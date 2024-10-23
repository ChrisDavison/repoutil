use clap::{Parser, Subcommand};
use rayon::prelude::*;
use std::path::PathBuf;

mod git;
mod util;

#[derive(Debug, Parser)]
#[command(name = "repoutil", about = "Operations on multiple git repos")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /// Use JSON rather than plaintext output
    #[arg(long, short)]
    json: bool,
}

#[derive(Debug, Subcommand, PartialEq)]
enum Command {
    /// Push commits
    #[command(alias = "p")]
    Push,
    /// Fetch commits and tags
    #[command(alias = "f")]
    Fetch,
    /// Show short status
    #[command(alias = "s")]
    Stat,
    /// List tracked repos
    #[command(alias = "l")]
    List,
    /// List repos with local changes
    #[command(alias = "u")]
    Unclean,
    /// List short status of all branches
    #[command(alias = "bs")]
    Branchstat,
    /// List all branches
    #[command(alias = "b")]
    Branches,
    /// List all untracked folders
    #[command(alias = "un")]
    Untracked,
}

fn main() {
    let opts = Cli::parse();

    let json = opts.json;

    let cmd = match opts.command {
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
    let repos = if opts.command == Command::Untracked {
        excludes
    } else {
        includes
    };

    let common = util::common_ancestor(&repos);
    let outs: Vec<_> = repos
        .par_iter()
        .filter_map(|repo| match (opts.json, cmd(repo)) {
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
