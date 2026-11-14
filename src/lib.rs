pub mod align;
pub mod ansi;
pub mod badge;
pub mod box_styles;
pub mod callout;
pub mod cells;
pub mod color;
pub mod columns;
pub mod console;
pub mod constrain;
pub mod control;
pub mod default_styles;
pub mod emoji;
pub mod errors;
pub mod export;
pub mod filesize;
pub mod group;
pub mod highlighter;
pub mod inspect;
pub mod interactive;
pub mod join;
#[cfg(feature = "json")]
pub mod json;
pub mod layout;
pub mod live;
#[cfg(feature = "tracing")]
pub mod logging;
pub mod macros;
#[cfg(feature = "markdown")]
pub mod markdown;
pub mod markup;
pub mod measure;
pub mod padding;
pub mod panel;
pub mod pretty;
pub mod progress;
pub mod prompt;
pub mod protocol;
pub mod ratio;
pub mod rule;
pub mod screen;
pub mod segment;
pub mod sparkline;
pub mod spinner_bar;
pub mod status;
pub mod style;
pub mod styled;
#[cfg(feature = "syntax")]
pub mod syntax;
pub mod table;
pub mod terminal_theme;
pub mod text;
pub mod theme;
pub mod traceback;
pub mod tree;

// Top-level re-exports
pub use align::{Align, AlignMethod, VerticalAlignMethod};
pub use ansi::AnsiDecoder;
pub use badge::{Badge, BadgeStyle, Breadcrumbs};
pub use box_styles::{
    ASCII, ASCII2, Box as BoxStyle, DOUBLE, DOUBLE_EDGE, HEAVY, HEAVY_HEAD, HORIZONTALS, MARKDOWN,
    MINIMAL, ROUNDED, SIMPLE, SQUARE, SQUARE_DOUBLE_HEAD,
};
pub use callout::{AdmonitionType, Callout};
pub use cells::{cell_len, get_character_cell_size, set_cell_size, split_cells};
pub use color::{Color, ColorSystem, ColorTriplet, ColorType};
pub use columns::Columns;
pub use console::{Console, get_console};
pub use constrain::Constrain;
pub use control::ControlCode;
pub use emoji::Emoji;
pub use errors::RurichError;
pub use export::{export_html, export_svg};
pub use filesize::{binary as format_binary_size, decimal as format_decimal_size};
pub use group::Group;
pub use highlighter::{
    Highlighter, ISO8601Highlighter, JSONHighlighter, NullHighlighter, ReprHighlighter,
};
pub use inspect::inspect;
pub use interactive::{
    Choose, ConfirmDialog, FilePicker, Filter, Input, InteractiveAction, MultiChoose, Pager,
    TextArea,
};
pub use join::{AlignJoin, JoinedBlock, join_horizontal, join_vertical};
#[cfg(feature = "json")]
pub use json::Json;
pub use layout::{Layout, SplitDirection};
pub use live::Live;
#[cfg(feature = "tracing")]
pub use logging::{ConsoleLayer, RuRichLayer};
#[cfg(feature = "markdown")]
pub use markdown::Markdown;
pub use markup::{escape, render as parse_markup};
pub use measure::Measurement;
pub use padding::Padding;
pub use panel::Panel;
pub use pretty::Pretty;
pub use progress::{
    BarColumn, DownloadColumn, FileSizeColumn, MofNCompleteColumn, Progress, ProgressColumn,
    SpinnerColumn, Task, TaskId, TaskProgressColumn, TextColumn, TimeElapsedColumn,
    TimeRemainingColumn, TotalFileSizeColumn, TrackIter, TransferSpeedColumn, track,
};
pub use prompt::{Confirm, FloatPrompt, IntPrompt, Prompt, PromptError};
pub use protocol::{ConsoleDimensions, ConsoleOptions, JustifyMethod, OverflowMethod, Renderable};
pub use ratio::{RatioEdge, ratio_distribute, ratio_reduce, ratio_resolve};
pub use rule::Rule;
pub use screen::{Screen, ScreenGuard};
pub use segment::Segment;
pub use sparkline::Sparkline;
pub use spinner_bar::{ProgressBar, Spinner, SpinnerPreset};
pub use status::Status;
pub use style::{Style, StyleAttributes};
pub use styled::Styled;
#[cfg(feature = "syntax")]
pub use syntax::Syntax;
pub use table::{Column, Row, Table};
pub use text::{Span, Text};
pub use theme::{Theme, get_global_theme, get_theme_style, set_global_theme, update_global_theme};
pub use traceback::{StackFrame, Traceback};
pub use tree::{ASCII_GUIDES, BOLD_GUIDES, DEFAULT_GUIDES, DOUBLE_GUIDES, Tree, TreeGuides};

// Alias box_styles as r#box
pub use box_styles as r#box;

pub mod prelude {
    pub use crate::align::{Align, AlignMethod, VerticalAlignMethod};
    pub use crate::ansi::AnsiDecoder;
    pub use crate::badge::{Badge, BadgeStyle, Breadcrumbs};
    pub use crate::box_styles::{
        ASCII, ASCII2, Box as BoxStyle, DOUBLE, DOUBLE_EDGE, HEAVY, HEAVY_HEAD, HORIZONTALS,
        MARKDOWN, MINIMAL, ROUNDED, SIMPLE, SQUARE, SQUARE_DOUBLE_HEAD,
    };
    pub use crate::callout::{AdmonitionType, Callout};
    pub use crate::color::{Color, ColorSystem, ColorTriplet};
    pub use crate::columns::Columns;
    pub use crate::console::{Console, get_console};
    pub use crate::constrain::Constrain;
    pub use crate::emoji::Emoji;
    pub use crate::group::Group;
    pub use crate::inspect::inspect;
    pub use crate::interactive::{
        Choose, ConfirmDialog, FilePicker, Filter, Input, InteractiveAction, MultiChoose, Pager,
        RawGuard, TextArea, redraw_view,
    };
    pub use crate::join::{AlignJoin, JoinedBlock, join_horizontal, join_vertical};
    #[cfg(feature = "json")]
    pub use crate::json::Json;
    pub use crate::layout::{Layout, SplitDirection};
    pub use crate::live::Live;
    #[cfg(feature = "markdown")]
    pub use crate::markdown::Markdown;
    pub use crate::markup::escape;
    pub use crate::padding::Padding;
    pub use crate::panel::Panel;
    pub use crate::pretty::Pretty;
    pub use crate::progress::{
        BarColumn, DownloadColumn, FileSizeColumn, MofNCompleteColumn, Progress, ProgressColumn,
        SpinnerColumn, Task, TaskId, TaskProgressColumn, TextColumn, TimeElapsedColumn,
        TimeRemainingColumn, TotalFileSizeColumn, TrackIter, TransferSpeedColumn, track,
    };
    pub use crate::prompt::{Confirm, Prompt};
    pub use crate::protocol::{ConsoleOptions, JustifyMethod, OverflowMethod, Renderable};
    pub use crate::reprint;
    pub use crate::reprintln;
    pub use crate::rprint;
    pub use crate::rprintln;
    pub use crate::rule::Rule;
    pub use crate::screen::Screen;
    pub use crate::segment::Segment;
    pub use crate::sparkline::Sparkline;
    pub use crate::spinner_bar::{ProgressBar, Spinner, SpinnerPreset};
    pub use crate::status::Status;
    pub use crate::style::Style;
    pub use crate::styled::Styled;
    #[cfg(feature = "syntax")]
    pub use crate::syntax::Syntax;
    pub use crate::table::{Column, Table};
    pub use crate::text::Text;
    pub use crate::traceback::Traceback;
    pub use crate::tree::Tree;
}

/// Print markup to standard output
pub fn print(markup: &str) {
    get_console().print_markup(markup);
}

/// Print markup to standard output followed by a newline
pub fn println(markup: &str) {
    get_console().println_markup(markup);
}

/// Print markup to standard error
pub fn eprint(markup: &str) {
    match parse_markup(markup) {
        Ok(text) => get_console().eprint(text),
        Err(_) => get_console().eprint(markup),
    }
}

/// Print markup to standard error followed by a newline
pub fn eprintln(markup: &str) {
    match parse_markup(markup) {
        Ok(text) => get_console().eprintln(text),
        Err(_) => get_console().eprintln(markup),
    }
}

/// Print formatted and syntax-highlighted JSON to console.
#[cfg(feature = "json")]
pub fn print_json(json_str: &str) {
    match Json::from_str(json_str) {
        Ok(j) => get_console().println(j),
        Err(_) => get_console().println_markup(json_str),
    }
}

/// Print serializable data structure as formatted, highlighted JSON.
#[cfg(feature = "json")]
pub fn print_serializable<T: serde::Serialize>(data: &T) {
    if let Ok(j) = Json::from_serializable(data) {
        get_console().println(j);
    }
}
