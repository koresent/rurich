use crate::default_styles::DEFAULT_STYLES;
use crate::text::{Span, Text};
use regex::Regex;
use std::sync::LazyLock;

pub trait Highlighter: Send + Sync {
    fn highlight(&self, text: &mut Text);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NullHighlighter;

impl Highlighter for NullHighlighter {
    fn highlight(&self, _text: &mut Text) {}
}

pub struct ReprHighlighter;

static RE_BRACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\[\]{}()]").unwrap());
static RE_NUMBERS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:\d+\.?\d*|\d*\.?\d+)(?:[eE][+-]?\d+)?\b|0x[0-9a-fA-F]+\b").unwrap()
});
static RE_STRINGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'"#).unwrap());
static RE_BOOLEANS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:true|false|True|False|None|null)\b").unwrap());
static RE_URLS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://[^\s<>]+").unwrap());
static RE_UUID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}\b")
        .unwrap()
});
static RE_IPV4: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b").unwrap()
});

impl Highlighter for ReprHighlighter {
    fn highlight(&self, text: &mut Text) {
        let plain = &text.plain;

        if let Some(style) = DEFAULT_STYLES.get("repr.url") {
            for m in RE_URLS.find_iter(plain) {
                text.spans
                    .push(Span::new(m.start(), m.end(), style.clone()));
            }
        }

        if let Some(style) = DEFAULT_STYLES.get("repr.uuid") {
            for m in RE_UUID.find_iter(plain) {
                text.spans
                    .push(Span::new(m.start(), m.end(), style.clone()));
            }
        }

        if let Some(style) = DEFAULT_STYLES.get("repr.number") {
            for m in RE_IPV4.find_iter(plain) {
                text.spans
                    .push(Span::new(m.start(), m.end(), style.clone()));
            }
        }

        if let Some(style) = DEFAULT_STYLES.get("repr.str") {
            for m in RE_STRINGS.find_iter(plain) {
                text.spans
                    .push(Span::new(m.start(), m.end(), style.clone()));
            }
        }

        for m in RE_BOOLEANS.find_iter(plain) {
            let s = m.as_str();
            let style_name = if s == "true" || s == "True" {
                "repr.bool_true"
            } else if s == "false" || s == "False" {
                "repr.bool_false"
            } else {
                "repr.none"
            };
            if let Some(style) = DEFAULT_STYLES.get(style_name) {
                text.spans
                    .push(Span::new(m.start(), m.end(), style.clone()));
            }
        }

        if let Some(style) = DEFAULT_STYLES.get("repr.number") {
            for m in RE_NUMBERS.find_iter(plain) {
                text.spans
                    .push(Span::new(m.start(), m.end(), style.clone()));
            }
        }

        if let Some(style) = DEFAULT_STYLES.get("repr.brace") {
            for m in RE_BRACES.find_iter(plain) {
                text.spans
                    .push(Span::new(m.start(), m.end(), style.clone()));
            }
        }
    }
}

pub struct ISO8601Highlighter;

static RE_ISO8601: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{4}-\d{2}-\d{2}(?:[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?)?\b")
        .unwrap()
});

impl Highlighter for ISO8601Highlighter {
    fn highlight(&self, text: &mut Text) {
        let plain = &text.plain;
        let date_style = crate::style::Style::parse("cyan").unwrap_or_default();
        for m in RE_ISO8601.find_iter(plain) {
            text.spans
                .push(Span::new(m.start(), m.end(), date_style.clone()));
        }
    }
}

pub struct JSONHighlighter;

static RE_JSON_KEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""([^"\\]+)":\s*"#).unwrap());

impl Highlighter for JSONHighlighter {
    fn highlight(&self, text: &mut Text) {
        ReprHighlighter.highlight(text);
        let plain = &text.plain;
        let key_style = crate::style::Style::parse("bold cyan").unwrap_or_default();
        for m in RE_JSON_KEY.find_iter(plain) {
            text.spans.push(Span::new(
                m.start(),
                m.end().saturating_sub(1),
                key_style.clone(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repr_highlighter() {
        let mut text = Text::from_plain("user_id = 42, active = true, url = https://example.com");
        let highlighter = ReprHighlighter;
        highlighter.highlight(&mut text);
        assert!(!text.spans.is_empty());
    }
}
