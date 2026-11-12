use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignMethod {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalAlignMethod {
    #[default]
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct Align<T> {
    pub renderable: T,
    pub align: AlignMethod,
    pub vertical: Option<VerticalAlignMethod>,
    pub style: Option<Style>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub pad: bool,
}

impl<T> Align<T> {
    pub fn new(renderable: T, align: AlignMethod) -> Self {
        Self {
            renderable,
            align,
            vertical: None,
            style: None,
            width: None,
            height: None,
            pad: true,
        }
    }

    pub fn left(renderable: T) -> Self {
        Self::new(renderable, AlignMethod::Left)
    }

    pub fn center(renderable: T) -> Self {
        Self::new(renderable, AlignMethod::Center)
    }

    pub fn right(renderable: T) -> Self {
        Self::new(renderable, AlignMethod::Right)
    }

    pub fn vertical(mut self, vertical: VerticalAlignMethod) -> Self {
        self.vertical = Some(vertical);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    pub fn pad(mut self, pad: bool) -> Self {
        self.pad = pad;
        self
    }
}

impl<T: Renderable> Renderable for Align<T> {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let max_width = self.width.unwrap_or(options.max_width);
        let child_opts = options.update_width(max_width);
        let lines = console.render_lines(&self.renderable, Some(&child_opts), false);

        // Calculate vertical padding if height is set
        let target_height = self.height.unwrap_or(lines.len());
        let top_pad = if target_height > lines.len() {
            match self.vertical.unwrap_or_default() {
                VerticalAlignMethod::Top => 0,
                VerticalAlignMethod::Middle => (target_height - lines.len()) / 2,
                VerticalAlignMethod::Bottom => target_height - lines.len(),
            }
        } else {
            0
        };
        let bottom_pad = if target_height > lines.len() {
            target_height - lines.len() - top_pad
        } else {
            0
        };

        let blank_line = " ".repeat(max_width);

        for _ in 0..top_pad {
            sink.push(Segment {
                text: Cow::Owned(blank_line.clone()),
                style: self.style.clone(),
                control: None,
            });
            sink.push(Segment::line());
        }

        let num_lines = lines.len();
        for (i, line) in lines.into_iter().enumerate() {
            let line_len = Segment::get_line_length(&line);
            if line_len < max_width {
                let diff = max_width - line_len;
                let (left, right) = match self.align {
                    AlignMethod::Left => (0, diff),
                    AlignMethod::Right => (diff, 0),
                    AlignMethod::Center => {
                        let l = diff / 2;
                        (l, diff - l)
                    }
                };

                if left > 0 {
                    sink.push(Segment {
                        text: Cow::Owned(" ".repeat(left)),
                        style: self.style.clone(),
                        control: None,
                    });
                }
                sink.extend(line);
                if self.pad && right > 0 {
                    sink.push(Segment {
                        text: Cow::Owned(" ".repeat(right)),
                        style: self.style.clone(),
                        control: None,
                    });
                }
            } else {
                sink.extend(line);
            }

            if i + 1 < num_lines || bottom_pad > 0 {
                sink.push(Segment::line());
            }
        }

        for i in 0..bottom_pad {
            sink.push(Segment {
                text: Cow::Owned(blank_line.clone()),
                style: self.style.clone(),
                control: None,
            });
            if i + 1 < bottom_pad {
                sink.push(Segment::line());
            }
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.renderable.measure(console, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_align_center() {
        let console = Console::with_dimensions(15, 3).with_color_system(ColorSystem::NoColor);
        let aligned = Align::center("Hi");
        let out = console.capture(|c| {
            c.print(aligned);
        });

        // 15 width, "Hi" is 2 -> left is 6, right is 7
        assert_eq!(out, "      Hi       ");
    }

    #[test]
    fn test_align_right() {
        let console = Console::with_dimensions(10, 3).with_color_system(ColorSystem::NoColor);
        let aligned = Align::right("ABC");
        let out = console.capture(|c| {
            c.print(aligned);
        });

        assert_eq!(out, "       ABC");
    }
}
