use std::io::IsTerminal;

pub const BOLD: &str = "\x1b[1m";
pub const FAINT: &str = "\x1b[2m";

pub const BLACK: &str = "\x1b[90m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
#[cfg(feature = "jj")]
pub const PURPLE: &str = "\x1b[35m";

pub const ANSI_RESET: &str = "\x1b[0m";

pub fn should_color_stdout() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty())
}

/// Wrap `text` in the given ANSI codes, unless colour is disabled
pub fn apply_color(text: impl Into<String>, no_color: bool, codes: &[&str]) -> String {
    let text = text.into();
    if no_color || codes.is_empty() {
        text
    } else {
        format!("{}{text}{ANSI_RESET}", codes.concat())
    }
}
