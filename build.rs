use clap::Command;
use clap_complete::{generate_to, shells};
use std::env;
use std::path::PathBuf;

fn build_cli() -> Command {
    use clap::{Arg, Command, ValueEnum};

    #[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
    enum ColorChoice {
        Auto,
        Always,
        Never,
    }

    Command::new("repoutil")
        .about("Run common operations across many repositories")
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .help("Output as JSON"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .value_parser(clap::builder::EnumValueParser::<ColorChoice>::new())
                .default_value("auto")
                .help("Colorize output"),
        )
        .arg(
            Arg::new("threads")
                .long("threads")
                .value_parser(clap::value_parser!(usize))
                .help("Limit thread pool size"),
        )
        .arg(Arg::new("keep_home").short('k').long("keep-home"))
        .subcommand(Command::new("list").about("List directories tracked in ~/.repoutilrc"))
        .subcommand(Command::new("add").about("Add current directory to ~/.repoutilrc"))
        .subcommand(
            Command::new("git")
                .about("Operations on git repositories")
                .subcommand(Command::new("stat").about("Show short status"))
                .subcommand(Command::new("fetch").about("Fetch commits and tags"))
                .subcommand(Command::new("pull").about("Pull commits"))
                .subcommand(Command::new("push").about("Push commits"))
                .subcommand(Command::new("branches").about("List all branches"))
                .subcommand(Command::new("branchstat").about("List short status of all branches"))
                .subcommand(Command::new("stashcount").about("Count stashes"))
                .subcommand(Command::new("unclean").about("List repos with local changes"))
                .subcommand(Command::new("dashboard").about("Display git dashboard"))
                .subcommand(Command::new("untracked").about("List all untracked folders")),
        )
        .subcommand(
            Command::new("jj")
                .about("Operations on git repositories")
                .subcommand(Command::new("stat").about("Get status of all repositories"))
                .subcommand(Command::new("sync").about("Pull all repositories")),
        )
}

fn main() {
    let outdir = match env::var_os("CARGO_CFG_TARGET_OS") {
        None => return,
        Some(_) => PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("completions"),
    };

    std::fs::create_dir_all(&outdir).unwrap();

    let mut cmd = build_cli();
    let _ = generate_to(shells::Bash, &mut cmd, "repoutil", &outdir);
    let _ = generate_to(shells::Zsh, &mut cmd, "repoutil", &outdir);
    let _ = generate_to(shells::Fish, &mut cmd, "repoutil", &outdir);
}
