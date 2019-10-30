use std::thread;

mod git;

type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

enum Errs {
    BadUsage = -1,
    UnknownCommand = -2,
    BadRepos = -3,
}

const USAGE: &str = "usage: repoutil (stat|fetch) [DIRS...]";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("{}", USAGE);
        std::process::exit(Errs::BadUsage as i32);
    }
    let cmd = match args[0].as_ref() {
        "fetch" => git::fetch,
        "stat" => git::stat,
        _ => {
            eprintln!("Error: unrecognised command `{}`", args[0]);
            eprintln!("{}", USAGE);
            std::process::exit(Errs::UnknownCommand as i32);
        }
    };
    let dirs: Option<&[String]> = args.get(1..);
    let repos = match git::get_repos(dirs) {
        Ok(r) => r,
        _ => {
            eprintln!("Error: couldn't get repos");
            std::process::exit(Errs::BadRepos as i32);
        }
    };

    let mut handles = Vec::new();
    for repo in repos {
        // Spawn a thread for each repo
        // and run the chosen command.
        // The handle must 'move' to take ownership of `cmd`
        let handle = thread::spawn(move || match cmd(repo.clone()) {
            Some(out) => println!("{}\n{}\n", repo.display(), out),
            None => return,
        });
        handles.push(handle);
    }
    for h in handles {
        h.join().unwrap();
    }
    Ok(())
}
