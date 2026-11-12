use crate::console::Console;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use std::borrow::Cow;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Spinner {
    pub frames: Vec<String>,
    pub interval: Duration,
    pub start_time: Instant,
}

impl Spinner {
    pub fn new(frames: Vec<&str>, interval_ms: u64) -> Self {
        Self {
            frames: frames.into_iter().map(|s| s.to_string()).collect(),
            interval: Duration::from_millis(interval_ms),
            start_time: Instant::now(),
        }
    }

    pub fn get_frame(&self) -> &str {
        let elapsed = self.start_time.elapsed();
        let index = (elapsed.as_millis() / self.interval.as_millis()) as usize % self.frames.len();
        &self.frames[index]
    }
}

impl Renderable for Spinner {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        _options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        sink.push(Segment::unstyled(Cow::Owned(self.get_frame().to_string())));
    }

    fn measure(
        &self,
        _console: &Console,
        _options: &ConsoleOptions,
    ) -> crate::measure::Measurement {
        crate::measure::Measurement::new(1, 1)
    }
}

pub struct ProgressBar {
    pub total: usize,
    pub completed: usize,
    pub width: usize,
    pub style: Style,
    pub pulse: bool,
}

impl ProgressBar {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            completed: 0,
            width: 20,
            style: Style::new(),
            pulse: false,
        }
    }

    pub fn update(&mut self, completed: usize) {
        self.completed = completed.min(self.total);
    }
}

impl Renderable for ProgressBar {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        _options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let percent = if self.total > 0 {
            self.completed as f64 / self.total as f64
        } else {
            0.0
        };
        let filled = (percent * self.width as f64).round() as usize;

        let mut bar = String::new();
        for i in 0..self.width {
            if i < filled {
                bar.push('█');
            } else {
                bar.push('░');
            }
        }

        sink.push(Segment {
            text: Cow::Owned(bar),
            style: Some(self.style.clone()),
            control: None,
        });
    }

    fn measure(
        &self,
        _console: &Console,
        _options: &ConsoleOptions,
    ) -> crate::measure::Measurement {
        crate::measure::Measurement::new(self.width, self.width)
    }
}
