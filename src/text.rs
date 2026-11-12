use crate::cells::{cell_len, split_cells};
use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, JustifyMethod, OverflowMethod, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub style: Style,
}

impl Span {
    pub fn new(start: usize, end: usize, style: Style) -> Self {
        Self { start, end, style }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Text {
    pub plain: String,
    pub spans: Vec<Span>,
    pub style: Option<Style>,
    pub justify: Option<JustifyMethod>,
    pub overflow: Option<OverflowMethod>,
    pub no_wrap: bool,
}

impl Text {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_plain(plain: impl Into<String>) -> Self {
        Self {
            plain: plain.into(),
            spans: Vec::new(),
            style: None,
            justify: None,
            overflow: None,
            no_wrap: false,
        }
    }

    pub fn styled(plain: impl Into<String>, style: Style) -> Self {
        Self {
            plain: plain.into(),
            spans: Vec::new(),
            style: Some(style),
            justify: None,
            overflow: None,
            no_wrap: false,
        }
    }

    pub fn cell_len(&self) -> usize {
        cell_len(&self.plain)
    }

    pub fn is_empty(&self) -> bool {
        self.plain.is_empty()
    }

    pub fn len(&self) -> usize {
        self.plain.len()
    }

    pub fn justify(mut self, justify: JustifyMethod) -> Self {
        self.justify = Some(justify);
        self
    }

    pub fn overflow(mut self, overflow: OverflowMethod) -> Self {
        self.overflow = Some(overflow);
        self
    }

    pub fn no_wrap(mut self, no_wrap: bool) -> Self {
        self.no_wrap = no_wrap;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn append(&mut self, text: &str, style: Option<Style>) {
        if text.is_empty() {
            return;
        }
        let start = self.plain.len();
        self.plain.push_str(text);
        let end = self.plain.len();
        if let Some(s) = style
            && !s.is_null()
        {
            self.spans.push(Span::new(start, end, s));
        }
    }

    pub fn append_styled(&mut self, text: &str, style: Style) {
        self.append(text, Some(style));
    }

    pub fn append_text(&mut self, other: &Text) {
        let offset = self.plain.len();
        self.plain.push_str(&other.plain);
        for span in &other.spans {
            self.spans.push(Span::new(
                span.start + offset,
                span.end + offset,
                span.style.clone(),
            ));
        }
    }

    /// Split text by newline into a vector of Text objects
    pub fn split_lines(&self) -> Vec<Text> {
        let mut lines = Vec::new();
        let parts: Vec<&str> = self.plain.split('\n').collect();
        let mut current_offset = 0;

        for part in parts {
            let part_len = part.len();
            let start = current_offset;
            let end = current_offset + part_len;

            // Extract spans that overlap with this line
            let mut line_spans = Vec::new();
            for span in &self.spans {
                if span.end > start && span.start < end {
                    let clamped_start = span.start.max(start) - start;
                    let clamped_end = span.end.min(end) - start;
                    if clamped_start < clamped_end {
                        line_spans.push(Span::new(clamped_start, clamped_end, span.style.clone()));
                    }
                }
            }

            lines.push(Text {
                plain: part.to_string(),
                spans: line_spans,
                style: self.style.clone(),
                justify: self.justify,
                overflow: self.overflow,
                no_wrap: self.no_wrap,
            });

            current_offset += part_len + 1; // +1 for '\n'
        }

        lines
    }

    /// Pad left and right with spaces
    pub fn pad(&mut self, left: usize, right: usize) {
        if left > 0 {
            let pad_left = " ".repeat(left);
            self.plain.insert_str(0, &pad_left);
            for span in &mut self.spans {
                span.start += left;
                span.end += left;
            }
        }
        if right > 0 {
            self.plain.push_str(&" ".repeat(right));
        }
    }

    /// Justify a single-line Text to a given width
    pub fn justify_line(&self, width: usize, method: JustifyMethod) -> Text {
        let length = self.cell_len();
        if length >= width || method == JustifyMethod::Default {
            return self.clone();
        }

        let diff = width - length;
        let mut result = self.clone();

        match method {
            JustifyMethod::Default => result,
            JustifyMethod::Left => {
                result.pad(0, diff);
                result
            }
            JustifyMethod::Right => {
                result.pad(diff, 0);
                result
            }
            JustifyMethod::Center => {
                let left = diff / 2;
                let right = diff - left;
                result.pad(left, right);
                result
            }
            JustifyMethod::Full => {
                let words: Vec<&str> = self.plain.split_whitespace().collect();
                if words.len() <= 1 {
                    result.pad(0, diff);
                    result
                } else {
                    let total_gaps = words.len() - 1;
                    let extra_spaces = diff / total_gaps;
                    let remainder = diff % total_gaps;

                    let mut new_text = Text::new();
                    new_text.style = self.style.clone();

                    for (i, &word) in words.iter().enumerate() {
                        new_text.append(word, self.style.clone());
                        if i < total_gaps {
                            let gap_len = 1 + extra_spaces + if i < remainder { 1 } else { 0 };
                            new_text.append(&" ".repeat(gap_len), None);
                        }
                    }
                    new_text
                }
            }
        }
    }

    /// Strip trailing whitespace from this Text
    pub fn rstrip(&mut self) {
        let trimmed_len = self.plain.trim_end().len();
        if trimmed_len < self.plain.len() {
            self.plain.truncate(trimmed_len);
            self.spans.retain_mut(|span| {
                if span.start >= trimmed_len {
                    false
                } else {
                    if span.end > trimmed_len {
                        span.end = trimmed_len;
                    }
                    true
                }
            });
        }
    }

    /// Wrap text at max_width cells
    pub fn wrap(&self, width: usize) -> Vec<Text> {
        if self.no_wrap || width == 0 {
            return self.split_lines();
        }

        let mut lines = Vec::new();
        for raw_line in self.split_lines() {
            if raw_line.cell_len() <= width {
                lines.push(raw_line);
                continue;
            }

            let mut current_line = Text::new();
            current_line.style = raw_line.style.clone();
            current_line.justify = raw_line.justify;

            let words: Vec<&str> = raw_line.plain.split(' ').collect();
            for (i, &word) in words.iter().enumerate() {
                let word_len = cell_len(word);
                let current_len = current_line.cell_len();
                let need_space = i > 0 && !current_line.is_empty();
                let space_len = if need_space { 1 } else { 0 };

                if current_len + space_len + word_len <= width {
                    if need_space {
                        current_line.append(" ", None);
                    }
                    current_line.append(word, None);
                } else {
                    if !current_line.is_empty() {
                        current_line.rstrip();
                        lines.push(std::mem::take(&mut current_line));
                    }
                    // If a single word is wider than width, break it down
                    if word_len > width {
                        let (left, right) = split_cells(word, width);
                        current_line.append(&left, None);
                        current_line.rstrip();
                        lines.push(std::mem::take(&mut current_line));
                        current_line.append(&right, None);
                    } else {
                        current_line.append(word, None);
                    }
                }
            }

            if !current_line.is_empty() {
                current_line.rstrip();
                lines.push(current_line);
            }
        }

        if lines.is_empty() {
            lines.push(Text::new());
        }

        lines
    }

    /// Render text intervals into Segments
    pub fn render_to_segments<'a>(&'a self) -> Vec<Segment<'a>> {
        if self.plain.is_empty() {
            return Vec::new();
        }

        if self.spans.is_empty() {
            return vec![Segment {
                text: Cow::Borrowed(&self.plain),
                style: self.style.clone(),
                control: None,
            }];
        }

        // Interval sweep algorithm for style spans
        let mut events: Vec<(usize, bool, usize)> = Vec::new();
        for (i, span) in self.spans.iter().enumerate() {
            let start = span.start.min(self.plain.len());
            let end = span.end.min(self.plain.len());
            if start < end {
                events.push((start, false, i)); // false = enter
                events.push((end, true, i)); // true = leave
            }
        }

        // Sort events by offset, with leaving (true) before entering (false) at the same offset
        events.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        let mut segments = Vec::new();
        let mut active_indices: Vec<usize> = Vec::new();
        let mut last_offset = 0;

        for (offset, leaving, span_idx) in events {
            if offset > last_offset {
                let slice = &self.plain[last_offset..offset];
                let mut combined_style = self.style.clone().unwrap_or_default();
                for &idx in &active_indices {
                    combined_style = combined_style.combine(&self.spans[idx].style);
                }
                segments.push(Segment {
                    text: Cow::Borrowed(slice),
                    style: if combined_style.is_null() {
                        None
                    } else {
                        Some(combined_style)
                    },
                    control: None,
                });
                last_offset = offset;
            }

            if leaving {
                if let Some(pos) = active_indices.iter().position(|&x| x == span_idx) {
                    active_indices.remove(pos);
                }
            } else {
                active_indices.push(span_idx);
            }
        }

        if last_offset < self.plain.len() {
            let slice = &self.plain[last_offset..];
            segments.push(Segment {
                text: Cow::Borrowed(slice),
                style: self.style.clone(),
                control: None,
            });
        }

        Segment::simplify(segments)
    }
}

impl Renderable for Text {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let max_width = options.max_width;
        let wrapped_lines = self.wrap(max_width);
        let num_lines = wrapped_lines.len();

        for (i, line) in wrapped_lines.into_iter().enumerate() {
            let justify = self.justify.or(options.justify).unwrap_or_default();
            let justified = line.justify_line(max_width, justify);
            sink.extend(justified.render_to_segments().into_iter().map(|s| Segment {
                text: Cow::Owned(s.text.into_owned()),
                style: s.style,
                control: s.control,
            }));
            if i + 1 < num_lines {
                sink.push(Segment::line());
            }
        }
    }

    fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let lines = self.split_lines();
        let max_width = lines.iter().map(|l| l.cell_len()).max().unwrap_or(0);
        let min_width = self
            .plain
            .split_whitespace()
            .map(cell_len)
            .max()
            .unwrap_or(0);
        Measurement::new(min_width, max_width)
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.plain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_append_and_render() {
        let mut t = Text::new();
        t.append("Hello ", None);
        let bold_style = Style::new().bold();
        t.append_styled("World", bold_style.clone());

        let segs = t.render_to_segments();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].text, "Hello ");
        assert_eq!(segs[0].style, None);
        assert_eq!(segs[1].text, "World");
        assert_eq!(segs[1].style, Some(bold_style));
    }

    #[test]
    fn test_text_wrap() {
        let t = Text::from_plain("The quick brown fox jumps over the lazy dog");
        let lines = t.wrap(15);
        assert!(lines.len() >= 3);
        for line in lines {
            assert!(line.cell_len() <= 15);
        }
    }
}
