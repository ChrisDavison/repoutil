use anyhow::{anyhow, Result};
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use crate::git;

pub fn homedir(s: &str) -> Result<PathBuf> {
    let mut home = PathBuf::from(std::env::var("HOME")?);
    if s.contains("~") {
        let p = PathBuf::from(s);
        for cmp in p.components().skip(1) {
            home.push(cmp);
        }
        Ok(home)
    } else {
        home.push(s);
        Ok(home)
    }
}

pub fn common_ancestor(ss: &[PathBuf]) -> PathBuf {
    if ss.len() == 1 {
        return PathBuf::new();
    }
    let mut idx = 0;
    let components = ss
        .iter()
        .map(|x| x.components().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let entry0 = &components[0];
    loop {
        let first = entry0.get(idx);
        if !components.iter().all(|w| w.get(idx) == first) {
            let first = entry0.get(idx);
            if !components.iter().all(|w| w.get(idx) == first) {
                break;
            }
        }
        idx += 1;
    }
    entry0.iter().take(idx).collect()
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
        .filter(|d| git::is_git_repo(d))
        .collect();
    repos.sort();
    Ok(repos)
}

pub fn get_repos_from_config() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let (inc, exc) = get_dirs_from_config()?;
    let excludes: Vec<_> = exc
        .iter()
        .filter(|dir| git::is_git_repo(dir))
        .cloned()
        .collect();

    let mut includes = Vec::with_capacity(inc.len());
    for dir in inc {
        if git::is_git_repo(&dir) {
            includes.push(dir);
        } else {
            let repos = match get_repos_from_dir(&dir) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Couldn't get repos from '{:?}': '{}'\n", dir, e);
                    continue;
                }
            };
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
