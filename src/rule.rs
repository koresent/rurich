use crate::align::AlignMethod;
use crate::cells::cell_len;
use crate::console::Console;
use crate::markup::render as render_markup;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use std::borrow::Cow;

/// Standalone horizontal rule renderable with optional centered title.
#[derive(Clone, Debug)]
pub struct Rule {
    pub title: Option<String>,
    pub characters: String,
    pub style: Style,
    pub align: AlignMethod,
}

impl Rule {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            characters: "─".to_string(),
            style: Style::parse("rule.line")
                .unwrap_or_else(|_| Style::parse("dim").unwrap_or_default()),
            align: AlignMethod::Center,
        }
    }

    pub fn line() -> Self {
        Self {
            title: None,
            characters: "─".to_string(),
            style: Style::parse("rule.line")
                .unwrap_or_else(|_| Style::parse("dim").unwrap_or_default()),
            align: AlignMethod::Center,
        }
    }

    pub fn characters(mut self, characters: impl Into<String>) -> Self {
        self.characters = characters.into();
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn align(mut self, align: AlignMethod) -> Self {
        self.align = align;
        self
    }
}

impl Renderable for Rule {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let width = options.max_width;
        let char_str = if self.characters.is_empty() {
            "─"
        } else {
            &self.characters
        };
        let char_len = cell_len(char_str).max(1);

        if let Some(ref title) = self.title {
            let title_text = render_markup(title).unwrap_or_else(|_| Text::from_plain(title));
            let title_width = title_text.cell_len();
            let total_margin = 2; // 1 space on each side

            let available = width.saturating_sub(title_width + total_margin);
            let (left_w, right_w) = match self.align {
                AlignMethod::Left => (char_len * 2, available.saturating_sub(char_len * 2)),
                AlignMethod::Right => (available.saturating_sub(char_len * 2), char_len * 2),
                AlignMethod::Center => {
                    let left = available / 2;
                    (left, available - left)
                }
            };

            let left_chars = left_w / char_len;
            let right_chars = right_w / char_len;

            sink.push(Segment {
                text: Cow::Owned(char_str.repeat(left_chars)),
                style: Some(self.style.clone()),
                control: None,
            });
            sink.push(Segment {
                text: Cow::Borrowed(" "),
                style: None,
                control: None,
            });

            for seg in title_text.render_to_segments() {
                sink.push(seg.into_segment());
            }

            sink.push(Segment {
                text: Cow::Borrowed(" "),
                style: None,
                control: None,
            });
            sink.push(Segment {
                text: Cow::Owned(char_str.repeat(right_chars)),
                style: Some(self.style.clone()),
                control: None,
            });
        } else {
            let count = width / char_len;
            sink.push(Segment {
                text: Cow::Owned(char_str.repeat(count)),
                style: Some(self.style.clone()),
                control: None,
            });
        }
    }

    fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        Measurement::new(10, options.max_width)
    }
}
