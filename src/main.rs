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
        "usage: repoutil COMMAND [-j|--json] [-d|--dont-strip-home]

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

#[derive(Clone)]
struct FormatOpts<'a> {
    use_json: bool,
    common_prefix: &'a PathBuf,
}

fn parse_args() -> Result<(Command, bool, bool)> {
    let mut use_json = false;
    let mut words = Vec::new();
    let mut keep_home = false;
    for arg in std::env::args().skip(1) {
        if matches!(arg.as_str(), "-j" | "-json" | "--json" | "--j") {
            use_json = true;
        } else if matches!(arg.as_str(), "-d" | "--dont-strip-home") {
            keep_home = true;
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
        Ok((cmd, use_json, keep_home))
    }
}

fn main() {
    let (command, json, keep_home) = match parse_args() {
        Ok((command, json, keep_home)) => (command, json, keep_home),
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
    let common = if keep_home {
        PathBuf::new()
    } else {
        util::common_ancestor(&repos)
    };

    let fmt = FormatOpts {
        use_json: json,
        common_prefix: &common,
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
        Command::Add => {
            if let Err(e) = git::add() {
                println!("{}", e);
                std::process::exit(1);
            }
            return;
        }
    };

    let outs: Vec<_> = repos
        .par_iter()
        .filter_map(|repo| cmd(repo, &fmt).ok())
        .filter_map(|r| r)
        .collect();

    if json {
        println!(r#"{{"items": [{{{}}}]}}"#, outs.join(", "));
    } else {
        println!("{}", outs.join("\n"));
    }
}
