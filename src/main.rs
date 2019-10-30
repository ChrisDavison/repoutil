use std::env;
use std::process::exit;
use std::thread;

mod git;

type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

enum Errs {
    BadUsage = -1,
    UnknownCommand = -2,
    NoParentDirs = -3
}

const USAGE: &str = "usage: repoutil (stat|fetch) [DIRS...]";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("{}", USAGE);
        exit(Errs::BadUsage as i32);
    }
    let cmd = match args[0].as_ref() {
        "fetch" => git::fetch,
        "stat" => git::stat,
        _ => {
            eprintln!("Error: unrecognised command `{}`", args[0]);
            eprintln!("{}", USAGE);
            exit(Errs::UnknownCommand as i32);
        }
    };
    let dirs = match args.get(1..) {
        Some(dirs) => dirs.to_vec(),
        None => vec![env::var("CODEDIR")?],
    };
    if dirs.is_empty() {
        eprintln!("Must pass dirs or set CODEDIR to a parent dir of multiple repos");
        exit(Errs::NoParentDirs as i32);
    }

    let mut all_repos = Vec::new();
    for dir in dirs {
        let repos = match git::get_repos(&dir) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: couldn't get repos: {}", e);
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
            Ok(Some(out)) => println!("{}\n{}\n", repo.display(), out),
            _ => return,
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }
    Ok(())
}
