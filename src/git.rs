use super::Result;
use std::fs::read_dir;
use std::path::PathBuf;
use std::process::Command;

pub fn is_git_repo(mut p: PathBuf) -> bool {
    p.push(".git");
    p.exists()
}

fn command_output(dir: &PathBuf, args: &[&str]) -> Result<Vec<String>> {
    let out = Command::new("git")
        .current_dir(dir.clone())
        .args(args)
        .output()
        .map_err(|_| format!("couldn't run command `git {:?}` on `{:?}`", args, dir))?;
    Ok(std::str::from_utf8(&out.stdout)?
        .to_string()
        .lines()
        .map(|x| x.to_string())
        .collect())
}

pub fn fetch(p: &PathBuf) -> Result<Option<String>> {
    let out_lines = command_output(p, &["fetch", "--all"])?;
    let status: String = out_lines[1..].iter().cloned().collect();
    if status.is_empty() {
        Ok(None)
    } else {
        Ok(Some(status))
    }
}

pub fn stat(p: &PathBuf) -> Result<Option<String>> {
    let out_lines = command_output(p, &["status", "-s", "-b"])?;
    // let lines: Vec<String> = out.lines().map(|x| x.to_string()).collect();
    if out_lines[0].ends_with(']') {
        return Ok(Some(out_lines.join("\n")));
    }
    let status: Vec<String> = out_lines.iter().skip(1).map(|x| x.to_string()).collect();
    if status.is_empty() {
        Ok(None)
    } else {
        Ok(Some(status.join("\n")))
    }
}

pub fn get_repos(dir: &str) -> Result<Vec<PathBuf>> {
    let mut repos = Vec::new();
    let repos_for_dir: Vec<_> = read_dir(dir)?
        .filter_map(|d| d.ok())
        .map(|d| d.path())
        .filter(|d| d.is_dir() && is_git_repo(d.to_path_buf()))
        .collect();
    repos.extend(repos_for_dir.iter().cloned());
    Ok(repos)
}
