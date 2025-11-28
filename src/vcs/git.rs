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
        .args(["fetch", "--all", "--tags", "--prune"])
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
    let stdout = Command::new("git")
        .current_dir(p)
        .args(["status", "-s", "-b"])
        .output()?
        .stdout;
    let out_lines: Vec<&str> = std::str::from_utf8(&stdout)?.lines().collect();
    let status = if out_lines.is_empty() || out_lines[0].ends_with(']') {
        out_lines
    } else {
        out_lines[1..].to_vec()
    };
    if status.is_empty() {
        return Ok(None);
    }

    let s = if fmt.use_json {
        format_json(p, Some(&status.join(", ")), true, fmt.common_prefix)
    } else {
        format!(
            "{}\n{}\n",
            remove_common_ancestor(p, fmt.common_prefix),
            status.join("\n")
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

/// Dashboard ... repo status, and recent commits
pub fn dashboard(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    if let Ok(Some(_)) = needs_attention(p, fmt) {
        let stdout = Command::new("git")
            .current_dir(p)
            .args(["dashboard", "--color=always"])
            .output()?
            .stdout;
        let resp: Vec<&str> = std::str::from_utf8(&stdout)?.lines().collect();
        Ok(Some(format!(
            "\n\x1b[42m\x1b[1;30m {} {}\x1b[0m\n{}",
            remove_common_ancestor(p, fmt.common_prefix),
            "·".repeat(0),
            resp.iter()
                .map(|l| format!("  {l}"))
                .collect::<Vec<_>>()
                .join("\n")
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
    let jj_mutable = std::str::from_utf8(&stdout)?
        .lines().count();

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
