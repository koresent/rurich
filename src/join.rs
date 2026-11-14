use crate::cells::cell_len;
use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use std::borrow::Cow;

/// Alignment for joining blocks horizontally or vertically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignJoin {
    #[default]
    Top,
    Center,
    Bottom,
    Left,
    Right,
}

/// A container that combines multiple renderables horizontally or vertically.
pub struct JoinedBlock<'a> {
    pub items: Vec<&'a dyn Renderable>,
    pub horizontal: bool,
    pub align: AlignJoin,
}

impl<'a> JoinedBlock<'a> {
    pub fn horizontal(align: AlignJoin, items: Vec<&'a dyn Renderable>) -> Self {
        Self {
            items,
            horizontal: true,
            align,
        }
    }

    pub fn vertical(align: AlignJoin, items: Vec<&'a dyn Renderable>) -> Self {
        Self {
            items,
            horizontal: false,
            align,
        }
    }
}

pub fn join_horizontal<'a>(align: AlignJoin, items: &[&'a dyn Renderable]) -> JoinedBlock<'a> {
    JoinedBlock::horizontal(align, items.to_vec())
}

pub fn join_vertical<'a>(align: AlignJoin, items: &[&'a dyn Renderable]) -> JoinedBlock<'a> {
    JoinedBlock::vertical(align, items.to_vec())
}

impl<'b> Renderable for JoinedBlock<'b> {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        if self.items.is_empty() {
            return;
        }

        if self.horizontal {
            // Render each item into lines of segments
            let mut rendered_items: Vec<(Vec<Vec<Segment<'static>>>, usize)> = Vec::new();
            let mut max_lines = 0;

            for item in &self.items {
                let lines = console.render_lines_owned(*item, Some(options), false);
                let item_width = lines
                    .iter()
                    .map(|l| l.iter().map(|s| cell_len(&s.text)).sum::<usize>())
                    .max()
                    .unwrap_or(0);
                max_lines = max_lines.max(lines.len());
                rendered_items.push((lines, item_width));
            }

            if max_lines == 0 {
                return;
            }

            for row in 0..max_lines {
                for (lines, item_width) in &rendered_items {
                    let total_item_lines = lines.len();
                    let padding_lines = max_lines.saturating_sub(total_item_lines);

                    let (pad_top, _) = match self.align {
                        AlignJoin::Top | AlignJoin::Left => (0, padding_lines),
                        AlignJoin::Bottom | AlignJoin::Right => (padding_lines, 0),
                        AlignJoin::Center => {
                            let top = padding_lines / 2;
                            (top, padding_lines - top)
                        }
                    };

                    if row < pad_top || row >= pad_top + total_item_lines {
                        // Empty line padding for this column
                        if *item_width > 0 {
                            sink.push(Segment::unstyled(Cow::Owned(" ".repeat(*item_width))));
                        }
                    } else {
                        let line_idx = row - pad_top;
                        let line = &lines[line_idx];
                        let current_line_len: usize =
                            line.iter().map(|s| cell_len(&s.text)).sum();
                        sink.extend(line.iter().cloned());
                        if current_line_len < *item_width {
                            sink.push(Segment::unstyled(Cow::Owned(
                                " ".repeat(*item_width - current_line_len),
                            )));
                        }
                    }
                }
                if row + 1 < max_lines {
                    sink.push(Segment::line());
                }
            }
        } else {
            // Vertical join
            let mut rendered_items: Vec<Vec<Vec<Segment<'static>>>> = Vec::new();
            let mut max_width = 0;

            for item in &self.items {
                let lines = console.render_lines_owned(*item, Some(options), false);
                for line in &lines {
                    let w: usize = line.iter().map(|s| cell_len(&s.text)).sum();
                    max_width = max_width.max(w);
                }
                rendered_items.push(lines);
            }

            let mut first = true;
            for lines in rendered_items {
                for line in lines {
                    if !first {
                        sink.push(Segment::line());
                    }
                    first = false;

                    let line_w: usize = line.iter().map(|s| cell_len(&s.text)).sum();
                    let excess = max_width.saturating_sub(line_w);

                    let (pad_left, pad_right) = match self.align {
                        AlignJoin::Left | AlignJoin::Top => (0, excess),
                        AlignJoin::Right | AlignJoin::Bottom => (excess, 0),
                        AlignJoin::Center => {
                            let l = excess / 2;
                            (l, excess - l)
                        }
                    };

                    if pad_left > 0 {
                        sink.push(Segment::unstyled(Cow::Owned(" ".repeat(pad_left))));
                    }
                    sink.extend(line);
                    if pad_right > 0 {
                        sink.push(Segment::unstyled(Cow::Owned(" ".repeat(pad_right))));
                    }
                }
            }
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        if self.items.is_empty() {
            return Measurement::new(0, 0);
        }

        if self.horizontal {
            let mut min = 0;
            let mut max = 0;
            for item in &self.items {
                let m = item.measure(console, options);
                min += m.minimum;
                max += m.maximum;
            }
            Measurement::new(min, max)
        } else {
            let mut min = 0;
            let mut max = 0;
            for item in &self.items {
                let m = item.measure(console, options);
                min = min.max(m.minimum);
                max = max.max(m.maximum);
            }
            Measurement::new(min, max)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;
    use crate::text::Text;

    #[test]
    fn test_join_horizontal() {
        let console = Console::with_dimensions(30, 5).with_color_system(ColorSystem::NoColor);
        let t1 = Text::from_plain("A\nB");
        let t2 = Text::from_plain("1\n2");

        let joined = join_horizontal(AlignJoin::Top, &[&t1, &t2]);
        let out = console.capture(|c| c.print(joined));
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "A1");
        assert_eq!(lines[1], "B2");
    }

    #[test]
    fn test_join_vertical_center() {
        let console = Console::with_dimensions(30, 5).with_color_system(ColorSystem::NoColor);
        let t1 = Text::from_plain("Short");
        let t2 = Text::from_plain("Very Long Text");

        let joined = join_vertical(AlignJoin::Center, &[&t1, &t2]);
        let out = console.capture(|c| c.print(joined));
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with("    Short"));
        assert_eq!(lines[1], "Very Long Text");
    }
}
