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
    /// Add the current directory to ~/.repoutilrc
    Add,
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
    a add
        Add the current working directory to ~/.repoutilrc
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
        let cmd = match w.as_str() {
            "p" | "push" => Command::Push,
            "f" | "fetch" => Command::Fetch,
            "s" | "stat" | "status" => Command::Stat,
            "l" | "ls" | "list" => Command::List,
            "u" | "unclean" | "dirty" => Command::Unclean,
            "bs" | "branchstat" => Command::Branchstat,
            "b" | "branches" | "branch" => Command::Branches,
            "un" | "untracked" => Command::Untracked,
            "a" | "add" => Command::Add,
            _ => return Err(anyhow!("Unrecognised command `{w}`")),
        };
        Ok((cmd, use_json))
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

    let cmd = match command {
        Command::Push => git::push,
        Command::Fetch => git::fetch,
        Command::Stat => git::stat,
        Command::List => git::list,
        Command::Unclean => git::needs_attention,
        Command::Branchstat => git::branchstat,
        Command::Branches => git::branches,
        Command::Untracked => git::untracked,
        Command::Add => {
            if let Err(e) = git::add() {
                println!("{}", e);
                std::process::exit(1);
            }
            return;
        }
    };

    let formatter = if json { git::as_json } else { git::as_plain };

    let fmt_output = |repo| match cmd(repo) {
        Ok(output) => output.and_then(|r| formatter(r, &common)),
        Err(e) => {
            eprintln!("ERR `{}`: {}", repo.display(), e);
            None
        }
    };

    let outs: Vec<_> = repos
        .par_iter()
        .filter_map(fmt_output)
        .filter(|s| !s.is_empty())
        .collect();

    match (json, outs.is_empty()) {
        (true, true) => println!(r#"{{"items": [{{"title": "NO ITEMS"}}]}}"#),
        (true, false) => println!("{{\"items\": [{}]}}", outs.join(",")),
        (false, false) => println!("{}", outs.join("\n")),
        (false, true) => {}
    }
}
