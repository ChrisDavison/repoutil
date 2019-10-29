use std::thread;

type Result<T> = ::std::result::Result<T, Box<dyn::std::error::Error>>;

enum Errs {
    BadUsage = -1,
    UnknownCommand = -2,
    BadRepos = -3,
}

const USAGE: &'static str = "usage: repoutil (stat|fetch) [DIRS...]";

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

mod git {
    use super::Result;
    use std::env;

    pub fn is_git_repo(mut p: std::path::PathBuf) -> bool {
        p.push(".git");
        p.exists()
    }

    fn command_output(dir: std::path::PathBuf, args: &[&str], err_msg: Option<String>) -> String {
        let err_msg = match err_msg {
            Some(d) => d,
            None => format!("couldn't run command `git {:?}`", args),
        };
        let out = std::process::Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .expect(&err_msg);
        std::str::from_utf8(&out.stdout)
            .expect("couldn't convert stdout")
            .to_string()
    }

    pub fn fetch(p: std::path::PathBuf) -> Option<String> {
        let err_msg = Some(format!("couldn't fetch {:?}", p));
        let out = command_output(p, &["fetch", "--all"], err_msg);
        let status: String = out.lines().skip(1).collect();
        match status.is_empty() {
            true => None,
            false => Some(status),
        }
    }

    pub fn stat(p: std::path::PathBuf) -> Option<String> {
        let err_msg = Some(format!("couldn't stat {:?}", p));
        let out = command_output(p, &["status", "-s", "-b"], err_msg);
        let lines: Vec<String> = out.lines().map(|x| x.to_string()).collect();
        if lines[0].ends_with(']') {
            return Some(lines.join("\n"));
        }
        let status: Vec<String> = lines.iter().skip(1).map(|x| x.to_string()).collect();
        match status.is_empty() {
            true => None,
            false => Some(status.join("\n")),
        }
    }

    pub fn get_repos(dirs: Option<&[String]>) -> Result<Vec<::std::path::PathBuf>> {
        let dirs = match dirs {
            Some(d) => d.to_owned(),
            None => vec![env::var("CODEDIR")?],
        };
        let mut repos = Vec::new();
        for dir in dirs {
            let repos_for_dir: Vec<_> = std::fs::read_dir(dir)?
                .filter(|d| d.is_ok())
                .filter(|d| {
                    let entry = d.as_ref().unwrap().path();
                    entry.is_dir() && is_git_repo(entry)
                })
                .map(|d| d.unwrap().path())
                .collect();
            repos.extend(repos_for_dir.iter().cloned());
        }
        Ok(repos)
    }
}
