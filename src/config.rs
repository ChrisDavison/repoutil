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

fn get_dirs_from_config() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let p = config_path()?;

    if !p.exists() {
        return Err(anyhow!("No ~/.repoutilrc"));
    }

    let mut includes = Vec::new();
    let mut excludes = Vec::new();
    let mut exclude_set: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    for line in std::fs::read_to_string(p)?.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(stripped) = line.strip_prefix('!') {
            let path = expand_home(stripped)?;
            // Strip 'exclusion-marking' ! from start of path, and add to excludes list
            exclude_set.insert(path.clone());
            excludes.push(path);
        } else {
            let path = expand_home(line)?;
            // Only skip if the exclude appeared earlier in the config
            if !exclude_set.contains(&path) {
                includes.push(path);
            }
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

pub fn get_repos_from_config() -> Result<Vec<PathBuf>> {
    let (inc, exc) = get_dirs_from_config()?;
    let excludes: Vec<_> = exc.into_iter().filter(|dir| git::is_repo(dir)).collect();

    let mut includes = Vec::with_capacity(inc.len());
    for dir in inc {
        if git::is_repo(&dir) {
            includes.push(dir);
        } else if let Ok(repos) = get_repos_from_dir(&dir) {
            includes.extend(repos);
        }
    }
    includes.sort();
    includes.dedup();
    let exclude_set: std::collections::HashSet<_> = excludes.iter().collect();
    includes.retain(|x| !exclude_set.contains(x));
    Ok(includes)
}
