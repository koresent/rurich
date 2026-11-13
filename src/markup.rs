use crate::errors::RurichError;
use crate::style::Style;
use crate::text::{Span, Text};
use regex::Regex;
use std::sync::LazyLock;

static RE_TAGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\\*)\[([a-z#/@][^\[]*?)]").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub parameters: Option<String>,
}

impl Tag {
    pub fn new(name: impl Into<String>, parameters: Option<String>) -> Self {
        Self {
            name: name.into(),
            parameters,
        }
    }

    pub fn to_style(&self) -> Result<Style, RurichError> {
        let style_def = if let Some(ref params) = self.parameters {
            if self.name == "link" {
                format!("link {params}")
            } else {
                format!("{} {params}", self.name)
            }
        } else {
            self.name.clone()
        };
        Style::parse(&style_def)
    }
}

/// Escape a string so that square bracket tags won't be interpreted as markup
pub fn escape(text: &str) -> String {
    let mut result = String::new();
    let mut last_idx = 0;

    for mat in RE_TAGS.captures_iter(text) {
        let full = mat.get(0).unwrap();
        let escapes = mat.get(1).unwrap().as_str();
        let tag_content = mat.get(2).unwrap().as_str();

        result.push_str(&text[last_idx..full.start()]);
        // Double the backslashes and escape the tag
        result.push_str(escapes);
        result.push_str(escapes);
        result.push('\\');
        result.push('[');
        result.push_str(tag_content);
        result.push(']');

        last_idx = full.end();
    }

    result.push_str(&text[last_idx..]);

    if result.ends_with('\\') && !result.ends_with("\\\\") {
        result.push('\\');
    }

    result
}

/// Parse markup string and render into a styled Text instance
pub fn render(markup: &str) -> Result<Text, RurichError> {
    if !markup.contains('[') {
        return Ok(Text::from_plain(markup));
    }

    let mut text = Text::new();
    let mut style_stack: Vec<(usize, Tag)> = Vec::new();
    let mut spans: Vec<Span> = Vec::new();

    let mut last_idx = 0;

    for caps in RE_TAGS.captures_iter(markup) {
        let full_match = caps.get(0).unwrap();
        let escapes = caps.get(1).unwrap().as_str();
        let tag_inner = caps.get(2).unwrap().as_str();

        let match_start = full_match.start();
        let match_end = full_match.end();

        // Plain text before tag
        if match_start > last_idx {
            let plain_chunk = &markup[last_idx..match_start];
            let clean = plain_chunk.replace("\\[", "[");
            text.append(&clean, None);
        }

        let num_escapes = escapes.len();
        let (even_backslashes, is_escaped) = (num_escapes / 2, num_escapes % 2 == 1);

        if even_backslashes > 0 {
            text.append(&"\\".repeat(even_backslashes), None);
        }

        if is_escaped {
            // Escaped tag literal
            text.append(&format!("[{tag_inner}]"), None);
            last_idx = match_end;
            continue;
        }

        // Parse tag and optional parameter (e.g. "link=https://...")
        let (tag_name, tag_param) = if let Some((name, param)) = tag_inner.split_once('=') {
            (name.trim(), Some(param.trim().to_string()))
        } else {
            (tag_inner.trim(), None)
        };

        if let Some(stripped_close) = tag_name.strip_prefix('/') {
            // Closing tag
            let close_name = stripped_close.trim();
            if close_name.is_empty() {
                // Implicit close: [/]
                if let Some((start_pos, open_tag)) = style_stack.pop() {
                    let end_pos = text.len();
                    if start_pos < end_pos {
                        let style = open_tag.to_style()?;
                        spans.push(Span::new(start_pos, end_pos, style));
                    }
                } else {
                    return Err(RurichError::MarkupError(format!(
                        "Closing tag '[/]' has nothing to close at offset {match_start}"
                    )));
                }
            } else {
                // Explicit close: [/tag_name]
                let mut found_idx = None;
                for (idx, (_, open_tag)) in style_stack.iter().enumerate().rev() {
                    if open_tag.name == close_name {
                        found_idx = Some(idx);
                        break;
                    }
                }
                if let Some(idx) = found_idx {
                    let (start_pos, open_tag) = style_stack.remove(idx);
                    let end_pos = text.len();
                    if start_pos < end_pos {
                        let style = open_tag.to_style()?;
                        spans.push(Span::new(start_pos, end_pos, style));
                    }
                } else {
                    return Err(RurichError::MarkupError(format!(
                        "Closing tag '[/{close_name}]' doesn't match any open tag at offset {match_start}"
                    )));
                }
            }
        } else {
            // Opening tag
            let tag = Tag::new(tag_name, tag_param);
            style_stack.push((text.len(), tag));
        }

        last_idx = match_end;
    }

    // Remaining text after last tag
    if last_idx < markup.len() {
        let plain_chunk = &markup[last_idx..];
        let clean = plain_chunk.replace("\\[", "[");
        text.append(&clean, None);
    }

    // Close any unclosed tags at end of text
    let text_len = text.len();
    while let Some((start_pos, open_tag)) = style_stack.pop() {
        if start_pos < text_len {
            let style = open_tag.to_style()?;
            spans.push(Span::new(start_pos, text_len, style));
        }
    }

    text.spans = spans;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(escape("foo[bar]"), "foo\\[bar]");
        assert_eq!(escape("[5]"), "[5]");
        assert_eq!(escape("[@foo]"), "\\[@foo]");
    }

    #[test]
    fn test_render_simple() {
        let text = render("[bold]Hello[/bold] World").unwrap();
        assert_eq!(text.plain, "Hello World");
        assert_eq!(text.spans.len(), 1);
        assert_eq!(text.spans[0].start, 0);
        assert_eq!(text.spans[0].end, 5);
    }

    #[test]
    fn test_render_nested_implicit_close() {
        let text = render("[red]Hello [bold]Cruel[/] World[/]").unwrap();
        assert_eq!(text.plain, "Hello Cruel World");
        assert_eq!(text.spans.len(), 2);
    }

    #[test]
    fn test_render_link() {
        let text = render("Click [link=https://example.com]here[/link]!").unwrap();
        assert_eq!(text.plain, "Click here!");
        assert_eq!(text.spans.len(), 1);
        assert_eq!(
            text.spans[0].style.link.as_deref(),
            Some("https://example.com")
        );
    }

    #[test]
    fn test_render_semantic_and_theme_tags() {
        let text = render("[header]Title[/header] - [ok]Done[/ok] - [warn]Warning[/warn] - [err]Error[/err] - [orange]10[/orange]").unwrap();
        assert_eq!(text.plain, "Title - Done - Warning - Error - 10");
        assert_eq!(text.spans.len(), 5);
    }
}
