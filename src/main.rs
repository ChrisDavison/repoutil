use clap::{Parser, Subcommand, ValueEnum};
use rayon::prelude::*;
use std::path::PathBuf;

mod ansi_escape;
mod util;
mod vcs;

/// Run common operations across many repositories
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "repoutil")]
#[command(about = "Run common operations across many repositories", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(short, long)]
    use_json: bool,
    #[arg(short, long, hide = true)]
    no_colour: bool,
    /// Colorize output: auto (default), always, or never
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto)]
    color: ColorChoice,
    /// Limit thread pool size for parallel repo ops
    #[arg(long)]
    threads: Option<usize>,
    #[arg(short, long)]
    keep_home: bool,
}

#[derive(Debug, Subcommand, PartialEq)]
enum Command {
    /// Add the current directory to ~/.repoutilrc
    #[command(aliases = &["a"])]
    Add,
    /// List directories tracked in ~/.repoutilrc
    #[command(aliases = &["ls", "l"])]
    List,

    #[cfg(feature = "git")]
    #[command(subcommand)]
    /// Operations on git repositories
    Git(GitCommand),

    #[cfg(feature = "jj")]
    #[command(subcommand)]
    /// Operations on git repositories
    Jj(JjCommand),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[cfg(feature = "git")]
#[derive(Debug, Subcommand, PartialEq)]
enum GitCommand {
    /// Fetch commits and tags
    #[command(alias = "f")]
    Fetch,
    /// Show short status
    #[command(aliases = &["s", "st"])]
    Stat,
    /// Display git dashboard
    #[command(aliases = &["d", "dash"])]
    Dashboard,
    /// List short status of all branches
    #[command(aliases = &["bs"])]
    Branchstat,
    /// Push commits
    #[command(alias = "p", hide = true)]
    Push,
    /// Pull commits
    #[command(alias = "pu", hide = true)]
    Pull,
    /// List repos with local changes
    #[command(aliases = &["u"], hide=true)]
    Unclean,
    /// List all branches
    #[command(aliases = &["b"], hide=true)]
    Branches,
    /// List all untracked folders
    #[command(aliases = &["un"], hide=true)]
    Untracked,
    /// Count stashes
    #[command(aliases = &["sc"], hide=true)]
    Stashcount,
}

#[cfg(feature = "jj")]
#[derive(Debug, Subcommand, PartialEq)]
enum JjCommand {
    /// Get status of all repositories
    Stat,
    /// Pull all repositories
    Sync,
}

#[derive(Clone)]
struct FormatOpts<'a> {
    use_json: bool,
    common_prefix: Option<&'a PathBuf>,
    no_colour: bool,
}

fn main() {
    let args = Cli::parse();

    if let Some(n) = args.threads {
        let _ = rayon::ThreadPoolBuilder::new().num_threads(n).build_global();
    }

    let (includes, excludes) = match util::get_repos_from_config() {
        Ok((i, e)) => (i, e),
        Err(err) => {
            eprintln!("ERR `{}`", err);
            std::process::exit(1);
        }
    };

    let repos = if args.command == Command::Git(GitCommand::Untracked) {
        excludes
    } else {
        includes
    };
    let common = if args.keep_home {
        PathBuf::new()
    } else {
        util::common_ancestor(&repos)
    };

    let env_no_color = std::env::var_os("NO_COLOR").is_some();
    let computed_no_colour = match args.color {
        ColorChoice::Always => false,
        ColorChoice::Never => true,
        ColorChoice::Auto => env_no_color,
    } || args.no_colour;

    let fmt = FormatOpts {
        use_json: args.use_json,
        common_prefix: if args.keep_home { None } else { Some(&common) },
        no_colour: computed_no_colour,
    };

    let cmd = match args.command {
        // Works with any directory
        Command::Add => {
            if let Err(e) = vcs::add() {
                println!("{}", e);
                std::process::exit(1);
            }
            return;
        }
        Command::List => vcs::list,
        // Git commands
        #[cfg(feature = "git")]
        Command::Git(gc) => match gc {
            GitCommand::Push => vcs::git::push,
            GitCommand::Fetch => vcs::git::fetch,
            GitCommand::Stat => vcs::git::stat,
            GitCommand::Pull => vcs::git::pull,
            GitCommand::Unclean => vcs::git::needs_attention,
            GitCommand::Branchstat => vcs::git::branchstat,
            GitCommand::Stashcount => vcs::git::stashcount,
            GitCommand::Branches => vcs::git::branches,
            GitCommand::Untracked => vcs::git::untracked,
            GitCommand::Dashboard => vcs::git::dashboard,
        },
        // JJ commands
        #[cfg(feature = "jj")]
        Command::Jj(jjc) => match jjc {
            JjCommand::Stat => vcs::jj::stat,
            JjCommand::Sync => vcs::jj::sync,
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
