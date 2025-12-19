#![allow(dead_code)]

// Colour attributes
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const ITALIC: &str = "\x1b[3m";
pub const UNDERLINE: &str = "\x1b[4m";

// Foreground colours
pub const BLACK: &str = "\x1b[30m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const PURPLE: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";
pub const INTENSE_BLACK: &str = "\x1b[90m";
pub const INTENSE_RED: &str = "\x1b[91m";
pub const INTENSE_GREEN: &str = "\x1b[92m";
pub const INTENSE_YELLOW: &str = "\x1b[93m";
pub const INTENSE_BLUE: &str = "\x1b[94m";
pub const INTENSE_PURPLE: &str = "\x1b[95m";
pub const INTENSE_CYAN: &str = "\x1b[96m";
pub const INTENSE_WHITE: &str = "\x1b[97m";

// Background colours
pub const BG_BLACK: &str = "\x1b[40m";
pub const BG_RED: &str = "\x1b[41m";
pub const BG_GREEN: &str = "\x1b[42m";
pub const BG_YELLOW: &str = "\x1b[43m";
pub const BG_BLUE: &str = "\x1b[44m";
pub const BG_PURPLE: &str = "\x1b[45m";
pub const BG_CYAN: &str = "\x1b[46m";
pub const BG_WHITE: &str = "\x1b[47m";
pub const BG_INTENSE_BLACK: &str = "\x1b[100m";
pub const BG_INTENSE_RED: &str = "\x1b[101m";
pub const BG_INTENSE_GREEN: &str = "\x1b[102m";
pub const BG_INTENSE_YELLOW: &str = "\x1b[103m";
pub const BG_INTENSE_BLUE: &str = "\x1b[104m";
pub const BG_INTENSE_PURPLE: &str = "\x1b[105m";
pub const BG_INTENSE_CYAN: &str = "\x1b[106m";
pub const BG_INTENSE_WHITE: &str = "\x1b[107m";

pub const ANSI_RESET: &str = "\x1b[0m";

/// Check if colors should be enabled based on TTY and NO_COLOR
pub fn should_color_stdout() -> bool {
    anstream::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

pub fn colour(text: impl Into<String> + std::fmt::Display, colours: &[&str]) -> String {
    let colourcode = colours.join("");
    format!("{colourcode}{text}{ANSI_RESET}")
}
