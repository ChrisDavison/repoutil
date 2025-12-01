use clap::{Parser, Subcommand};
use rayon::prelude::*;
use std::path::PathBuf;

mod ansi_escape;
mod util;
mod vcs;

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
    no_colour: bool,
    #[arg(short, long)]
    keep_home: bool,
}

#[derive(Debug, Subcommand, PartialEq)]
enum Command {
    /// Push commits
    #[command(alias = "p", hide = true)]
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
    #[command(aliases = &["u"], hide=true)]
    Unclean,
    /// List short status of all branches
    #[command(aliases = &["bs"])]
    Branchstat,
    /// JJ status
    #[command(aliases = &["jj"], hide=true)]
    JjStat,
    /// JJ sync all repos
    #[command(aliases = &["jjs"], hide=true)]
    JjSync,
    /// List all branches
    #[command(aliases = &["b"], hide=true)]
    Branches,
    /// List all untracked folders
    #[command(aliases = &["un"], hide=true)]
    Untracked,

    /// Add the current directory to ~/.repoutilrc
    #[command(aliases = &["a"])]
    Add,

    /// Display git dashboard
    #[command(aliases = &["d"])]
    Dashboard,
}

#[derive(Clone)]
struct FormatOpts<'a> {
    use_json: bool,
    common_prefix: Option<&'a PathBuf>,
    no_colour: bool,
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
        no_colour: args.no_colour,
    };

    let cmd = match args.command {
        Command::Push => vcs::git::push,
        Command::Fetch => vcs::git::fetch,
        Command::Stat => vcs::git::stat,
        Command::List => vcs::list,
        Command::Unclean => vcs::git::needs_attention,
        Command::Branchstat => vcs::git::branchstat,
        Command::JjStat => vcs::jj::stat,
        Command::JjSync => vcs::jj::sync,
        Command::Branches => vcs::git::branches,
        Command::Untracked => vcs::git::untracked,
        Command::Dashboard => vcs::git::dashboard,
        Command::Add => {
            if let Err(e) = vcs::add() {
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
        println!(r#"{{"items": [{}]}}"#, outs.join(", "));
    } else {
        println!("{}", outs.join("\n"));
    }
}
