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

    let (includes, excludes) = util::get_repos_from_config().expect("Couldn't get repos");
    let repos = if opts.command == OptCommand::Untracked {
        excludes
    } else {
        includes
    };

    let common = util::common_substring(
        &repos
            .iter()
            .map(|x| format!("{}", x.display()))
            .collect::<Vec<String>>(),
    );
    let outs: Vec<_> = repos
        .par_iter()
        .filter_map(|repo| match cmd(repo, json) {
            Ok(rr) => {
                if rr.output.is_empty() {
                    None
                } else {
                    Some(rr.output.replace(&common, ""))
                }
            }
            Err(e) => {
                eprintln!("ERR `{}`: {}", repo.display(), e);
                None
            }
        })
        .collect();
    if json {
        println!("{{\"items\": [{}]}}", outs.join(","));
    } else {
        println!("{}", outs.join("\n"));
    }
}
