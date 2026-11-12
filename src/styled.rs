use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;

/// Applies a Style to any renderable content.
#[derive(Clone, Debug)]
pub struct Styled<T> {
    pub renderable: T,
    pub style: Style,
}

impl<T: Renderable> Styled<T> {
    pub fn new(renderable: T, style: Style) -> Self {
        Self { renderable, style }
    }
}

impl<T: Renderable> Renderable for Styled<T> {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let mut inner_segments = Vec::new();
        self.renderable
            .render_to(console, options, &mut inner_segments);

        for mut seg in inner_segments {
            let combined = match seg.style {
                Some(s) => s.combine(&self.style),
                None => self.style.clone(),
            };
            seg.style = Some(combined);
            sink.push(seg);
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.renderable.measure(console, options)
    }
}
