use crate::{FormatOpts, Row};

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ansi_escape::*;

const SYM_AHEAD: &str = "↑";
const SYM_BEHIND: &str = "↓";
const SYM_MODIFIED: &str = "±";
#[cfg(feature = "jj")]
const SYM_JJ_MUTABLE: &str = "*";

/// Run `git` in the given directory, returning stdout
fn git(p: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").current_dir(p).args(args).output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8(output.stdout)?)
}

/// Root directory of the repo containing `p`
pub fn toplevel(p: &Path) -> Result<PathBuf> {
    Ok(PathBuf::from(
        git(p, &["rev-parse", "--show-toplevel"])?.trim(),
    ))
}

/// List each repo found
pub fn list(_: &Path, _: &FormatOpts) -> Result<Option<Row>> {
    Ok(Some(Row::default()))
}

pub fn is_repo(p: &Path) -> bool {
    p.join(".git").exists()
}

/// Fetch all branches of a git repo
pub fn fetch(p: &Path, fmt: &FormatOpts) -> Result<Option<Row>> {
    git(p, &["fetch", "--all", "--quiet"])?;
    branchstat(p, fmt)
}

/// Get a list of branches for the given git path
pub fn branches(p: &Path, _: &FormatOpts) -> Result<Option<Row>> {
    // Mark the current branch with `*` and those checked out elsewhere with `+`,
    // as plain `git branch` does, while keeping its (configurable) sort order
    let stdout = git(
        p,
        &[
            "branch",
            "--format=%(if)%(HEAD)%(then)* %(else)%(if)%(worktreepath)%(then)+ %(end)%(end)%(refname:short)",
        ],
    )?;
    let branches = stdout.lines().collect::<Vec<_>>().join(" // ");
    Ok(Some(Row {
        detail: branches,
        ..Row::default()
    }))
}

/// Summary of `git status --porcelain=v2 --branch`
#[derive(Debug, Default, PartialEq)]
struct RepoStatus {
    /// `None` when HEAD is detached
    branch: Option<String>,
    ahead: usize,
    behind: usize,
    /// Upstream is configured but no longer exists on the remote
    upstream_gone: bool,
    added: usize,
    deleted: usize,
    modified: usize,
    untracked: usize,
}

impl RepoStatus {
    fn parse(output: &str) -> RepoStatus {
        let mut status = RepoStatus::default();
        let mut has_upstream = false;
        let mut has_ab = false;
        for line in output.lines() {
            if let Some(header) = line.strip_prefix("# ") {
                match header.split_once(' ') {
                    Some(("branch.head", "(detached)")) => status.branch = None,
                    Some(("branch.head", name)) => status.branch = Some(name.to_string()),
                    Some(("branch.upstream", _)) => has_upstream = true,
                    Some(("branch.ab", ab)) => {
                        has_ab = true;
                        for n in ab.split(' ') {
                            if let Some(n) = n.strip_prefix('+') {
                                status.ahead = n.parse().unwrap_or(0);
                            } else if let Some(n) = n.strip_prefix('-') {
                                status.behind = n.parse().unwrap_or(0);
                            }
                        }
                    }
                    _ => {}
                }
                continue;
            }
            // Changed entries are `<kind> <XY> ...`, where X is staged and Y unstaged
            match line.split(' ').take(2).collect::<Vec<_>>()[..] {
                ["?", _] => status.untracked += 1,
                ["1" | "2" | "u", xy] => {
                    if xy.starts_with('A') {
                        status.added += 1;
                    } else if xy.contains('D') {
                        status.deleted += 1;
                    } else {
                        status.modified += 1;
                    }
                }
                _ => {}
            }
        }
        status.upstream_gone = has_upstream && !has_ab;
        status
    }
}

/// Get the status _of each branch_
pub fn branchstat(p: &Path, fmt: &FormatOpts) -> Result<Option<Row>> {
    let status = RepoStatus::parse(&git(p, &["status", "--porcelain=v2", "--branch"])?);

    let mut parts = Vec::new();

    let mut tracking = Vec::new();
    if status.ahead > 0 {
        tracking.push(format!("{SYM_AHEAD}{}", status.ahead));
    }
    if status.behind > 0 {
        tracking.push(format!("{SYM_BEHIND}{}", status.behind));
    }
    if status.upstream_gone {
        tracking.push("gone".to_string());
    }
    if !tracking.is_empty() {
        parts.push(fmt.paint(tracking.join(", "), &[BLUE]));
    }

    let counts = [
        (status.added, "+", GREEN),
        (status.deleted, "-", RED),
        (status.modified, SYM_MODIFIED, GREEN),
        (status.untracked, "?", YELLOW),
    ];
    for (n, sym, color) in counts {
        if n > 0 {
            parts.push(fmt.paint(format!("{n}{sym}"), &[color]));
        }
    }

    #[cfg(feature = "jj")]
    if let Some(n) = jj_mutable_count(p).filter(|&n| n > 0) {
        parts.push(fmt.paint(format!("{n}{SYM_JJ_MUTABLE}"), &[PURPLE]));
    }

    if parts.is_empty() {
        return Ok(None);
    }
    let branch = status.branch.unwrap_or_else(|| "HEAD".to_string());
    Ok(Some(Row {
        path_style: &[BOLD, RED],
        note: (!matches!(branch.as_str(), "main" | "master")).then(|| format!("on {branch}")),
        detail: parts.join(fmt.separator),
    }))
}

/// Number of non-empty mutable jj commits, or `None` if this isn't a jj repo
/// or jj can't be run
#[cfg(feature = "jj")]
fn jj_mutable_count(p: &Path) -> Option<usize> {
    if !p.join(".jj").exists() {
        return None;
    }
    let output = Command::new("jj")
        .current_dir(p)
        .args([
            "log",
            "-r",
            "mutable()::",
            "--no-graph",
            "-T",
            r#"if(!empty, "x\n")"#,
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    Some(String::from_utf8_lossy(&output.stdout).lines().count())
}

#[cfg(test)]
mod tests {
    use super::RepoStatus;

    #[test]
    fn test_parse_status() {
        let output = "\
# branch.oid 0123456789abcdef
# branch.head feat.something
# branch.upstream origin/feat.something
# branch.ab +2 -1
1 M. N... 100644 100644 100644 aaa bbb staged.rs
1 .M N... 100644 100644 100644 aaa bbb unstaged.rs
1 MM N... 100644 100644 100644 aaa bbb both.rs
1 A. N... 000000 100644 100644 000 bbb new.rs
1 .D N... 100644 100644 000000 aaa aaa gone.rs
2 R. N... 100644 100644 100644 aaa bbb R100 to.rs\tfrom.rs
? untracked.rs
";
        assert_eq!(
            RepoStatus::parse(output),
            RepoStatus {
                branch: Some("feat.something".to_string()),
                ahead: 2,
                behind: 1,
                upstream_gone: false,
                added: 1,
                deleted: 1,
                modified: 4,
                untracked: 1,
            }
        );
    }

    #[test]
    fn test_parse_status_detached() {
        let status = RepoStatus::parse("# branch.oid 0123\n# branch.head (detached)\n");
        assert_eq!(status.branch, None);
        assert!(!status.upstream_gone);
    }

    #[test]
    fn test_parse_status_upstream_gone() {
        let status = RepoStatus::parse("# branch.head old\n# branch.upstream origin/old\n");
        assert!(status.upstream_gone);
    }
}
