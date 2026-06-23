use crate::FormatOpts;

use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::ansi_escape::*;
use crate::util::{path_output, remove_common_ancestor};

const SYM_AHEAD: &str = "↑";
const SYM_BEHIND: &str = "↓";
const SYM_MODIFIED: &str = "±";
#[cfg(feature = "jj")]
const SYM_JJ_MUTABLE: &str = "*";

pub fn add() -> Result<()> {
    let config_filename = crate::util::homedir(".repoutilrc")?;
    let curdir = std::env::current_dir()?;
    if is_repo(&curdir) {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(config_filename)?;

        writeln!(file, "{}", curdir.to_string_lossy())?;
        Ok(())
    } else {
        Err(anyhow!("Don't appear to be in the root of a git repo."))
    }
}

/// List each repo found
pub fn list(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    Ok(Some(path_output(p, fmt.common_prefix)))
}

pub fn is_repo(p: &Path) -> bool {
    let mut p = p.to_path_buf();
    p.push(".git");
    p.exists()
}

/// Fetch all branches of a git repo
pub fn fetch(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    let status = Command::new("git")
        .current_dir(p)
        .args(["fetch", "--all", "--tags"])
        .status()?;
    if !status.success() {
        return Err(anyhow!("git fetch failed for {}", p.display()));
    }
    branchstat(p, fmt)
}

// /// Get the name of any repo with local or remote changes
// pub fn needs_attention(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
//     match stat(p, &fmt.clone()) {
//         Ok(Some(output)) => {
//             if output.is_empty() {
//                 Ok(None)
//             } else {
//                 Ok(Some(path_output(p, fmt.common_prefix)))
//             }
//         }
//         Ok(None) => Ok(None),
//         Err(e) => Err(e),
//     }
// }

// /// List each untracked repo found
// pub fn untracked(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
//     Ok(Some(path_output(p, fmt.common_prefix)))
// }

// /// Get the short status (ahead, behind, and modified files) of a repo
// pub fn stat(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
//     let (branch, status_vec) = parse_git_status(p)?;
//
//     let formatted_branch = apply_color(branch.clone(), fmt.no_colour, &[BLUE]);
//     let commit_hash = get_short_commit_hash(p)?;
//     let formatted_commit = apply_color(commit_hash.clone(), fmt.no_colour, &[YELLOW]);
//
//     let file_list = format_file_list(&status_vec, fmt);
//     if file_list.is_empty() {
//         return Ok(None);
//     }
//
//     let s = format!(
//         "{} on {formatted_branch} at {formatted_commit}\n{}\n",
//         apply_color(
//             remove_common_ancestor(p, fmt.common_prefix),
//             fmt.no_colour,
//             &[PURPLE]
//         ),
//         file_list
//     );
//
//     Ok(Some(s))
// }

// /// Get a count of stashes
// pub fn stashcount(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
//     let stdout = Command::new("git")
//         .current_dir(p)
//         .args(["stash", "list"])
//         .output()?
//         .stdout;
//     let stashes: usize = std::str::from_utf8(&stdout)?.lines().count();
//     if stashes == 0 {
//         Ok(None)
//     } else {
//         let simple_path = remove_common_ancestor(p, fmt.common_prefix);
//         let s = format!(
//             "{:30}\t{}",
//             apply_color(simple_path, fmt.no_colour, &[RED]),
//             apply_color(stashes.to_string(), fmt.no_colour, &[GREEN]),
//         );
//         Ok(Some(s))
//     }
// }

// /// Get a list of branches for the given git path
// pub fn branches(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
//     let stdout = Command::new("git")
//         .current_dir(p)
//         .args(["branch"])
//         .output()?
//         .stdout;
//     let mut branches: Vec<&str> = std::str::from_utf8(&stdout)?.lines().collect();
//     branches.sort();
//     branches.reverse();
//     let branches: String = branches
//         .iter()
//         .map(|x| x.trim().to_string())
//         .collect::<Vec<_>>()
//         .join(", ");
//     let s = format!(
//         "{:30}\t{}",
//         remove_common_ancestor(p, fmt.common_prefix),
//         branches
//     );
//     Ok(Some(s))
// }

// /// Parse git status with robust null-separated porcelain format
// fn parse_git_status(p: &Path) -> Result<(String, Vec<(String, String)>)> {
//     let branch_stdout = Command::new("git")
//         .current_dir(p)
//         .args(["symbolic-ref", "--short", "HEAD"])
//         .output()?
//         .stdout;
//
//     let branch = if branch_stdout.is_empty() {
//         "HEAD".to_string()
//     } else {
//         std::str::from_utf8(&branch_stdout)?.trim().to_string()
//     };
//
//     let stdout = Command::new("git")
//         .current_dir(p)
//         .args(["status", "--porcelain=v1", "-z"])
//         .output()?
//         .stdout;
//
//     let mut result = Vec::new();
//     for entry in stdout.split(|&b| b == 0) {
//         if entry.is_empty() {
//             continue;
//         }
//         if let Ok(s) = std::str::from_utf8(entry) {
//             if s.len() > 3 {
//                 let status = s[..2].to_string();
//                 let filepath = s[3..].to_string();
//                 result.push((status, filepath));
//             }
//         }
//     }
//
//     Ok((branch, result))
// }

// /// Get short commit hash for current HEAD
// fn get_short_commit_hash(p: &Path) -> Result<String> {
//     let stdout = Command::new("git")
//         .current_dir(p)
//         .args(["rev-parse", "--short", "HEAD"])
//         .output()?
//         .stdout;
//     Ok(std::str::from_utf8(&stdout)?.trim().to_string())
// }

// /// Format the file list from status vector for display
// fn format_file_list(status_vec: &[(String, String)], fmt: &FormatOpts) -> String {
//     if status_vec.is_empty() {
//         return String::new();
//     }
//
//     status_vec
//         .iter()
//         .map(|(status, filepath)| {
//             let formatted_status = apply_color(status.clone(), fmt.no_colour, &[GREEN]);
//             format!("{} {}", formatted_status, filepath)
//         })
//         .collect::<Vec<_>>()
//         .join("\n")
// }

/// Get the status _of each branch_
pub fn branchstat(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    let stdout = Command::new("git")
        .current_dir(p)
        .args(["status", "--porcelain", "--ahead-behind", "-b"])
        .output()?
        .stdout;
    let mut response = std::str::from_utf8(&stdout)?.lines();

    let branch_line = response.next();

    let mut parts = Vec::new();
    if let Some(response) = branch_line.filter(|x| x.contains('[')) {
        let start = response.find('[').unwrap();
        let end = response.find(']').unwrap();
        let s = response[start + 1..end]
            .replace("ahead ", SYM_AHEAD)
            .replace("behind ", SYM_BEHIND);
        parts.push(apply_color(s, fmt.no_colour, &[BLUE]));
    }

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
        parts.push(apply_color(
            format!("{}{SYM_MODIFIED}", n_modified),
            fmt.no_colour,
            &[GREEN],
        ));
    }
    if n_untracked > 0 {
        parts.push(apply_color(
            format!("{}?", n_untracked),
            fmt.no_colour,
            &[YELLOW],
        ));
    }

    #[cfg(feature = "jj")]
    {
        let jj_mutable = {
            let stdout = Command::new("jj")
                .current_dir(p)
                .args(["list_mut"])
                .output()?
                .stdout;
            std::str::from_utf8(&stdout)?.lines().count()
        };
        if jj_mutable != 0 {
            parts.push(apply_color(
                format!("{}{SYM_JJ_MUTABLE}", jj_mutable),
                fmt.no_colour,
                &[YELLOW],
            ));
        }
    }

    let joined = parts.join(", ");
    if joined.is_empty() {
        return Ok(None);
    }

    let path = remove_common_ancestor(p, fmt.common_prefix);
    let s = format!(
        "{:40} {}",
        apply_color(path, fmt.no_colour, &[BOLD, RED]),
        joined
    );
    Ok(Some(s))
}
