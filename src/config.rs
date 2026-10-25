use anyhow::{anyhow, Result};
use std::fs::read_dir;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::git;

/// Resolve `s` relative to $HOME, expanding a leading `~/`
pub fn expand_home(s: &str) -> Result<PathBuf> {
    let mut home = PathBuf::from(std::env::var("HOME")?);
    if let Some(rest) = s.strip_prefix("~/") {
        let p = PathBuf::from(rest);
        for cmp in p.components() {
            home.push(cmp);
        }
        Ok(home)
    } else {
        Ok(home.join(s))
    }
}

fn config_path() -> Result<PathBuf> {
    expand_home(".repoutilrc")
}

/// Add the current directory to ~/.repoutilrc
pub fn add() -> Result<()> {
    let config_filename = config_path()?;
    let curdir = std::env::current_dir()?;
    if git::is_repo(&curdir) {
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

/// Read the include and exclude (`!`-prefixed) paths from ~/.repoutilrc
fn get_dirs_from_config() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let p = config_path()?;

    if !p.exists() {
        return Err(anyhow!("No ~/.repoutilrc"));
    }

    let mut includes = Vec::new();
    let mut excludes = Vec::new();
    for line in std::fs::read_to_string(p)?.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(stripped) = line.strip_prefix('!') {
            excludes.push(expand_home(stripped)?);
        } else {
            includes.push(expand_home(line)?);
        }
    }
    Ok((includes, excludes))
}

// Get every repo from subdirs of `dir`
fn get_repos_from_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut repos: Vec<PathBuf> = read_dir(dir)?
        .filter_map(|d| d.ok())
        .map(|d| d.path())
        .filter(|d| git::is_repo(d))
        .collect();
    repos.sort();
    Ok(repos)
}

/// Every repo in ~/.repoutilrc, skipping any that are at or under an excluded path
pub fn get_repos_from_config() -> Result<Vec<PathBuf>> {
    let (dirs, excludes) = get_dirs_from_config()?;

    let mut repos = Vec::with_capacity(dirs.len());
    for dir in dirs {
        if git::is_repo(&dir) {
            repos.push(dir);
        } else if let Ok(found) = get_repos_from_dir(&dir) {
            repos.extend(found);
        }
    }
    repos.retain(|repo| !excludes.iter().any(|exc| repo.starts_with(exc)));
    repos.sort();
    repos.dedup();
    Ok(repos)
}
