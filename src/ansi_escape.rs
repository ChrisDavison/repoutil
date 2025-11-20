pub enum ColourAttribute {
    Bold,
    Dim,
    Italic,
    Underline,
}

// ESC[mode1m
// ESC[mode1;mode2m
// ESC[mode1;mode2;mode3m
// ...so take a list of modes and sum them
// if combining text colours, keep rightmost
// if combining bg colours, keep rightmost
// if combining text and bg colours, keep rightmost of each

// BG Colour
impl std::ops::Add<BGColour> for BGColour {
    type Output = String;
    fn add(self, other: BGColour) -> String {
        format!("{self}{other}")
    }
}

impl std::ops::Add<Colour> for BGColour {
    type Output = String;
    fn add(self, other: Colour) -> String {
        format!("{self}{other}")
    }
}

impl std::ops::Add<ColourAttribute> for BGColour {
    type Output = String;
    fn add(self, other: ColourAttribute) -> String {
        format!("{self}{other}")
    }
}

// Colour
impl std::ops::Add<BGColour> for Colour {
    type Output = String;
    fn add(self, other: BGColour) -> String {
        format!("{self}{other}")
    }
}

impl std::ops::Add<Colour> for Colour {
    type Output = String;
    fn add(self, other: Colour) -> String {
        format!("{self}{other}")
    }
}

impl std::ops::Add<ColourAttribute> for Colour {
    type Output = String;
    fn add(self, other: ColourAttribute) -> String {
        format!("{self}{other}")
    }
}

// ColourAttribute
impl std::ops::Add<BGColour> for ColourAttribute {
    type Output = String;
    fn add(self, other: BGColour) -> String {
        format!("{self}{other}")
    }
}

impl std::ops::Add<Colour> for ColourAttribute {
    type Output = String;
    fn add(self, other: Colour) -> String {
        format!("{self}{other}")
    }
}

impl std::ops::Add<ColourAttribute> for ColourAttribute {
    type Output = String;
    fn add(self, other: ColourAttribute) -> String {
        format!("{self}{other}")
    }
}

pub enum BGColour {
    // regular
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
    // intense
    IntenseBlack,
    IntenseRed,
    IntenseGreen,
    IntenseYellow,
    IntenseBlue,
    IntensePurple,
    IntenseCyan,
    IntenseWhite,
}

pub enum Colour {
    // regular
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
    // intense
    IntenseBlack,
    IntenseRed,
    IntenseGreen,
    IntenseYellow,
    IntenseBlue,
    IntensePurple,
    IntenseCyan,
    IntenseWhite,
}

impl std::fmt::Display for ColourAttribute {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let code = match self {
            ColourAttribute::Bold => "1",
            ColourAttribute::Dim => "2",
            ColourAttribute::Italic => "3",
            ColourAttribute::Underline => "4",
        };
        write!(f, "\x1b[{code}m")
    }
}

impl std::fmt::Display for BGColour {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let code = match self {
            // background
            BGColour::Black => "40",
            BGColour::Red => "41",
            BGColour::Green => "42",
            BGColour::Yellow => "43",
            BGColour::Blue => "44",
            BGColour::Purple => "45",
            BGColour::Cyan => "46",
            BGColour::White => "47",
            // High Intensity backgrounds
            BGColour::IntenseBlack => "100",
            BGColour::IntenseRed => "101",
            BGColour::IntenseGreen => "102",
            BGColour::IntenseYellow => "103",
            BGColour::IntenseBlue => "104",
            BGColour::IntensePurple => "105",
            BGColour::IntenseCyan => "106",
            BGColour::IntenseWhite => "107",
        };
        write!(f, "\x1b[{code}m")
    }
}

impl std::fmt::Display for Colour {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let code = match self {
            // regular
            Colour::Black => "30",
            Colour::Red => "31",
            Colour::Green => "32",
            Colour::Yellow => "33",
            Colour::Blue => "34",
            Colour::Purple => "35",
            Colour::Cyan => "36",
            Colour::White => "37",
            //High Intensity
            Colour::IntenseBlack => "90",
            Colour::IntenseRed => "91",
            Colour::IntenseGreen => "92",
            Colour::IntenseYellow => "93",
            Colour::IntenseBlue => "94",
            Colour::IntensePurple => "95",
            Colour::IntenseCyan => "96",
            Colour::IntenseWhite => "97",
        };
        write!(f, "\x1b[{code}m")
    }
}

type AnsiEscape = String;

impl From<Colour> for AnsiEscape {
    fn from(value: Colour) -> Self {
        format!("{value}")
    }
}

impl From<&Colour> for AnsiEscape {
    fn from(value: &Colour) -> Self {
        format!("{value}")
    }
}

impl From<BGColour> for AnsiEscape {
    fn from(value: BGColour) -> Self {
        format!("{value}")
    }
}

impl From<&BGColour> for AnsiEscape {
    fn from(value: &BGColour) -> Self {
        format!("{value}")
    }
}

impl From<ColourAttribute> for AnsiEscape {
    fn from(value: ColourAttribute) -> Self {
        format!("{value}")
    }
}

impl From<&ColourAttribute> for AnsiEscape {
    fn from(value: &ColourAttribute) -> Self {
        format!("{value}")
    }
}

pub fn colour<T: Into<AnsiEscape> + std::fmt::Display>(
    c: T,
    text: impl Into<String> + std::fmt::Display,
) -> String {
    format!("{c}{text}\x1b[0m")
}
