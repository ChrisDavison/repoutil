use super::Result;

pub fn is_git_repo(mut p: std::path::PathBuf) -> bool {
    p.push(".git");
    p.exists()
}

fn command_output(dir: std::path::PathBuf, args: &[&str]) -> String {
    let err_msg = format!("couldn't run command `git {:?}` on `{:?}`", args, dir);
    let out = std::process::Command::new("git")
        .current_dir(dir.clone())
        .args(args)
        .output()
        .expect(&err_msg);
    std::str::from_utf8(&out.stdout)
        .expect("couldn't convert stdout")
        .to_string()
}

pub fn fetch(p: std::path::PathBuf) -> Option<String> {
    let out = command_output(p, &["fetch", "--all"]);
    let status: String = out.lines().skip(1).collect();
    if status.is_empty() {
        None
    } else {
        Some(status)
    }
}

pub fn stat(p: std::path::PathBuf) -> Option<String> {
    let out = command_output(p, &["status", "-s", "-b"]);
    let lines: Vec<String> = out.lines().map(|x| x.to_string()).collect();
    if lines[0].ends_with(']') {
        return Some(lines.join("\n"));
    }
    let status: Vec<String> = lines.iter().skip(1).map(|x| x.to_string()).collect();
    if status.is_empty() {
        None
    } else {
        Some(status.join("\n"))
    }
}

pub fn get_repos(dir: &str) -> Result<Vec<::std::path::PathBuf>> {
    let mut repos = Vec::new();
    let repos_for_dir: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|d| d.ok())
        .map(|d| d.path())
        .filter(|d| d.is_dir() && is_git_repo(d.to_path_buf()))
        .collect();
    repos.extend(repos_for_dir.iter().cloned());
    Ok(repos)
}
