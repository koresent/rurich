use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use serde_json::Value;

pub struct Json {
    pub value: Value,
    pub indent: usize,
}

impl std::str::FromStr for Json {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: Value = serde_json::from_str(s)?;
        Ok(Self { value, indent: 2 })
    }
}

impl Json {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(json_str: &str) -> Result<Self, serde_json::Error> {
        json_str.parse()
    }

    pub fn from_json_str(json_str: &str) -> Result<Self, serde_json::Error> {
        json_str.parse()
    }

    pub fn from_value(value: Value) -> Self {
        Self { value, indent: 2 }
    }

    pub fn from_serializable<T: serde::Serialize>(data: &T) -> Result<Self, serde_json::Error> {
        let value = serde_json::to_value(data)?;
        Ok(Self { value, indent: 2 })
    }

    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    fn render_value(&self, value: &Value, depth: usize, text: &mut Text) {
        let indent_str = " ".repeat(depth * self.indent);
        let brace_style = Style::parse("bold dim").unwrap_or_default();
        let key_style = Style::parse("bold cyan").unwrap_or_default();
        let str_style = Style::parse("green").unwrap_or_default();
        let num_style = Style::parse("cyan").unwrap_or_default();
        let bool_style = Style::parse("italic yellow").unwrap_or_default();
        let null_style = Style::parse("italic magenta").unwrap_or_default();

        match value {
            Value::Null => {
                text.append("null", Some(null_style));
            }
            Value::Bool(b) => {
                text.append(if *b { "true" } else { "false" }, Some(bool_style));
            }
            Value::Number(n) => {
                text.append(&n.to_string(), Some(num_style));
            }
            Value::String(s) => {
                text.append(&format!("\"{}\"", s), Some(str_style));
            }
            Value::Array(arr) => {
                if arr.is_empty() {
                    text.append("[]", Some(brace_style));
                } else {
                    text.append("[\n", Some(brace_style.clone()));
                    for (i, item) in arr.iter().enumerate() {
                        text.append(&" ".repeat((depth + 1) * self.indent), None);
                        self.render_value(item, depth + 1, text);
                        if i + 1 < arr.len() {
                            text.append(",", Some(brace_style.clone()));
                        }
                        text.append("\n", None);
                    }
                    text.append(&indent_str, None);
                    text.append("]", Some(brace_style));
                }
            }
            Value::Object(obj) => {
                if obj.is_empty() {
                    text.append("{}", Some(brace_style));
                } else {
                    text.append("{\n", Some(brace_style.clone()));
                    let len = obj.len();
                    for (i, (k, v)) in obj.iter().enumerate() {
                        text.append(&" ".repeat((depth + 1) * self.indent), None);
                        text.append(&format!("\"{}\"", k), Some(key_style.clone()));
                        text.append(": ", Some(brace_style.clone()));
                        self.render_value(v, depth + 1, text);
                        if i + 1 < len {
                            text.append(",", Some(brace_style.clone()));
                        }
                        text.append("\n", None);
                    }
                    text.append(&indent_str, None);
                    text.append("}", Some(brace_style));
                }
            }
        }
    }
}

impl Renderable for Json {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        _options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let mut text = Text::new();
        self.render_value(&self.value, 0, &mut text);
        let segs = text.render_to_segments();
        for seg in segs {
            sink.push(seg.into_segment());
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let mut text = Text::new();
        self.render_value(&self.value, 0, &mut text);
        text.measure(console, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_json_rendering() {
        let json_str = r#"{"name": "rurich", "stars": 1000, "active": true}"#;
        let json = Json::from_str(json_str).unwrap();

        let console = Console::with_dimensions(40, 10).with_color_system(ColorSystem::NoColor);
        let out = console.capture(|c| {
            c.print(json);
        });

        assert!(out.contains("\"name\": \"rurich\""));
        assert!(out.contains("\"stars\": 1000"));
        assert!(out.contains("\"active\": true"));
    }
}
