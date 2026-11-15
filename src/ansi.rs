use crate::color::Color;
use crate::style::Style;
use crate::text::Text;
use regex::Regex;
use std::sync::LazyLock;

static RE_ANSI: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:\x1b\[([0-9;]*)m)|(?:\x1b\]8;[^;\x1b\x07]*;([^\x1b\x07]*)(?:\x1b\\|\x07))|(?:\x1b\][^\x1b\x07]*(?:\x1b\\|\x07))|(?:\x1b\[[0-9;?]*[a-zA-Z])").unwrap()
});

/// Decodes raw terminal output containing ANSI escape sequences into styled `Text`.
#[derive(Clone, Debug, Default)]
pub struct AnsiDecoder {
    current_style: Style,
}

impl AnsiDecoder {
    pub fn new() -> Self {
        Self {
            current_style: Style::null(),
        }
    }

    /// Decode terminal text into a single `Text` instance with preserved styles.
    pub fn decode(&mut self, text: &str) -> Text {
        let mut result = Text::new();
        let mut last_idx = 0;

        for caps in RE_ANSI.captures_iter(text) {
            let mat = caps.get(0).unwrap();
            let start = mat.start();
            let end = mat.end();

            if start > last_idx {
                let slice = &text[last_idx..start];
                let st = if self.current_style.is_null() {
                    None
                } else {
                    Some(self.current_style.clone())
                };
                result.append(slice, st);
            }

            if let Some(sgr) = caps.get(1) {
                self.apply_sgr(sgr.as_str());
            } else if let Some(url) = caps.get(2) {
                let url_str = url.as_str();
                if url_str.is_empty() {
                    self.current_style.link = None;
                } else {
                    self.current_style.link = Some(url_str.to_string());
                }
            }

            last_idx = end;
        }

        if last_idx < text.len() {
            let slice = &text[last_idx..];
            let st = if self.current_style.is_null() {
                None
            } else {
                Some(self.current_style.clone())
            };
            result.append(slice, st);
        }

        result
    }

    /// Decode a single line of ANSI text.
    pub fn decode_line(&mut self, line: &str) -> Text {
        self.decode(line)
    }

    fn apply_sgr(&mut self, sgr: &str) {
        if sgr.is_empty() || sgr == "0" {
            self.current_style = Style::null();
            return;
        }

        let parts: Vec<u8> = sgr
            .split(';')
            .filter_map(|s| s.parse::<u8>().ok())
            .collect();

        let mut st = std::mem::take(&mut self.current_style);
        let mut idx = 0;
        while idx < parts.len() {
            let code = parts[idx];
            match code {
                0 => st = Style::null(),
                1 => st = st.bold(),
                2 => st = st.dim(),
                3 => st = st.italic(),
                4 => st = st.underline(),
                7 => st = st.reverse(),
                9 => st = st.strike(),
                22 => {
                    st.attributes.remove(crate::style::StyleAttributes::BOLD);
                    st.attributes.remove(crate::style::StyleAttributes::DIM);
                }
                23 => {
                    st.attributes.remove(crate::style::StyleAttributes::ITALIC);
                }
                24 => {
                    st.attributes
                        .remove(crate::style::StyleAttributes::UNDERLINE);
                }
                30..=37 => {
                    st = st.color(Color::from_ansi(code - 30));
                }
                39 => {
                    st.color = None;
                }
                40..=47 => {
                    st = st.bgcolor(Color::from_ansi(code - 40));
                }
                49 => {
                    st.bgcolor = None;
                }
                90..=97 => {
                    st = st.color(Color::from_ansi(code - 90 + 8));
                }
                100..=107 => {
                    st = st.bgcolor(Color::from_ansi(code - 100 + 8));
                }
                38 => {
                    if idx + 2 < parts.len() && parts[idx + 1] == 5 {
                        let color_num = parts[idx + 2];
                        st = st.color(Color::from_ansi(color_num));
                        idx += 2;
                    } else if idx + 4 < parts.len() && parts[idx + 1] == 2 {
                        let (r, g, b) = (parts[idx + 2], parts[idx + 3], parts[idx + 4]);
                        st = st.color(Color::from_rgb(r, g, b));
                        idx += 4;
                    }
                }
                48 => {
                    if idx + 2 < parts.len() && parts[idx + 1] == 5 {
                        let color_num = parts[idx + 2];
                        st = st.bgcolor(Color::from_ansi(color_num));
                        idx += 2;
                    } else if idx + 4 < parts.len() && parts[idx + 1] == 2 {
                        let (r, g, b) = (parts[idx + 2], parts[idx + 3], parts[idx + 4]);
                        st = st.bgcolor(Color::from_rgb(r, g, b));
                        idx += 4;
                    }
                }
                _ => {}
            }
            idx += 1;
        }

        self.current_style = st;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_decoder_simple() {
        let mut decoder = AnsiDecoder::new();
        let raw = "\x1b[1mBold\x1b[0m \x1b[31mRed\x1b[0m";
        let text = decoder.decode(raw);

        assert_eq!(text.plain, "Bold Red");
        assert_eq!(text.spans.len(), 2);
        assert!(
            text.spans[0]
                .style
                .attributes
                .contains(crate::style::StyleAttributes::BOLD)
        );
        assert!(text.spans[1].style.color.is_some());
    }

    #[test]
    fn test_ansi_decoder_truecolor() {
        let mut decoder = AnsiDecoder::new();
        let raw = "\x1b[38;2;255;128;0mOrange\x1b[0m";
        let text = decoder.decode(raw);

        assert_eq!(text.plain, "Orange");
        let color = text.spans[0].style.color.as_ref().unwrap();
        let triplet = color.get_truecolor();
        assert_eq!(triplet.0, 255);
        assert_eq!(triplet.1, 128);
        assert_eq!(triplet.2, 0);
    }

    #[test]
    fn test_ansi_decoder_osc8_link() {
        let mut decoder = AnsiDecoder::new();
        let raw = "\x1b]8;;https://github.com\x1b\\Click Here\x1b]8;;\x1b\\";
        let text = decoder.decode(raw);

        assert_eq!(text.plain, "Click Here");
        assert_eq!(text.spans.len(), 1);
        assert_eq!(
            text.spans[0].style.link.as_deref(),
            Some("https://github.com")
        );
    }
}
