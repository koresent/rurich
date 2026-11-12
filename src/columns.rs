use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use std::borrow::Cow;

pub struct Columns {
    pub renderables: Vec<Box<dyn Renderable>>,
    pub padding: usize,
    pub equal: bool,
    pub expand: bool,
}

impl Default for Columns {
    fn default() -> Self {
        Self::new()
    }
}

impl Columns {
    pub fn new() -> Self {
        Self {
            renderables: Vec::new(),
            padding: 2,
            equal: false,
            expand: false,
        }
    }

    pub fn add(&mut self, renderable: impl Renderable + 'static) {
        self.renderables.push(Box::new(renderable));
    }

    pub fn with_renderables(renderables: Vec<Box<dyn Renderable>>) -> Self {
        Self {
            renderables,
            padding: 2,
            equal: false,
            expand: false,
        }
    }

    pub fn padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }

    pub fn equal(mut self, equal: bool) -> Self {
        self.equal = equal;
        self
    }
}

impl Renderable for Columns {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        if self.renderables.is_empty() {
            return;
        }

        let max_width = options.max_width;
        let measurements: Vec<Measurement> = self
            .renderables
            .iter()
            .map(|r| r.measure(console, options))
            .collect();

        let max_item_width = measurements.iter().map(|m| m.maximum).max().unwrap_or(1);
        let col_width = max_item_width;
        let pad = self.padding;

        // How many columns fit?
        let divisor = col_width + pad;
        let num_cols = (max_width + pad).checked_div(divisor).unwrap_or(1).max(1);

        // Render each item into lines
        let rendered_items: Vec<Vec<Vec<Segment<'static>>>> = self
            .renderables
            .iter()
            .map(|r| {
                let opts = options.update_width(col_width);
                console.render_lines_owned(r.as_ref(), Some(&opts), true)
            })
            .collect();

        // Chunk into rows of `num_cols`
        let rows: Vec<&[Vec<Vec<Segment<'static>>>]> = rendered_items.chunks(num_cols).collect();

        for (row_idx, row) in rows.iter().enumerate() {
            let max_lines_in_row = row.iter().map(|item| item.len()).max().unwrap_or(0);

            for line_idx in 0..max_lines_in_row {
                for (col_idx, item) in row.iter().enumerate() {
                    if let Some(line) = item.get(line_idx) {
                        sink.extend(line.clone());
                    } else {
                        sink.push(Segment {
                            text: Cow::Owned(" ".repeat(col_width)),
                            style: None,
                            control: None,
                        });
                    }

                    if col_idx + 1 < row.len() {
                        sink.push(Segment {
                            text: Cow::Owned(" ".repeat(pad)),
                            style: None,
                            control: None,
                        });
                    }
                }
                sink.push(Segment::line());
            }

            if row_idx + 1 < rows.len() {
                // optional row separator spacing
            }
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let max_w = self
            .renderables
            .iter()
            .map(|r| r.measure(console, options).maximum)
            .max()
            .unwrap_or(0);
        Measurement::new(max_w, options.max_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_columns_flow() {
        let console = Console::with_dimensions(30, 10).with_color_system(ColorSystem::NoColor);
        let mut cols = Columns::new().padding(2);
        cols.add("Item 1");
        cols.add("Item 2");
        cols.add("Item 3");

        let out = console.capture(|c| {
            c.print(cols);
        });

        assert!(out.contains("Item 1"));
        assert!(out.contains("Item 2"));
    }
}
