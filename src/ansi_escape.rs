#![allow(dead_code)]

// Colour attributes
pub const Bold: &str = "\x1b[1m";
pub const Dim: &str = "\x1b[2m";
pub const Italic: &str = "\x1b[3m";
pub const Underline: &str = "\x1b[4m";

// Foreground colours
pub const Black: &str = "\x1b[30m";
pub const Red: &str = "\x1b[31m";
pub const Green: &str = "\x1b[32m";
pub const Yellow: &str = "\x1b[33m";
pub const Blue: &str = "\x1b[34m";
pub const Purple: &str = "\x1b[35m";
pub const Cyan: &str = "\x1b[36m";
pub const White: &str = "\x1b[37m";
pub const IntenseBlack: &str = "\x1b[90m";
pub const IntenseRed: &str = "\x1b[91m";
pub const IntenseGreen: &str = "\x1b[92m";
pub const IntenseYellow: &str = "\x1b[93m";
pub const IntenseBlue: &str = "\x1b[94m";
pub const IntensePurple: &str = "\x1b[95m";
pub const IntenseCyan: &str = "\x1b[96m";
pub const IntenseWhite: &str = "\x1b[97m";

// Background colours
pub const BGBlack: &str = "\x1b[40m";
pub const BGRed: &str = "\x1b[41m";
pub const BGGreen: &str = "\x1b[42m";
pub const BGYellow: &str = "\x1b[43m";
pub const BGBlue: &str = "\x1b[44m";
pub const BGPurple: &str = "\x1b[45m";
pub const BGCyan: &str = "\x1b[46m";
pub const BGWhite: &str = "\x1b[47m";
pub const BGIntenseBlack: &str = "\x1b[100m";
pub const BGIntenseRed: &str = "\x1b[101m";
pub const BGIntenseGreen: &str = "\x1b[102m";
pub const BGIntenseYellow: &str = "\x1b[103m";
pub const BGIntenseBlue: &str = "\x1b[104m";
pub const BGIntensePurple: &str = "\x1b[105m";
pub const BGIntenseCyan: &str = "\x1b[106m";
pub const BGIntenseWhite: &str = "\x1b[107m";

const ANSI_RESET: &str = "\x1b[0m";

pub fn colour(text: impl Into<String> + std::fmt::Display, colours: &[&str]) -> String {
    let colourcode = colours.join("");
    format!("{colourcode}{text}{ANSI_RESET}")
}
