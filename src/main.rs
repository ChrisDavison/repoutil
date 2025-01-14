use anyhow::{anyhow, Result};
use rayon::prelude::*;
use std::path::PathBuf;

mod git;
mod util;

#[derive(PartialEq)]
enum Command {
    /// Push commits
    Push,
    /// Fetch commits and tags
    Fetch,
    /// Show short status
    Stat,
    /// List tracked repos
    List,
    /// List repos with local changes
    Unclean,
    /// List short status of all branches
    Branchstat,
    /// List all branches
    Branches,
    /// List all untracked folders
    Untracked,
}

fn print_help() {
    println!(
        "usage: repoutil COMMAND [-j|--json]

commands:
    p push
        Push commits
    f fetch
        Fetch commits and tags
    s stat status
        Short status
    l ls list
        List repos found
    u unclean
        List repos with changes
    bs branchstat
        List short status of all branches
    b branches
        List branches
    un untracked
        List all untracked repos
"
    );
}

fn parse_args() -> Result<(Command, bool)> {
    let mut use_json = false;
    let mut words = Vec::new();
    for arg in std::env::args().skip(1) {
        if matches!(arg.as_str(), "-j" | "-json" | "--json" | "--j") {
            use_json = true;
        } else {
            words.push(arg)
        }
    }
    if words.is_empty() {
        Err(anyhow!("No command given."))
    } else if words.len() > 1 {
        Err(anyhow!("Too many arguments? {words:?}"))
    } else {
        let w = words[0].to_lowercase();
        match w.as_str() {
            "p" | "push" => Ok((Command::Push, use_json)),
            "f" | "fetch" => Ok((Command::Fetch, use_json)),
            "s" | "stat" | "status" => Ok((Command::Stat, use_json)),
            "l" | "ls" | "list" => Ok((Command::List, use_json)),
            "u" | "unclean" | "dirty" => Ok((Command::Unclean, use_json)),
            "bs" | "branchstat" => Ok((Command::Branchstat, use_json)),
            "b" | "branches" | "branch" => Ok((Command::Branches, use_json)),
            "un" | "untracked" => Ok((Command::Untracked, use_json)),
            _ => Err(anyhow!("Unrecognised command `{w}`")),
        }
    }
}

fn main() {
    let (command, json) = match parse_args() {
        Ok((command, json)) => (command, json),
        Err(e) => {
            print_help();
            println!("{}", e);
            std::process::exit(1);
        }
    };

    let cmd = match command {
        Command::Push => git::push,
        Command::Fetch => git::fetch,
        Command::Stat => git::stat,
        Command::List => git::list,
        Command::Unclean => git::needs_attention,
        Command::Branchstat => git::branchstat,
        Command::Branches => git::branches,
        Command::Untracked => git::untracked,
    };

    let (includes, excludes) = match util::get_repos_from_config() {
        Ok((i, e)) => (i, e),
        Err(err) => {
            eprintln!("ERR `{}`", err);
            std::process::exit(1);
        }
    };
    let repos = if command == Command::Untracked {
        excludes
    } else {
        includes
    };

    let common = util::common_ancestor(&repos);
    let outs: Vec<_> = repos
        .par_iter()
        .filter_map(|repo| match (json, cmd(repo)) {
            (false, Ok(rr)) => rr.plain(&common),
            (true, Ok(rr)) => rr.json(&common),
            (_, Err(e)) => {
                eprintln!("ERR `{}`: {}", repo.display(), e);
                None
            }
        })
        .filter(|s| !s.is_empty())
        .collect();
    if outs.is_empty() {
        if json {
            println!(r#"{{"items": [{{"title": "NO ITEMS"}}]}}"#);
        } else {
            println!("No output");
        }
    } else {
        if json {
            println!("{{\"items\": [{}]}}", outs.join(","));
        } else {
            println!("{}", outs.join("\n"));
        }
    }
}
