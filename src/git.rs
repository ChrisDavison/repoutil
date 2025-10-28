use crate::{FormatOpts, PathBuf};

use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn remove_common_ancestor(repo: &Path, common: Option<&PathBuf>) -> String {
    if let Some(prefix) = common {
        repo.strip_prefix(prefix).unwrap().display().to_string()
    } else {
        repo.display().to_string()
    }
}

fn format_json(
    title: &Path,
    subtitle: Option<&str>,
    path_as_arg: bool,
    common: Option<&PathBuf>,
) -> String {
    let arg = if path_as_arg {
        title.display().to_string()
    } else {
        String::new()
    };
    let title = remove_common_ancestor(title, common);
    let mut fields = format!(r#""title": "{title}", "arg": "{arg}""#);
    if let Some(sub) = subtitle {
        fields += &format!(r#", "subtitle": "{sub}""#);
    }
    format!(r#"{{{fields}}}"#)
}

pub fn is_git_repo(p: &Path) -> bool {
    let mut p = p.to_path_buf();
    p.push(".git");
    p.exists()
}

pub fn jjsync(dir: &Path, _fmt: &FormatOpts) -> Result<Option<String>> {
    Command::new("jj")
        .current_dir(dir)
        .args(&["sync"])
        .output()?;
    Ok(None)
}

pub fn jjstat(dir: &Path, _fmt: &FormatOpts) -> Result<Option<String>> {
    let stdout = Command::new("jj")
        .current_dir(dir)
        .args(&["status", "--color=always"])
        .output()?
        .stdout;
    let lines: Vec<&str> = std::str::from_utf8(&stdout)?.lines().collect();
    if lines.is_empty() {
        Ok(None)
    } else if lines[0] == "The working copy has no changes." {
        Ok(None)
    } else {
        Ok(Some(format!(
            "{}\n{}\n",
            dir.to_string_lossy().to_string(),
            lines
                .iter()
                .map(|x| format!("    {x}"))
                .collect::<Vec<_>>()
                .join("\n")
        )))
    }
}

// Run a git command and return the lines of the output
fn command_output(dir: &Path, command: &str) -> Result<Vec<String>> {
    let stdout = Command::new("git")
        .current_dir(dir)
        .args(command.split(' '))
        .output()?
        .stdout;
    Ok(std::str::from_utf8(&stdout)?
        .lines()
        .map(|x| x.to_string())
        .collect())
}

/// Push all changes to the branch
///
/// On success, returns nothing.
pub fn push(p: &Path, _fmt: &FormatOpts) -> Result<Option<String>> {
    command_output(p, "push --all --tags")?;
    Ok(None)
}

/// Fetch all branches of a git repo
pub fn fetch(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    command_output(p, "fetch --all --tags --prune")?;
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

/// List each repo found
pub fn list(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    let s = if fmt.use_json {
        format_json(p, None, true, fmt.common_prefix)
    } else {
        remove_common_ancestor(p, fmt.common_prefix)
    };
    Ok(Some(s))
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
    let out_lines = command_output(p, "status -s -b")?;
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
    let mut branches: Vec<_> = command_output(p, "branch")?;
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

/// Get the status _of each branch_
pub fn branchstat(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    let mut response = command_output(p, "status --porcelain --ahead-behind -b")?.into_iter();

    let branch_line = response.next();

    // Get the 'ahead/behind' status
    let mut parts = Vec::new();
    if let Some(response) = branch_line.filter(|x| x.contains('[')) {
        // We're already filtering on contains, so safe to unwrap
        let start = response.find('[').unwrap();
        let end = response.find(']').unwrap();
        parts.push(
            response[start + 1..end]
                .replace("ahead ", "↑")
                .replace("behind ", "↓")
                .to_string(),
        )
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
        parts.push(format!("{}±", n_modified))
    };
    if n_untracked > 0 {
        parts.push(format!("{}?", n_untracked))
    };

    let joined = parts.join(", ");

    if joined.is_empty() {
        return Ok(None);
    }

    let s = if fmt.use_json {
        format_json(p, Some(&joined), true, fmt.common_prefix)
    } else {
        format!(
            "{:30} | {}",
            remove_common_ancestor(p, fmt.common_prefix),
            joined
        )
    };
    Ok(Some(s))
}

pub fn add() -> Result<()> {
    let config_filename = crate::util::homedir(".repoutilrc")?;
    let curdir = std::env::current_dir()?;
    if is_git_repo(&curdir) {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(config_filename)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", curdir.to_string_lossy()) {
            eprintln!("Couldn't write to file: {}", e);
        }
        Ok(())
    } else {
        Err(anyhow!("Don't appear to be in the root of a git repo."))
    }
}
