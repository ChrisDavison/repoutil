use anyhow::{anyhow, Result};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::thread;
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
    };

    let (inc, exc) = match get_dirs_from_config() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let mut all_repos = Vec::new();
    for dir in inc {
        if git::is_git_repo(&dir) {
            all_repos.push(dir);
        } else {
            let repos = match get_repos(&dir) {
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

    let mut handles = Vec::new();
    for repo in all_repos {
        // Spawn a thread for each repo
        // and run the chosen command.
        // The handle must 'move' to take ownership of `cmd`
        let handle = thread::spawn(move || match cmd(&repo, json) {
            Ok(Some(out)) => out,
            Err(e) => format!("ERR Repo {}: {}", repo.display(), e),
            _ => String::new(),
        });
        handles.push(handle);
    }

    let mut messages = Vec::new();
    for h in handles {
        match h.join() {
            Ok(msg) => messages.push(msg),
            Err(e) => eprintln!("A child git command panic'd: {:?}", e),
        }
    }
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
        for msg in messages {
            println!("{}", msg)
        }
    }
}

fn get_dirs_from_config() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let repoutil_config = tilde("~/.repoutilrc").to_string();
    let p = std::path::Path::new(&repoutil_config);
    if !p.exists() {
        Err(anyhow!("No ~/.repoutilrc, or passed dirs"))
    } else {
        let contents = std::fs::read_to_string(p)?;
        let (inc, exc): (Vec<_>, Vec<_>) = contents.lines().partition(|p| !p.starts_with('!'));
        Ok((
            inc.iter()
                .map(|x| PathBuf::from(tilde(x).to_string()))
                .collect(),
            exc.iter()
                .map(|x| PathBuf::from(tilde(&x[1..]).to_string()))
                .collect(),
        ))
    }
}

// Get every repo from subdirs of `dir`
fn get_repos(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut repos: Vec<PathBuf> = read_dir(dir)?
        .filter_map(|d| d.ok())
        .map(|d| d.path())
        .filter(|d| git::is_git_repo(d))
        .collect();
    repos.sort();
    Ok(repos)
}
