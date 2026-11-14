use crate::color::{Color, ColorTriplet};
use crate::console::Console;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use std::borrow::Cow;
use std::time::{Duration, Instant};

const FRACTIONAL_BLOCKS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerPreset {
    Dots,
    Dots2,
    Line,
    MiniDot,
    Jump,
    Pulse,
    Points,
    Globe,
    Moon,
    Clock,
    BouncingBar,
    Pipe,
    Arc,
    SquareCorners,
    CircleHalves,
    Arrow,
    Triangle,
    GrowHorizontal,
    GrowVertical,
    Hamburger,
    Monkey,
}

impl SpinnerPreset {
    pub const ALL: &'static [SpinnerPreset] = &[
        SpinnerPreset::Dots,
        SpinnerPreset::Dots2,
        SpinnerPreset::Line,
        SpinnerPreset::MiniDot,
        SpinnerPreset::Jump,
        SpinnerPreset::Pulse,
        SpinnerPreset::Points,
        SpinnerPreset::Globe,
        SpinnerPreset::Moon,
        SpinnerPreset::Clock,
        SpinnerPreset::BouncingBar,
        SpinnerPreset::Pipe,
        SpinnerPreset::Arc,
        SpinnerPreset::SquareCorners,
        SpinnerPreset::CircleHalves,
        SpinnerPreset::Arrow,
        SpinnerPreset::Triangle,
        SpinnerPreset::GrowHorizontal,
        SpinnerPreset::GrowVertical,
        SpinnerPreset::Hamburger,
        SpinnerPreset::Monkey,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            SpinnerPreset::Dots => "dots",
            SpinnerPreset::Dots2 => "dots2",
            SpinnerPreset::Line => "line",
            SpinnerPreset::MiniDot => "mini_dot",
            SpinnerPreset::Jump => "jump",
            SpinnerPreset::Pulse => "pulse",
            SpinnerPreset::Points => "points",
            SpinnerPreset::Globe => "globe",
            SpinnerPreset::Moon => "moon",
            SpinnerPreset::Clock => "clock",
            SpinnerPreset::BouncingBar => "bouncing_bar",
            SpinnerPreset::Pipe => "pipe",
            SpinnerPreset::Arc => "arc",
            SpinnerPreset::SquareCorners => "square_corners",
            SpinnerPreset::CircleHalves => "circle_halves",
            SpinnerPreset::Arrow => "arrow",
            SpinnerPreset::Triangle => "triangle",
            SpinnerPreset::GrowHorizontal => "grow_horizontal",
            SpinnerPreset::GrowVertical => "grow_vertical",
            SpinnerPreset::Hamburger => "hamburger",
            SpinnerPreset::Monkey => "monkey",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spinner {
    pub frames: Vec<String>,
    pub interval: Duration,
    pub start_time: Instant,
    pub style: Option<Style>,
}

impl Spinner {
    pub fn new(frames: Vec<&str>, interval_ms: u64) -> Self {
        Self {
            frames: frames.into_iter().map(|s| s.to_string()).collect(),
            interval: Duration::from_millis(interval_ms),
            start_time: Instant::now(),
            style: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn from_preset(preset: SpinnerPreset) -> Self {
        match preset {
            SpinnerPreset::Dots => {
                Self::new(vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"], 80)
            }
            SpinnerPreset::Dots2 => Self::new(vec!["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"], 80),
            SpinnerPreset::Line => Self::new(vec!["-", "\\", "|", "/"], 130),
            SpinnerPreset::MiniDot => {
                Self::new(vec!["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠲", "⠳", "⠓"], 80)
            }
            SpinnerPreset::Jump => Self::new(vec!["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"], 100),
            SpinnerPreset::Pulse => Self::new(vec!["█", "▓", "▒", "░"], 100),
            SpinnerPreset::Points => Self::new(vec!["∙∙∙", "●∙∙", "∙●∙", "∙∙●", "∙∙∙"], 120),
            SpinnerPreset::Globe => Self::new(vec!["🌍", "🌎", "🌏"], 180),
            SpinnerPreset::Moon => {
                Self::new(vec!["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"], 100)
            }
            SpinnerPreset::Clock => Self::new(
                vec![
                    "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛",
                ],
                100,
            ),
            SpinnerPreset::BouncingBar => Self::new(
                vec![
                    "[    ]", "[=   ]", "[==  ]", "[=== ]", "[====]", "[ ===]", "[  ==]", "[   =]",
                    "[    ]",
                ],
                80,
            ),
            SpinnerPreset::Pipe => Self::new(vec!["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"], 100),
            SpinnerPreset::Arc => Self::new(vec!["◜", "◠", "◝", "◞", "◡", "◟"], 100),
            SpinnerPreset::SquareCorners => Self::new(vec!["◰", "◳", "◲", "◱"], 100),
            SpinnerPreset::CircleHalves => Self::new(vec!["◐", "◓", "◑", "◒"], 100),
            SpinnerPreset::Arrow => Self::new(vec!["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"], 100),
            SpinnerPreset::Triangle => Self::new(vec!["◢", "◣", "◤", "◥"], 100),
            SpinnerPreset::GrowHorizontal => Self::new(
                vec!["▏", "▎", "▍", "▌", "▋", "▊", "▉", "▊", "▋", "▌", "▍", "▎"],
                100,
            ),
            SpinnerPreset::GrowVertical => {
                Self::new(vec![" ", "▃", "▄", "▅", "▆", "▇", "▆", "▅", "▄", "▃"], 100)
            }
            SpinnerPreset::Hamburger => Self::new(vec!["☱", "☲", "☴"], 100),
            SpinnerPreset::Monkey => Self::new(vec!["🙈", "🙉", "🙊"], 300),
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        let preset = match name.to_lowercase().as_str() {
            "dots" => SpinnerPreset::Dots,
            "dots2" => SpinnerPreset::Dots2,
            "line" => SpinnerPreset::Line,
            "minidot" | "mini_dot" => SpinnerPreset::MiniDot,
            "jump" => SpinnerPreset::Jump,
            "pulse" => SpinnerPreset::Pulse,
            "points" => SpinnerPreset::Points,
            "globe" => SpinnerPreset::Globe,
            "moon" => SpinnerPreset::Moon,
            "clock" => SpinnerPreset::Clock,
            "bouncingbar" | "bouncing_bar" => SpinnerPreset::BouncingBar,
            "pipe" => SpinnerPreset::Pipe,
            "arc" => SpinnerPreset::Arc,
            "squarecorners" | "square_corners" => SpinnerPreset::SquareCorners,
            "circlehalves" | "circle_halves" => SpinnerPreset::CircleHalves,
            "arrow" => SpinnerPreset::Arrow,
            "triangle" => SpinnerPreset::Triangle,
            "growhorizontal" | "grow_horizontal" => SpinnerPreset::GrowHorizontal,
            "growvertical" | "grow_vertical" => SpinnerPreset::GrowVertical,
            "hamburger" => SpinnerPreset::Hamburger,
            "monkey" => SpinnerPreset::Monkey,
            _ => return None,
        };
        Some(Self::from_preset(preset))
    }

    pub fn get_frame(&self) -> &str {
        let elapsed = self.start_time.elapsed();
        let index =
            (elapsed.as_millis() / self.interval.as_millis().max(1)) as usize % self.frames.len();
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
        sink.push(Segment::styled(
            Cow::Owned(self.get_frame().to_string()),
            self.style.clone().unwrap_or_default(),
        ));
    }

    fn measure(
        &self,
        _console: &Console,
        _options: &ConsoleOptions,
    ) -> crate::measure::Measurement {
        crate::measure::Measurement::new(1, 1)
    }
}

#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub total: usize,
    pub completed: usize,
    pub width: usize,
    pub style: Style,
    pub complete_style: Option<Style>,
    pub finished_style: Option<Style>,
    pub remaining_style: Option<Style>,
    pub complete_char: Option<char>,
    pub remaining_char: Option<char>,
    pub pulse: bool,
    pub pulse_style: Option<Style>,
    pub smooth: bool,
    pub line_mode: bool,
    pub start_time: Instant,
}

impl ProgressBar {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            completed: 0,
            width: 20,
            style: Style::new(),
            complete_style: None,
            finished_style: None,
            remaining_style: None,
            complete_char: None,
            remaining_char: None,
            pulse: false,
            pulse_style: None,
            smooth: true,
            line_mode: false,
            start_time: Instant::now(),
        }
    }

    pub fn line(mut self, line: bool) -> Self {
        self.line_mode = line;
        if line {
            if self.complete_style.is_none() {
                self.complete_style = Some(Style::parse("color(#f92672)").unwrap_or_default());
            }
            if self.finished_style.is_none() {
                self.finished_style = Some(Style::parse("color(#729c1f)").unwrap_or_default());
            }
            if self.remaining_style.is_none() {
                self.remaining_style = Some(Style::parse("color(#3a3a3a)").unwrap_or_default());
            }
        }
        self
    }

    pub fn as_line(mut self) -> Self {
        self.line_mode = true;
        if self.complete_style.is_none() {
            self.complete_style = Some(Style::parse("color(#f92672)").unwrap_or_default());
        }
        if self.finished_style.is_none() {
            self.finished_style = Some(Style::parse("color(#729c1f)").unwrap_or_default());
        }
        if self.remaining_style.is_none() {
            self.remaining_style = Some(Style::parse("color(#3a3a3a)").unwrap_or_default());
        }
        self
    }

    pub fn update(&mut self, completed: usize) {
        self.completed = completed.min(self.total);
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn complete_style(mut self, style: Style) -> Self {
        self.complete_style = Some(style);
        self
    }

    pub fn finished_style(mut self, style: Style) -> Self {
        self.finished_style = Some(style);
        self
    }

    pub fn remaining_style(mut self, style: Style) -> Self {
        self.remaining_style = Some(style);
        self
    }

    pub fn complete_char(mut self, ch: char) -> Self {
        self.complete_char = Some(ch);
        self
    }

    pub fn remaining_char(mut self, ch: char) -> Self {
        self.remaining_char = Some(ch);
        self
    }

    pub fn pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }

    pub fn pulse_style(mut self, style: Style) -> Self {
        self.pulse_style = Some(style);
        self
    }

    pub fn smooth(mut self, smooth: bool) -> Self {
        self.smooth = smooth;
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }
}

impl Renderable for ProgressBar {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        _options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        if self.pulse {
            let bar_ch = self.complete_char.unwrap_or('━');
            let fore_color = self
                .pulse_style
                .as_ref()
                .and_then(|s| s.color.as_ref().map(|c| c.get_truecolor()))
                .unwrap_or(ColorTriplet(249, 38, 114));
            let back_color = self
                .remaining_style
                .as_ref()
                .or_else(|| {
                    if self.style.color.is_some() || self.style.bgcolor.is_some() {
                        Some(&self.style)
                    } else {
                        None
                    }
                })
                .and_then(|s| s.color.as_ref().map(|c| c.get_truecolor()))
                .unwrap_or(ColorTriplet(58, 58, 58));

            let elapsed_secs = self.start_time.elapsed().as_secs_f64();
            let pi2 = std::f64::consts::PI * 2.0;
            let pulse_size = 20.0;
            let speed = 15.0;

            for i in 0..self.width {
                let pos = ((i as f64 - elapsed_secs * speed).rem_euclid(pulse_size)) / pulse_size;
                let fade = 0.5 + (pos * pi2).cos() * 0.5;
                let blended = fore_color.blend(&back_color, fade as f32);
                let mut cell_style = self.pulse_style.clone().unwrap_or_default();
                cell_style = cell_style.color(Color::from_rgb(blended.0, blended.1, blended.2));
                sink.push(Segment::styled(Cow::Owned(bar_ch.to_string()), cell_style));
            }
            return;
        }

        let percent = if self.total > 0 {
            self.completed as f64 / self.total as f64
        } else {
            0.0
        };

        if self.line_mode {
            let total_halves = self.width * 2;
            let complete_halves = if self.total > 0 {
                let comp = self.completed.min(self.total);
                (comp as f64 * total_halves as f64 / self.total as f64).round() as usize
            } else {
                total_halves
            };
            let bar_count = complete_halves / 2;
            let half_bar_count = complete_halves % 2;

            let is_finished = self.total > 0 && self.completed >= self.total;
            let c_style = if is_finished {
                self.finished_style
                    .clone()
                    .or_else(|| self.complete_style.clone())
                    .unwrap_or_else(|| Style::parse("color(#729c1f)").unwrap_or_default())
            } else {
                self.complete_style
                    .clone()
                    .unwrap_or_else(|| Style::parse("color(#f92672)").unwrap_or_default())
            };
            let r_style = self
                .remaining_style
                .clone()
                .unwrap_or_else(|| Style::parse("color(#3a3a3a)").unwrap_or_default());

            if bar_count > 0 {
                let s: String = std::iter::repeat_n('━', bar_count).collect();
                sink.push(Segment::styled(Cow::Owned(s), c_style.clone()));
            }
            if half_bar_count > 0 {
                sink.push(Segment::styled(Cow::Borrowed("╸"), c_style));
            }

            let mut remaining_bars = self.width.saturating_sub(bar_count + half_bar_count);
            if remaining_bars > 0 {
                if half_bar_count == 0 && bar_count > 0 {
                    sink.push(Segment::styled(Cow::Borrowed("╺"), r_style.clone()));
                    remaining_bars -= 1;
                }
                if remaining_bars > 0 {
                    let s: String = std::iter::repeat_n('━', remaining_bars).collect();
                    sink.push(Segment::styled(Cow::Owned(s), r_style));
                }
            }
            return;
        }

        let full_ch = self.complete_char.unwrap_or('█');
        let empty_ch = self.remaining_char.unwrap_or('░');
        let comp_style = self
            .complete_style
            .clone()
            .or_else(|| Some(self.style.clone()));
        let rem_style = self
            .remaining_style
            .clone()
            .or_else(|| Some(self.style.clone()));

        let mut completed_str = String::new();
        let mut remaining_str = String::new();

        if self.smooth && self.complete_char.is_none() {
            let total_steps = self.width * 8;
            let completed_steps = (percent * total_steps as f64).round() as usize;
            let full_cells = completed_steps / 8;
            let remainder = completed_steps % 8;

            for i in 0..self.width {
                if i < full_cells {
                    completed_str.push(full_ch);
                } else if i == full_cells && remainder > 0 {
                    completed_str.push(FRACTIONAL_BLOCKS[remainder]);
                } else {
                    remaining_str.push(empty_ch);
                }
            }
        } else {
            let filled = (percent * self.width as f64).round() as usize;
            for i in 0..self.width {
                if i < filled {
                    completed_str.push(full_ch);
                } else {
                    remaining_str.push(empty_ch);
                }
            }
        }

        if !completed_str.is_empty() {
            sink.push(Segment {
                text: Cow::Owned(completed_str),
                style: comp_style,
                control: None,
            });
        }
        if !remaining_str.is_empty() {
            sink.push(Segment {
                text: Cow::Owned(remaining_str),
                style: rem_style,
                control: None,
            });
        }
    }

    fn measure(
        &self,
        _console: &Console,
        _options: &ConsoleOptions,
    ) -> crate::measure::Measurement {
        crate::measure::Measurement::new(self.width, self.width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_spinner_presets() {
        let spinner = Spinner::from_preset(SpinnerPreset::Moon);
        assert_eq!(spinner.frames.len(), 8);

        let by_name = Spinner::by_name("moon");
        assert!(by_name.is_some());
    }

    #[test]
    fn test_smooth_progress_bar() {
        let mut pb = ProgressBar::new(100).smooth(true);
        pb.width = 10;
        pb.update(25); // 25% of 10 cells = 2.5 cells (2 full, remainder 4)

        let console = Console::with_dimensions(20, 5).with_color_system(ColorSystem::NoColor);
        let out = console.capture(|c| c.print(&pb));
        assert!(out.contains("██▌░░░░░░░"));
    }
}
