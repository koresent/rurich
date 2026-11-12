use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Padding<T> {
    pub renderable: T,
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
    pub left: usize,
    pub style: Option<Style>,
}

impl<T> Padding<T> {
    pub fn new(renderable: T, pad: (usize, usize, usize, usize)) -> Self {
        Self {
            renderable,
            top: pad.0,
            right: pad.1,
            bottom: pad.2,
            left: pad.3,
            style: None,
        }
    }

    pub fn all(renderable: T, pad: usize) -> Self {
        Self::new(renderable, (pad, pad, pad, pad))
    }

    pub fn horizontal(renderable: T, pad: usize) -> Self {
        Self::new(renderable, (0, pad, 0, pad))
    }

    pub fn vertical(renderable: T, pad: usize) -> Self {
        Self::new(renderable, (pad, 0, pad, 0))
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }
}

impl<T: Renderable> Renderable for Padding<T> {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let max_width = options.max_width;
        let pad_width = self.left + self.right;
        let inner_width = max_width.saturating_sub(pad_width).max(1);

        let child_opts = options.update_width(inner_width);
        let lines = console.render_lines(&self.renderable, Some(&child_opts), false);

        let blank_line = if max_width > 0 {
            " ".repeat(max_width)
        } else {
            String::new()
        };

        // Top padding
        for _ in 0..self.top {
            sink.push(Segment {
                text: Cow::Owned(blank_line.clone()),
                style: self.style.clone(),
                control: None,
            });
            sink.push(Segment::line());
        }

        // Inner lines
        let num_lines = lines.len();
        let left_space = if self.left > 0 {
            " ".repeat(self.left)
        } else {
            String::new()
        };
        let right_space = if self.right > 0 {
            " ".repeat(self.right)
        } else {
            String::new()
        };

        for (i, line) in lines.into_iter().enumerate() {
            if self.left > 0 {
                sink.push(Segment {
                    text: Cow::Owned(left_space.clone()),
                    style: self.style.clone(),
                    control: None,
                });
            }
            sink.extend(line);
            if self.right > 0 {
                sink.push(Segment {
                    text: Cow::Owned(right_space.clone()),
                    style: self.style.clone(),
                    control: None,
                });
            }
            if i + 1 < num_lines || self.bottom > 0 {
                sink.push(Segment::line());
            }
        }

        // Bottom padding
        for i in 0..self.bottom {
            sink.push(Segment {
                text: Cow::Owned(blank_line.clone()),
                style: self.style.clone(),
                control: None,
            });
            if i + 1 < self.bottom {
                sink.push(Segment::line());
            }
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let extra = self.left + self.right;
        let inner = self.renderable.measure(console, options);
        Measurement::new(inner.minimum + extra, inner.maximum + extra)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_padding() {
        let console = Console::with_dimensions(20, 5).with_color_system(ColorSystem::NoColor);
        let padded = Padding::new("Hello", (1, 2, 1, 2));
        let out = console.capture(|c| {
            c.print(padded);
        });

        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[1], "  Hello  ");
    }
}
