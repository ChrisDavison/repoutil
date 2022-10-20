use anyhow::{anyhow, Result};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::thread;

use shellexpand::tilde;

mod git;

const USAGE: &str = "usage: repoutil stat|fetch|list|unclean|branchstat|branches|help";
const LONG_USAGE: &str = "usage:
    repoutil <command>

Commands:
    p|push            Push commits
    f|fetch           Fetch commits and tags
    s|stat            Show short status
    l|list            List tracked repos
    u|unclean         List repos with local changes
    bs|branchstat     List short status of all branches
    b|branches        List all branches
    h|help            Display this help message
";

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();

    // if args.is_empty() {
    //     println!(@
    //     std::process::exit(0);
    // }

    let cmd = match args.get(0).unwrap_or(&String::from("NO COMMAND")).as_ref() {
        "p" | "push" => git::push,
        "f" | "fetch" => git::fetch,
        "s" | "stat" => git::stat,
        "l" | "list" => git::list,
        "u" | "unclean" => git::needs_attention,
        "bs" | "branchstat" => git::branchstat,
        "b" | "branches" => git::branches,
        "h" | "help" => {
            println!("{LONG_USAGE}");
            std::process::exit(1);
        }
        _ => {
            println!("{USAGE}");
            std::process::exit(1);
        }
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
            Ok(Some(out)) => out.trim_end().into(),
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
    for msg in messages.iter().filter(|msg| !msg.is_empty()) {
        println!("{}", msg)
    }
}

fn get_dirs_from_config() -> Result<Vec<PathBuf>> {
    let repoutil_config = tilde("~/.repoutilrc").to_string();
    let p = std::path::Path::new(&repoutil_config);
    if !p.exists() {
        Err(anyhow!("No ~/.repoutilrc, or passed dirs"))
    } else {
        let contents = std::fs::read_to_string(p)?;
        Ok(contents
            .lines()
            .map(|x| PathBuf::from(tilde(x).to_string()))
            .collect())
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
