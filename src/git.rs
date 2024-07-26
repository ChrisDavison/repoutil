use crate::PathBuf;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub struct RepoResult<'a> {
    pub path: &'a PathBuf,
    pub output: String,
}

pub fn is_git_repo(p: &Path) -> bool {
    let mut p = p.to_path_buf();
    p.push(".git");
    p.exists()
}

// Run a git command and return the lines of the output
fn command_output(dir: &PathBuf, command: &str) -> Result<Vec<String>> {
    let stdout = Command::new("git")
        .current_dir(dir)
        .args(command.split(' '))
        .output()?.stdout;
    Ok(std::str::from_utf8(&stdout)?
        .lines()
        .map(|x| x.to_string())
        .collect())
}

/// Push all changes to the branch
///
/// On success, returns nothing.
pub fn push(p: &PathBuf, _: bool) -> Result<RepoResult> {
    // We don't care about the output
    command_output(p, "push --all --tags").map(|_| RepoResult {
        path: p,
        output: String::new(),
    })
}

/// Fetch all branches of a git repo
pub fn fetch(p: &PathBuf, _: bool) -> Result<RepoResult> {
    // We don't care about the output
    command_output(p, "fetch --all --tags --prune").map(|_| RepoResult {
        path: p,
        output: String::new(),
    })
}

/// Get the short status (ahead, behind, and modified files) of a repo
pub fn stat(p: &PathBuf, as_json: bool) -> Result<RepoResult> {
    let out_lines = command_output(p, "status -s -b")?;
    let output = if out_lines.is_empty() {
        String::new()
    } else if out_lines[0].ends_with(']') {
        // We have an 'ahead', 'behind' or similar, so free to return the status early
        if as_json {
            format!(
                "{{\"title\": \"{}\", \"subtitle\": \"{}\"}}",
                p.display(),
                out_lines[0]
            )
        } else {
            format!("{}\n{}\n", p.display(), out_lines.join("\n"))
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
            String::new()
        } else if as_json {
            format!(
                "{{\"title\": \"{}\", \"subtitle\": \"{}\"}}",
                p.display(),
                status
            )
        } else {
            format!("{}\n{}\n", p.display(), status)
        }
    };
    Ok(RepoResult { path: p, output })
}

fn ahead_behind(p: &PathBuf) -> Result<Option<String>> {
    let response: String = command_output(p, "status --porcelain --ahead-behind -b")?
        .into_iter()
        .next()
        .filter(|x| x.contains('['))
        .unwrap_or(String::new());
    if response.is_empty() {
        Ok(None)
    } else {
        let start = response.find('[').unwrap();
        let end = response.find(']').unwrap();
        Ok(Some(
            response[start + 1..end]
                .replace("ahead ", "↑")
                .replace("behind ", "↓")
                .to_string(),
        ))
    }
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
    Ok(Some(
        [modif_str, untrack_str]
            .iter()
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", "),
    ))
}

/// Get a list of branches for the given git path
pub fn branches(p: &PathBuf, as_json: bool) -> Result<RepoResult> {
    let mut branches: Vec<_> = command_output(p, "branch")?;
    branches.sort();
    branches.reverse();
    let branches: String = branches
        .iter()
        .map(|x| x.trim().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Ok(RepoResult {
        path: p,
        output: if as_json {
            format!(
                "{{\"path\": \"{}\", \"subtitle\": \"{}\"}}",
                p.display(), branches
            )
        } else {
            format!("{:40}\t{}", p.display(), branches)
        },
    })
}

/// Get the status _of each branch_
pub fn branchstat(p: &PathBuf, as_json: bool) -> Result<RepoResult> {
    let outputs = [ahead_behind(p)?, modified(p)?]
        .iter()
        .filter(|&x| x.is_some())
        .map(|x| x.as_ref().unwrap().as_str())
        .collect::<Vec<&str>>()
        .join(", ");

    let output = if outputs.is_empty() {
        String::new()
    } else if as_json {
        format!(
            "{{\"title\": \"{}\", \"subtitle\": \"{}\", \"arg\": \"{}\"}}",
            p.display(),
            outputs,
            p.display(),
        )
    } else {
        format!("{:50} | {}", p.display(), outputs)
    };
    Ok(RepoResult { path: p, output })
}

/// Get the name of any repo with local or remote changes
pub fn needs_attention(p: &PathBuf, as_json: bool) -> Result<RepoResult> {
    let pstr = p.display().to_string();
    stat(p, as_json).map(|_| RepoResult {
        path: p,
        output: if as_json {
            format!("{{\"path\": {pstr}}}")
        } else {
            pstr
        },
    })
}

/// List each repo found
pub fn list(p: &PathBuf, as_json: bool) -> Result<RepoResult> {
    let pstr = p.display().to_string();
    Ok(RepoResult {
        path: p,
        output: if as_json {
            format!("{{\"title\": \"{pstr}\", \"arg\": \"{pstr}\"}}")
        } else {
            pstr
        },
    })
}

/// List each untracked repo found
pub fn untracked(p: &PathBuf, as_json: bool) -> Result<RepoResult> {
    let pstr = p.display().to_string();
    Ok(RepoResult {
        path: p,
        output: if as_json {
            format!("{{\"title\": \"! {pstr}\", \"arg\": \"{pstr}\"}}")
        } else {
            format!("! {}", pstr)
        },
    })
}
