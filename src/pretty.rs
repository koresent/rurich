use crate::console::Console;
use crate::highlighter::{Highlighter, ReprHighlighter};
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, JustifyMethod, OverflowMethod, Renderable};
use crate::segment::Segment;
use crate::text::Text;
use std::fmt::Debug;

/// Formats Rust data structures or debug representations with syntax highlighting and layout controls.
#[derive(Clone, Debug)]
pub struct Pretty {
    pub text: Text,
    pub justify: Option<JustifyMethod>,
    pub overflow: Option<OverflowMethod>,
    pub no_wrap: bool,
}

impl Pretty {
    /// Create a new `Pretty` renderable from any type implementing `Debug`.
    pub fn new<T: Debug>(value: &T) -> Self {
        let plain = format!("{:#?}", value);
        let mut text = Text::from_plain(plain);
        ReprHighlighter.highlight(&mut text);
        Self {
            text,
            justify: None,
            overflow: None,
            no_wrap: false,
        }
    }

    /// Create a `Pretty` renderable directly from a debug string representation.
    pub fn from_debug_str(repr: &str) -> Self {
        let mut text = Text::from_plain(repr);
        ReprHighlighter.highlight(&mut text);
        Self {
            text,
            justify: None,
            overflow: None,
            no_wrap: false,
        }
    }

    /// Set a custom highlighter.
    pub fn with_highlighter<H: Highlighter>(mut self, highlighter: &H) -> Self {
        self.text.spans.clear();
        highlighter.highlight(&mut self.text);
        self
    }

    /// Set text justification.
    pub fn justify(mut self, justify: JustifyMethod) -> Self {
        self.justify = Some(justify);
        self
    }

    /// Set text overflow method.
    pub fn overflow(mut self, overflow: OverflowMethod) -> Self {
        self.overflow = Some(overflow);
        self
    }

    /// Disable or enable line wrapping.
    pub fn no_wrap(mut self, no_wrap: bool) -> Self {
        self.no_wrap = no_wrap;
        self
    }
}

impl Renderable for Pretty {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let mut render_opts = options.clone();
        if let Some(j) = self.justify {
            render_opts.justify = Some(j);
        }
        if let Some(o) = self.overflow {
            render_opts.overflow = Some(o);
        }
        if self.no_wrap {
            render_opts.no_wrap = true;
        }

        self.text.render_to(console, &render_opts, sink);
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.text.measure(console, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[allow(dead_code)]
    #[derive(Debug)]
    struct User {
        name: String,
        age: u32,
        active: bool,
    }

    #[test]
    fn test_pretty_debug_struct() {
        let user = User {
            name: "Alice".to_string(),
            age: 30,
            active: true,
        };

        let pretty = Pretty::new(&user);
        let console = Console::with_dimensions(40, 10).with_color_system(ColorSystem::NoColor);
        let out = console.capture(|c| {
            c.print(pretty);
        });

        assert!(out.contains("User"));
        assert!(out.contains("\"Alice\""));
        assert!(out.contains("30"));
        assert!(out.contains("true"));
    }
}
