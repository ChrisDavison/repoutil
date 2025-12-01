use super::*;

pub fn is_repo(p: &Path) -> bool {
    let mut p = p.to_path_buf();
    p.push(".git");
    p.exists()
}

/// Push all changes to the branch
pub fn push(p: &Path, _fmt: &FormatOpts) -> Result<Option<String>> {
    Command::new("git")
        .current_dir(p)
        .args(["push", "--all", "--tags"])
        .status()?;
    Ok(None)
}

/// Fetch all branches of a git repo
pub fn fetch(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    Command::new("git")
        .current_dir(p)
        .args(["fetch", "--all", "--tags"])
        .status()?;
    branchstat(p, fmt)
}

/// Get the name of any repo with local or remote changes
pub fn needs_attention(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    match stat(p, &fmt.clone()) {
        Ok(Some(output)) => {
            if output.is_empty() {
                Ok(None)
            } else if fmt.use_json {
                Ok(Some(format_json(p, None, true, fmt.common_prefix)))
            } else {
                Ok(Some(remove_common_ancestor(p, fmt.common_prefix)))
            }
        }
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}

/// List each untracked repo found
pub fn untracked(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    let s = if fmt.use_json {
        format_json(p, None, true, fmt.common_prefix)
    } else {
        remove_common_ancestor(p, fmt.common_prefix)
    };
    Ok(Some(s))
}

/// Get the short status (ahead, behind, and modified files) of a repo
pub fn stat(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    // Parse git status into branch info and vector of (status, filepath) tuples
    let (branch, status_vec) = parse_git_status(p)?;

    let branch_colours = if fmt.no_colour { vec![] } else { vec![BLUE] };
    let commit_colours = if fmt.no_colour { vec![] } else { vec![YELLOW] };
    let formatted_branch = if fmt.no_colour {
        branch.clone()
    } else {
        colour(&branch, &branch_colours)
    };

    // Get short commit hash
    let commit_hash = get_short_commit_hash(p)?;

    let formatted_commit = if fmt.no_colour {
        commit_hash.clone()
    } else {
        colour(&commit_hash, &commit_colours)
    };

    // Format file list from status vector
    let file_list = format_file_list(&status_vec, fmt);
    if file_list.is_empty() {
        return Ok(None);
    }

    let s = if fmt.use_json {
        format_json(p, Some(&file_list), true, fmt.common_prefix)
    } else {
        format!(
            "{} on {formatted_branch} at {formatted_commit}\n{}\n",
            colour(remove_common_ancestor(p, fmt.common_prefix), &[PURPLE]),
            file_list
        )
    };

    Ok(Some(s))
}

/// Get a list of branches for the given git path
pub fn branches(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    let stdout = Command::new("git")
        .current_dir(p)
        .args(["branch"])
        .output()?
        .stdout;
    let mut branches: Vec<&str> = std::str::from_utf8(&stdout)?.lines().collect();
    branches.sort();
    branches.reverse();
    let branches: String = branches
        .iter()
        .map(|x| x.trim().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let s = if fmt.use_json {
        format_json(p, Some(&branches), false, fmt.common_prefix)
    } else {
        format!(
            "{:30}\t{}",
            remove_common_ancestor(p, fmt.common_prefix),
            branches
        )
    };
    Ok(Some(s))
}

/// Parse git status -s -b output into branch info and vector of (status, filepath) tuples
fn parse_git_status(p: &Path) -> Result<(String, Vec<(String, String)>)> {
    let stdout = Command::new("git")
        .current_dir(p)
        .args(["status", "-s", "-b"])
        .output()?
        .stdout;
    let out_lines: Vec<&str> = std::str::from_utf8(&stdout)?.lines().collect();

    // Extract branch info from first line
    let branch = if out_lines.is_empty() {
        "unknown".to_string()
    } else {
        let first_line = out_lines[0];
        if let Some(branch_part) = first_line.strip_prefix("## ") {
            // Remove any tracking info after "..."
            let branch_end = branch_part.find("...").unwrap_or(branch_part.len());
            let branch_name = &branch_part[..branch_end];
            if branch_name == "HEAD (no branch)" {
                "HEAD".to_string()
            } else {
                branch_name.to_string()
            }
        } else {
            "unknown".to_string()
        }
    };

    // Skip branch line and process status lines
    let status_lines = if out_lines.len() > 1 {
        &out_lines[1..]
    } else {
        &[]
    };

    let mut result = Vec::new();
    for line in status_lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            // Find where status codes end and filepath begins
            let status_end = trimmed.find(' ').unwrap_or(trimmed.len());
            let status = trimmed[..status_end].to_string();
            let filepath = trimmed[status_end..].trim().to_string();
            result.push((status, filepath));
        }
    }

    Ok((branch, result))
}

/// Get short commit hash for current HEAD
fn get_short_commit_hash(p: &Path) -> Result<String> {
    let stdout = Command::new("git")
        .current_dir(p)
        .args(["rev-parse", "--short", "HEAD"])
        .output()?
        .stdout;
    Ok(std::str::from_utf8(&stdout)?.trim().to_string())
}

/// Get recent commits - last week's commits if any, otherwise 10 most recent
fn get_recent_commits(p: &Path) -> Result<String> {
    // Try to get commits from last week first
    let stdout = Command::new("git")
        .current_dir(p)
        .args([
            "log",
            "--since=\"1 week ago\"",
            "--pretty=lo",
            "--color=always",
            "--date=short",
        ])
        .output()?
        .stdout;

    let commits = std::str::from_utf8(&stdout)?.trim();

    if !commits.is_empty() {
        Ok(commits.to_string())
    } else {
        // Fallback to 10 most recent commits
        let stdout = Command::new("git")
            .current_dir(p)
            .args([
                "log",
                "-n",
                "10",
                "--pretty=lo",
                "--color=always",
                "--date=short",
            ])
            .output()?
            .stdout;
        Ok(std::str::from_utf8(&stdout)?.trim().to_string())
    }
}

/// Format the file list from status vector for display
fn format_file_list(status_vec: &[(String, String)], fmt: &FormatOpts) -> String {
    if status_vec.is_empty() {
        return String::new();
    }

    let colours = if fmt.no_colour { vec![] } else { vec![GREEN] };
    status_vec
        .iter()
        .map(|(status, filepath)| {
            let formatted_status = if fmt.no_colour {
                status.clone()
            } else {
                colour(status, &colours)
            };
            format!("{} {}", formatted_status, filepath)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Dashboard ... repo status, and recent commits
pub fn dashboard(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    if let Ok(Some(_)) = needs_attention(p, fmt) {
        // Parse git status into branch info and vector of (status, filepath) tuples
        let (branch, status_vec) = parse_git_status(p)?;

        // Get short commit hash
        let commit_hash = get_short_commit_hash(p)?;

        // Format file list from status vector
        let file_list = format_file_list(&status_vec, fmt);

        // Get recent commits
        let commits = get_recent_commits(p)?;

        // Build the output
        let repo_name = remove_common_ancestor(p, fmt.common_prefix);

        // Add branch and commit info
        let branch_colours = if fmt.no_colour { vec![] } else { vec![BLUE] };
        let commit_colours = if fmt.no_colour { vec![] } else { vec![YELLOW] };
        let formatted_branch = if fmt.no_colour {
            branch.clone()
        } else {
            colour(&branch, &branch_colours)
        };
        let formatted_commit = if fmt.no_colour {
            commit_hash.clone()
        } else {
            colour(&commit_hash, &commit_colours)
        };
        let mut output = String::new();

        // Add file list if there are any files
        if !file_list.is_empty() {
            output.push_str(&file_list);
        }

        // Add commits if there are any
        if !commits.is_empty() {
            output.push_str("\n\n");
            output.push_str("Recent commits:\n");
            let commit_lines: Vec<&str> = commits.lines().collect();
            for line in commit_lines {
                output.push_str(&format!("{}\n", line));
            }
        }
        let bar = colour("░", &[BLACK]);
        Ok(Some(format!(
            "\n{} on {formatted_branch} at {formatted_commit}\n{}",
            colour(&repo_name, &[PURPLE]),
            output
                .lines()
                .map(|x| format!("{bar} {x}"))
                .collect::<Vec<_>>()
                .join("\n"),
        )))
    } else {
        Ok(None)
    }
}

/// Get the status _of each branch_
pub fn branchstat(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    let stdout = Command::new("git")
        .current_dir(p)
        .args(["status", "--porcelain", "--ahead-behind", "-b"])
        .output()?
        .stdout;
    let mut response = std::str::from_utf8(&stdout)?.lines();
    let stdout = Command::new("jj")
        .current_dir(p)
        .args(["list_mut"])
        .output()?
        .stdout;
    let jj_mutable = std::str::from_utf8(&stdout)?.lines().count();

    let branch_line = response.next();

    // Get the 'ahead/behind' status
    let mut parts = Vec::new();
    let colours = if fmt.no_colour { vec![] } else { vec![BLUE] };
    if let Some(response) = branch_line.filter(|x| x.contains('[')) {
        // We're already filtering on contains, so safe to unwrap
        let start = response.find('[').unwrap();
        let end = response.find(']').unwrap();
        let s = response[start + 1..end]
            .replace("ahead ", "↑")
            .replace("behind ", "↓")
            .to_string();
        parts.push(colour(s, &colours))
    }

    // Now go through each file reported, and count modified or untracked
    let mut n_modified = 0;
    let mut n_untracked = 0;
    for line in response {
        let trimmed = line.trim_start().to_string();
        let trimmed = if trimmed.starts_with('\u{1b}') {
            trimmed[5..6].to_string()
        } else {
            trimmed
        };
        if trimmed.starts_with("M") {
            n_modified += 1;
        }
        if trimmed.starts_with("?") {
            n_untracked += 1;
        }
    }

    if n_modified > 0 {
        let s = format!("{}±", n_modified);
        parts.push(if fmt.no_colour {
            s
        } else {
            colour(s, &[GREEN])
        })
    };
    if n_untracked > 0 {
        let s = format!("{}?", n_untracked);
        parts.push(if fmt.no_colour {
            s
        } else {
            colour(s, &[YELLOW])
        })
    };

    if jj_mutable != 0 {
        let s = format!("{}▲", jj_mutable);
        parts.push(if fmt.no_colour {
            s
        } else {
            colour(s, &[YELLOW])
        })
    }

    let joined = parts.join(", ");

    if joined.is_empty() {
        return Ok(None);
    }

    let s = if fmt.use_json {
        format_json(p, Some(&joined), true, fmt.common_prefix)
    } else {
        let s = remove_common_ancestor(p, fmt.common_prefix);
        format!(
            "{:40} | {}",
            if fmt.no_colour {
                s
            } else {
                colour(s, &[BOLD, RED])
            },
            joined
        )
    };
    Ok(Some(s))
}
