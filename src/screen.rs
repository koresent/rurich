use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crossterm::cursor::{Hide, Show};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use std::io::{self, Write};

/// A renderable that fills the terminal dimensions and pads/crops excess content.
pub struct Screen<T> {
    pub renderable: T,
    pub style: Option<Style>,
}

impl<T: Renderable> Screen<T> {
    pub fn new(renderable: T) -> Self {
        Self {
            renderable,
            style: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }
}

impl<T: Renderable> Renderable for Screen<T> {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let lines = console.render_lines_owned(&self.renderable, Some(options), true);
        for line in lines {
            for seg in line {
                sink.push(seg);
            }
            sink.push(Segment::line());
        }
    }

    fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        Measurement::new(options.max_width, options.max_width)
    }
}

/// RAII Guard that restores the normal terminal screen when dropped.
pub struct ScreenGuard;

impl ScreenGuard {
    pub fn enter() -> io::Result<Self> {
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, Hide)?;
        stdout.flush()?;
        Ok(Self)
    }
}

impl Drop for ScreenGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = crossterm::execute!(stdout, Show, LeaveAlternateScreen);
        let _ = stdout.flush();
    }
}
