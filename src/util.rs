#![allow(dead_code)]
use anyhow::{anyhow, Result};
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use crate::vcs;
use serde_json::json;

pub fn remove_common_ancestor(repo: &Path, common: Option<&PathBuf>) -> String {
    if let Some(prefix) = common {
        repo.strip_prefix(prefix).unwrap().display().to_string()
    } else {
        repo.display().to_string()
    }
}

pub fn format_json(
    title: &Path,
    subtitle: Option<&str>,
    path_as_arg: bool,
    common: Option<&PathBuf>,
) -> String {
    let arg = if path_as_arg {
        title.display().to_string()
    } else {
        String::new()
    };
    let title = remove_common_ancestor(title, common);
    let obj = if let Some(sub) = subtitle {
        json!({
            "title": title,
            "arg": arg,
            "subtitle": sub,
        })
    } else {
        json!({
            "title": title,
            "arg": arg,
        })
    };
    obj.to_string()
}

pub fn homedir(s: &str) -> Result<PathBuf> {
    let mut home = PathBuf::from(std::env::var("HOME")?);
    if let Some(rest) = s.strip_prefix("~") {
        let p = PathBuf::from(rest);
        for cmp in p.components() {
            home.push(cmp);
        }
        Ok(home)
    } else {
        home.push(s);
        Ok(home)
    }
}

pub fn common_ancestor(ss: &[PathBuf]) -> PathBuf {
    if ss.is_empty() || ss.len() == 1 {
        return PathBuf::new();
    }
    let mut iterators: Vec<_> = ss.iter().map(|p| p.components()).collect();
    let mut prefix: Vec<std::path::Component> = Vec::new();
    'outer: loop {
        let mut next: Option<std::path::Component> = None;
        for it in iterators.iter_mut() {
            if let Some(c) = it.next() {
                if let Some(prev) = next {
                    if prev != c {
                        break 'outer;
                    }
                } else {
                    next = Some(c);
                }
            } else {
                break 'outer;
            }
        }
        if let Some(c) = next {
            prefix.push(c);
        } else {
            break;
        }
    }
    prefix.iter().collect()
}

pub fn get_dirs_from_config() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let p = homedir(".repoutilrc")?;
    // let p = std::path::Path::new(&repoutil_config);

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
            let path = homedir(stripped)?;
            // Strip 'exclusion-marking' ! from start of path, and add to excludes list
            excludes.push(path);
        } else {
            let path = homedir(line)?;
            if !excludes.contains(&path) {
                includes.push(path);
            }
        }
    }
    Ok((includes, excludes))
}

// Get every repo from subdirs of `dir`
pub fn get_repos_from_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut repos: Vec<PathBuf> = read_dir(dir)?
        .filter_map(|d| d.ok())
        .map(|d| d.path())
        .filter(|d| vcs::git::is_repo(d))
        .collect();
    repos.sort();
    Ok(repos)
}

pub fn get_repos_from_config() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let (inc, exc) = get_dirs_from_config()?;
    let excludes: Vec<_> = exc
        .iter()
        .filter(|dir| vcs::git::is_repo(dir))
        .cloned()
        .collect();

    let mut includes = Vec::with_capacity(inc.len());
    for dir in inc {
        if vcs::git::is_repo(&dir) {
            includes.push(dir);
        } else if let Ok(repos) = get_repos_from_dir(&dir) {
            includes.extend(repos.iter().map(|p| p.to_path_buf()));
        }
    }
    includes.sort();
    includes.dedup();
    let includes = includes
        .iter()
        .filter(|x| !excludes.contains(x))
        .cloned()
        .collect();
    Ok((includes, excludes))
}

#[cfg(test)]
mod tests {
    use super::common_ancestor;

    #[test]
    fn test_common_ancestor() {
        assert_eq!(
            common_ancestor(&[
                std::path::PathBuf::from("/home/cdavison/code"),
                std::path::PathBuf::from("/home/cdavison/code/recipes"),
                std::path::PathBuf::from("/home/cdavison/strathclyde")
            ]),
            std::path::PathBuf::from("/home/cdavison/")
        );
    }
}
