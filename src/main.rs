use std::env;
use std::thread;

mod git;

type Result<T> = ::std::result::Result<T, Box<dyn::std::error::Error>>;

const USAGE: &str = "usage: repoutil (stat|fetch|list|unclean) [DIRS...]";

type GitCommand = fn(&std::path::PathBuf) -> Result<Option<String>>;

fn main() {
    let (cmd, dirs) = match parse_args() {
        Ok((cmd, dirs)) => (cmd, dirs),
        Err(e) => {
            eprintln!("{}\n", e);
            eprintln!("{}", USAGE);
            return;
        }
    };

    let mut all_repos = Vec::new();
    for dir in dirs {
        let repos = match git::get_repos(&dir) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Couldn't get repos from '{}'", e);
                continue;
            }
        };
        all_repos.extend(repos);
    }

    let mut handles = Vec::new();
    for repo in all_repos {
        // Spawn a thread for each repo
        // and run the chosen command.
        // The handle must 'move' to take ownership of `cmd`
        let handle = thread::spawn(move || match cmd(&repo) {
            Ok(Some(out)) => println!("{}", out),
            Err(e) => eprintln!("Repo {}: {}", repo.display(), e),
            _ => (),
        });
        handles.push(handle);
    }

    for h in handles {
        if let Err(e) = h.join() {
            eprintln!("A child git command panic'd: {:?}", e);
        }
    }
}

fn parse_args() -> Result<(GitCommand, Vec<String>)> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return Err("No arguments given".into());
    }
    let cmd = match args[0].as_ref() {
        "fetch" => git::fetch,
        "stat" => git::stat,
        "list" => git::list,
        "unclean" => git::needs_attention,
        _ => {
            return Err(format!("Unrecognised command `{}`", args[0]).into());
        }
    };
    let dirs = match args.get(1..) {
        Some(dirs) => dirs.to_vec(),
        None => vec![env::var("CODEDIR")?],
    };
    if dirs.is_empty() {
        return Err("Must pass dirs or set CODEDIR to a parent dir of multiple repos".into());
    }

    Ok((cmd, dirs))
}
