use crate::color::Color;
use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
pub static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Syntax highlighted source code renderer powered by syntect.
#[derive(Clone, Debug)]
pub struct Syntax {
    pub code: String,
    pub lexer: String,
    pub theme: String,
    pub line_numbers: bool,
    pub start_line: usize,
    pub line_range: Option<(usize, usize)>,
    pub highlight_lines: HashSet<usize>,
    pub background_color: Option<Color>,
    pub word_wrap: bool,
}

impl Syntax {
    /// Create a new syntax highlighter for code in the given language.
    pub fn new(code: impl Into<String>, lexer: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            lexer: lexer.into(),
            theme: "base16-ocean.dark".to_string(),
            line_numbers: false,
            start_line: 1,
            line_range: None,
            highlight_lines: HashSet::new(),
            background_color: None,
            word_wrap: false,
        }
    }

    /// Set syntax theme name (e.g. "base16-ocean.dark", "InspiredGitHub", "Solarized (dark)").
    pub fn theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }

    /// Enable or disable line numbers.
    pub fn line_numbers(mut self, show: bool) -> Self {
        self.line_numbers = show;
        self
    }

    /// Starting line number offset.
    pub fn start_line(mut self, start: usize) -> Self {
        self.start_line = start;
        self
    }

    /// Limit rendering to a specific line range (inclusive).
    pub fn line_range(mut self, start: usize, end: usize) -> Self {
        self.line_range = Some((start, end));
        self
    }

    /// Add a line number to highlight.
    pub fn highlight_line(mut self, line: usize) -> Self {
        self.highlight_lines.insert(line);
        self
    }

    /// Highlight multiple lines.
    pub fn highlight_lines(mut self, lines: impl IntoIterator<Item = usize>) -> Self {
        self.highlight_lines.extend(lines);
        self
    }

    /// Set an explicit background color.
    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Enable or disable word wrapping.
    pub fn word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }
}

impl Renderable for Syntax {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let ps = &*SYNTAX_SET;
        let ts = &*THEME_SET;

        let syntax = ps
            .find_syntax_by_token(&self.lexer)
            .or_else(|| ps.find_syntax_by_extension(&self.lexer))
            .unwrap_or_else(|| ps.find_syntax_plain_text());

        let theme = ts
            .themes
            .get(&self.theme)
            .or_else(|| ts.themes.get("base16-ocean.dark"))
            .or_else(|| ts.themes.values().next())
            .unwrap();

        let mut h = HighlightLines::new(syntax, theme);

        let lines: Vec<&str> = LinesWithEndings::from(&self.code).collect();
        let total_lines = lines.len();
        let gutter_width = format!("{}", self.start_line + total_lines).len().max(3);

        let gutter_style = Style::parse("dim").unwrap_or_default();
        let highlight_gutter_style = Style::parse("bold yellow").unwrap_or_default();
        let highlight_bg_style = Style::parse("on grey23").unwrap_or_default();

        for (idx, raw_line) in lines.iter().enumerate() {
            let line_num = self.start_line + idx;

            if let Some((start, end)) = self.line_range
                && (line_num < start || line_num > end)
            {
                continue;
            }

            let is_highlighted = self.highlight_lines.contains(&line_num);

            // Render gutter / line number
            if self.line_numbers {
                let num_str = format!("{:>width$} │ ", line_num, width = gutter_width);
                let cur_gutter_style = if is_highlighted {
                    &highlight_gutter_style
                } else {
                    &gutter_style
                };
                sink.push(Segment {
                    text: Cow::Owned(num_str),
                    style: Some(cur_gutter_style.clone()),
                    control: None,
                });
            }

            // Syntect highlight line
            let clean_line = raw_line.trim_end_matches(['\r', '\n']);
            let ranges = h.highlight_line(clean_line, ps).unwrap_or_default();

            let mut line_text = Text::new();
            for (syn_style, text_slice) in ranges {
                let mut st = Style::new().color(Color::from_rgb(
                    syn_style.foreground.r,
                    syn_style.foreground.g,
                    syn_style.foreground.b,
                ));
                if syn_style.font_style.contains(FontStyle::BOLD) {
                    st = st.bold();
                }
                if syn_style.font_style.contains(FontStyle::ITALIC) {
                    st = st.italic();
                }
                if syn_style.font_style.contains(FontStyle::UNDERLINE) {
                    st = st.underline();
                }
                if is_highlighted {
                    st = st.combine(&highlight_bg_style);
                } else if let Some(ref bg) = self.background_color {
                    st = st.background(*bg);
                }

                line_text.append(text_slice, Some(st));
            }

            if self.word_wrap {
                let avail_width = if self.line_numbers {
                    options.max_width.saturating_sub(gutter_width + 3)
                } else {
                    options.max_width
                };
                let wrapped = line_text.wrap(avail_width);
                for (w_idx, subline) in wrapped.into_iter().enumerate() {
                    if w_idx > 0 && self.line_numbers {
                        let pad = format!("{:>width$} │ ", " ", width = gutter_width);
                        sink.push(Segment {
                            text: Cow::Owned(pad),
                            style: Some(gutter_style.clone()),
                            control: None,
                        });
                    }
                    let mut segs = Vec::new();
                    subline.render_to(console, options, &mut segs);
                    for s in segs {
                        sink.push(s.into_segment());
                    }
                    sink.push(Segment::line());
                }
            } else {
                let segs = line_text.render_to_segments();
                for s in segs {
                    sink.push(s.into_segment());
                }
                sink.push(Segment::line());
            }
        }
    }

    fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let max_line = self.code.lines().map(|l| l.len()).max().unwrap_or(0);
        let extra = if self.line_numbers { 8 } else { 0 };
        Measurement::new(10, max_line + extra)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_syntax_highlighting() {
        let rust_code = r#"fn main() {
    let message = "Hello, world!";
    println!("{}", message);
}"#;
        let syntax = Syntax::new(rust_code, "rust").line_numbers(true);
        let console = Console::with_dimensions(60, 10).with_color_system(ColorSystem::TrueColor);

        let out = console.capture(|c| {
            c.print(syntax);
        });

        assert!(out.contains("1 │"));
        assert!(out.contains("fn"));
        assert!(out.contains("main"));
        assert!(out.contains("Hello, world!"));
    }
}
