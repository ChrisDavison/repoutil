use anyhow::*;
use std::fs::read_dir;
use std::path::PathBuf;
use std::thread;

use shellexpand::tilde;
use structopt::clap::AppSettings;
use structopt::StructOpt;

mod git;

#[derive(StructOpt, Debug)]
#[structopt(name = "repoutil", setting=AppSettings::InferSubcommands)]
struct Opts {
    #[structopt(subcommand)]
    cmd: Repoutil,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "manage multiple git repos")]
enum Repoutil {
    /// Show short status
    #[structopt(alias = "s")]
    Stat,
    /// Fetch upstream changes
    #[structopt(alias = "f")]
    Fetch,
    /// List repos that would be operated on
    #[structopt(alias = "l")]
    List,
    /// List repos with local changes
    #[structopt(alias = "u")]
    Unclean,
    /// List short status of all branches
    #[structopt(alias = "b")]
    Branchstat,
    /// List all branches
    Branches,
}

const USAGE: &str = "usage: repoutil stat|fetch|list|unclean|branchstat|branches";

fn main() {
    let opts = Opts::from_args();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("{}", USAGE);
        return;
    }
    let cmd = match opts.cmd {
        Repoutil::Fetch => git::fetch,
        Repoutil::Stat => git::stat,
        Repoutil::List => git::list,
        Repoutil::Unclean => git::needs_attention,
        Repoutil::Branchstat => git::branchstat,
        Repoutil::Branches => git::branches,
    };
    let dirs = match get_dirs_from_config() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("{}", USAGE);
            return;
        }
    };
    let mut all_repos = Vec::new();
    for dir in dirs {
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
            all_repos.extend(repos);
        }
    }
    all_repos.sort();
    all_repos.dedup();

    let mut handles = Vec::new();
    for repo in all_repos {
        // Spawn a thread for each repo
        // and run the chosen command.
        // The handle must 'move' to take ownership of `cmd`
        let handle = thread::spawn(move || match cmd(&repo) {
            Ok(Some(out)) => println!("{}", out.trim_end()),
            Err(e) => eprintln!("Repo {}: {}", repo.display(), e),
            _ => (),
        });
        handles.push(handle);
    }

    for h in handles {
        if let Err(e) = h.join() {
            eprintln!("A child git command panic'd: {:?}", e);
        }
    }
}

fn get_dirs_from_config() -> Result<Vec<PathBuf>> {
    let repoutil_config = tilde("~/.repoutilrc").to_string();
    let p = std::path::Path::new(&repoutil_config);
    if p.exists() {
        let contents = std::fs::read_to_string(p)?;
        Ok(contents
            .lines()
            .map(|x| PathBuf::from(tilde(x).to_string()))
            .collect())
    } else {
        Err(anyhow!("No ~/.repoutilrc, or passed dirs"))
    }
}

// Get every repo from subdirs of `dir`
fn get_repos(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut repos: Vec<PathBuf> = read_dir(dir)?
        .filter_map(|d| d.ok())
        .map(|d| d.path())
        .filter(|d| git::is_git_repo(d))
        .collect();
    repos.sort();
    Ok(repos)
}
