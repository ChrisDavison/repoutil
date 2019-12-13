use std::thread;

use shellexpand::tilde;

mod git;

type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

const USAGE: &str = "usage: repoutil stat|fetch|list|unclean";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("{}", USAGE);
        return;
    }
    let cmd = match args[0].as_ref() {
        "fetch" | "f" => git::fetch,
        "stat" | "s" => git::stat,
        "list" | "l" => git::list,
        "unclean" | "u" => git::needs_attention,
        _ => {
            eprintln!("Command `{}` not valid.\n", args[0]);
            eprintln!("{}", USAGE);
            return;
        }
    };
    let dirs = match get_dirs_from_config() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("{}", USAGE);
            return;
        }
    };
    let mut all_repos = Vec::new();
    for dir in dirs {
        let repos = match git::get_repos(&dir) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Couldn't get repos from '{}': '{}'\n", dir, e);
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

fn get_dirs_from_config() -> Result<Vec<String>> {
    let repoutil_config = tilde("~/.repoutilrc").to_string();
    let p = std::path::Path::new(&repoutil_config);
    if p.exists() {
        let contents = std::fs::read_to_string(p)?;
        Ok(contents.lines().map(|x| tilde(x).to_string()).collect())
    } else {
        Err(format!("No ~/.repoutilrc, or passed dirs").into())
    }
}
