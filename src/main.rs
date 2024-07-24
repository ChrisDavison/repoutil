use anyhow::{anyhow, Result};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use shellexpand::tilde;

mod git;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum GitPath {
    Keep(PathBuf),
    Ignore(PathBuf),
}

impl GitPath {
    pub fn to_path_buf(&self) -> PathBuf {
        match self {
            GitPath::Keep(p) => p.to_path_buf(),
            GitPath::Ignore(p) => p.to_path_buf(),
        }
    }

    pub fn keep(&self) -> Option<PathBuf> {
        match self {
            GitPath::Keep(p) => Some(p.to_path_buf()),
            GitPath::Ignore(_) => None,
        }
    }

    pub fn ignore(&self) -> Option<PathBuf> {
        match self {
            GitPath::Ignore(p) => Some(p.to_path_buf()),
            GitPath::Keep(_) => None,
        }
    }
}

impl std::fmt::Display for GitPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            GitPath::Keep(p) => write!(f, "{}", p.display()),
            GitPath::Ignore(p) => write!(f, "! {}", p.display()),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "repoutil", about = "Operations on multiple git repos")]
struct Opts {
    #[structopt(subcommand)]
    command: OptCommand,
    /// Use JSON rather than plaintext output
    #[structopt(long, short)]
    json: bool,
}

#[derive(Debug, StructOpt)]
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

#[cfg(test)]
mod tests {
    use super::common_substring;

    #[test]
    fn test_common_substring() {
        assert_eq!(common_substring(&["aaa", "aab", "aac"]), "aa");
        assert_eq!(common_substring::<&str>(&[]), "");
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

    let all_repos = get_repos_from_config().expect("Couldn't get repos");

    let common = common_substring(
        &all_repos
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>(),
    );
    let mut outs = Vec::new();
    let out: Vec<_> = all_repos
        .par_iter()
        .progress_count(all_repos.len() as u64)
        .map(|repo| cmd(repo, json))
        .collect();
    for output in out {
        match output {
            Ok(Some(thing)) => {
                let thing = thing.replace(&common, "");
                if json {
                    outs.push(thing);
                } else {
                    println!("{}", thing)
                }
            }
            Ok(_) => (),
            Err(e) => eprintln!("ERR: {}", e),
        }
    }
    if json {
        println!("{{\"items\": [{}]}}", outs.join(","));
    }
}

fn get_dirs_from_config() -> Result<(Vec<GitPath>, Vec<GitPath>)> {
    let repoutil_config = tilde("~/.repoutilrc").to_string();
    let p = std::path::Path::new(&repoutil_config);

    if !p.exists() {
        return Err(anyhow!("No ~/.repoutilrc, or passed dirs"));
    }

    let mut includes = Vec::new();
    let mut excludes = Vec::new();
    for line in std::fs::read_to_string(p)?.lines() {
        if let Some(stripped) = line.strip_prefix('!') {
            // Strip 'exclusion-marking' ! from start of path, and add to excludes list
            excludes.push(GitPath::Ignore(PathBuf::from(tilde(stripped).to_string())));
        } else {
            includes.push(GitPath::Keep(PathBuf::from(tilde(&line).to_string())));
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

fn get_repos_from_config() -> Result<Vec<GitPath>> {
    let (inc, exc) = get_dirs_from_config()?;
    let mut all_repos = Vec::new();
    for dir in inc {
        let dir = dir.keep().unwrap();
        if git::is_git_repo(&dir) {
            all_repos.push(GitPath::Keep(dir));
        } else {
            let repos = match get_repos_from_dir(&dir) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Couldn't get repos from '{:?}': '{}'\n", dir, e);
                    continue;
                }
            };
            all_repos.extend(repos.iter().map(|p| GitPath::Keep(p.to_path_buf())));
        }
    }
    for dir in exc {
        let dir = dir.ignore().unwrap();
        if git::is_git_repo(&dir) {
            all_repos.push(GitPath::Ignore(dir));
        } else {
            let repos = match get_repos_from_dir(&dir) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Couldn't get repos from '{:?}': '{}'\n", dir, e);
                    continue;
                }
            };
            all_repos.extend(repos.iter().map(|p| GitPath::Ignore(p.to_path_buf())));
        }
    }
    all_repos.sort();
    all_repos.dedup();
    Ok(all_repos)
}
