use rayon::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

mod git;
mod util;

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
    /// List all untracked folders
    #[structopt(alias = "un")]
    Untracked,
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

    let (includes, excludes) = match util::get_repos_from_config() {
        Ok((i, e)) => (i, e),
        Err(err) => {
            eprintln!("ERR `{}`", err);
            std::process::exit(1);
        }
    };
    let repos = if opts.command == OptCommand::Untracked {
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
    if json {
        println!("{{\"items\": [{}]}}", outs.join(","));
    } else {
        println!("{}", outs.join("\n"));
    }
}
