use crate::PathBuf;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;

pub fn is_git_repo(p: &Path) -> bool {
    let mut p = p.to_path_buf();
    p.push(".git");
    p.exists()
}

// Run a git command and return the lines of the output
fn command_output(dir: &PathBuf, command: &str) -> Result<Vec<String>> {
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
pub fn push(p: &PathBuf, _: bool) -> Result<Option<String>> {
    // We don't care about the output
    command_output(p, "push --all --tags").map(|_| None)
}

/// Fetch all branches of a git repo
pub fn fetch(p: &PathBuf, _: bool) -> Result<Option<String>> {
    // We don't care about the output
    command_output(p, "fetch --all --tags --prune").map(|_| None)
}

/// Get the short status (ahead, behind, and modified files) of a repo
pub fn stat(p: &PathBuf, as_json: bool) -> Result<Option<String>> {
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
        let status: String = out_lines
            .iter()
            .skip(1)
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        if status.is_empty() {
            Ok(None)
        } else if as_json {
            Ok(Some(format!(
                "{{\"title\": \"{}\", \"subtitle\": \"{}\"}}",
                p.display(),
                status
            )))
        } else {
            Ok(Some(format!("{}\n{}\n", p.display(), status)))
        }
    }
}

fn ahead_behind(p: &PathBuf) -> Result<Option<String>> {
    let response: String = command_output(p, "status --porcelain --ahead-behind -b")?
        .into_iter()
        .next()
        .filter(|x| x.contains('['))
        .unwrap_or(String::new());
    Ok(if response.is_empty() {
        None
    } else {
        let start = response.find('[').unwrap();
        let end = response.find(']').unwrap();
        Some(
            response[start + 1..end]
                .replace("ahead ", "↑")
                .replace("behind ", "↓")
                .to_string(),
        )
    })
}

fn modified(p: &PathBuf) -> Result<Option<String>> {
    let mut modif = 0;
    let mut untracked = 0;
    for line in command_output(p, "status -s -b")?.into_iter().skip(1) {
        let trimmed = line.trim_start().to_string();
        let trimmed = if trimmed.starts_with('\u{1b}') {
            trimmed[5..6].to_string()
        } else {
            trimmed
        };
        if trimmed == "M" {
            modif += 1;
        }
        if trimmed == "?" {
            untracked += 1;
        }
    }
    let modif_str = if modif > 0 {
        format!("{}±", modif)
    } else {
        String::new()
    };
    let untrack_str = if untracked > 0 {
        format!("{}?", untracked)
    } else {
        String::new()
    };
    let outstr = [modif_str, untrack_str]
        .iter()
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    if !outstr.is_empty() {
        Ok(Some(outstr))
    } else {
        Ok(None)
    }
}

/// Get a list of branches for the given git path
pub fn branches(p: &PathBuf, as_json: bool) -> Result<Option<String>> {
    let mut branches: Vec<_> = command_output(p, "branch")?;
    branches.sort();
    branches.reverse();
    let branches: String = branches.iter().map(|x| x.trim().to_string()).collect::<Vec<_>>().join(", ");
    let parent_path = p.parent().ok_or_else(|| anyhow!("No parent for dir"))?;
    let parent_name = parent_path
        .file_stem()
        .ok_or_else(|| anyhow!("No stem for parent"))?
        .to_string_lossy();
    let dir_name = p
        .file_stem()
        .ok_or_else(|| anyhow!("No stem for dir"))?
        .to_string_lossy();
    let dirstr = format!("{}/{}", parent_name, dir_name);
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
pub fn branchstat(p: &PathBuf, as_json: bool) -> Result<Option<String>> {
    let outputs = [ahead_behind(p)?, modified(p)?]
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
            format!("{:50} | {}", p.display(), outputs)
        }))
    }
}

/// Get the name of any repo with local or remote changes
pub fn needs_attention(p: &PathBuf, as_json: bool) -> Result<Option<String>> {
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
pub fn list(p: &PathBuf, as_json: bool) -> Result<Option<String>> {
    let pstr = p.display().to_string();
    Ok(Some(if as_json {
        format!("{{\"title\": \"{pstr}\", \"arg\": \"{pstr}\"}}")
    } else {
        pstr
    }))
}

/// List each untracked repo found
pub fn untracked(p: &PathBuf, as_json: bool) -> Result<Option<String>> {
    let pstr = p.display().to_string();
    Ok(Some(if as_json {
        format!("{{\"title\": \"! {pstr}\", \"arg\": \"{pstr}\"}}")
    } else {
        format!("! {}", pstr)
    }))
}
