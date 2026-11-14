use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdmonitionType {
    #[default]
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl AdmonitionType {
    pub fn default_title(&self) -> &'static str {
        match self {
            AdmonitionType::Note => "Note",
            AdmonitionType::Tip => "Tip",
            AdmonitionType::Important => "Important",
            AdmonitionType::Warning => "Warning",
            AdmonitionType::Caution => "Caution",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AdmonitionType::Note => "ℹ ",
            AdmonitionType::Tip => "💡 ",
            AdmonitionType::Important => "❗ ",
            AdmonitionType::Warning => "⚠️  ",
            AdmonitionType::Caution => "🛑 ",
        }
    }

    pub fn default_style(&self) -> Style {
        match self {
            AdmonitionType::Note => Style::parse("bold cyan").unwrap_or_default(),
            AdmonitionType::Tip => Style::parse("bold green").unwrap_or_default(),
            AdmonitionType::Important => Style::parse("bold magenta").unwrap_or_default(),
            AdmonitionType::Warning => Style::parse("bold yellow").unwrap_or_default(),
            AdmonitionType::Caution => Style::parse("bold red").unwrap_or_default(),
        }
    }
}

pub struct Callout {
    pub admonition_type: AdmonitionType,
    pub title: Option<String>,
    pub renderable: Box<dyn Renderable>,
    pub style: Option<Style>,
}

impl Callout {
    pub fn new(admonition_type: AdmonitionType, renderable: impl Renderable + 'static) -> Self {
        Self {
            admonition_type,
            title: None,
            renderable: Box::new(renderable),
            style: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn note(renderable: impl Renderable + 'static) -> Self {
        Self::new(AdmonitionType::Note, renderable)
    }

    pub fn tip(renderable: impl Renderable + 'static) -> Self {
        Self::new(AdmonitionType::Tip, renderable)
    }

    pub fn important(renderable: impl Renderable + 'static) -> Self {
        Self::new(AdmonitionType::Important, renderable)
    }

    pub fn warning(renderable: impl Renderable + 'static) -> Self {
        Self::new(AdmonitionType::Warning, renderable)
    }

    pub fn caution(renderable: impl Renderable + 'static) -> Self {
        Self::new(AdmonitionType::Caution, renderable)
    }
}

impl Renderable for Callout {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let accent_style = self
            .style
            .clone()
            .unwrap_or_else(|| self.admonition_type.default_style());

        let title_str = self
            .title
            .as_deref()
            .unwrap_or_else(|| self.admonition_type.default_title());

        let mut header = Text::new();
        header.append(self.admonition_type.icon(), Some(accent_style.clone()));
        header.append(title_str, Some(accent_style.clone()));

        let inner_width = options.max_width.saturating_sub(4);
        let child_opts = options.update_width(inner_width);

        // Header lines
        let header_lines = console.render_lines_owned(&header, Some(&child_opts), true);
        for line in header_lines {
            sink.push(Segment {
                text: Cow::Borrowed("▌ "),
                style: Some(accent_style.clone()),
                control: None,
            });
            sink.extend(line);
            sink.push(Segment::line());
        }

        // Content lines
        let content_lines = console.render_lines_owned(&*self.renderable, Some(&child_opts), true);
        for line in content_lines {
            sink.push(Segment {
                text: Cow::Borrowed("▌ "),
                style: Some(accent_style.clone()),
                control: None,
            });
            sink.extend(line);
            sink.push(Segment::line());
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let m = self.renderable.measure(console, options);
        Measurement::new(m.minimum + 3, m.maximum + 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_callout_note() {
        let console = Console::with_dimensions(30, 5).with_color_system(ColorSystem::NoColor);
        let callout = Callout::note("This is important info");
        let out = console.capture(|c| c.print(callout));
        assert!(out.contains("Note"));
        assert!(out.contains("This is important info"));
    }
}
