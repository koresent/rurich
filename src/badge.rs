use crate::cells::cell_len;
use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeStyle {
    #[default]
    Square, // [ TEXT ]
    Rounded,   // ( TEXT )
    Powerline, // TEXT
    Minimal,   //  TEXT
}

#[derive(Debug, Clone)]
pub struct Badge {
    pub text: String,
    pub style: Style,
    pub badge_style: BadgeStyle,
}

impl Badge {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::parse("bold white on blue").unwrap_or_default(),
            badge_style: BadgeStyle::Square,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn badge_style(mut self, badge_style: BadgeStyle) -> Self {
        self.badge_style = badge_style;
        self
    }

    pub fn success(text: impl Into<String>) -> Self {
        Self::new(text).style(Style::parse("bold white on green").unwrap_or_default())
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self::new(text).style(Style::parse("bold white on red").unwrap_or_default())
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self::new(text).style(Style::parse("bold black on yellow").unwrap_or_default())
    }

    pub fn info(text: impl Into<String>) -> Self {
        Self::new(text).style(Style::parse("bold white on cyan").unwrap_or_default())
    }
}

impl Renderable for Badge {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        _options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let (open, close) = match self.badge_style {
            BadgeStyle::Square => ("[ ", " ]"),
            BadgeStyle::Rounded => ("( ", " )"),
            BadgeStyle::Powerline => ("", ""),
            BadgeStyle::Minimal => (" ", " "),
        };

        sink.push(Segment {
            text: Cow::Borrowed(open),
            style: Some(self.style.clone()),
            control: None,
        });
        sink.push(Segment {
            text: Cow::Owned(self.text.clone()),
            style: Some(self.style.clone()),
            control: None,
        });
        sink.push(Segment {
            text: Cow::Borrowed(close),
            style: Some(self.style.clone()),
            control: None,
        });
    }

    fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let extra = match self.badge_style {
            BadgeStyle::Square | BadgeStyle::Rounded => 4,
            BadgeStyle::Powerline => 2,
            BadgeStyle::Minimal => 2,
        };
        let w = cell_len(&self.text) + extra;
        Measurement::new(w, w)
    }
}

#[derive(Debug, Clone)]
pub struct Breadcrumbs {
    pub items: Vec<String>,
    pub separator: String,
    pub path_style: Style,
    pub active_style: Style,
    pub separator_style: Style,
}

impl Breadcrumbs {
    pub fn new(items: Vec<impl Into<String>>) -> Self {
        Self {
            items: items.into_iter().map(|s| s.into()).collect(),
            separator: " ❯ ".to_string(),
            path_style: Style::parse("dim").unwrap_or_default(),
            active_style: Style::parse("bold cyan").unwrap_or_default(),
            separator_style: Style::parse("bold yellow").unwrap_or_default(),
        }
    }

    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    pub fn path_style(mut self, style: Style) -> Self {
        self.path_style = style;
        self
    }

    pub fn active_style(mut self, style: Style) -> Self {
        self.active_style = style;
        self
    }

    pub fn separator_style(mut self, style: Style) -> Self {
        self.separator_style = style;
        self
    }
}

impl Renderable for Breadcrumbs {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        _options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let total = self.items.len();
        for (i, item) in self.items.iter().enumerate() {
            let is_last = i + 1 == total;
            let style = if is_last {
                self.active_style.clone()
            } else {
                self.path_style.clone()
            };

            sink.push(Segment {
                text: Cow::Owned(item.clone()),
                style: Some(style),
                control: None,
            });

            if !is_last {
                sink.push(Segment {
                    text: Cow::Owned(self.separator.clone()),
                    style: Some(self.separator_style.clone()),
                    control: None,
                });
            }
        }
    }

    fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let total = self.items.len();
        if total == 0 {
            return Measurement::new(0, 0);
        }
        let items_len: usize = self.items.iter().map(|s| cell_len(s)).sum();
        let seps_len = cell_len(&self.separator) * (total.saturating_sub(1));
        let w = items_len + seps_len;
        Measurement::new(w, w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_badge_square() {
        let console = Console::with_dimensions(20, 5).with_color_system(ColorSystem::NoColor);
        let badge = Badge::success("PASS");
        let out = console.capture(|c| c.print(badge));
        assert_eq!(out.trim(), "[ PASS ]");
    }

    #[test]
    fn test_breadcrumbs() {
        let console = Console::with_dimensions(30, 5).with_color_system(ColorSystem::NoColor);
        let bc = Breadcrumbs::new(vec!["root", "home", "user"]).separator(" / ");
        let out = console.capture(|c| c.print(bc));
        assert_eq!(out.trim(), "root / home / user");
    }
}
