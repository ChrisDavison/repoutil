use anyhow::{anyhow, Result};
use rayon::prelude::*;
use std::path::Path;
use std::process::Command;

pub fn is_git_repo(p: &Path) -> bool {
    let mut p = p.to_path_buf();
    p.push(".git");
    p.exists()
}

// Run a git command and return the lines of the output
fn command_output(dir: &Path, args: &[&str]) -> Result<Vec<String>> {
    let out = Command::new("git").current_dir(dir).args(args).output()?;
    Ok(std::str::from_utf8(&out.stdout)?
        .lines()
        .map(|x| x.to_string())
        .collect())
}

// Fetch all branches of a git repo
pub fn fetch(p: &Path) -> Result<Option<String>> {
    command_output(p, &["fetch", "--all", "--tags", "--prune"])?;
    Ok(None)
}

// Get the short status (ahead, behind, and modified files) of a repo
pub fn stat(p: &Path) -> Result<Option<String>> {
    let out_lines = command_output(p, &["status", "-s", "-b"])?;
    if out_lines.is_empty() {
        Ok(None)
    } else if out_lines[0].ends_with(']') {
        // We have an 'ahead', 'behind' or similar, so free to return the status early
        Ok(Some(format!("{}\n{}\n", p.display(), out_lines.join("\n"))))
    } else {
        // We aren't ahead or behind etc, but may have local uncommitted changes
        let status: Vec<String> = out_lines
            .par_iter()
            .skip(1)
            .map(|x| x.to_string())
            .collect();
        if status.is_empty() {
            Ok(None)
        } else {
            Ok(Some(format!("{}\n{}\n", p.display(), status.join("\n"))))
        }
    }
}

fn ahead_behind(p: &Path) -> Result<Option<String>> {
    let response: String = command_output(
        p,
        &[
            "for-each-ref",
            "--format='%(refname:short) %(upstream:track)'",
            "refs/heads",
        ],
    )?
    .par_iter()
    .map(|x| x.trim_matches('\'').trim())
    .filter(|x| {
        let splits: Vec<&str> = x.split(' ').collect();
        splits.get(1).is_some()
    })
    .collect();
    if !response.is_empty() {
        Ok(Some(response))
    } else {
        Ok(None)
    }
}

fn modified(p: &Path) -> Result<Option<String>> {
    let modified = command_output(p, &["diff", "--shortstat"])?.join("\n");
    if modified.contains("changed") {
        let num = modified.trim_start().split(' ').collect::<Vec<&str>>()[0];
        Ok(Some(format!("{}±", num)))
    } else {
        Ok(None)
    }
}

fn status(p: &Path) -> Result<Option<String>> {
    let response = command_output(p, &["diff", "--stat", "--cached"])?;
    if !response.is_empty() {
        Ok(Some(format!("Staged {}", response.len())))
    } else {
        Ok(None)
    }
}

fn untracked(p: &Path) -> Result<Option<String>> {
    let untracked = command_output(p, &["ls-files", "--others", "--exclude-standard"])?;
    if !untracked.is_empty() {
        Ok(Some(format!("{}?", untracked.len())))
    } else {
        Ok(None)
    }
}

pub fn branches(p: &Path) -> Result<Option<String>> {
    let branches: String = command_output(p, &["branch"])?
        .par_iter()
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
    Ok(Some(format!("{:40}\t{}", dirstr, branches)))
}

pub fn branchstat(p: &Path) -> Result<Option<String>> {
    let outputs = vec![ahead_behind(p)?, modified(p)?, status(p)?, untracked(p)?]
        .par_iter()
        .filter(|&x| x.is_some())
        .map(|x| x.as_ref().unwrap().as_str())
        .collect::<Vec<&str>>()
        .join(", ");

    if outputs.is_empty() {
        Ok(None)
    } else {
        let out = format!(
            "{:20} | {}",
            p.file_name().unwrap().to_string_lossy(),
            outputs
        );
        Ok(Some(out))
    }
}

// Get the name of any repo with local or remote changes
pub fn needs_attention(p: &Path) -> Result<Option<String>> {
    match stat(p) {
        Ok(Some(_)) => Ok(Some(p.display().to_string())),
        _ => Ok(None),
    }
}

// List each repo found
pub fn list(p: &Path) -> Result<Option<String>> {
    Ok(Some(p.display().to_string()))
}
