use crate::errors::RurichError;
use std::collections::HashMap;
use std::fmt;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorTriplet(pub u8, pub u8, pub u8);

impl ColorTriplet {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }

    pub fn hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }

    pub fn rgb(&self) -> String {
        format!("rgb({}, {}, {})", self.0, self.1, self.2)
    }

    pub fn blend(&self, other: &ColorTriplet, factor: f32) -> ColorTriplet {
        let f = factor.clamp(0.0, 1.0);
        let r = ((self.0 as f32 * (1.0 - f)) + (other.0 as f32 * f)).round() as u8;
        let g = ((self.1 as f32 * (1.0 - f)) + (other.1 as f32 * f)).round() as u8;
        let b = ((self.2 as f32 * (1.0 - f)) + (other.2 as f32 * f)).round() as u8;
        ColorTriplet(r, g, b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorType {
    Standard = 1,
    EightBit = 2,
    TrueColor = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorSystem {
    NoColor = 0,
    Standard = 1,
    EightBit = 2,
    TrueColor = 3,
    Windows = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub color_type: ColorType,
    pub number: Option<u8>,
    pub triplet: Option<ColorTriplet>,
}

impl Color {
    pub const fn from_ansi(number: u8) -> Self {
        if number < 16 {
            Self {
                color_type: ColorType::Standard,
                number: Some(number),
                triplet: None,
            }
        } else {
            Self {
                color_type: ColorType::EightBit,
                number: Some(number),
                triplet: None,
            }
        }
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            color_type: ColorType::TrueColor,
            number: None,
            triplet: Some(ColorTriplet(r, g, b)),
        }
    }

    pub const fn from_triplet(triplet: ColorTriplet) -> Self {
        Self {
            color_type: ColorType::TrueColor,
            number: None,
            triplet: Some(triplet),
        }
    }

    pub fn get_truecolor(&self) -> ColorTriplet {
        if let Some(triplet) = self.triplet {
            return triplet;
        }
        if let Some(n) = self.number {
            return ansi256_to_rgb(n);
        }
        ColorTriplet(192, 192, 192)
    }

    /// Parse a color string: named color, "#ffffff", "#fff", "rgb(r,g,b)", or "color(n)"
    pub fn parse(s: &str) -> Result<Self, RurichError> {
        let trimmed = s.trim().to_lowercase();
        if trimmed.is_empty() || trimmed == "default" || trimmed == "none" {
            return Err(RurichError::ColorParseError(
                "Empty or default color".into(),
            ));
        }

        // Hex format: #rrggbb or #rgb
        if let Some(hex) = trimmed.strip_prefix('#') {
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| {
                    RurichError::ColorParseError(format!("Invalid hex color: #{hex}"))
                })?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| {
                    RurichError::ColorParseError(format!("Invalid hex color: #{hex}"))
                })?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| {
                    RurichError::ColorParseError(format!("Invalid hex color: #{hex}"))
                })?;
                return Ok(Self::from_rgb(r, g, b));
            } else if hex.len() == 3 {
                let r_char = &hex[0..1];
                let g_char = &hex[1..2];
                let b_char = &hex[2..3];
                let r = u8::from_str_radix(&format!("{r_char}{r_char}"), 16).map_err(|_| {
                    RurichError::ColorParseError(format!("Invalid hex color: #{hex}"))
                })?;
                let g = u8::from_str_radix(&format!("{g_char}{g_char}"), 16).map_err(|_| {
                    RurichError::ColorParseError(format!("Invalid hex color: #{hex}"))
                })?;
                let b = u8::from_str_radix(&format!("{b_char}{b_char}"), 16).map_err(|_| {
                    RurichError::ColorParseError(format!("Invalid hex color: #{hex}"))
                })?;
                return Ok(Self::from_rgb(r, g, b));
            } else {
                return Err(RurichError::ColorParseError(format!(
                    "Invalid hex color: #{hex}"
                )));
            }
        }

        // rgb(r, g, b) format
        if let Some(rgb_inner) = trimmed
            .strip_prefix("rgb(")
            .and_then(|s| s.strip_suffix(')'))
        {
            let parts: Vec<&str> = rgb_inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 3 {
                let r: u8 = parts[0]
                    .parse()
                    .map_err(|_| RurichError::ColorParseError(s.into()))?;
                let g: u8 = parts[1]
                    .parse()
                    .map_err(|_| RurichError::ColorParseError(s.into()))?;
                let b: u8 = parts[2]
                    .parse()
                    .map_err(|_| RurichError::ColorParseError(s.into()))?;
                return Ok(Self::from_rgb(r, g, b));
            }
            return Err(RurichError::ColorParseError(format!(
                "Invalid rgb() format: {s}"
            )));
        }

        // color(n), color(#hex), color(rgb(...)) format
        if let Some(color_inner) = trimmed
            .strip_prefix("color(")
            .and_then(|s| s.strip_suffix(')'))
        {
            let inner = color_inner.trim();
            if let Ok(n) = inner.parse::<u8>() {
                return Ok(Self::from_ansi(n));
            }
            return Self::parse(inner);
        }

        // Check ANSI color names
        if let Some(&ansi_num) = ANSI_COLOR_NAMES.get(trimmed.as_str()) {
            return Ok(Self::from_ansi(ansi_num));
        }

        Err(RurichError::ColorParseError(format!("Unknown color: {s}")))
    }

    /// Convert color to SGR ANSI parameter sequence
    pub fn get_ansi_codes(&self, foreground: bool) -> Vec<String> {
        match self.color_type {
            ColorType::Standard => {
                let n = self.number.unwrap_or(0);
                let base = if foreground {
                    if n < 8 { 30 + n } else { 90 + (n - 8) }
                } else if n < 8 {
                    40 + n
                } else {
                    100 + (n - 8)
                };
                vec![base.to_string()]
            }
            ColorType::EightBit => {
                let n = self.number.unwrap_or(0);
                vec![
                    if foreground {
                        "38".to_string()
                    } else {
                        "48".to_string()
                    },
                    "5".to_string(),
                    n.to_string(),
                ]
            }
            ColorType::TrueColor => {
                let triplet = self.triplet.unwrap_or(ColorTriplet(0, 0, 0));
                vec![
                    if foreground {
                        "38".to_string()
                    } else {
                        "48".to_string()
                    },
                    "2".to_string(),
                    triplet.0.to_string(),
                    triplet.1.to_string(),
                    triplet.2.to_string(),
                ]
            }
        }
    }

    /// Downgrade color to match a destination color system
    pub fn downgrade(&self, system: ColorSystem) -> Option<Self> {
        match system {
            ColorSystem::NoColor => None,
            ColorSystem::TrueColor => Some(*self),
            ColorSystem::EightBit => match self.color_type {
                ColorType::TrueColor => {
                    let triplet = self.triplet.unwrap_or(ColorTriplet(0, 0, 0));
                    let code = rgb_to_256(triplet.0, triplet.1, triplet.2);
                    Some(Self::from_ansi(code))
                }
                _ => Some(*self),
            },
            ColorSystem::Standard | ColorSystem::Windows => {
                let ansi_16 = match self.color_type {
                    ColorType::Standard => self.number.unwrap_or(0),
                    ColorType::EightBit => ansi256_to_16(self.number.unwrap_or(0)),
                    ColorType::TrueColor => {
                        let triplet = self.triplet.unwrap_or(ColorTriplet(0, 0, 0));
                        let c256 = rgb_to_256(triplet.0, triplet.1, triplet.2);
                        ansi256_to_16(c256)
                    }
                };
                Some(Self::from_ansi(ansi_16))
            }
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.color_type {
            ColorType::Standard => write!(f, "color({})", self.number.unwrap_or(0)),
            ColorType::EightBit => write!(f, "color({})", self.number.unwrap_or(0)),
            ColorType::TrueColor => {
                let t = self.triplet.unwrap_or(ColorTriplet(0, 0, 0));
                write!(f, "{}", t.hex())
            }
        }
    }
}

/// Convert 24-bit RGB to closest 8-bit ANSI 256 color
fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
    // Check if it's grayscale
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return ((((r as f32 - 8.0) / 247.0) * 24.0).round() as u8) + 232;
    }

    // 6x6x6 color cube: 16 + 36 * r + 6 * g + b
    let to_cube = |v: u8| -> u8 {
        if v < 48 {
            0
        } else if v < 115 {
            1
        } else {
            ((v as f32 - 35.0) / 40.0).clamp(1.0, 5.0).round() as u8
        }
    };

    16 + 36 * to_cube(r) + 6 * to_cube(g) + to_cube(b)
}

/// Convert 256 color code to 16 ANSI color code
fn ansi256_to_rgb(n: u8) -> ColorTriplet {
    const STANDARD_16: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];

    if (n as usize) < STANDARD_16.len() {
        let (r, g, b) = STANDARD_16[n as usize];
        return ColorTriplet(r, g, b);
    }

    if (16..=231).contains(&n) {
        let n = n - 16;
        let r_idx = ((n / 36) % 6) as usize;
        let g_idx = ((n / 6) % 6) as usize;
        let b_idx = (n % 6) as usize;
        const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];
        return ColorTriplet(STEPS[r_idx], STEPS[g_idx], STEPS[b_idx]);
    }

    // 232..=255 grayscale
    let gray = 8 + (n.saturating_sub(232)) * 10;
    ColorTriplet(gray, gray, gray)
}

fn ansi256_to_16(n: u8) -> u8 {
    if n < 16 {
        return n;
    }
    if (232..=255).contains(&n) {
        return if n < 244 { 0 } else { 7 };
    }
    let idx = n - 16;
    let b = idx % 6;
    let g = (idx / 6) % 6;
    let r = idx / 36;
    let bright = if r > 2 || g > 2 || b > 2 { 8 } else { 0 };
    let code =
        (if r > 0 { 1 } else { 0 }) | (if g > 0 { 2 } else { 0 }) | (if b > 0 { 4 } else { 0 });
    code + bright
}

pub static ANSI_COLOR_NAMES: LazyLock<HashMap<&'static str, u8>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(250);
    let colors: &[(&'static str, u8)] = &[
        ("black", 0),
        ("red", 1),
        ("green", 2),
        ("yellow", 3),
        ("blue", 4),
        ("magenta", 5),
        ("cyan", 6),
        ("white", 7),
        ("orange", 214),
        ("bright_black", 8),
        ("bright_red", 9),
        ("bright_green", 10),
        ("bright_yellow", 11),
        ("bright_blue", 12),
        ("bright_magenta", 13),
        ("bright_cyan", 14),
        ("bright_white", 15),
        ("grey0", 16),
        ("gray0", 16),
        ("navy_blue", 17),
        ("dark_blue", 18),
        ("blue3", 20),
        ("blue1", 21),
        ("dark_green", 22),
        ("deep_sky_blue4", 25),
        ("dodger_blue3", 26),
        ("dodger_blue2", 27),
        ("green4", 28),
        ("spring_green4", 29),
        ("turquoise4", 30),
        ("deep_sky_blue3", 31),
        ("deep_sky_blue2", 38),
        ("dodger_blue1", 33),
        ("green3", 40),
        ("spring_green3", 41),
        ("dark_cyan", 36),
        ("light_sea_green", 37),
        ("deep_sky_blue1", 39),
        ("green1", 46),
        ("spring_green2", 47),
        ("spring_green1", 48),
        ("cyan3", 43),
        ("dark_turquoise", 44),
        ("turquoise2", 45),
        ("cyan2", 50),
        ("cyan1", 51),
        ("dark_red", 52),
        ("deep_pink4", 53),
        ("purple4", 54),
        ("purple3", 56),
        ("blue_violet", 57),
        ("orange4", 58),
        ("grey37", 59),
        ("medium_purple4", 60),
        ("slate_blue3", 61),
        ("slate_blue1", 63),
        ("wheat4", 101),
        ("cornflower_blue", 68),
        ("chartreuse4", 64),
        ("dark_sea_green4", 65),
        ("cadet_blue", 72),
        ("sky_blue3", 74),
        ("steel_blue1", 75),
        ("chartreuse3", 76),
        ("sea_green3", 78),
        ("aquamarine3", 79),
        ("dark_slate_gray2", 87),
        ("slate_blue3b", 62),
        ("sky_blue2", 111),
        ("sky_blue1", 117),
        ("chartreuse2", 112),
        ("sea_green2", 84),
        ("sea_green1", 85),
        ("aquamarine1", 122),
        ("dark_slate_gray1", 123),
        ("dark_red_b", 88),
        ("deep_pink4_b", 89),
        ("purple", 129),
        ("medium_purple3", 97),
        ("slate_blue1b", 99),
        ("yellow4", 106),
        ("grey53", 102),
        ("light_slate_grey", 103),
        ("medium_purple", 104),
        ("light_slate_blue", 105),
        ("dark_olive_green3", 113),
        ("dark_sea_green", 108),
        ("light_sky_blue3", 110),
        ("sky_blue3b", 116),
        ("chartreuse2b", 118),
        ("dark_olive_green2", 155),
        ("pale_green1", 120),
        ("dark_sea_green2", 157),
        ("dark_sea_green1", 158),
        ("deep_pink4c", 132),
        ("medium_orchid3", 133),
        ("medium_orchid", 134),
        ("medium_purple2", 139),
        ("dark_goldenrod", 136),
        ("light_salmon3", 137),
        ("rosy_brown", 138),
        ("grey63", 139),
        ("medium_purple1", 141),
        ("gold3", 142),
        ("dark_khaki", 143),
        ("navajo_white3", 144),
        ("grey69", 145),
        ("light_steel_blue3", 147),
        ("light_steel_blue", 147),
        ("yellow3", 149),
        ("dark_olive_green1", 192),
        ("khaki3", 185),
        ("light_goldenrod3", 186),
        ("light_yellow3", 187),
        ("grey84", 188),
        ("light_steel_blue1", 189),
        ("yellow2", 190),
        ("dark_olive_green1b", 191),
        ("green_yellow", 154),
        ("honeydew2", 194),
        ("indian_red1", 203),
        ("deep_pink3", 162),
        ("magenta3", 164),
        ("medium_orchid1", 207),
        ("dark_orange3", 166),
        ("indian_red", 167),
        ("hot_pink3", 168),
        ("hot_pink2", 169),
        ("orchid", 170),
        ("medium_orchid1b", 171),
        ("orange3", 172),
        ("light_salmon2", 173),
        ("light_pink3", 174),
        ("pink3", 175),
        ("plum3", 176),
        ("violet", 177),
        ("gold3b", 178),
        ("light_goldenrod3b", 179),
        ("tan", 180),
        ("misty_rose3", 181),
        ("thistle3", 182),
        ("plum2", 183),
        ("yellow3b", 184),
        ("khaki3b", 185),
        ("light_goldenrod2", 221),
        ("light_yellow2", 222),
        ("grey85", 223),
        ("light_goldenrod1", 227),
        ("grey89", 225),
        ("grey93", 226),
        ("red3", 160),
        ("deep_pink2", 197),
        ("magenta2", 200),
        ("magenta1", 201),
        ("orange_red1", 202),
        ("indian_red1b", 204),
        ("orchid2", 205),
        ("medium_orchid1c", 206),
        ("dark_orange", 208),
        ("salmon1", 209),
        ("light_coral", 210),
        ("pale_violet_red1", 211),
        ("orchid1", 213),
        ("orange1", 214),
        ("sandy_brown", 215),
        ("light_salmon1", 216),
        ("light_pink1", 217),
        ("plum1", 219),
        ("gold1", 220),
        ("navajo_white1", 223),
        ("wheat1", 224),
        ("grey100", 231),
        ("grey3", 232),
        ("grey7", 233),
        ("grey11", 234),
        ("grey15", 235),
        ("grey19", 236),
        ("grey23", 237),
        ("grey27", 238),
        ("grey30", 239),
        ("grey35", 240),
        ("grey39", 241),
        ("grey42", 242),
        ("grey46", 243),
        ("grey50", 244),
        ("grey54", 245),
        ("grey58", 246),
        ("grey62", 247),
        ("grey66", 248),
        ("grey70", 249),
        ("grey74", 250),
        ("grey78", 251),
        ("grey82", 252),
        ("grey86", 253),
        ("grey90", 254),
        ("grey93b", 255),
    ];
    for &(name, num) in colors {
        m.insert(name, num);
    }
    m
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_parse_hex() {
        let c = Color::parse("#ff0000").unwrap();
        assert_eq!(c.color_type, ColorType::TrueColor);
        assert_eq!(c.triplet, Some(ColorTriplet(255, 0, 0)));

        let c2 = Color::parse("#0f0").unwrap();
        assert_eq!(c2.triplet, Some(ColorTriplet(0, 255, 0)));
    }

    #[test]
    fn test_color_parse_rgb() {
        let c = Color::parse("rgb(10, 20, 30)").unwrap();
        assert_eq!(c.triplet, Some(ColorTriplet(10, 20, 30)));
    }

    #[test]
    fn test_color_parse_named() {
        let c = Color::parse("red").unwrap();
        assert_eq!(c.color_type, ColorType::Standard);
        assert_eq!(c.number, Some(1));

        let c2 = Color::parse("bright_green").unwrap();
        assert_eq!(c2.number, Some(10));
    }

    #[test]
    fn test_ansi_codes() {
        let red = Color::parse("red").unwrap();
        assert_eq!(red.get_ansi_codes(true), vec!["31"]);
        assert_eq!(red.get_ansi_codes(false), vec!["41"]);

        let bright_red = Color::parse("bright_red").unwrap();
        assert_eq!(bright_red.get_ansi_codes(true), vec!["91"]);

        let tc = Color::from_rgb(10, 20, 30);
        assert_eq!(tc.get_ansi_codes(true), vec!["38", "2", "10", "20", "30"]);
        assert_eq!(tc.get_ansi_codes(false), vec!["48", "2", "10", "20", "30"]);
    }
}
