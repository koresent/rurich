use crate::cells::cell_len;
use crate::console::Console;
use crate::measure::Measurement;
use crate::segment::Segment;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsoleDimensions {
    pub width: usize,
    pub height: usize,
}

impl ConsoleDimensions {
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JustifyMethod {
    #[default]
    Default,
    Left,
    Center,
    Right,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverflowMethod {
    #[default]
    Fold,
    Crop,
    Ellipsis,
    Ignore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleOptions {
    pub size: ConsoleDimensions,
    pub min_width: usize,
    pub max_width: usize,
    pub is_terminal: bool,
    pub max_height: usize,
    pub justify: Option<JustifyMethod>,
    pub overflow: Option<OverflowMethod>,
    pub no_wrap: bool,
    pub highlight: bool,
    pub markup: bool,
    pub height: Option<usize>,
}

impl ConsoleOptions {
    pub fn new(width: usize, height: usize, is_terminal: bool) -> Self {
        Self {
            size: ConsoleDimensions::new(width, height),
            min_width: 1,
            max_width: width,
            is_terminal,
            max_height: height,
            justify: None,
            overflow: None,
            no_wrap: false,
            highlight: true,
            markup: true,
            height: None,
        }
    }

    pub fn update_width(&self, width: usize) -> Self {
        let mut copy = self.clone();
        copy.min_width = width;
        copy.max_width = width;
        copy
    }

    pub fn update_height(&self, height: usize) -> Self {
        let mut copy = self.clone();
        copy.max_height = height;
        copy.height = Some(height);
        copy
    }

    pub fn reset_height(&self) -> Self {
        let mut copy = self.clone();
        copy.height = None;
        copy
    }

    pub fn update_dimensions(&self, width: usize, height: usize) -> Self {
        let mut copy = self.clone();
        copy.min_width = width;
        copy.max_width = width;
        copy.height = Some(height);
        copy.max_height = height;
        copy
    }
}

/// The universal interface for any terminal renderable component.
pub trait Renderable {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    );

    fn render<'a>(&'a self, console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        let mut sink = Vec::new();
        self.render_to(console, options, &mut sink);
        sink
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement;
}

// Blanket implementation for &T
impl<T: Renderable + ?Sized> Renderable for &T {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        (*self).render_to(console, options, sink);
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        (*self).measure(console, options)
    }
}

// Blanket implementation for Box<T>
impl<T: Renderable + ?Sized> Renderable for Box<T> {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        (**self).render_to(console, options, sink);
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        (**self).measure(console, options)
    }
}

// Blanket implementation for Arc<T>
impl<T: Renderable + ?Sized> Renderable for std::sync::Arc<T> {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        (**self).render_to(console, options, sink);
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        (**self).measure(console, options)
    }
}

// Implement Renderable for &str
impl Renderable for &str {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        _options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        sink.push(Segment::unstyled(Cow::Borrowed(*self)));
    }

    fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let max_line = self.lines().map(cell_len).max().unwrap_or(0);
        let min_line = self.split_whitespace().map(cell_len).max().unwrap_or(0);
        Measurement::new(min_line, max_line)
    }
}

// Implement Renderable for String
impl Renderable for String {
    fn render_to<'a>(
        &'a self,
        _console: &Console,
        _options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        sink.push(Segment::unstyled(Cow::Borrowed(self.as_str())));
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.as_str().measure(console, options)
    }
}
