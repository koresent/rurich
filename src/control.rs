use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControlCode {
    Bell,
    CarriageReturn,
    Home,
    Clear,
    ShowCursor,
    HideCursor,
    EnableAltScreen,
    DisableAltScreen,
    CursorUp(u32),
    CursorDown(u32),
    CursorForward(u32),
    CursorBackward(u32),
    CursorMoveToColumn(u32),
    CursorMoveTo(u32, u32),
    EraseInLine(u32),
    SetWindowTitle(String),
}

impl ControlCode {
    pub fn render_ansi(&self) -> String {
        match self {
            Self::Bell => "\x07".to_string(),
            Self::CarriageReturn => "\r".to_string(),
            Self::Home => "\x1b[H".to_string(),
            Self::Clear => "\x1b[2J".to_string(),
            Self::ShowCursor => "\x1b[?25h".to_string(),
            Self::HideCursor => "\x1b[?25l".to_string(),
            Self::EnableAltScreen => "\x1b[?1049h".to_string(),
            Self::DisableAltScreen => "\x1b[?1049l".to_string(),
            Self::CursorUp(n) => format!("\x1b[{n}A"),
            Self::CursorDown(n) => format!("\x1b[{n}B"),
            Self::CursorForward(n) => format!("\x1b[{n}C"),
            Self::CursorBackward(n) => format!("\x1b[{n}D"),
            Self::CursorMoveToColumn(col) => format!("\x1b[{col}G"),
            Self::CursorMoveTo(x, y) => format!("\x1b[{y};{x}H"),
            Self::EraseInLine(mode) => format!("\x1b[{mode}K"),
            Self::SetWindowTitle(title) => format!("\x1b]0;{title}\x07"),
        }
    }
}

impl fmt::Display for ControlCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render_ansi())
    }
}
