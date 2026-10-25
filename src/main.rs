use clap::{Parser, Subcommand, ValueEnum};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

use crate::ansi_escape::{apply_color, BLACK, BOLD};
mod ansi_escape;
mod config;
mod git;
mod util;

/// Run common operations across many repositories
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "repoutil")]
#[command(about = "Run common operations across many repositories", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    /// Colorize output: auto (default), always, or never
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto)]
    color: ColorChoice,
    #[arg(short, long)]
    keep_home: bool,
}

#[derive(Debug, Subcommand, PartialEq)]
enum Command {
    /// Add the repo containing the current directory to ~/.repoutilrc
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
    common_prefix: Option<&'a Path>,
    no_color: bool,
    separator: &'a str,
}

impl FormatOpts<'_> {
    fn paint(&self, text: impl Into<String>, codes: &[&str]) -> String {
        apply_color(text, self.no_color, codes)
    }
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let cmd = match args.command.unwrap_or(Command::Branchstat) {
        // Runs before reading the config, as it may create it
        Command::Add => return config::add(),
        Command::List => git::list,
        Command::Fetch => git::fetch,
        Command::Branchstat => git::branchstat,
        Command::Branches => git::branches,
    };

    let repos = config::get_repos_from_config()?;
    let common = if args.keep_home {
        PathBuf::new()
    } else {
        util::common_ancestor(&repos)
    };

    let no_color = match args.color {
        ColorChoice::Always => false,
        ColorChoice::Never => true,
        ColorChoice::Auto => !ansi_escape::should_color_stdout(),
    };

    let separator = apply_color(" · ", no_color, &[BOLD, BLACK]);

    let fmt = FormatOpts {
        common_prefix: if args.keep_home {
            None
        } else {
            Some(common.as_path())
        },
        no_color,
        separator: &separator,
    };

    let outs: Vec<_> = repos
        .par_iter()
        .filter_map(|repo| match cmd(repo, &fmt) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("ERR {}: {}", repo.display(), e);
                None
            }
        })
        .collect();

    for line in outs {
        println!("{line}");
    }
    Ok(())
}
