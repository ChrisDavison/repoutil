use anyhow::{anyhow, Result};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use shellexpand::tilde;

mod git;

#[derive(Debug, StructOpt)]
#[structopt(name = "repoutil", about = "Operations on multiple git repos")]
struct Opts {
    #[structopt(subcommand)]
    command: OptCommand,
    /// Use JSON rather than plaintext output
    #[structopt(long, short)]
    json: bool,
}

#[derive(Debug, StructOpt, PartialEq)]
enum OptCommand {
    /// Push commits
    #[structopt(alias = "p")]
    Push,
    // /// Fetch commits and tags
    #[structopt(alias = "f")]
    Fetch,
    /// Show short status
    #[structopt(alias = "s")]
    Stat,
    /// List tracked repos
    #[structopt(alias = "l")]
    List,
    /// List repos with local changes
    #[structopt(alias = "u")]
    Unclean,
    /// List short status of all branches
    #[structopt(alias = "bs")]
    Branchstat,
    /// List all branches
    #[structopt(alias = "b")]
    Branches,
    /// List all untracked folders
    #[structopt(alias = "un")]
    Untracked,
}

fn common_substring<T: ToString>(ss: &[T]) -> String {
    if ss.len() == 1 {
        return String::new();
    }
    let mut idx = 0;
    let charlists = ss
        .iter()
        .map(|x| x.to_string().chars().collect())
        .collect::<Vec<Vec<char>>>();
    if charlists.is_empty() {
        return "".to_string();
    }
    let first_charlist = &charlists[0];
    loop {
        let first = first_charlist.get(idx);
        if !charlists.iter().all(|w| w.get(idx) == first) {
            let first = first_charlist.get(idx);
            if !charlists.iter().all(|w| w.get(idx) == first) {
                break;
            }
        }
        idx += 1;
    }
    ss[0].to_string().chars().take(idx).collect()
}


fn main() {
    let opts = Opts::from_args();

    let json = opts.json;

    let cmd = match opts.command {
        OptCommand::Push => git::push,
        OptCommand::Fetch => git::fetch,
        OptCommand::Stat => git::stat,
        OptCommand::List => git::list,
        OptCommand::Unclean => git::needs_attention,
        OptCommand::Branchstat => git::branchstat,
        OptCommand::Branches => git::branches,
        OptCommand::Untracked => git::untracked,
    };

    let (includes, excludes) = get_repos_from_config().expect("Couldn't get repos");
    let repos = if opts.command == OptCommand::Untracked { excludes } else { includes };

    let common = common_substring(
        &repos
            .iter()
            .map(|x| format!("{}", x.display()))
            .collect::<Vec<String>>(),
    );
    let outs: Vec<_> = repos
        .par_iter()
        .progress_count(repos.len() as u64)
        .filter_map(|repo| match cmd(repo, json) {
            Ok(Some(thing)) => {
                Some(thing.replace(&common, ""))
            }
            Ok(_) => None,
            Err(e) => {
                eprintln!("ERR `{}`: {}", repo.display(), e);
                None
            },
        })
        .collect();
    if json {
        println!("{{\"items\": [{}]}}", outs.join(","));
    } else {
        println!("{}", outs.join("\n"));
    }
}

fn get_dirs_from_config() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let repoutil_config = tilde("~/.repoutilrc").to_string();
    let p = std::path::Path::new(&repoutil_config);

    if !p.exists() {
        return Err(anyhow!("No ~/.repoutilrc"));
    }

    let mut includes = Vec::new();
    let mut excludes = Vec::new();
    for line in std::fs::read_to_string(p)?.lines() {
        if let Some(stripped) = line.strip_prefix('!') {
            let path = PathBuf::from(tilde(&stripped).to_string());
            // Strip 'exclusion-marking' ! from start of path, and add to excludes list
            excludes.push(path);
        } else {
            let path = PathBuf::from(tilde(&line).to_string());
            if !excludes.contains(&path) {
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
        .filter(|d| git::is_git_repo(d))
        .collect();
    repos.sort();
    Ok(repos)
}

fn get_repos_from_config() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let (inc, exc) = get_dirs_from_config()?;
    let mut excludes = Vec::with_capacity(exc.len());
    for dir in exc {
        if git::is_git_repo(&dir) {
            excludes.push(dir);
        }
    }

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
    let includes = includes.iter().filter(|x| !excludes.contains(x)).cloned().collect();
    Ok((includes, excludes))
}

#[cfg(test)]
mod tests {
    use super::common_substring;

    #[test]
    fn test_common_substring() {
        assert_eq!(common_substring(&["aaa", "aab", "aac"]), "aa");
        assert_eq!(common_substring::<&str>(&[]), "");
        assert_eq!(common_substring(&["Something"]), "");
        assert_eq!(
            common_substring(&[
                "/home/cdavison/code",
                "/home/cdavison/code/recipes",
                "/home/cdavison/strathclyde"
            ]),
            "/home/cdavison/"
        );
    }
}

