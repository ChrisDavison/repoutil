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
        .args(["fetch", "--all", "--quiet"])
        .status()?;
    if !status.success() {
        return Err(anyhow!("git fetch failed for {}", p.display()));
    }
    branchstat(p, fmt)
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
    let s = format!(
        "{:30}\t{}",
        remove_common_ancestor(p, fmt.common_prefix),
        branches
    );
    Ok(Some(s))
}

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
    let mut n_added = 0;
    let mut n_deleted = 0;
    for line in response {
        let trimmed = line.trim_start();
        let status_char = if trimmed.starts_with('\u{1b}') {
            trimmed.chars().nth(5)
        } else {
            trimmed.chars().next()
        };
        match status_char {
            Some('M') => n_modified += 1,
            Some('?') => n_untracked += 1,
            Some('A') => n_added += 1,
            Some('D') => n_deleted += 1,
            _ => {}
        }
    }

    if n_added > 0 {
        parts.push(apply_color(
            format!("{}+", n_added),
            fmt.no_colour,
            &[GREEN],
        ));
    }

    if n_deleted > 0 {
        parts.push(apply_color(
            format!("{}-", n_deleted),
            fmt.no_colour,
            &[RED],
        ));
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
                &[PURPLE],
            ));
        }
    }

    if parts.is_empty() {
        return Ok(None);
    }
    let joined = parts.join(&apply_color(" · ", fmt.no_colour, &[BOLD, BLACK]));

    let path = remove_common_ancestor(p, fmt.common_prefix);
    let s = format!(
        "{:40} {}",
        apply_color(path, fmt.no_colour, &[BOLD, RED]),
        joined
    );
    Ok(Some(s))
}
