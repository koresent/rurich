use crate::console::Console;
use crate::measure::Measurement;
use crate::panel::Panel;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use std::backtrace::Backtrace;
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::panic::PanicHookInfo;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct StackFrame {
    pub filename: String,
    pub lineno: usize,
    pub function: String,
    pub code_context: Option<Vec<(usize, String, bool)>>, // (line_number, line_text, is_fault)
}

impl StackFrame {
    pub fn new(filename: impl Into<String>, lineno: usize, function: impl Into<String>) -> Self {
        let f_name = filename.into();
        let context = Self::read_file_context(&f_name, lineno, 2);
        Self {
            filename: f_name,
            lineno,
            function: function.into(),
            code_context: context,
        }
    }

    fn read_file_context(
        filename: &str,
        lineno: usize,
        margin: usize,
    ) -> Option<Vec<(usize, String, bool)>> {
        let path = Path::new(filename);
        if !path.is_file() {
            return None;
        }

        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);

        let start_line = lineno.saturating_sub(margin).max(1);
        let end_line = lineno + margin;

        let mut lines = Vec::new();
        for (idx, line_res) in reader.lines().enumerate() {
            let cur_line = idx + 1;
            if cur_line >= start_line
                && cur_line <= end_line
                && let Ok(line_str) = line_res
            {
                lines.push((cur_line, line_str, cur_line == lineno));
            }
            if cur_line > end_line {
                break;
            }
        }

        if lines.is_empty() { None } else { Some(lines) }
    }
}

/// Formatted error and panic traceback renderer with source code snippets.
#[derive(Clone, Debug)]
pub struct Traceback {
    pub title: String,
    pub message: String,
    pub frames: Vec<StackFrame>,
    pub extra_lines: usize,
}

impl Traceback {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            title: "Traceback (most recent call last)".to_string(),
            message: message.into(),
            frames: Vec::new(),
            extra_lines: 2,
        }
    }

    pub fn add_frame(&mut self, frame: StackFrame) -> &mut Self {
        self.frames.push(frame);
        self
    }

    pub fn from_panic_hook(info: &PanicHookInfo) -> Self {
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<dyn Any>".to_string()
        };

        let mut tb = Self::new(format!("Panic: {}", payload));

        if let Some(loc) = info.location() {
            let frame = StackFrame::new(loc.file(), loc.line() as usize, "panic");
            tb.add_frame(frame);
        }

        // Capture backtrace frames
        let bt = Backtrace::force_capture();
        let bt_str = format!("{:#?}", bt);
        for line in bt_str.lines() {
            let trimmed = line.trim();
            if trimmed.contains(" at ") {
                let parts: Vec<&str> = trimmed.split(" at ").collect();
                if parts.len() == 2 {
                    let fn_name = parts[0].trim();
                    let loc_part = parts[1].trim();
                    if let Some((f, l)) = loc_part.rsplit_once(':')
                        && let Ok(line_num) = l.parse::<usize>()
                        && !f.contains("/rustc/")
                        && !f.contains("/library/std/")
                    {
                        tb.add_frame(StackFrame::new(f, line_num, fn_name));
                    }
                }
            }
        }

        tb
    }

    /// Install panic handler into standard library `std::panic`.
    pub fn install() {
        std::panic::set_hook(Box::new(|info| {
            let tb = Traceback::from_panic_hook(info);
            eprintln!();
            crate::console::get_console().eprint(tb);
            eprintln!();
        }));
    }
}

impl Renderable for Traceback {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let title_text = format!(" {} ", self.title);
        let rule_len = options.max_width.saturating_sub(title_text.len());
        let left_len = rule_len / 2;
        let right_len = rule_len - left_len;

        let rule_style = Style::parse("bold red").unwrap_or_default();
        sink.push(Segment {
            text: Cow::Owned("━".repeat(left_len)),
            style: Some(rule_style.clone()),
            control: None,
        });
        sink.push(Segment {
            text: Cow::Owned(title_text),
            style: Some(rule_style.clone()),
            control: None,
        });
        sink.push(Segment {
            text: Cow::Owned("━".repeat(right_len)),
            style: Some(rule_style),
            control: None,
        });
        sink.push(Segment::line());

        for frame in &self.frames {
            let mut frame_header = Text::new();
            frame_header.append("╭─ ", Some(Style::parse("dim").unwrap_or_default()));
            frame_header.append(
                &frame.filename,
                Some(Style::parse("bold cyan").unwrap_or_default()),
            );
            frame_header.append(":", Some(Style::parse("dim").unwrap_or_default()));
            frame_header.append(
                &frame.lineno.to_string(),
                Some(Style::parse("bold magenta").unwrap_or_default()),
            );
            frame_header.append(" in ", Some(Style::parse("dim").unwrap_or_default()));
            frame_header.append(
                &frame.function,
                Some(Style::parse("bold yellow").unwrap_or_default()),
            );

            let segs = frame_header.render_to_segments();
            for s in segs {
                sink.push(s.into_segment());
            }
            sink.push(Segment::line());

            if let Some(ref context) = frame.code_context {
                let gutter_w = format!("{}", frame.lineno + 2).len().max(3);
                for (ln, code, is_target) in context {
                    let mut line_text = Text::new();
                    if *is_target {
                        line_text
                            .append("│ > ", Some(Style::parse("bold red").unwrap_or_default()));
                        line_text.append(
                            &format!("{:>width$} ", ln, width = gutter_w),
                            Some(Style::parse("bold red").unwrap_or_default()),
                        );
                        line_text.append("│ ", Some(Style::parse("dim red").unwrap_or_default()));
                        line_text.append(
                            code,
                            Some(Style::parse("bold white on grey19").unwrap_or_default()),
                        );
                    } else {
                        line_text.append("│   ", Some(Style::parse("dim").unwrap_or_default()));
                        line_text.append(
                            &format!("{:>width$} ", ln, width = gutter_w),
                            Some(Style::parse("dim").unwrap_or_default()),
                        );
                        line_text.append("│ ", Some(Style::parse("dim").unwrap_or_default()));
                        line_text.append(code, Some(Style::parse("dim").unwrap_or_default()));
                    }

                    let segs = line_text.render_to_segments();
                    for s in segs {
                        sink.push(s.into_segment());
                    }
                    sink.push(Segment::line());
                }
            }

            sink.push(Segment {
                text: Cow::Borrowed("╰────────────────────────────────────"),
                style: Some(Style::parse("dim").unwrap_or_default()),
                control: None,
            });
            sink.push(Segment::line());
        }

        // Bottom error panel
        let mut err_text = Text::new();
        err_text.append(
            &self.message,
            Some(Style::parse("bold red").unwrap_or_default()),
        );
        let panel = Panel::new(err_text)
            .box_style(crate::box_styles::HEAVY)
            .border_style(Style::parse("bold red").unwrap_or_default());

        let mut panel_segs = Vec::new();
        panel.render_to(console, options, &mut panel_segs);
        for s in panel_segs {
            sink.push(s.into_segment());
        }
    }

    fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        Measurement::new(20, options.max_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_traceback_rendering() {
        let mut tb = Traceback::new("Divide by zero error");
        tb.add_frame(StackFrame {
            filename: "src/math.rs".to_string(),
            lineno: 42,
            function: "divide".to_string(),
            code_context: Some(vec![
                (41, "let x = 10;".to_string(), false),
                (42, "let y = x / 0;".to_string(), true),
                (43, "println!(\"{}\", y);".to_string(), false),
            ]),
        });

        let console = Console::with_dimensions(60, 20).with_color_system(ColorSystem::NoColor);
        let out = console.capture(|c| {
            c.print(tb);
        });

        assert!(out.contains("Traceback (most recent call last)"));
        assert!(out.contains("src/math.rs:42 in divide"));
        assert!(out.contains(">  42 │ let y = x / 0;"));
        assert!(out.contains("Divide by zero error"));
    }
}
