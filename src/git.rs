use crate::PathBuf;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub enum GitOutput<'a> {
    #[allow(dead_code)]
    Push(&'a PathBuf),
    List(&'a PathBuf),
    Unclean(&'a PathBuf),
    Untracked(&'a PathBuf),
    Branches(&'a PathBuf, String),
    Branchstat(&'a PathBuf, String),
    Stat(&'a PathBuf, Vec<String>),
}

impl<'a> GitOutput<'a> {
    pub fn plain(&self, common_ancestor: &PathBuf) -> Option<String> {
        let f = |repo: &PathBuf| {
            repo.strip_prefix(common_ancestor)
                .unwrap()
                .display()
                .to_string()
        };
        let outstr = match self {
            // Don't want output for these cases
            GitOutput::Push(_) => return None,
            // Just show the shortened repo path
            GitOutput::List(p) => f(p),
            GitOutput::Unclean(p) => f(p),
            GitOutput::Untracked(p) => f(p),
            // More complicated outputs
            GitOutput::Branches(p, b) => format!("{:30}\t{}", f(p), b),
            GitOutput::Branchstat(p, o) => {
                if o.is_empty() {
                    return None;
                }
                format!("{:30} | {}", f(p), o)
            }
            GitOutput::Stat(p, ss) => {
                if ss.is_empty() {
                    return None;
                }
                format!("{}\n{}\n", f(p), ss.join("\n"))
            }
        };
        Some(outstr)
    }
    pub fn json(&self, common_ancestor: &PathBuf) -> Option<String> {
        let disp = |p: &PathBuf| p.display().to_string();
        let (title, subtitle, arg) = match self {
            // Don't want the outputs for these cases
            GitOutput::Push(_) => return None, // early return. don't care about output
            // Just show the shortened repo path
            GitOutput::List(p) => (p, None, disp(p)),
            GitOutput::Unclean(p) => (p, None, disp(p)),
            GitOutput::Untracked(p) => (p, None, disp(p)),
            // More complicated outputs
            GitOutput::Branches(p, b) => (p, Some(b.clone()), String::new()),
            GitOutput::Branchstat(p, o) => {
                if o.is_empty() {
                    return None;
                }
                (p, Some(o.clone()), disp(p))
            }
            GitOutput::Stat(p, ss) => {
                if ss.is_empty() {
                    return None;
                }
                (p, Some(ss.join(", ")), disp(p))
            }
        };
        let title = title
            .strip_prefix(common_ancestor)
            .ok()?
            .display()
            .to_string();
        let mut fields = format!(r#""title": "{title}", "arg": "{arg}""#);
        if let Some(sub) = subtitle {
            fields += &format!(r#", "subtitle": "{sub}""#);
        }
        Some(format!(r#"{{{fields}}}"#))
    }
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
pub fn push(p: &PathBuf) -> Result<GitOutput> {
    command_output(p, "push --all --tags").map(|_| GitOutput::Push(p))
}

/// Fetch all branches of a git repo
pub fn fetch(p: &PathBuf) -> Result<GitOutput> {
    command_output(p, "fetch --all --tags --prune")?;
    branchstat(p)
}

/// Get the name of any repo with local or remote changes
pub fn needs_attention(p: &PathBuf) -> Result<GitOutput> {
    stat(p).map(|_| GitOutput::Unclean(p))
}

/// List each repo found
pub fn list(p: &PathBuf) -> Result<GitOutput> {
    Ok(GitOutput::List(p))
}

/// List each untracked repo found
pub fn untracked(p: &PathBuf) -> Result<GitOutput> {
    Ok(GitOutput::Untracked(p))
}

/// Get the short status (ahead, behind, and modified files) of a repo
pub fn stat(p: &PathBuf) -> Result<GitOutput> {
    let out_lines = command_output(p, "status -s -b")?;
    let status = if out_lines.is_empty() || out_lines[0].ends_with(']') {
        out_lines
    } else {
        out_lines[1..].to_vec()
    };
    Ok(GitOutput::Stat(p, status))
}

fn ahead_behind(p: &PathBuf) -> Result<Option<String>> {
    let first_line = command_output(p, "status --porcelain --ahead-behind -b")?
        .into_iter()
        .next();

    match first_line.filter(|x| x.contains('[')) {
        Some(response) => {
            // We're already filtering on contains, so safe to unwrap
            let start = response.find('[').unwrap();
            let end = response.find(']').unwrap();
            Ok(Some(
                response[start + 1..end]
                    .replace("ahead ", "↑")
                    .replace("behind ", "↓")
                    .to_string(),
            ))
        }
        // We should _always_ have a 'next' on the command output as the above
        // command outputs the branch info as the first line.
        // Therefore, 'none' only occurs with an empty filter response.
        None => Ok(None),
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
pub fn branches(p: &PathBuf) -> Result<GitOutput> {
    let mut branches: Vec<_> = command_output(p, "branch")?;
    branches.sort();
    branches.reverse();
    let branches: String = branches
        .iter()
        .map(|x| x.trim().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Ok(GitOutput::Branches(p, branches))
}

/// Get the status _of each branch_
pub fn branchstat(p: &PathBuf) -> Result<GitOutput> {
    let outputs = [ahead_behind(p)?, modified(p)?]
        .iter()
        .filter(|&x| x.is_some())
        .map(|x| x.as_ref().unwrap().as_str())
        .collect::<Vec<&str>>()
        .join(", ");

    Ok(GitOutput::Branchstat(p, outputs))
}
