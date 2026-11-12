use crate::box_styles::{Box as BoxStyle, ROUNDED};
use crate::console::Console;
use crate::markup::render as render_markup;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, JustifyMethod, Renderable};
use crate::ratio::{ratio_distribute, ratio_reduce};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Column {
    pub header: Option<Text>,
    pub footer: Option<Text>,
    pub justify: JustifyMethod,
    pub style: Option<Style>,
    pub header_style: Option<Style>,
    pub footer_style: Option<Style>,
    pub no_wrap: bool,
    pub width: Option<usize>,
    pub min_width: Option<usize>,
    pub max_width: Option<usize>,
    pub ratio: Option<usize>,
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl Column {
    pub fn new() -> Self {
        Self {
            header: None,
            footer: None,
            justify: JustifyMethod::Left,
            style: None,
            header_style: None,
            footer_style: None,
            no_wrap: false,
            width: None,
            min_width: None,
            max_width: None,
            ratio: None,
        }
    }

    pub fn header(mut self, header: impl Into<String>) -> Self {
        let s = header.into();
        self.header = Some(render_markup(&s).unwrap_or_else(|_| Text::from_plain(s)));
        self
    }

    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        let s = footer.into();
        self.footer = Some(render_markup(&s).unwrap_or_else(|_| Text::from_plain(s)));
        self
    }

    pub fn justify(mut self, justify: JustifyMethod) -> Self {
        self.justify = justify;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = Some(style);
        self
    }

    pub fn no_wrap(mut self, no_wrap: bool) -> Self {
        self.no_wrap = no_wrap;
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    pub fn ratio(mut self, ratio: usize) -> Self {
        self.ratio = Some(ratio);
        self
    }
}

pub trait IntoCell {
    fn into_cell(self) -> Box<dyn Renderable>;
}

impl IntoCell for &str {
    fn into_cell(self) -> Box<dyn Renderable> {
        if self.contains('[')
            && let Ok(text) = render_markup(self)
        {
            return Box::new(text);
        }
        Box::new(self.to_string())
    }
}

impl IntoCell for String {
    fn into_cell(self) -> Box<dyn Renderable> {
        if self.contains('[')
            && let Ok(text) = render_markup(&self)
        {
            return Box::new(text);
        }
        Box::new(self)
    }
}

impl IntoCell for Text {
    fn into_cell(self) -> Box<dyn Renderable> {
        Box::new(self)
    }
}

impl<T: Renderable + 'static> IntoCell for Box<T> {
    fn into_cell(self) -> Box<dyn Renderable> {
        self
    }
}

pub struct Row {
    pub cells: Vec<Box<dyn Renderable>>,
    pub style: Option<Style>,
    pub end_section: bool,
}

pub struct Table {
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
    pub title: Option<Text>,
    pub caption: Option<Text>,
    pub box_style: Option<BoxStyle>,
    pub show_header: bool,
    pub show_footer: bool,
    pub show_edge: bool,
    pub show_lines: bool,
    pub padding: (usize, usize), // (vertical, horizontal)
    pub expand: bool,
    pub width: Option<usize>,
    pub min_width: Option<usize>,
    pub border_style: Option<Style>,
    pub title_style: Option<Style>,
    pub caption_style: Option<Style>,
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            title: None,
            caption: None,
            box_style: Some(ROUNDED),
            show_header: true,
            show_footer: false,
            show_edge: true,
            show_lines: false,
            padding: (0, 1),
            expand: false,
            width: None,
            min_width: None,
            border_style: None,
            title_style: None,
            caption_style: None,
        }
    }

    /// Borderless grid convenience constructor
    pub fn grid() -> Self {
        let mut t = Self::new();
        t.box_style = None;
        t.show_header = false;
        t.show_edge = false;
        t.show_lines = false;
        t
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        let s = title.into();
        self.title = Some(render_markup(&s).unwrap_or_else(|_| Text::from_plain(s)));
        self
    }

    pub fn set_title(&mut self, title: impl Into<String>) -> &mut Self {
        let s = title.into();
        self.title = Some(render_markup(&s).unwrap_or_else(|_| Text::from_plain(s)));
        self
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        let s = caption.into();
        self.caption = Some(render_markup(&s).unwrap_or_else(|_| Text::from_plain(s)));
        self
    }

    pub fn box_style(mut self, box_style: BoxStyle) -> Self {
        self.box_style = Some(box_style);
        self
    }

    pub fn without_box(mut self) -> Self {
        self.box_style = None;
        self
    }

    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    pub fn show_footer(mut self, show: bool) -> Self {
        self.show_footer = show;
        self
    }

    pub fn show_edge(mut self, show: bool) -> Self {
        self.show_edge = show;
        self
    }

    pub fn show_lines(mut self, show: bool) -> Self {
        self.show_lines = show;
        self
    }

    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    pub fn padding(mut self, pad_v: usize, pad_h: usize) -> Self {
        self.padding = (pad_v, pad_h);
        self
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = Some(style);
        self
    }

    pub fn add_column(&mut self, col: Column) -> &mut Self {
        self.columns.push(col);
        self
    }

    pub fn add_row<I, T>(&mut self, cells: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: IntoCell,
    {
        let boxed_cells: Vec<Box<dyn Renderable>> =
            cells.into_iter().map(|c| c.into_cell()).collect();
        self.rows.push(Row {
            cells: boxed_cells,
            style: None,
            end_section: false,
        });
        self
    }

    pub fn add_section(&mut self) -> &mut Self {
        if let Some(last_row) = self.rows.last_mut() {
            last_row.end_section = true;
        }
        self
    }

    /// Calculate column widths based on console width and content
    fn calculate_column_widths(&self, console: &Console, options: &ConsoleOptions) -> Vec<usize> {
        let num_cols = self.columns.len();
        if num_cols == 0 {
            return Vec::new();
        }

        let max_width = self.width.unwrap_or(options.max_width);
        let pad_h = self.padding.1 * 2;
        let border_overhead = if self.box_style.is_some() {
            (if self.show_edge { 2 } else { 0 }) + num_cols.saturating_sub(1)
        } else {
            0
        };

        let available_content_width = max_width.saturating_sub(border_overhead);

        // Measure columns
        let mut min_widths = vec![0; num_cols];
        let mut max_widths = vec![0; num_cols];

        for (col_idx, col) in self.columns.iter().enumerate() {
            if let Some(fixed_w) = col.width {
                min_widths[col_idx] = fixed_w;
                max_widths[col_idx] = fixed_w;
                continue;
            }

            let mut col_min = 1;
            let mut col_max = 1;

            if self.show_header
                && let Some(ref h) = col.header
            {
                let m = h.measure(console, options);
                col_min = col_min.max(m.minimum);
                col_max = col_max.max(m.maximum);
            }

            for row in &self.rows {
                if let Some(cell) = row.cells.get(col_idx) {
                    let m = cell.measure(console, options);
                    col_min = col_min.max(m.minimum);
                    col_max = col_max.max(m.maximum);
                }
            }

            col_min += pad_h;
            col_max += pad_h;

            if let Some(min_w) = col.min_width {
                col_min = col_min.max(min_w);
                col_max = col_max.max(min_w);
            }
            if let Some(max_w) = col.max_width {
                col_min = col_min.min(max_w);
                col_max = col_max.min(max_w);
            }

            min_widths[col_idx] = col_min;
            max_widths[col_idx] = col_max;
        }

        let mut widths = max_widths.clone();
        let total_desired: usize = widths.iter().sum();

        if total_desired > available_content_width {
            // Need to shrink columns
            let excess = total_desired - available_content_width;
            let ratios: Vec<usize> = self.columns.iter().map(|c| c.ratio.unwrap_or(1)).collect();
            let maximum_cuts: Vec<usize> = widths
                .iter()
                .zip(&min_widths)
                .map(|(&w, &min)| w.saturating_sub(min))
                .collect();

            let reduced = ratio_reduce(excess, &ratios, &maximum_cuts, &widths);
            widths = reduced;
        } else if self.expand && total_desired < available_content_width {
            // Expand flexible columns
            let extra_space = available_content_width - total_desired;
            let ratios: Vec<usize> = self.columns.iter().map(|c| c.ratio.unwrap_or(1)).collect();
            let distributed = ratio_distribute(extra_space, &ratios, None);
            for (w, add) in widths.iter_mut().zip(distributed) {
                *w += add;
            }
        }

        widths
    }
}

impl Renderable for Table {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        if self.columns.is_empty() {
            return;
        }

        let widths = self.calculate_column_widths(console, options);
        let border_box = self.box_style;
        let border_style = self.border_style.clone();
        let pad_h = self.padding.1;

        // Render Title
        if let Some(ref title) = self.title {
            let total_w = widths.iter().sum::<usize>()
                + if border_box.is_some() {
                    (if self.show_edge { 2 } else { 0 }) + widths.len().saturating_sub(1)
                } else {
                    0
                };
            let title_opts = options.update_width(total_w);
            let mut title_segs = Vec::new();
            title.render_to(console, &title_opts, &mut title_segs);
            sink.extend(title_segs);
            sink.push(Segment::line());
        }

        // Top Border
        if let Some(b) = border_box
            && self.show_edge
        {
            let top_line = b.get_top(&widths);
            sink.push(Segment {
                text: Cow::Owned(top_line),
                style: border_style.clone(),
                control: None,
            });
            sink.push(Segment::line());
        }

        // Header Row
        if self.show_header {
            let mut header_rendered_cols: Vec<Vec<Vec<Segment<'static>>>> = Vec::new();
            let mut max_header_lines = 1;

            for (col_idx, col) in self.columns.iter().enumerate() {
                let w = widths[col_idx].saturating_sub(pad_h * 2);
                let col_opts = options.update_width(w);
                let lines = if let Some(ref h) = col.header {
                    let mut styled_header = h.clone();
                    if let Some(ref s) = col.header_style {
                        styled_header = styled_header.style(s.clone());
                    }
                    console.render_lines_owned(&styled_header, Some(&col_opts), false)
                } else {
                    Vec::new()
                };
                max_header_lines = max_header_lines.max(lines.len());
                header_rendered_cols.push(lines);
            }

            for line_idx in 0..max_header_lines {
                if let Some(b) = border_box
                    && self.show_edge
                {
                    sink.push(Segment {
                        text: Cow::Owned(b.head_left.to_string()),
                        style: border_style.clone(),
                        control: None,
                    });
                }

                for (col_idx, col) in self.columns.iter().enumerate() {
                    let col_w = widths[col_idx];
                    let inner_w = col_w.saturating_sub(pad_h * 2);
                    let left_pad = " ".repeat(pad_h);
                    let right_pad = " ".repeat(pad_h);

                    sink.push(Segment::unstyled(Cow::Owned(left_pad)));

                    let empty_line = Vec::new();
                    let line = header_rendered_cols[col_idx]
                        .get(line_idx)
                        .unwrap_or(&empty_line);
                    let line_len = Segment::get_line_length(line);

                    let (align_left, align_right) = if line_len < inner_w {
                        let diff = inner_w - line_len;
                        match col.justify {
                            JustifyMethod::Left | JustifyMethod::Default => (0, diff),
                            JustifyMethod::Right => (diff, 0),
                            JustifyMethod::Center => {
                                let l = diff / 2;
                                (l, diff - l)
                            }
                            JustifyMethod::Full => (0, diff),
                        }
                    } else {
                        (0, 0)
                    };

                    if align_left > 0 {
                        sink.push(Segment::unstyled(Cow::Owned(" ".repeat(align_left))));
                    }
                    sink.extend(line.clone());
                    if align_right > 0 {
                        sink.push(Segment::unstyled(Cow::Owned(" ".repeat(align_right))));
                    }

                    sink.push(Segment::unstyled(Cow::Owned(right_pad)));

                    if let Some(b) = border_box
                        && col_idx + 1 < self.columns.len()
                    {
                        sink.push(Segment {
                            text: Cow::Owned(b.head_vertical.to_string()),
                            style: border_style.clone(),
                            control: None,
                        });
                    }
                }

                if let Some(b) = border_box
                    && self.show_edge
                {
                    sink.push(Segment {
                        text: Cow::Owned(b.head_right.to_string()),
                        style: border_style.clone(),
                        control: None,
                    });
                }
                sink.push(Segment::line());
            }

            // Head row separator
            if let Some(b) = border_box {
                let sep = b.get_row(&widths, "head");
                sink.push(Segment {
                    text: Cow::Owned(sep),
                    style: border_style.clone(),
                    control: None,
                });
                sink.push(Segment::line());
            }
        }

        // Data Rows
        let num_rows = self.rows.len();
        for (row_idx, row) in self.rows.iter().enumerate() {
            let mut rendered_cells: Vec<Vec<Vec<Segment<'static>>>> = Vec::new();
            let mut max_cell_lines = 1;

            for (col_idx, _col) in self.columns.iter().enumerate() {
                let w = widths[col_idx].saturating_sub(pad_h * 2);
                let col_opts = options.update_width(w);

                let cell_lines = if let Some(cell) = row.cells.get(col_idx) {
                    console.render_lines_owned(cell.as_ref(), Some(&col_opts), false)
                } else {
                    Vec::new()
                };
                max_cell_lines = max_cell_lines.max(cell_lines.len());
                rendered_cells.push(cell_lines);
            }

            for line_idx in 0..max_cell_lines {
                if let Some(b) = border_box
                    && self.show_edge
                {
                    sink.push(Segment {
                        text: Cow::Owned(b.mid_left.to_string()),
                        style: border_style.clone(),
                        control: None,
                    });
                }

                for (col_idx, col) in self.columns.iter().enumerate() {
                    let col_w = widths[col_idx];
                    let inner_w = col_w.saturating_sub(pad_h * 2);
                    let left_pad = " ".repeat(pad_h);
                    let right_pad = " ".repeat(pad_h);

                    sink.push(Segment::unstyled(Cow::Owned(left_pad)));

                    let empty_line = Vec::new();
                    let line = rendered_cells[col_idx].get(line_idx).unwrap_or(&empty_line);
                    let line_len = Segment::get_line_length(line);

                    let (align_left, align_right) = if line_len < inner_w {
                        let diff = inner_w - line_len;
                        match col.justify {
                            JustifyMethod::Left | JustifyMethod::Default => (0, diff),
                            JustifyMethod::Right => (diff, 0),
                            JustifyMethod::Center => {
                                let l = diff / 2;
                                (l, diff - l)
                            }
                            JustifyMethod::Full => (0, diff),
                        }
                    } else {
                        (0, 0)
                    };

                    if align_left > 0 {
                        sink.push(Segment::unstyled(Cow::Owned(" ".repeat(align_left))));
                    }
                    sink.extend(line.clone());
                    if align_right > 0 {
                        sink.push(Segment::unstyled(Cow::Owned(" ".repeat(align_right))));
                    }

                    sink.push(Segment::unstyled(Cow::Owned(right_pad)));

                    if let Some(b) = border_box
                        && col_idx + 1 < self.columns.len()
                    {
                        sink.push(Segment {
                            text: Cow::Owned(b.mid_vertical.to_string()),
                            style: border_style.clone(),
                            control: None,
                        });
                    }
                }

                if let Some(b) = border_box
                    && self.show_edge
                {
                    sink.push(Segment {
                        text: Cow::Owned(b.mid_right.to_string()),
                        style: border_style.clone(),
                        control: None,
                    });
                }
                sink.push(Segment::line());
            }

            // Row separator
            if ((self.show_lines && row_idx + 1 < num_rows) || row.end_section)
                && let Some(b) = border_box
            {
                let sep = b.get_row(&widths, "row");
                sink.push(Segment {
                    text: Cow::Owned(sep),
                    style: border_style.clone(),
                    control: None,
                });
                sink.push(Segment::line());
            }
        }

        // Bottom Border
        if let Some(b) = border_box
            && self.show_edge
        {
            let bottom_line = b.get_bottom(&widths);
            sink.push(Segment {
                text: Cow::Owned(bottom_line),
                style: border_style,
                control: None,
            });
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let widths = self.calculate_column_widths(console, options);
        let border_overhead = if self.box_style.is_some() {
            (if self.show_edge { 2 } else { 0 }) + widths.len().saturating_sub(1)
        } else {
            0
        };
        let total: usize = widths.iter().sum::<usize>() + border_overhead;
        Measurement::new(total, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_styles::ASCII;
    use crate::color::ColorSystem;

    #[test]
    fn test_table_render() {
        let console = Console::with_dimensions(30, 10).with_color_system(ColorSystem::NoColor);
        let mut table = Table::new().box_style(ASCII);
        table.add_column(Column::new().header("Name"));
        table.add_column(Column::new().header("Age").justify(JustifyMethod::Right));
        table.add_row(["Alice", "24"]);
        table.add_row(["Bob", "30"]);

        let out = console.capture(|c| {
            c.print(table);
        });

        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "+-------------+");
        assert_eq!(lines[1], "| Name  | Age |");
        assert_eq!(lines[2], "|-------+-----|");
        assert_eq!(lines[3], "| Alice |  24 |");
        assert_eq!(lines[4], "| Bob   |  30 |");
        assert_eq!(lines[5], "+-------------+");
    }
}
