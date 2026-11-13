use crate::cells::cell_len;
use crate::color::ColorSystem;
use crate::markup::render as render_markup;
use crate::protocol::{ConsoleDimensions, ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use crate::theme::Theme;
use crossterm::terminal;
use std::borrow::Cow;
use std::env;
use std::io::{self, IsTerminal, Write};
use std::sync::{Arc, Mutex, OnceLock};

pub struct Console {
    pub color_system: ColorSystem,
    pub is_terminal: bool,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub theme: Theme,
    pub no_color: bool,
    buffer: Arc<Mutex<Vec<Segment<'static>>>>,
    is_capturing: Arc<Mutex<bool>>,
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

impl Console {
    pub fn new() -> Self {
        let is_terminal = io::stdout().is_terminal();
        let no_color =
            env::var("NO_COLOR").is_ok() || env::var("CLICOLOR").map(|v| v == "0").unwrap_or(false);

        let color_system = if no_color {
            ColorSystem::NoColor
        } else {
            Self::detect_color_system(is_terminal)
        };

        Self {
            color_system,
            is_terminal,
            width: None,
            height: None,
            theme: crate::theme::get_global_theme(),
            no_color,
            buffer: Arc::new(Mutex::new(Vec::new())),
            is_capturing: Arc::new(Mutex::new(false)),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_dimensions(width: usize, height: usize) -> Self {
        let mut console = Self::new();
        console.width = Some(width);
        console.height = Some(height);
        console
    }

    pub fn with_color_system(mut self, system: ColorSystem) -> Self {
        self.color_system = system;
        self
    }

    pub fn detect_color_system(is_terminal: bool) -> ColorSystem {
        if !is_terminal
            && !env::var("CLICOLOR_FORCE")
                .map(|v| v != "0")
                .unwrap_or(false)
        {
            return ColorSystem::NoColor;
        }

        if let Ok(colorterm) = env::var("COLORTERM")
            && (colorterm == "truecolor" || colorterm == "24bit")
        {
            return ColorSystem::TrueColor;
        }

        if let Ok(term) = env::var("TERM") {
            if term.contains("256color") {
                return ColorSystem::EightBit;
            }
            if term == "dumb" {
                return ColorSystem::NoColor;
            }
        }

        #[cfg(windows)]
        {
            return ColorSystem::TrueColor;
        }

        ColorSystem::Standard
    }

    pub fn size(&self) -> ConsoleDimensions {
        if let (Some(w), Some(h)) = (self.width, self.height) {
            return ConsoleDimensions::new(w, h);
        }

        if let Ok((cols, rows)) = terminal::size() {
            let w = self.width.unwrap_or(cols as usize);
            let h = self.height.unwrap_or(rows as usize);
            ConsoleDimensions::new(w, h)
        } else {
            ConsoleDimensions::new(self.width.unwrap_or(80), self.height.unwrap_or(25))
        }
    }

    pub fn options(&self) -> ConsoleOptions {
        let size = self.size();
        ConsoleOptions::new(size.width, size.height, self.is_terminal)
    }

    pub fn get_style(&self, name_or_style: &str) -> Style {
        if let Some(theme_style) = self.theme.get(name_or_style) {
            return theme_style;
        }
        Style::parse(name_or_style).unwrap_or_default()
    }

    pub fn begin_capture(&self) {
        let mut capturing = self.is_capturing.lock().unwrap();
        *capturing = true;
        let mut buf = self.buffer.lock().unwrap();
        buf.clear();
    }

    pub fn end_capture(&self) -> String {
        let mut capturing = self.is_capturing.lock().unwrap();
        *capturing = false;
        let mut buf = self.buffer.lock().unwrap();
        let rendered = Segment::render_ansi(&buf, self.color_system);
        buf.clear();
        rendered
    }

    pub fn capture(&self, f: impl FnOnce(&Console)) -> String {
        self.begin_capture();
        f(self);
        self.end_capture()
    }

    pub fn render_lines<'a>(
        &self,
        renderable: &'a dyn Renderable,
        options: Option<&ConsoleOptions>,
        pad: bool,
    ) -> Vec<Vec<Segment<'a>>> {
        let default_opts = self.options();
        let opts = options.unwrap_or(&default_opts);
        let segments = renderable.render(self, opts);
        Segment::split_and_crop_lines(segments, opts.max_width, false, pad)
    }

    pub fn render_lines_owned(
        &self,
        renderable: &dyn Renderable,
        options: Option<&ConsoleOptions>,
        pad: bool,
    ) -> Vec<Vec<Segment<'static>>> {
        let default_opts = self.options();
        let opts = options.unwrap_or(&default_opts);
        let segments = renderable.render(self, opts);
        let lines = Segment::split_and_crop_lines(segments, opts.max_width, false, pad);
        lines
            .into_iter()
            .map(|line| line.into_iter().map(|s| s.into_segment()).collect())
            .collect()
    }

    /// Export a renderable directly to standalone HTML.
    pub fn export_html(&self, renderable: &dyn Renderable) -> String {
        let lines = self.render_lines_owned(renderable, None, false);
        crate::export::export_html(&lines)
    }

    /// Export a renderable directly to a stylized SVG terminal window image.
    pub fn export_svg(&self, renderable: &dyn Renderable, title: &str) -> String {
        let lines = self.render_lines_owned(renderable, None, false);
        crate::export::export_svg(&lines, title)
    }

    /// Run a closure inside an alternate screen buffer (fullscreen mode), automatically restoring when done.
    pub fn screen<F, R>(&self, f: F) -> io::Result<R>
    where
        F: FnOnce(&Console) -> R,
    {
        let _guard = crate::screen::ScreenGuard::enter()?;
        let res = f(self);
        Ok(res)
    }

    pub fn print(&self, renderable: impl Renderable) {
        let options = self.options();
        let mut segments = Vec::new();
        renderable.render_to(self, &options, &mut segments);

        let owned_segments: Vec<Segment<'static>> = segments
            .into_iter()
            .map(|s| Segment {
                text: Cow::Owned(s.text.into_owned()),
                style: s.style,
                control: s.control,
            })
            .collect();

        let is_cap = *self.is_capturing.lock().unwrap();
        if is_cap {
            let mut buf = self.buffer.lock().unwrap();
            buf.extend(owned_segments);
        } else {
            let ansi = Segment::render_ansi(&owned_segments, self.color_system);
            let mut stdout = io::stdout().lock();
            let _ = stdout.write_all(ansi.as_bytes());
            let _ = stdout.flush();
        }
    }

    pub fn eprint(&self, renderable: impl Renderable) {
        let options = self.options();
        let mut segments = Vec::new();
        renderable.render_to(self, &options, &mut segments);

        let owned_segments: Vec<Segment<'static>> = segments
            .into_iter()
            .map(|s| Segment {
                text: Cow::Owned(s.text.into_owned()),
                style: s.style,
                control: s.control,
            })
            .collect();

        let ansi = Segment::render_ansi(&owned_segments, self.color_system);
        let mut stderr = io::stderr().lock();
        let _ = stderr.write_all(ansi.as_bytes());
        let _ = stderr.flush();
    }

    pub fn eprintln(&self, renderable: impl Renderable) {
        self.eprint(renderable);
        let mut stderr = io::stderr().lock();
        let _ = stderr.write_all(b"\n");
        let _ = stderr.flush();
    }

    pub fn println(&self, renderable: impl Renderable) {
        self.print(renderable);
        let is_cap = *self.is_capturing.lock().unwrap();
        if is_cap {
            let mut buf = self.buffer.lock().unwrap();
            buf.push(Segment::line());
        } else {
            let mut stdout = io::stdout().lock();
            let _ = stdout.write_all(b"\n");
            let _ = stdout.flush();
        }
    }

    pub fn print_markup(&self, markup: &str) {
        match render_markup(markup) {
            Ok(text) => self.print(text),
            Err(_) => self.print(markup),
        }
    }

    pub fn println_markup(&self, markup: &str) {
        match render_markup(markup) {
            Ok(text) => self.println(text),
            Err(_) => self.println(markup),
        }
    }

    pub fn rule(&self, title: Option<&str>) {
        let width = self.size().width;
        let line_char = "─";
        let rule_style = self.get_style("rule.line");

        if let Some(t) = title {
            let title_len = cell_len(t);
            let total_margin = 2; // 1 space before, 1 space after
            if width > title_len + total_margin {
                let side_len = (width - title_len - total_margin) / 2;
                let left_side = line_char.repeat(side_len);
                let right_len = width - title_len - total_margin - side_len;
                let right_side = line_char.repeat(right_len);

                let mut text = Text::new();
                text.append(&left_side, Some(rule_style.clone()));
                text.append(" ", None);
                let text_style = self.get_style("rule.text");
                text.append(t, Some(text_style));
                text.append(" ", None);
                text.append(&right_side, Some(rule_style));
                self.println(text);
                return;
            }
        }

        let line = line_char.repeat(width);
        let mut text = Text::new();
        text.append(&line, Some(rule_style));
        self.println(text);
    }
}

/// Global shared Console instance
pub static GLOBAL_CONSOLE: OnceLock<Console> = OnceLock::new();

pub fn get_console() -> &'static Console {
    GLOBAL_CONSOLE.get_or_init(Console::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_capture() {
        let console = Console::with_dimensions(80, 24).with_color_system(ColorSystem::Standard);
        let output = console.capture(|c| {
            c.print_markup("[bold red]Error:[/] Something went wrong");
        });
        assert!(output.contains("\x1b[1;31mError:\x1b[0m Something went wrong"));
    }

    #[test]
    fn test_console_rule() {
        let console = Console::with_dimensions(20, 10).with_color_system(ColorSystem::NoColor);
        let output = console.capture(|c| {
            c.rule(Some("Test"));
        });
        assert!(output.contains("Test"));
        assert!(output.contains("─"));
    }
}
