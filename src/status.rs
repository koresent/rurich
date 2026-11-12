use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::spinner_bar::Spinner;
use crate::style::Style;
use crate::text::Text;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Status {
    status_text: Arc<Mutex<String>>,
    spinner: Arc<Mutex<Spinner>>,
    spinner_style: Option<Style>,
}

impl Status {
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            status_text: Arc::new(Mutex::new(status.into())),
            spinner: Arc::new(Mutex::new(Spinner::new(
                vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
                80,
            ))),
            spinner_style: Some(Style::parse("bold cyan").unwrap_or_default()),
        }
    }

    pub fn update(&self, status: impl Into<String>) {
        let mut text = self.status_text.lock().unwrap();
        *text = status.into();
    }

    pub fn spinner_style(mut self, style: Style) -> Self {
        self.spinner_style = Some(style);
        self
    }
}

impl Renderable for Status {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        _options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let spinner = self.spinner.lock().unwrap();
        let frame = spinner.get_frame();
        let text_guard = self.status_text.lock().unwrap();

        let mut text = Text::new();
        text.append(frame, self.spinner_style.clone());
        text.append(" ", None);
        text.append(&text_guard, None);

        let segments = text.render_to_segments();
        for seg in segments {
            sink.push(seg.into_segment());
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let text_guard = self.status_text.lock().unwrap();
        let text = Text::from_plain(format!("{} {}", "⠋", text_guard));
        text.measure(console, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_status_render() {
        let status = Status::new("Processing batch jobs...");
        let console = Console::with_dimensions(40, 2).with_color_system(ColorSystem::NoColor);
        let out = console.capture(|c| {
            c.print(&status);
        });

        assert!(out.contains("Processing batch jobs..."));
    }
}
