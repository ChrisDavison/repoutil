use anyhow::*;

use std::path::PathBuf;
use std::process::Command;

pub fn is_git_repo(p: &PathBuf) -> bool {
    let mut p = p.clone();
    p.push(".git");
    p.exists()
}

// Run a git command and return the lines of the output
fn command_output(dir: &PathBuf, args: &[&str]) -> Result<Vec<String>> {
    let out = Command::new("git")
        .current_dir(dir.clone())
        .args(args)
        .output()?;
    Ok(std::str::from_utf8(&out.stdout)?
        .lines()
        .map(|x| x.to_string())
        .collect())
}

// Fetch all branches of a git repo
pub fn fetch(p: &PathBuf) -> Result<Option<String>> {
    let out_lines = command_output(p, &["fetch", "--all"])?;
    match out_lines.get(1..) {
        Some(lines) => Ok(Some(format!("{}\n{}", p.display(), lines.join("")))),
        None => Ok(None),
    }
}

// Get the short status (ahead, behind, and modified files) of a repo
pub fn stat(p: &PathBuf) -> Result<Option<String>> {
    let out_lines = command_output(p, &["status", "-s", "-b"])?;
    if out_lines.is_empty() {
        Ok(None)
    } else if out_lines[0].ends_with(']') {
        // We have an 'ahead', 'behind' or similar, so free to return the status early
        Ok(Some(format!("{}\n{}\n", p.display(), out_lines.join("\n"))))
    } else {
        // We aren't ahead or behind etc, but may have local uncommitted changes
        let status: Vec<String> = out_lines.iter().skip(1).map(|x| x.to_string()).collect();
        if status.is_empty() {
            Ok(None)
        } else {
            Ok(Some(format!("{}\n{}\n", p.display(), status.join("\n"))))
        }
    }
}

fn ahead_behind(p: &PathBuf) -> Result<Option<String>> {
    let response: String = command_output(
        p,
        &[
            "for-each-ref",
            "--format='%(refname:short) %(upstream:track)'",
            "refs/heads",
        ],
    )?
    .iter()
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

fn modified(p: &PathBuf) -> Result<Option<String>> {
    let modified = command_output(p, &["diff", "--shortstat"])?.join("\n");
    if modified.contains("changed") {
        let num = modified.trim_start().split(' ').collect::<Vec<&str>>()[0];
        Ok(Some(format!("Modified {}", num)))
    } else {
        Ok(None)
    }
}

fn status(p: &PathBuf) -> Result<Option<String>> {
    let response = command_output(p, &["diff", "--stat", "--cached"])?;
    if !response.is_empty() {
        Ok(Some(format!("Staged {}", response.len())))
    } else {
        Ok(None)
    }
}

fn untracked(p: &PathBuf) -> Result<Option<String>> {
    let untracked = command_output(p, &["ls-files", "--others", "--exclude-standard"])?;
    if !untracked.is_empty() {
        Ok(Some(format!("Untracked {}", untracked.len())))
    } else {
        Ok(None)
    }
}

pub fn branches(p: &PathBuf) -> Result<Option<String>> {
    let branches: String = command_output(p, &["branch"])?
        .iter()
        .map(|x| x.trim())
        .filter(|x| x.starts_with('*'))
        .map(|x| &x[2..])
        .collect();
    let parentpath = p.parent().context("No parent for dir")?;
    let parentname = parentpath
        .file_stem()
        .context("No stem for parent")?
        .to_string_lossy();
    let dirname = p.file_stem().context("No stem for dir")?.to_string_lossy();
    let dirstr = format!("{}/{}", parentname, dirname);
    Ok(Some(format!("{:40}\t{}", dirstr, branches)))
}

pub fn branchstat(p: &PathBuf) -> Result<Option<String>> {
    let outputs = vec![ahead_behind(p)?, modified(p)?, status(p)?, untracked(p)?]
        .iter()
        .filter(|&x| x.is_some())
        .map(|x| x.as_ref().unwrap().as_str())
        .collect::<Vec<&str>>()
        .join(", ");

    if outputs.is_empty() {
        Ok(None)
    } else {
        let out = format!("{:40} | {}", p.display().to_string(), outputs);
        Ok(Some(out))
    }
}

// Get the name of any repo with local or remote changes
pub fn needs_attention(p: &PathBuf) -> Result<Option<String>> {
    match stat(p) {
        Ok(Some(_)) => Ok(Some(p.display().to_string())),
        _ => Ok(None),
    }
}

// List each repo found
pub fn list(p: &PathBuf) -> Result<Option<String>> {
    Ok(Some(p.display().to_string()))
}
