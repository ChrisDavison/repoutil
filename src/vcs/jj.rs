use super::*;

/// Sync jujutsu repo
pub fn sync(dir: &Path, _fmt: &FormatOpts) -> Result<Option<String>> {
    Command::new("jj")
        .current_dir(dir)
        .args(["sync"])
        .output()?;

    stat(dir, _fmt)
}

/// Show jujutsu status
pub fn stat(dir: &Path, _fmt: &FormatOpts) -> Result<Option<String>> {
    let no_modifications = std::str::from_utf8(
        &Command::new("git")
            .current_dir(dir)
            .args(["status", "-s", "-b"])
            .output()?
            .stdout,
    )?
    .lines()
    .count()
        == 1;
    let stdout = Command::new("jj")
        .current_dir(dir)
        .args(["status", "--color=always"])
        .output()?
        .stdout;
    let lines: Vec<&str> = std::str::from_utf8(&stdout)?.lines().collect();
    if lines.is_empty() || no_modifications {
        Ok(None)
    } else {
        let s = dir.to_string_lossy().to_string();
        Ok(Some(format!(
            "{} {}\n{}\n",
            colour(s, &[YELLOW]),
            colour("·".repeat(0), &[BOLD, RED]),
            lines
                .iter()
                .map(|x| format!("░ {x}"))
                .collect::<Vec<_>>()
                .join("\n")
        )))
    }
}
