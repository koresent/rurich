use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::ratio::{RatioEdge, ratio_resolve};
use crate::segment::Segment;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitDirection {
    #[default]
    Column, // Split vertically into rows
    Row, // Split horizontally into columns
}

pub struct Layout {
    pub name: Option<String>,
    pub renderable: Option<Box<dyn Renderable>>,
    pub size: Option<usize>,
    pub minimum_size: usize,
    pub ratio: usize,
    pub direction: SplitDirection,
    pub children: Vec<Layout>,
    pub visible: bool,
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

impl Layout {
    pub fn new() -> Self {
        Self {
            name: None,
            renderable: None,
            size: None,
            minimum_size: 1,
            ratio: 1,
            direction: SplitDirection::Column,
            children: Vec::new(),
            visible: true,
        }
    }

    pub fn with_renderable(renderable: impl Renderable + 'static) -> Self {
        let mut l = Self::new();
        l.renderable = Some(Box::new(renderable));
        l
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn minimum_size(mut self, min: usize) -> Self {
        self.minimum_size = min;
        self
    }

    pub fn ratio(mut self, ratio: usize) -> Self {
        self.ratio = ratio;
        self
    }

    pub fn update(&mut self, renderable: impl Renderable + 'static) {
        self.renderable = Some(Box::new(renderable));
    }

    pub fn split_row(&mut self, children: Vec<Layout>) {
        self.direction = SplitDirection::Row;
        self.children = children;
    }

    pub fn split_column(&mut self, children: Vec<Layout>) {
        self.direction = SplitDirection::Column;
        self.children = children;
    }

    /// Render layout tree into a 2D grid of lines
    fn render_tree(
        &self,
        console: &Console,
        width: usize,
        height: usize,
    ) -> Vec<Vec<Segment<'static>>> {
        if !self.children.is_empty() {
            match self.direction {
                SplitDirection::Column => {
                    // Split vertically into child rows
                    let edges: Vec<RatioEdge> = self
                        .children
                        .iter()
                        .map(|c| RatioEdge::new(c.size, c.ratio, c.minimum_size))
                        .collect();
                    let heights = ratio_resolve(height, &edges);
                    let mut lines = Vec::with_capacity(height);

                    for (child, child_h) in self.children.iter().zip(heights) {
                        if child_h > 0 {
                            let child_lines = child.render_tree(console, width, child_h);
                            lines.extend(child_lines);
                        }
                    }
                    lines
                }
                SplitDirection::Row => {
                    // Split horizontally into child columns
                    let edges: Vec<RatioEdge> = self
                        .children
                        .iter()
                        .map(|c| RatioEdge::new(c.size, c.ratio, c.minimum_size))
                        .collect();
                    let widths = ratio_resolve(width, &edges);

                    let child_rendered: Vec<Vec<Vec<Segment<'static>>>> = self
                        .children
                        .iter()
                        .zip(&widths)
                        .map(|(c, &w)| c.render_tree(console, w, height))
                        .collect();

                    let mut lines = Vec::with_capacity(height);
                    for line_idx in 0..height {
                        let mut combined_line = Vec::new();
                        for (col_idx, &col_w) in widths.iter().enumerate() {
                            let empty = Vec::new();
                            let segs = child_rendered[col_idx].get(line_idx).unwrap_or(&empty);
                            let segs_len = Segment::get_line_length(segs);
                            combined_line.extend(segs.clone());
                            if segs_len < col_w {
                                combined_line.push(Segment {
                                    text: Cow::Owned(" ".repeat(col_w - segs_len)),
                                    style: None,
                                    control: None,
                                });
                            }
                        }
                        lines.push(combined_line);
                    }
                    lines
                }
            }
        } else if let Some(ref r) = self.renderable {
            let opts = console.options().update_dimensions(width, height);
            let mut lines = console.render_lines_owned(r.as_ref(), Some(&opts), true);
            // Ensure lines fill the height
            while lines.len() < height {
                lines.push(vec![Segment {
                    text: Cow::Owned(" ".repeat(width)),
                    style: None,
                    control: None,
                }]);
            }
            lines.truncate(height);
            lines
        } else {
            // Empty placeholder box
            let blank = " ".repeat(width);
            vec![
                vec![Segment {
                    text: Cow::Owned(blank),
                    style: None,
                    control: None,
                }];
                height
            ]
        }
    }
}

impl Renderable for Layout {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let width = self.size.unwrap_or(options.max_width);
        let height = options.height.unwrap_or(options.size.height);
        let lines = self.render_tree(console, width, height);

        let num_lines = lines.len();
        for (i, line) in lines.into_iter().enumerate() {
            sink.extend(line);
            if i + 1 < num_lines {
                sink.push(Segment::line());
            }
        }
    }

    fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        Measurement::new(self.minimum_size, options.max_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_layout_split_row() {
        let console = Console::with_dimensions(20, 2).with_color_system(ColorSystem::NoColor);
        let mut layout = Layout::new();
        layout.split_row(vec![
            Layout::with_renderable("Left"),
            Layout::with_renderable("Right"),
        ]);

        let out = console.capture(|c| {
            c.print(layout);
        });

        assert!(out.contains("Left"));
        assert!(out.contains("Right"));
    }
}
