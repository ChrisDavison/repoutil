use clap::{Parser, Subcommand, ValueEnum};
use rayon::prelude::*;
use std::path::PathBuf;

use crate::ansi_escape::{apply_color, BLACK, BLUE, BOLD, FAINT};
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
    no_color: bool,
    separator: &'a str,
}

impl FormatOpts<'_> {
    fn paint(&self, text: impl Into<String>, codes: &[&str]) -> String {
        apply_color(text, self.no_color, codes)
    }
}

/// One repo's line of output
#[derive(Default)]
struct Row {
    /// ANSI codes for the repo path
    path_style: &'static [&'static str],
    /// Shown faintly after the path, e.g. the current branch
    note: Option<String>,
    /// Already-coloured text for the second column
    detail: String,
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

    let common_prefix = (!args.keep_home).then_some(common.as_path());
    let fmt = FormatOpts {
        no_color,
        separator: &separator,
    };

    let rows: Vec<_> = repos
        .par_iter()
        .filter_map(|repo| match cmd(repo, &fmt) {
            Ok(row) => row.map(|row| (repo, row)),
            Err(e) => {
                eprintln!("ERR {}: {}", repo.display(), e);
                None
            }
        })
        .collect();

    // Colour codes don't take up space, so track each header's visible width
    let headers: Vec<_> = rows
        .iter()
        .map(|(repo, row)| {
            let path = util::remove_common_ancestor(repo, common_prefix);
            let mut width = path.chars().count();
            let mut header = fmt.paint(path, row.path_style);
            if let Some(note) = &row.note {
                width += note.chars().count() + 1;
                header.push_str(&fmt.paint(format!(" {note}"), &[FAINT, BLUE]));
            }
            (header, width)
        })
        .collect();
    let column = headers.iter().map(|(_, w)| *w).max().unwrap_or(0);

    for ((header, width), (_, row)) in headers.iter().zip(&rows) {
        if row.detail.is_empty() {
            println!("{header}");
        } else {
            let padding = " ".repeat(column - width);
            println!("{header}{padding}  {}", row.detail);
        }
    }
    Ok(())
}
