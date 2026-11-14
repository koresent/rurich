use crate::cells::cell_len;
use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use std::borrow::Cow;

const SPARK_BARS: [char; 8] = ['\u{2581}', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// A mini-chart widget that visualizes a sequence of numeric values.
#[derive(Debug, Clone)]
pub struct Sparkline {
    pub data: Vec<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub style: Option<Style>,
}

impl Sparkline {
    pub fn new(data: impl IntoIterator<Item = f64>) -> Self {
        Self {
            data: data.into_iter().collect(),
            min: None,
            max: None,
            style: None,
        }
    }

    pub fn from_integers(data: impl IntoIterator<Item = i64>) -> Self {
        Self::new(data.into_iter().map(|v| v as f64))
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn render_string(&self) -> String {
        if self.data.is_empty() {
            return String::new();
        }

        let computed_min = self
            .min
            .unwrap_or_else(|| self.data.iter().copied().fold(f64::INFINITY, f64::min));
        let computed_max = self
            .max
            .unwrap_or_else(|| self.data.iter().copied().fold(f64::NEG_INFINITY, f64::max));

        let range = computed_max - computed_min;

        let mut res = String::with_capacity(self.data.len());
        for &val in &self.data {
            let clamped = val.clamp(computed_min, computed_max);
            let idx = if range <= 0.0 {
                0
            } else {
                let frac = (clamped - computed_min) / range;
                let scaled = (frac * 7.0).round() as usize;
                scaled.min(7)
            };
            res.push(SPARK_BARS[idx]);
        }
        res
    }
}

impl Renderable for Sparkline {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        _options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let s = self.render_string();
        sink.push(Segment {
            text: Cow::Owned(s),
            style: self.style.clone(),
            control: None,
        });
    }

    fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let w = cell_len(&self.render_string());
        Measurement::new(w, w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_sparkline_rendering() {
        let spark = Sparkline::new(vec![0.0, 3.5, 7.0]);
        assert_eq!(spark.render_string(), "\u{2581}▅█");
    }

    #[test]
    fn test_sparkline_console() {
        let console = Console::with_dimensions(20, 5).with_color_system(ColorSystem::NoColor);
        let spark = Sparkline::new(vec![0.0, 3.5, 7.0]);
        let out = console.capture(|c| c.print(spark));
        assert_eq!(out.trim(), "\u{2581}▅█");
    }
}
