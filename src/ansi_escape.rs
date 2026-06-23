pub const BOLD: &str = "\x1b[1m";

// pub const BLACK: &str = "\x1b[30m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
// pub const PURPLE: &str = "\x1b[35m";

pub const ANSI_RESET: &str = "\x1b[0m";

pub fn should_color_stdout() -> bool {
    anstream::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

pub fn colour(text: impl Into<String> + std::fmt::Display, colours: &[&str]) -> String {
    let colourcode = colours.join("");
    format!("{colourcode}{text}{ANSI_RESET}")
}

pub fn apply_color(text: impl Into<String>, no_colour: bool, codes: &[&str]) -> String {
    let text = text.into();
    if no_colour || codes.is_empty() {
        text
    } else {
        colour(text, codes)
    }
}
