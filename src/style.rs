use crate::color::{Color, ColorSystem};
use crate::errors::RurichError;
use bitflags::bitflags;
use std::fmt;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct StyleAttributes: u16 {
        const BOLD             = 1 << 0;  // 1 (off 22)
        const DIM              = 1 << 1;  // 2 (off 22)
        const ITALIC           = 1 << 2;  // 3 (off 23)
        const UNDERLINE        = 1 << 3;  // 4 (off 24)
        const BLINK            = 1 << 4;  // 5 (off 25)
        const RAPID_BLINK      = 1 << 5;  // 6 (off 25)
        const REVERSE          = 1 << 6;  // 7 (off 27)
        const CONCEAL          = 1 << 7;  // 8 (off 28)
        const STRIKE           = 1 << 8;  // 9 (off 29)
        const DOUBLE_UNDERLINE = 1 << 9;  // 21 (off 24)
        const OVERLINE         = 1 << 10; // 53 (off 55)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Style {
    pub color: Option<Color>,
    pub bgcolor: Option<Color>,
    pub attributes: StyleAttributes,
    pub set_attributes: StyleAttributes,
    pub link: Option<String>,
}

impl Style {
    pub const fn new() -> Self {
        Self {
            color: None,
            bgcolor: None,
            attributes: StyleAttributes::empty(),
            set_attributes: StyleAttributes::empty(),
            link: None,
        }
    }

    pub const fn null() -> Self {
        Self::new()
    }

    pub fn is_null(&self) -> bool {
        self.color.is_none()
            && self.bgcolor.is_none()
            && self.set_attributes.is_empty()
            && self.link.is_none()
    }

    // Builder methods
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn bgcolor(mut self, bgcolor: Color) -> Self {
        self.bgcolor = Some(bgcolor);
        self
    }

    pub fn background(self, color: Color) -> Self {
        self.bgcolor(color)
    }

    pub fn link(mut self, link: impl Into<String>) -> Self {
        self.link = Some(link.into());
        self
    }

    pub fn bold(mut self) -> Self {
        self.attributes.insert(StyleAttributes::BOLD);
        self.set_attributes.insert(StyleAttributes::BOLD);
        self
    }

    pub fn dim(mut self) -> Self {
        self.attributes.insert(StyleAttributes::DIM);
        self.set_attributes.insert(StyleAttributes::DIM);
        self
    }

    pub fn italic(mut self) -> Self {
        self.attributes.insert(StyleAttributes::ITALIC);
        self.set_attributes.insert(StyleAttributes::ITALIC);
        self
    }

    pub fn underline(mut self) -> Self {
        self.attributes.insert(StyleAttributes::UNDERLINE);
        self.set_attributes.insert(StyleAttributes::UNDERLINE);
        self
    }

    pub fn strike(mut self) -> Self {
        self.attributes.insert(StyleAttributes::STRIKE);
        self.set_attributes.insert(StyleAttributes::STRIKE);
        self
    }

    pub fn reverse(mut self) -> Self {
        self.attributes.insert(StyleAttributes::REVERSE);
        self.set_attributes.insert(StyleAttributes::REVERSE);
        self
    }

    pub fn blink(mut self) -> Self {
        self.attributes.insert(StyleAttributes::BLINK);
        self.set_attributes.insert(StyleAttributes::BLINK);
        self
    }

    pub fn overline(mut self) -> Self {
        self.attributes.insert(StyleAttributes::OVERLINE);
        self.set_attributes.insert(StyleAttributes::OVERLINE);
        self
    }

    /// Combine this style with another (other overrides set fields in this)
    pub fn combine(&self, other: &Self) -> Self {
        let mut new_style = self.clone();

        if other.color.is_some() {
            new_style.color = other.color;
        }
        if other.bgcolor.is_some() {
            new_style.bgcolor = other.bgcolor;
        }
        if other.link.is_some() {
            new_style.link = other.link.clone();
        }

        // Apply attribute overrides
        let unset = other.set_attributes - other.attributes;
        new_style.attributes.remove(unset);
        new_style.attributes.insert(other.attributes);
        new_style.set_attributes.insert(other.set_attributes);

        new_style
    }

    /// Parse style string, e.g. "bold red on white link https://example.com"
    pub fn parse(s: &str) -> Result<Self, RurichError> {
        let s = s.trim();
        if s.is_empty() || s == "none" {
            return Ok(Self::null());
        }

        let mut style = Self::new();
        let mut words = s.split_whitespace().peekable();

        while let Some(word) = words.next() {
            let lower = word.to_lowercase();
            match lower.as_str() {
                "on" => {
                    let bg_word = words.next().ok_or_else(|| {
                        RurichError::StyleSyntaxError("Color expected after 'on'".into())
                    })?;
                    let color = Color::parse(bg_word)?;
                    style.bgcolor = Some(color);
                }
                "link" => {
                    let link_url = words.next().ok_or_else(|| {
                        RurichError::StyleSyntaxError("URL expected after 'link'".into())
                    })?;
                    style.link = Some(link_url.to_string());
                }
                "bold" | "b" => {
                    style.attributes.insert(StyleAttributes::BOLD);
                    style.set_attributes.insert(StyleAttributes::BOLD);
                }
                "not" => {
                    let next_word = words.next().ok_or_else(|| {
                        RurichError::StyleSyntaxError("Attribute expected after 'not'".into())
                    })?;
                    match next_word.to_lowercase().as_str() {
                        "bold" | "b" => {
                            style.attributes.remove(StyleAttributes::BOLD);
                            style.set_attributes.insert(StyleAttributes::BOLD);
                        }
                        "dim" | "d" => {
                            style.attributes.remove(StyleAttributes::DIM);
                            style.set_attributes.insert(StyleAttributes::DIM);
                        }
                        "italic" | "i" => {
                            style.attributes.remove(StyleAttributes::ITALIC);
                            style.set_attributes.insert(StyleAttributes::ITALIC);
                        }
                        "underline" | "u" => {
                            style.attributes.remove(StyleAttributes::UNDERLINE);
                            style.set_attributes.insert(StyleAttributes::UNDERLINE);
                        }
                        "strike" | "s" => {
                            style.attributes.remove(StyleAttributes::STRIKE);
                            style.set_attributes.insert(StyleAttributes::STRIKE);
                        }
                        "reverse" | "r" => {
                            style.attributes.remove(StyleAttributes::REVERSE);
                            style.set_attributes.insert(StyleAttributes::REVERSE);
                        }
                        "blink" => {
                            style.attributes.remove(StyleAttributes::BLINK);
                            style.set_attributes.insert(StyleAttributes::BLINK);
                        }
                        "overline" | "o" => {
                            style.attributes.remove(StyleAttributes::OVERLINE);
                            style.set_attributes.insert(StyleAttributes::OVERLINE);
                        }
                        other => {
                            return Err(RurichError::StyleSyntaxError(format!(
                                "Unknown attribute after 'not': {other}"
                            )));
                        }
                    }
                }
                "dim" | "d" => {
                    style.attributes.insert(StyleAttributes::DIM);
                    style.set_attributes.insert(StyleAttributes::DIM);
                }
                "italic" | "i" => {
                    style.attributes.insert(StyleAttributes::ITALIC);
                    style.set_attributes.insert(StyleAttributes::ITALIC);
                }
                "underline" | "u" => {
                    style.attributes.insert(StyleAttributes::UNDERLINE);
                    style.set_attributes.insert(StyleAttributes::UNDERLINE);
                }
                "strike" | "s" => {
                    style.attributes.insert(StyleAttributes::STRIKE);
                    style.set_attributes.insert(StyleAttributes::STRIKE);
                }
                "reverse" | "r" => {
                    style.attributes.insert(StyleAttributes::REVERSE);
                    style.set_attributes.insert(StyleAttributes::REVERSE);
                }
                "blink" => {
                    style.attributes.insert(StyleAttributes::BLINK);
                    style.set_attributes.insert(StyleAttributes::BLINK);
                }
                "overline" | "o" => {
                    style.attributes.insert(StyleAttributes::OVERLINE);
                    style.set_attributes.insert(StyleAttributes::OVERLINE);
                }
                _ => {
                    // Try parsing as foreground color
                    let color = Color::parse(word)?;
                    style.color = Some(color);
                }
            }
        }

        Ok(style)
    }

    /// Render SGR ANSI codes for this style
    pub fn render_sgr(&self, color_system: ColorSystem) -> String {
        if color_system == ColorSystem::NoColor || self.is_null() {
            return String::new();
        }

        let mut codes: Vec<String> = Vec::new();

        if self.attributes.contains(StyleAttributes::BOLD) {
            codes.push("1".into());
        }
        if self.attributes.contains(StyleAttributes::DIM) {
            codes.push("2".into());
        }
        if self.attributes.contains(StyleAttributes::ITALIC) {
            codes.push("3".into());
        }
        if self.attributes.contains(StyleAttributes::UNDERLINE) {
            codes.push("4".into());
        }
        if self.attributes.contains(StyleAttributes::BLINK) {
            codes.push("5".into());
        }
        if self.attributes.contains(StyleAttributes::REVERSE) {
            codes.push("7".into());
        }
        if self.attributes.contains(StyleAttributes::CONCEAL) {
            codes.push("8".into());
        }
        if self.attributes.contains(StyleAttributes::STRIKE) {
            codes.push("9".into());
        }
        if self.attributes.contains(StyleAttributes::DOUBLE_UNDERLINE) {
            codes.push("21".into());
        }
        if self.attributes.contains(StyleAttributes::OVERLINE) {
            codes.push("53".into());
        }

        if let Some(ref color) = self.color
            && let Some(downgraded) = color.downgrade(color_system)
        {
            codes.extend(downgraded.get_ansi_codes(true));
        }

        if let Some(ref bgcolor) = self.bgcolor
            && let Some(downgraded) = bgcolor.downgrade(color_system)
        {
            codes.extend(downgraded.get_ansi_codes(false));
        }

        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }

    /// Render opening hyperlink (OSC 8)
    pub fn render_link_open(&self) -> Option<String> {
        self.link.as_ref().map(|url| format!("\x1b]8;;{url}\x1b\\"))
    }

    /// Render closing hyperlink (OSC 8)
    pub fn render_link_close(&self) -> Option<String> {
        if self.link.is_some() {
            Some("\x1b]8;;\x1b\\".to_string())
        } else {
            None
        }
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.attributes.contains(StyleAttributes::BOLD) {
            parts.push("bold");
        }
        if self.attributes.contains(StyleAttributes::ITALIC) {
            parts.push("italic");
        }
        if self.attributes.contains(StyleAttributes::UNDERLINE) {
            parts.push("underline");
        }
        if self.color.is_some() {
            parts.push("color");
        }
        if self.bgcolor.is_some() {
            parts.push("on");
            parts.push("color");
        }
        write!(f, "Style({})", parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_parse_basic() {
        let s = Style::parse("bold red on blue").unwrap();
        assert!(s.attributes.contains(StyleAttributes::BOLD));
        assert_eq!(s.color.unwrap(), Color::parse("red").unwrap());
        assert_eq!(s.bgcolor.unwrap(), Color::parse("blue").unwrap());
    }

    #[test]
    fn test_style_combine() {
        let base = Style::parse("bold red").unwrap();
        let override_style = Style::parse("italic on green").unwrap();
        let combined = base.combine(&override_style);

        assert!(combined.attributes.contains(StyleAttributes::BOLD));
        assert!(combined.attributes.contains(StyleAttributes::ITALIC));
        assert_eq!(combined.color.unwrap(), Color::parse("red").unwrap());
        assert_eq!(combined.bgcolor.unwrap(), Color::parse("green").unwrap());
    }

    #[test]
    fn test_style_render_sgr() {
        let s = Style::parse("bold red on white").unwrap();
        let sgr = s.render_sgr(ColorSystem::Standard);
        assert_eq!(sgr, "\x1b[1;31;47m");
    }
}
