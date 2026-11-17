pub mod choose;
pub mod confirm;
pub mod file_picker;
pub mod filter;
pub mod input;
pub mod pager;

use crate::color::Color;
use crossterm::cursor::{Hide, MoveToColumn, MoveUp, Show};
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use std::io::{self, IsTerminal, Write, stderr, stdout};

/// Helper to get an appropriate terminal writer for interactive UI rendering.
/// If stdout is a TTY, returns stdout. Otherwise, falls back to stderr (if TTY) or /dev/tty.
pub fn terminal_writer() -> Box<dyn Write> {
    if stdout().is_terminal() {
        Box::new(stdout())
    } else if stderr().is_terminal() {
        Box::new(stderr())
    } else if let Ok(f) = std::fs::OpenOptions::new().write(true).open("/dev/tty") {
        Box::new(f)
    } else {
        Box::new(stdout())
    }
}

/// Helper to resolve foreground ANSI sequence from environment variable with fallback.
pub fn env_fg_ansi(var: &str, fallback: &str) -> String {
    let val = std::env::var(var).unwrap_or_else(|_| fallback.to_string());
    if let Ok(c) = Color::parse(&val) {
        let codes = c.get_ansi_codes(true);
        if !codes.is_empty() {
            return format!("\x1b[{}m", codes.join(";"));
        }
    }
    "\x1b[38;2;250;189;47m".to_string() // #fabd2f (yellow)
}

/// Helper to resolve background ANSI sequence from environment variable with fallback.
pub fn env_bg_ansi(var: &str, fallback: &str) -> String {
    let val = std::env::var(var).unwrap_or_else(|_| fallback.to_string());
    if let Ok(c) = Color::parse(&val) {
        let codes = c.get_ansi_codes(false);
        if !codes.is_empty() {
            return format!("\x1b[{}m", codes.join(";"));
        }
    }
    "\x1b[48;2;184;187;38m".to_string() // #b8bb26 (olive/green)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractiveAction<T> {
    Continue,
    Submit(T),
    Cancel,
}

pub use choose::{Choose, MultiChoose};
pub use confirm::ConfirmDialog;
pub use file_picker::FilePicker;
pub use filter::Filter;
pub use input::{Input, TextArea};
pub use pager::Pager;

pub struct RawGuard;

impl RawGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut out = terminal_writer();
        let _ = crossterm::execute!(out, Hide);
        let _ = out.flush();
        Ok(Self)
    }
}

impl Drop for RawGuard {
    fn drop(&mut self) {
        let mut out = terminal_writer();
        let _ = crossterm::execute!(out, Show, MoveToColumn(0));
        let _ = out.flush();
        let _ = disable_raw_mode();
    }
}

pub fn redraw_view<W: Write>(
    stdout: &mut W,
    view: &str,
    printed_lines: &mut usize,
) -> io::Result<()> {
    if *printed_lines > 0 {
        crossterm::execute!(
            stdout,
            MoveToColumn(0),
            MoveUp(*printed_lines as u16),
            Clear(ClearType::FromCursorDown)
        )?;
    }

    let mut count = 0;
    for line in view.lines() {
        write!(stdout, "\r{line}\x1b[K\r\n")?;
        count += 1;
    }
    stdout.flush()?;
    *printed_lines = count;
    Ok(())
}
