use clap::{Parser, Subcommand, ValueEnum};
use rayon::prelude::*;
use std::path::PathBuf;

mod ansi_escape;
mod git;
mod util;

/// Run common operations across many repositories
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "repoutil")]
#[command(about = "Run common operations across many repositories", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(short, long, hide = true)]
    no_colour: bool,
    /// Colorize output: auto (default), always, or never
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto)]
    color: ColorChoice,
    #[arg(short, long)]
    keep_home: bool,
}

#[derive(Debug, Subcommand, PartialEq)]
enum Command {
    /// Add the current directory to ~/.repoutilrc
    #[command(aliases = &["a"])]
    Add,
    /// List directories (or glob matches) that will be tracked from ~/.repoutilrc
    #[command(aliases = &["ls", "l"])]
    List,

    /// Fetch commits and tags
    #[command(alias = "f")]
    Fetch,
    /// List short status of all branches
    #[command(aliases = &["bs"])]
    Branchstat,

    /// List branches
    #[command(aliases = &["b"])]
    Branches,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Clone)]
struct FormatOpts<'a> {
    common_prefix: Option<&'a PathBuf>,
    no_colour: bool,
}

fn main() {
    let args = Cli::parse();

    let (includes, _excludes) = match util::get_repos_from_config() {
        Err(err) => {
            eprintln!("ERR `{}`", err);
            std::process::exit(1);
        }
        Ok((i, e)) => (i, e),
    };

    let repos = includes;
    let common = if args.keep_home {
        PathBuf::new()
    } else {
        util::common_ancestor(&repos)
    };

    let computed_no_colour = match args.color {
        ColorChoice::Always => false,
        ColorChoice::Never => true,
        ColorChoice::Auto => !ansi_escape::should_color_stdout(),
    } || args.no_colour;

    let fmt = FormatOpts {
        common_prefix: if args.keep_home { None } else { Some(&common) },
        no_colour: computed_no_colour,
    };

    let cmd = match args.command.unwrap_or(Command::Branchstat) {
        // Works with any directory
        Command::Add => {
            if let Err(e) = git::add() {
                println!("{}", e);
                std::process::exit(1);
            }
            return;
        }
        Command::List => git::list,
        Command::Fetch => git::fetch,
        Command::Branchstat => git::branchstat,
        Command::Branches => git::branches,
    };

    let outs: Vec<_> = repos
        .par_iter()
        .filter_map(|repo| cmd(repo, &fmt).ok())
        .filter_map(|r| r)
        .collect();

    println!("{}", outs.join("\n"));
}
