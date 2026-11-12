use crate::cells::{cell_len, split_cells};
use crate::color::ColorSystem;
use crate::control::ControlCode;
use crate::style::Style;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment<'a> {
    pub text: Cow<'a, str>,
    pub style: Option<Style>,
    pub control: Option<Vec<ControlCode>>,
}

impl<'a> Segment<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>, style: Option<Style>) -> Self {
        Self {
            text: text.into(),
            style,
            control: None,
        }
    }

    pub fn styled(text: impl Into<Cow<'a, str>>, style: Style) -> Self {
        Self::new(text, Some(style))
    }

    pub fn unstyled(text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(text, None)
    }

    pub fn control(control: Vec<ControlCode>) -> Self {
        Self {
            text: Cow::Borrowed(""),
            style: None,
            control: Some(control),
        }
    }

    pub fn line() -> Self {
        Self::unstyled("\n")
    }

    pub fn cell_length(&self) -> usize {
        if self.control.is_some() {
            0
        } else {
            cell_len(&self.text)
        }
    }

    pub fn is_control(&self) -> bool {
        self.control.is_some()
    }

    pub fn into_segment<'b>(self) -> Segment<'b> {
        Segment {
            text: Cow::Owned(self.text.into_owned()),
            style: self.style,
            control: self.control,
        }
    }

    /// Split segment at given cell cut position
    pub fn split_cells(&self, cut: usize) -> (Segment<'static>, Segment<'static>) {
        if self.is_control() {
            return (
                Segment {
                    text: Cow::Borrowed(""),
                    style: self.style.clone(),
                    control: self.control.clone(),
                },
                Segment {
                    text: Cow::Borrowed(""),
                    style: self.style.clone(),
                    control: None,
                },
            );
        }

        let (left_str, right_str) = split_cells(&self.text, cut);
        (
            Segment {
                text: Cow::Owned(left_str),
                style: self.style.clone(),
                control: None,
            },
            Segment {
                text: Cow::Owned(right_str),
                style: self.style.clone(),
                control: None,
            },
        )
    }

    /// Split segments into lines at '\n' boundaries
    pub fn split_lines(segments: impl IntoIterator<Item = Segment<'a>>) -> Vec<Vec<Segment<'a>>> {
        let mut lines = Vec::new();
        let mut current_line = Vec::new();

        for seg in segments {
            if seg.is_control() {
                current_line.push(seg);
                continue;
            }

            let text = &seg.text;
            if !text.contains('\n') {
                current_line.push(seg);
            } else {
                let parts: Vec<&str> = text.split('\n').collect();
                for (i, part) in parts.iter().enumerate() {
                    if !part.is_empty() {
                        current_line.push(Segment {
                            text: Cow::Owned(part.to_string()),
                            style: seg.style.clone(),
                            control: None,
                        });
                    }
                    if i + 1 < parts.len() {
                        lines.push(std::mem::take(&mut current_line));
                    }
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Get total cell width of a line of segments
    pub fn get_line_length(line: &[Segment<'a>]) -> usize {
        line.iter().map(|s| s.cell_length()).sum()
    }

    /// Adjust line length by cropping or padding
    pub fn adjust_line_length(
        mut line: Vec<Segment<'a>>,
        max_width: usize,
        style: Option<Style>,
        pad: bool,
    ) -> Vec<Segment<'a>> {
        let current_length = Self::get_line_length(&line);

        if current_length < max_width {
            if pad {
                let padding = max_width - current_length;
                line.push(Segment {
                    text: Cow::Owned(" ".repeat(padding)),
                    style,
                    control: None,
                });
            }
            return line;
        }

        if current_length > max_width {
            let mut cropped = Vec::new();
            let mut total_width = 0;

            for seg in line {
                let seg_len = seg.cell_length();
                if total_width + seg_len <= max_width {
                    total_width += seg_len;
                    cropped.push(seg);
                } else {
                    let needed = max_width - total_width;
                    if needed > 0 {
                        let (left, _) = seg.split_cells(needed);
                        cropped.push(left);
                    }
                    break;
                }
            }
            return cropped;
        }

        line
    }

    /// Split lines, crop at max_width, and optionally pad
    pub fn split_and_crop_lines(
        segments: impl IntoIterator<Item = Segment<'a>>,
        max_width: usize,
        include_new_lines: bool,
        pad: bool,
    ) -> Vec<Vec<Segment<'a>>> {
        let raw_lines = Self::split_lines(segments);
        let mut processed = Vec::with_capacity(raw_lines.len());

        for line in raw_lines {
            let mut line = Self::adjust_line_length(line, max_width, None, pad);
            if include_new_lines {
                line.push(Self::line());
            }
            processed.push(line);
        }

        processed
    }

    /// Combine adjacent segments that have the exact same style
    pub fn simplify(segments: impl IntoIterator<Item = Segment<'a>>) -> Vec<Segment<'a>> {
        let mut result: Vec<Segment<'a>> = Vec::new();

        for seg in segments {
            if seg.is_control() || seg.text.is_empty() {
                result.push(seg);
                continue;
            }

            if let Some(last) = result.last_mut()
                && !last.is_control()
                && last.style == seg.style
            {
                let mut combined = last.text.clone().into_owned();
                combined.push_str(&seg.text);
                last.text = Cow::Owned(combined);
                continue;
            }
            result.push(seg);
        }

        result
    }

    /// Strip all styles from segments
    pub fn strip_styles(segments: &[Segment<'a>]) -> String {
        let mut out = String::new();
        for seg in segments {
            if !seg.is_control() {
                out.push_str(&seg.text);
            }
        }
        out
    }

    /// Render a slice of segments to an ANSI string
    pub fn render_ansi(segments: &[Segment<'_>], color_system: ColorSystem) -> String {
        let mut output = String::new();

        for seg in segments {
            if let Some(ref controls) = seg.control {
                for c in controls {
                    output.push_str(&c.render_ansi());
                }
            }

            if !seg.text.is_empty() {
                if color_system == ColorSystem::NoColor {
                    output.push_str(&seg.text);
                } else if let Some(ref style) = seg.style {
                    let sgr = style.render_sgr(color_system);
                    let link_open = style.render_link_open();
                    let link_close = style.render_link_close();

                    if let Some(ref lo) = link_open {
                        output.push_str(lo);
                    }
                    if !sgr.is_empty() {
                        output.push_str(&sgr);
                    }
                    output.push_str(&seg.text);
                    if !sgr.is_empty() {
                        output.push_str("\x1b[0m");
                    }
                    if let Some(ref lc) = link_close {
                        output.push_str(lc);
                    }
                } else {
                    output.push_str(&seg.text);
                }
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    #[test]
    fn test_segment_split_lines() {
        let segs = vec![Segment::unstyled("hello\nworld"), Segment::unstyled("!")];
        let lines = Segment::split_lines(segs);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0][0].text, "hello");
        assert_eq!(lines[1][0].text, "world");
        assert_eq!(lines[1][1].text, "!");
    }

    #[test]
    fn test_segment_render_ansi() {
        let style = Style::parse("bold red").unwrap();
        let segs = vec![Segment::styled("Hello", style), Segment::unstyled(" World")];
        let ansi = Segment::render_ansi(&segs, ColorSystem::Standard);
        assert_eq!(ansi, "\x1b[1;31mHello\x1b[0m World");
    }
}
