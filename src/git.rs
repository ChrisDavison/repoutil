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

pub fn as_json(g: GitOutput, common_substr: &PathBuf) -> Option<String> {
    g.json(common_substr)
}

pub fn as_plain(g: GitOutput, common_substr: &PathBuf) -> Option<String> {
    g.plain(common_substr)
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
pub fn push(p: &PathBuf) -> Result<Option<GitOutput>> {
    command_output(p, "push --all --tags").map(|_| Some(GitOutput::Push(p)))
}

/// Fetch all branches of a git repo
pub fn fetch(p: &PathBuf) -> Result<Option<GitOutput>> {
    command_output(p, "fetch --all --tags --prune")?;
    branchstat(p)
}

/// Get the name of any repo with local or remote changes
pub fn needs_attention(p: &PathBuf) -> Result<Option<GitOutput>> {
    match stat(p) {
        Ok(Some(GitOutput::Stat(name, lines))) => {
            if lines.is_empty() {
                Ok(None)
            } else {
                Ok(Some(GitOutput::Unclean(name)))
            }
        }
        Ok(_) => unreachable!(),
        Err(e) => Err(e),
    }

    // stat(p).map(|something| {
    //     if let GitOutput::Stat(a, b) = something {
    //         dbg!(a);
    //         dbg!(b);
    //             GitOutput::Unclean(p)
    // })
}

/// List each repo found
pub fn list(p: &PathBuf) -> Result<Option<GitOutput>> {
    Ok(Some(GitOutput::List(p)))
}

/// List each untracked repo found
pub fn untracked(p: &PathBuf) -> Result<Option<GitOutput>> {
    Ok(Some(GitOutput::Untracked(p)))
}

/// Get the short status (ahead, behind, and modified files) of a repo
pub fn stat(p: &PathBuf) -> Result<Option<GitOutput>> {
    let out_lines = command_output(p, "status -s -b")?;
    let status = if out_lines.is_empty() || out_lines[0].ends_with(']') {
        out_lines
    } else {
        out_lines[1..].to_vec()
    };
    Ok(Some(GitOutput::Stat(p, status)))
}

/// Get a list of branches for the given git path
pub fn branches(p: &PathBuf) -> Result<Option<GitOutput>> {
    let mut branches: Vec<_> = command_output(p, "branch")?;
    branches.sort();
    branches.reverse();
    let branches: String = branches
        .iter()
        .map(|x| x.trim().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Ok(Some(GitOutput::Branches(p, branches)))
}

/// Get the status _of each branch_
pub fn branchstat(p: &PathBuf) -> Result<Option<GitOutput>> {
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

    Ok(Some(GitOutput::Branchstat(p, parts.join(", "))))
}
