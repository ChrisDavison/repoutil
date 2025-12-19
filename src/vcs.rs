use crate::FormatOpts;

use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::ansi_escape::*;
use crate::util::{format_json, remove_common_ancestor};

pub mod git;

#[cfg(feature = "jj")]
pub mod jj;

pub fn add() -> Result<()> {
    let config_filename = crate::util::homedir(".repoutilrc")?;
    let curdir = std::env::current_dir()?;
    if git::is_repo(&curdir) {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(config_filename)?;

        writeln!(file, "{}", curdir.to_string_lossy())?;
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
