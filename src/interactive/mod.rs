pub mod choose;
pub mod confirm;
pub mod file_picker;
pub mod filter;
pub mod input;
pub mod pager;

use crossterm::cursor::{Hide, MoveToColumn, MoveUp, Show};
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use std::io::{self, Write, stdout};

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
        let mut out = stdout();
        let _ = crossterm::execute!(out, Hide);
        let _ = out.flush();
        Ok(Self)
    }
}

impl Drop for RawGuard {
    fn drop(&mut self) {
        let mut out = stdout();
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
