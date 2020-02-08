use super::Result;
use std::fs::read_dir;
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
        .output()
        .map_err(|_| format!("counldn't run command `git {:?}` on `{:?}`", args, dir))?;
    Ok(std::str::from_utf8(&out.stdout)?
        .lines()
        .map(|x| x.to_string())
        .collect())
}

// Fetch all branches of a git repo
pub fn fetch(p: &PathBuf) -> Result<Option<String>> {
    let out_lines = command_output(p, &["fetch", "--all"])?;
    let status: String = out_lines[1..].iter().cloned().collect();
    if status.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format!("{}\n{}\n", p.display(), status)))
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
        Ok(Some(format!("{}", response)))
    } else {
        Ok(None)
    }
}

fn modified(p: &PathBuf) -> Result<Option<String>> {
    let modified = command_output(p, &["diff", "--shortstat"])?.join("\n");
    if modified.contains("changed") {
        let num = modified.trim_start().split(" ").collect::<Vec<&str>>()[0];
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

fn path_and_parent(p: &PathBuf) -> String {
    let parent = p.parent().unwrap().file_stem().unwrap().to_string_lossy();
    let dir = p.file_stem().unwrap().to_string_lossy();
    format!("{}/{}", parent, dir)
}

pub fn branches(p: &PathBuf) -> Result<Option<String>> {
    let branches: String = command_output(p, &["branch"])?
        .iter()
        .map(|x| x.trim())
        .filter(|x| x.starts_with("*")).
        map(|x| &x[2..])
        .collect();
    Ok(Some(format!("{:40}\t{}", path_and_parent(p), branches)))
}

pub fn branchstat(p: &PathBuf) -> Result<Option<String>> {
    // ahead-behind
    let outputs = vec![ahead_behind(p)?, modified(p)?, status(p)?, untracked(p)?]
        .iter()
        .filter(|&x| x.is_some())
        .map(|x| x.as_ref().unwrap().as_str())
        .collect::<Vec<&str>>()
        .join(", ");

    if outputs.is_empty() {
        Ok(None)
    } else {
        let out = format!("{}\n{}\n", p.display().to_string(), outputs);
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

// Get every repo from subdirs of `dir`
pub fn get_repos(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut repos: Vec<PathBuf> = read_dir(dir)?
        .filter_map(|d| d.ok())
        .map(|d| d.path())
        .filter(|d| is_git_repo(d))
        .collect();
    repos.sort();
    Ok(repos)
}
