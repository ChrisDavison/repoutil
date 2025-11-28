use crate::FormatOpts;

use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::ansi_escape::*;
use crate::util::{remove_common_ancestor, format_json};

pub mod jj;
pub mod git;

pub fn add() -> Result<()> {
    let config_filename = crate::util::homedir(".repoutilrc")?;
    let curdir = std::env::current_dir()?;
    if git::is_repo(&curdir) {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(config_filename)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", curdir.to_string_lossy()) {
            eprintln!("Couldn't write to file: {}", e);
        }
        Ok(())
    } else {
        Err(anyhow!("Don't appear to be in the root of a git repo."))
    }
}

/// List each repo found
pub fn list(p: &Path, fmt: &FormatOpts) -> Result<Option<String>> {
    let s = if fmt.use_json {
        format_json(p, None, true, fmt.common_prefix)
    } else {
        remove_common_ancestor(p, fmt.common_prefix)
    };
    Ok(Some(s))
}
