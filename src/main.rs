use clap::{Parser, Subcommand};
use rayon::prelude::*;
use std::path::PathBuf;

mod git;
mod util;

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "git")]
#[command(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(short, long)]
    use_json: bool,
    #[arg(short, long)]
    keep_home: bool,
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
    #[command(aliases = &["s", "st"])]
    Stat,
    /// List tracked repos
    #[command(aliases = &["ls", "l"])]
    List,
    /// List repos with local changes
    #[command(aliases = &["u"])]
    Unclean,
    /// List short status of all branches
    #[command(aliases = &["bs"])]
    Branchstat,
    /// JJ status
    #[command(aliases = &["jj"])]
    JjStat,
    /// JJ sync all repos
    #[command(aliases = &["jjs"])]
    JjSync,
    /// List all branches
    #[command(aliases = &["b"])]
    Branches,
    /// List all untracked folders
    #[command(aliases = &["un"])]
    Untracked,
    /// Add the current directory to ~/.repoutilrc
    #[command(aliases = &["a"])]
    Add,
}

#[derive(Clone)]
struct FormatOpts<'a> {
    use_json: bool,
    common_prefix: Option<&'a PathBuf>,
}

fn main() {
    let args = Cli::parse();

    let (includes, excludes) = match util::get_repos_from_config() {
        Ok((i, e)) => (i, e),
        Err(err) => {
            eprintln!("ERR `{}`", err);
            std::process::exit(1);
        }
    };

    let repos = if args.command == Command::Untracked {
        excludes
    } else {
        includes
    };
    let common = if args.keep_home {
        PathBuf::new()
    } else {
        util::common_ancestor(&repos)
    };

    let fmt = FormatOpts {
        use_json: args.use_json,
        common_prefix: if args.keep_home { None } else { Some(&common) },
    };

    let cmd = match args.command {
        Command::Push => git::push,
        Command::Fetch => git::fetch,
        Command::Stat => git::stat,
        Command::List => git::list,
        Command::Unclean => git::needs_attention,
        Command::Branchstat => git::branchstat,
        Command::JjStat => git::jjstat,
        Command::JjSync => git::jjsync,
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

    if args.use_json {
        dbg!(&outs);
        println!(r#"{{"items": [{}]}}"#, outs.join(", "));
    } else {
        println!("{}", outs.join("\n"));
    }
}
