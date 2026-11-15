use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::borrow::Cow;

pub struct Markdown {
    pub text: String,
    pub code_theme: Option<String>,
}

impl Markdown {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            code_theme: None,
        }
    }
}

impl Renderable for Markdown {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(&self.text, opts);

        let mut current_text = Text::new();
        let mut style_stack: Vec<Style> = Vec::new();
        let mut code_block_lang = String::new();
        let mut in_blockquote = false;
        let mut list_stack: Vec<Option<u64>> = Vec::new(); // None = bullet, Some(n) = ordered
        let mut heading_level: Option<HeadingLevel> = None;

        let active_style = |stack: &[Style]| -> Style {
            let mut res = Style::new();
            for s in stack {
                res = res.combine(s);
            }
            res
        };

        for event in parser {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading { level, .. } => {
                        heading_level = Some(level);
                        let s = match level {
                            HeadingLevel::H1 => Style::parse("bold underline").unwrap_or_default(),
                            HeadingLevel::H2 => Style::parse("bold yellow").unwrap_or_default(),
                            HeadingLevel::H3 => Style::parse("bold green").unwrap_or_default(),
                            _ => Style::parse("bold italic").unwrap_or_default(),
                        };
                        style_stack.push(s);
                    }
                    Tag::Paragraph => {}
                    Tag::BlockQuote(_) => {
                        in_blockquote = true;
                    }
                    Tag::CodeBlock(kind) => {
                        code_block_lang = match kind {
                            CodeBlockKind::Fenced(lang) => lang.to_string(),
                            CodeBlockKind::Indented => String::new(),
                        };
                    }
                    Tag::List(first_num) => {
                        list_stack.push(first_num);
                    }
                    Tag::Item => {
                        let depth = list_stack.len().saturating_sub(1);
                        let indent = "  ".repeat(depth);
                        if let Some(Some(num)) = list_stack.last_mut() {
                            current_text.append(
                                &format!("{indent}{}. ", *num),
                                Some(Style::parse("bold").unwrap_or_default()),
                            );
                            *num += 1;
                        } else {
                            current_text.append(
                                &format!("{indent}• "),
                                Some(Style::parse("bold cyan").unwrap_or_default()),
                            );
                        }
                    }
                    Tag::Emphasis => {
                        style_stack.push(Style::new().italic());
                    }
                    Tag::Strong => {
                        style_stack.push(Style::new().bold());
                    }
                    Tag::Strikethrough => {
                        style_stack.push(Style::new().strike());
                    }
                    Tag::Link { dest_url, .. } => {
                        style_stack.push(Style::new().underline().link(dest_url.as_ref()));
                    }
                    _ => {}
                },
                Event::End(tag_end) => match tag_end {
                    TagEnd::Heading(_) => {
                        style_stack.pop();
                        let h_lvl = heading_level.take();

                        if let Some(HeadingLevel::H1) = h_lvl {
                            let aligned = crate::align::Align::center(&current_text).pad(false);
                            let lines = console.render_lines_owned(&aligned, Some(options), false);
                            for line in lines {
                                sink.extend(line);
                            }
                        } else {
                            let mut segs = Vec::new();
                            current_text.render_to(console, options, &mut segs);
                            sink.extend(segs.into_iter().map(|s| s.into_segment()));
                        }
                        sink.push(Segment::line());
                        sink.push(Segment::line());
                        current_text = Text::new();
                    }
                    TagEnd::Paragraph => {
                        if in_blockquote {
                            let bq_style = Style::parse("italic dim").unwrap_or_default();
                            let bq_border = Style::parse("dim cyan").unwrap_or_default();

                            let wrapped = current_text.wrap(options.max_width.saturating_sub(4));
                            for line in wrapped {
                                sink.push(Segment {
                                    text: Cow::Borrowed("  ▌ "),
                                    style: Some(bq_border.clone()),
                                    control: None,
                                });
                                let mut line_segs = Vec::new();
                                line.render_to(console, options, &mut line_segs);
                                for seg in line_segs {
                                    let merged = seg
                                        .style
                                        .map(|s| s.combine(&bq_style))
                                        .or_else(|| Some(bq_style.clone()));
                                    sink.push(Segment {
                                        text: Cow::Owned(seg.text.into_owned()),
                                        style: merged,
                                        control: None,
                                    });
                                }
                                sink.push(Segment::line());
                            }
                        } else {
                            let mut segs = Vec::new();
                            current_text.render_to(console, options, &mut segs);
                            sink.extend(segs.into_iter().map(|s| s.into_segment()));
                            sink.push(Segment::line());
                        }
                        sink.push(Segment::line());
                        current_text = Text::new();
                    }
                    TagEnd::BlockQuote(_) => {
                        in_blockquote = false;
                    }
                    TagEnd::CodeBlock => {
                        let code_content = current_text.plain.clone();
                        let code_style = Style::parse("dim white on grey19").unwrap_or_default();

                        // Render code block with header if language specified
                        if !code_block_lang.is_empty() {
                            sink.push(Segment {
                                text: Cow::Owned(format!(" ```{} ", code_block_lang)),
                                style: Some(Style::parse("dim cyan").unwrap_or_default()),
                                control: None,
                            });
                            sink.push(Segment::line());
                        }

                        for line in code_content.lines() {
                            sink.push(Segment {
                                text: Cow::Owned(format!("  {line}")),
                                style: Some(code_style.clone()),
                                control: None,
                            });
                            sink.push(Segment::line());
                        }
                        sink.push(Segment::line());
                        current_text = Text::new();
                    }
                    TagEnd::List(_) => {
                        list_stack.pop();
                        sink.push(Segment::line());
                    }
                    TagEnd::Item => {
                        let mut segs = Vec::new();
                        current_text.render_to(console, options, &mut segs);
                        sink.extend(segs.into_iter().map(|s| s.into_segment()));
                        sink.push(Segment::line());
                        current_text = Text::new();
                    }
                    TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                        style_stack.pop();
                    }
                    _ => {}
                },
                Event::Text(t) => {
                    let cur_style = active_style(&style_stack);
                    current_text.append(
                        t.as_ref(),
                        if cur_style.is_null() {
                            None
                        } else {
                            Some(cur_style)
                        },
                    );
                }
                Event::Code(c) => {
                    let inline_code_style = Style::parse("bold cyan").unwrap_or_default();
                    current_text.append(&format!("`{}`", c.as_ref()), Some(inline_code_style));
                }
                Event::Rule => {
                    let line = "─".repeat(options.max_width);
                    sink.push(Segment {
                        text: Cow::Owned(line),
                        style: Some(Style::parse("dim").unwrap_or_default()),
                        control: None,
                    });
                    sink.push(Segment::line());
                    sink.push(Segment::line());
                }
                Event::SoftBreak => {
                    current_text.append(" ", None);
                }
                Event::HardBreak => {
                    current_text.append("\n", None);
                }
                _ => {}
            }
        }
    }

    fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        Measurement::new(10, options.max_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_markdown_heading_and_paragraphs() {
        let md_text = "# Title\n\nThis is a **bold** paragraph with `code`.\n\n- Item 1\n- Item 2";
        let md = Markdown::new(md_text);

        let console = Console::with_dimensions(40, 15).with_color_system(ColorSystem::NoColor);
        let out = console.capture(|c| {
            c.print(md);
        });

        assert!(out.contains("Title"));
        assert!(out.contains("bold"));
        assert!(out.contains("`code`"));
        assert!(out.contains("• Item 1"));
        assert!(out.contains("• Item 2"));
    }
}
