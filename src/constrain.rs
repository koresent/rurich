use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;

/// Constrains the width of a child renderable to a specified number of characters.
#[derive(Clone, Debug)]
pub struct Constrain<T> {
    pub renderable: T,
    pub width: Option<usize>,
}

impl<T: Renderable> Constrain<T> {
    pub fn new(renderable: T, width: usize) -> Self {
        Self {
            renderable,
            width: Some(width),
        }
    }

    pub fn unconstrained(renderable: T) -> Self {
        Self {
            renderable,
            width: None,
        }
    }
}

impl<T: Renderable> Renderable for Constrain<T> {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        if let Some(w) = self.width {
            let child_opts = options.update_width(w.min(options.max_width));
            self.renderable.render_to(console, &child_opts, sink);
        } else {
            self.renderable.render_to(console, options, sink);
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        if let Some(w) = self.width {
            let child_opts = options.update_width(w);
            let m = self.renderable.measure(console, &child_opts);
            Measurement::new(m.minimum.min(w), m.maximum.min(w))
        } else {
            self.renderable.measure(console, options)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::Text;

    #[test]
    fn test_constrain_measure() {
        let text = Text::from_plain("A very long text that should be measured");
        let constrained = Constrain::new(text, 10);
        let console = Console::new();
        let m = constrained.measure(&console, &console.options());
        assert!(m.maximum <= 10);
    }
}
