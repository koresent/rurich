use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;

/// Takes a group of renderables and renders them sequentially as a single unit.
pub struct Group {
    pub renderables: Vec<Box<dyn Renderable>>,
    pub fit: bool,
}

impl Group {
    pub fn new() -> Self {
        Self {
            renderables: Vec::new(),
            fit: true,
        }
    }

    pub fn with_renderables(renderables: Vec<Box<dyn Renderable>>) -> Self {
        Self {
            renderables,
            fit: true,
        }
    }

    pub fn add(&mut self, renderable: impl Renderable + 'static) -> &mut Self {
        self.renderables.push(Box::new(renderable));
        self
    }

    pub fn fit(mut self, fit: bool) -> Self {
        self.fit = fit;
        self
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderable for Group {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let count = self.renderables.len();
        for (i, r) in self.renderables.iter().enumerate() {
            let mut segs = Vec::new();
            r.render_to(console, options, &mut segs);

            let has_trailing_newline = segs.last().map(|s| s.text == "\n").unwrap_or(false);
            for s in segs {
                sink.push(s.into_segment());
            }

            if i + 1 < count && !has_trailing_newline {
                sink.push(Segment::line());
            }
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        if self.renderables.is_empty() {
            return Measurement::new(0, 0);
        }

        let mut max_min = 0;
        let mut max_max = 0;
        for r in &self.renderables {
            let m = r.measure(console, options);
            max_min = max_min.max(m.minimum);
            max_max = max_max.max(m.maximum);
        }

        Measurement::new(max_min, max_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::Panel;
    use crate::text::Text;

    #[test]
    fn test_group_rendering() {
        let mut group = Group::new();
        group.add(Text::from_plain("First"));
        group.add(Panel::from_markup("[bold]Second[/]"));

        let console = Console::with_dimensions(30, 10);
        let out = console.capture(|c| {
            c.print(group);
        });

        assert!(out.contains("First"));
        assert!(out.contains("Second"));
    }
}
