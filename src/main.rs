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

    #[cfg(feature = "jj")]
    /// Get status of all repositories
    #[command(aliases = &["jj"], hide=true)]
    JJStat,
    #[cfg(feature = "jj")]
    #[command(aliases = &["jjs"], hide=true)]
    /// Pull all repositories
    JJSync,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ColorChoice {
    Auto,
    Always,
    Never,
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
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global();
    }

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

    let computed_no_colour = match args.color {
        ColorChoice::Always => false,
        ColorChoice::Never => true,
        ColorChoice::Auto => !ansi_escape::should_color_stdout(),
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
        Command::Push => vcs::git::push,
        Command::Fetch => vcs::git::fetch,
        Command::Stat => vcs::git::stat,
        Command::Pull => vcs::git::pull,
        Command::Unclean => vcs::git::needs_attention,
        Command::Branchstat => vcs::git::branchstat,
        Command::Stashcount => vcs::git::stashcount,
        Command::Branches => vcs::git::branches,
        Command::Untracked => vcs::git::untracked,
        Command::Dashboard => vcs::git::dashboard,
        #[cfg(feature = "jj")]
        Command::JJStat => vcs::jj::stat,
        #[cfg(feature = "jj")]
        Command::JJSync => vcs::jj::sync,
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
