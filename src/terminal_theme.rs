use crate::color::ColorTriplet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalTheme {
    pub background: ColorTriplet,
    pub foreground: ColorTriplet,
    pub ansi_colors: [ColorTriplet; 16],
}

impl Default for TerminalTheme {
    fn default() -> Self {
        Self::dimmed_monokai()
    }
}

impl TerminalTheme {
    pub const fn dimmed_monokai() -> Self {
        Self {
            background: ColorTriplet(30, 30, 30),
            foreground: ColorTriplet(185, 188, 186),
            ansi_colors: [
                ColorTriplet(0, 0, 0),       // black
                ColorTriplet(190, 63, 72),   // red
                ColorTriplet(135, 154, 59),  // green
                ColorTriplet(201, 162, 79),  // yellow
                ColorTriplet(78, 126, 179),  // blue
                ColorTriplet(143, 107, 167), // magenta
                ColorTriplet(78, 154, 158),  // cyan
                ColorTriplet(197, 200, 198), // white
                ColorTriplet(80, 80, 80),    // bright black
                ColorTriplet(222, 102, 110), // bright red
                ColorTriplet(168, 187, 92),  // bright green
                ColorTriplet(234, 195, 112), // bright yellow
                ColorTriplet(111, 159, 212), // bright blue
                ColorTriplet(176, 140, 200), // bright magenta
                ColorTriplet(111, 187, 191), // bright cyan
                ColorTriplet(255, 255, 255), // bright white
            ],
        }
    }
}
