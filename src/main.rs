use anyhow::{anyhow, Result};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use rayon::prelude::*;

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

#[derive(Debug, StructOpt)]
enum OptCommand {
    /// Push commits
    #[structopt(alias = "p")]
    Push,
    /// Fetch commits and tags
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
}

fn common_substring<T: ToString>(ss: &[T]) -> String {
    let mut idx = 0;
    loop {
        if !index_is_common(ss, idx) {
            break;
        }
        idx += 1;
    }
    ss[0].to_string().chars().take(idx).collect()
}

fn index_is_common<T: ToString>(ss: &[T], idx: usize) -> bool {
    let first = ss[0].to_string().chars().nth(idx).unwrap();
    ss.iter().all(|w| w.to_string().chars().nth(idx).unwrap() == first)
}

#[cfg(test)]
mod tests {
    use super::common_substring;

    #[test]
    fn test_common_substring() {
        assert_eq!(common_substring(&["aaa", "aab", "aac"]), "aa");
        assert_eq!(common_substring(&["/home/cdavison/code", "/home/cdavison/code/recipes", "/home/cdavison/strathclyde"]), "/home/cdavison/");
    }
}


type Command = fn(&Path, bool) -> Result<Option<String>>;

fn parse_args() -> (Command, bool) {
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
    };
    (cmd, json)
}

fn main() {
    let (cmd, json) = parse_args();
    
    let all_repos = get_repos_from_config().expect("Couldn't get repos");

    let mut messages: Vec<_> = all_repos.par_iter().map(|repo| {
    match cmd(repo, json) {
            Ok(Some(out)) => out,
            Err(e) => format!("ERR Repo {}: {}", repo.display(), e),
            _ => String::new(),
        }
    }).collect(); 

    messages.sort();
    let messages = messages.iter().filter(|msg| !msg.is_empty());
    if json {
        println!(
            "{{\"items\": [{}]}}",
            messages
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );
    } else {
        let messages = messages
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
        let common = common_substring(&messages);
        dbg!(&common);
        for msg in messages {
            println!("{}", msg.replace(&common, ""))
        }
    }
}

fn get_dirs_from_config() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let repoutil_config = tilde("~/.repoutilrc").to_string();
    let p = std::path::Path::new(&repoutil_config);

    if !p.exists() {
        return Err(anyhow!("No ~/.repoutilrc, or passed dirs"));
    } 

    let mut includes = Vec::new();
    let mut excludes = Vec::new();
    for line in std::fs::read_to_string(p)?.lines() {
        if line.starts_with('!') {
            // Strip 'exclusion-marking' ! from start of path, and add to excludes list
            excludes.push(PathBuf::from(tilde(&line[1..]).to_string()));
        } else {
            includes.push(PathBuf::from(tilde(&line).to_string()));
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

fn get_repos_from_config() -> Result<Vec<PathBuf>> {
    let (inc, exc) = get_dirs_from_config()?;
    let mut all_repos = Vec::new();
    for dir in inc {
        if git::is_git_repo(&dir) {
            all_repos.push(dir);
        } else {
            let repos = match get_repos_from_dir(&dir) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Couldn't get repos from '{:?}': '{}'\n", dir, e);
                    continue;
                }
            };
            all_repos.extend(repos.iter().filter(|r| !exc.contains(r)).cloned());
        }
    }
    all_repos.sort();
    all_repos.dedup();
    Ok(all_repos)
}
