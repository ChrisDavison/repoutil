use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;

pub fn is_git_repo(p: &Path) -> bool {
    let mut p = p.to_path_buf();
    p.push(".git");
    p.exists()
}

// Run a git command and return the lines of the output
fn command_output(dir: &Path, command: &str) -> Result<Vec<String>> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(command.split(' '))
        .output()?;
    Ok(std::str::from_utf8(&out.stdout)?
        .lines()
        .map(|x| x.to_string())
        .collect())
}

/// Push all changes to the branch
///
/// On success, returns nothing.
pub fn push(p: &Path, _: bool) -> Result<Option<String>> {
    // We don't care about the output
    command_output(p, "push --all --tags").map(|_| None)
}

/// Fetch all branches of a git repo
pub fn fetch(p: &Path, _: bool) -> Result<Option<String>> {
    // We don't care about the output
    command_output(p, "fetch --all --tags --prune").map(|_| None)
}

/// Get the short status (ahead, behind, and modified files) of a repo
pub fn stat(p: &Path, as_json: bool) -> Result<Option<String>> {
    let out_lines = command_output(p, "status -s -b")?;
    if out_lines.is_empty() {
        Ok(None)
    } else if out_lines[0].ends_with(']') {
        // We have an 'ahead', 'behind' or similar, so free to return the status early
        if as_json {
            Ok(Some(format!(
            "{{\"title\": \"{}\", \"subtitle\": \"{}\"}}",
            p.display(),
            out_lines[0]
            )))
        } else {
            Ok(Some(format!("{}\n{}\n", p.display(), out_lines.join("\n"))))
        }
    } else {
        // We aren't ahead or behind etc, but may have local uncommitted changes
        let status: String = out_lines.iter().skip(1).map(|x| x.to_string()).collect::<Vec<String>>().join("\n");
        if status.is_empty() {
            Ok(None)
        } else {
            if as_json {
                Ok(Some(format!(
                    "{{\"title\": \"{}\", \"subtitle\": \"{}\"}}",
                    p.display(),
                    status
                ))
                )
            } else {
                Ok(Some(format!("{}\n{}\n", p.display(), status)))
            }
        }
    }
}

fn ahead_behind(p: &Path) -> Result<Option<String>> {
    let response: String = command_output(
        p,
        "status --porcelain --ahead-behind -b",
    )?.into_iter().next().filter(|x| x.contains("[")).unwrap_or(String::new());
    Ok(if response.is_empty() {
        None
    } else {
        let start = response.find('[').unwrap();
        let end = response.find(']').unwrap();
        Some(response[start + 1..end].to_string())
    })
}

fn modified(p: &Path) -> Result<Option<String>> {
    let modified = command_output(p, "diff --shortstat")?.join("\n");
    if modified.contains("changed") {
        let num = modified.trim_start().split(' ').collect::<Vec<&str>>()[0];
        Ok(Some(format!("{}±", num)))
    } else {
        Ok(None)
    }
}

fn status(p: &Path) -> Result<Option<String>> {
    let response = command_output(p, "diff --stat --cached")?;
    if !response.is_empty() {
        Ok(Some(format!("Staged {}", response.len())))
    } else {
        Ok(None)
    }
}

fn untracked(p: &Path) -> Result<Option<String>> {
    let untracked = command_output(p, "ls-files --others --exclude-standard")?;
    Ok(if untracked.is_empty() {
        None
    } else {
        Some(format!("{}?", untracked.len()))
    })
}

/// Get a list of branches for the given git path
pub fn branches(p: &Path, as_json: bool) -> Result<Option<String>> {
    let branches: String = command_output(p, "branch")?
        .iter()
        .map(|x| x.trim())
        .filter(|x| x.starts_with('*'))
        .map(|x| &x[2..])
        .collect();
    let parentpath = p.parent().ok_or_else(|| anyhow!("No parent for dir"))?;
    let parentname = parentpath
        .file_stem()
        .ok_or_else(|| anyhow!("No stem for parent"))?
        .to_string_lossy();
    let dirname = p
        .file_stem()
        .ok_or_else(|| anyhow!("No stem for dir"))?
        .to_string_lossy();
    let dirstr = format!("{}/{}", parentname, dirname);
    Ok(Some(if as_json {
        format!(
            "{{\"path\": \"{}\", \"subtitle\": \"{}\"}}",
            dirstr, branches
        )
    } else {
        format!("{:40}\t{}", dirstr, branches)
    }))
}

/// Get the status _of each branch_
pub fn branchstat(p: &Path, as_json: bool) -> Result<Option<String>> {
    let outputs = [ahead_behind(p)?, modified(p)?, status(p)?, untracked(p)?]
        .iter()
        .filter(|&x| x.is_some())
        .map(|x| x.as_ref().unwrap().as_str())
        .collect::<Vec<&str>>()
        .join(", ");

    if outputs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(if as_json {
            format!(
                "{{\"title\": \"{}\", \"subtitle\": \"{}\", \"arg\": \"{}\"}}",
                p.display(),
                outputs,
                p.display(),
            )
        } else {
            format!(
                "{:40} | {}",
                p.display(),
                outputs
            )
        }))
    }
}

/// Get the name of any repo with local or remote changes
pub fn needs_attention(p: &Path, as_json: bool) -> Result<Option<String>> {
    let pstr = p.display().to_string();
    stat(p, as_json).map(|_| {
        Some(if as_json {
            format!("{{\"path\": {pstr}}}")
        } else {
            pstr
        })
    })
}

/// List each repo found
pub fn list(p: &Path, as_json: bool) -> Result<Option<String>> {
    let pstr = p.display().to_string();
    Ok(Some(if as_json {
        format!("{{\"path\": {pstr}}}")
    } else {
        pstr
    }))
}
