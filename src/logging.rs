use crate::console::Console;
use crate::highlighter::{Highlighter, ReprHighlighter};
use crate::markup::render as render_markup;
use crate::style::Style;
use crate::text::Text;
use std::fmt;
use std::sync::Arc;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

struct FieldVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: Vec::new(),
        }
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}").trim_matches('"').to_string());
        } else {
            self.fields
                .push((field.name().to_string(), format!("{value:?}")));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields
                .push((field.name().to_string(), value.to_string()));
        }
    }
}

/// Tracing subscriber layer that formats logs with styles, level badges, and file targets.
#[derive(Clone)]
pub struct ConsoleLayer {
    console: Arc<Console>,
    show_time: bool,
    show_level: bool,
    show_target: bool,
}

pub type RuRichLayer = ConsoleLayer;

impl ConsoleLayer {
    pub fn new() -> Self {
        Self {
            console: Arc::new(Console::new()),
            show_time: true,
            show_level: true,
            show_target: true,
        }
    }

    pub fn with_console(mut self, console: Console) -> Self {
        self.console = Arc::new(console);
        self
    }

    pub fn show_time(mut self, show: bool) -> Self {
        self.show_time = show;
        self
    }

    pub fn show_level(mut self, show: bool) -> Self {
        self.show_level = show;
        self
    }

    pub fn show_target(mut self, show: bool) -> Self {
        self.show_target = show;
        self
    }
}

impl Default for ConsoleLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for ConsoleLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        let mut row = Text::new();

        // Level badge
        if self.show_level {
            let (level_str, level_style) = match *metadata.level() {
                Level::TRACE => ("TRACE", Style::parse("bold magenta").unwrap_or_default()),
                Level::DEBUG => ("DEBUG", Style::parse("bold blue").unwrap_or_default()),
                Level::INFO => ("INFO ", Style::parse("bold green").unwrap_or_default()),
                Level::WARN => ("WARN ", Style::parse("bold yellow").unwrap_or_default()),
                Level::ERROR => ("ERROR", Style::parse("bold red").unwrap_or_default()),
            };
            row.append(level_str, Some(level_style));
            row.append(" ", None);
        }

        // Target / file location
        if self.show_target {
            let target_str = if let (Some(file), Some(line)) = (metadata.file(), metadata.line()) {
                format!("{file}:{line} ")
            } else {
                format!("{} ", metadata.target())
            };
            row.append(
                &target_str,
                Some(Style::parse("dim cyan").unwrap_or_default()),
            );
        }

        // Message
        if let Some(msg) = visitor.message {
            let mut msg_text = render_markup(&msg).unwrap_or_else(|_| Text::from_plain(msg));
            ReprHighlighter.highlight(&mut msg_text);
            row.append_text(&msg_text);
        }

        // Extra fields
        for (k, v) in visitor.fields {
            row.append(" ", None);
            row.append(
                &format!("{k}="),
                Some(Style::parse("italic dim").unwrap_or_default()),
            );
            let mut val_text = Text::from_plain(v);
            ReprHighlighter.highlight(&mut val_text);
            row.append_text(&val_text);
        }

        if *metadata.level() == Level::ERROR {
            self.console.eprintln(row);
        } else {
            self.console.println(row);
        }
    }
}
