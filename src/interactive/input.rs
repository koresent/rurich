use crate::interactive::{redraw_view, InteractiveAction, RawGuard};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::io::{self, stdout};

/// Single-line interactive input prompt with placeholder and password masking.
pub struct Input {
    pub value: String,
    pub prompt: String,
    pub placeholder: String,
    pub password: bool,
    pub cursor: usize,
}

impl Input {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            prompt: "> ".to_string(),
            placeholder: String::new(),
            password: false,
            cursor: 0,
        }
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    pub fn value(mut self, val: impl Into<String>) -> Self {
        self.value = val.into();
        self.cursor = self.value.chars().count();
        self
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InteractiveAction<String> {
        let char_count = self.value.chars().count();
        match key.code {
            KeyCode::Char(c) => {
                let mut chars: Vec<char> = self.value.chars().collect();
                chars.insert(self.cursor, c);
                self.value = chars.into_iter().collect();
                self.cursor += 1;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let mut chars: Vec<char> = self.value.chars().collect();
                    chars.remove(self.cursor - 1);
                    self.value = chars.into_iter().collect();
                    self.cursor -= 1;
                }
            }
            KeyCode::Delete => {
                if self.cursor < char_count {
                    let mut chars: Vec<char> = self.value.chars().collect();
                    chars.remove(self.cursor);
                    self.value = chars.into_iter().collect();
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor < char_count {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => {
                self.cursor = 0;
            }
            KeyCode::End => {
                self.cursor = char_count;
            }
            KeyCode::Enter => {
                return InteractiveAction::Submit(self.value.clone());
            }
            KeyCode::Esc => {
                return InteractiveAction::Cancel;
            }
            _ => {}
        }
        InteractiveAction::Continue
    }

    pub fn run_headless(&mut self, keys: impl IntoIterator<Item = KeyEvent>) -> Option<String> {
        for key in keys {
            match self.handle_key(key) {
                InteractiveAction::Submit(res) => return Some(res),
                InteractiveAction::Cancel => return None,
                InteractiveAction::Continue => continue,
            }
        }
        None
    }

    pub fn render_view(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.prompt);

        if self.value.is_empty() && !self.placeholder.is_empty() {
            out.push_str(&format!("\x1b[2m{}\x1b[0m", self.placeholder));
        } else {
            let chars: Vec<char> = self.value.chars().collect();
            for (i, &ch) in chars.iter().enumerate() {
                let display_ch = if self.password { '*' } else { ch };
                if i == self.cursor {
                    out.push_str(&format!("\x1b[7m{display_ch}\x1b[0m"));
                } else {
                    out.push(display_ch);
                }
            }
            if self.cursor == chars.len() {
                out.push_str("\x1b[7m \x1b[0m");
            }
        }
        out.push('\n');
        out
    }

    pub fn run(&mut self) -> io::Result<Option<String>> {
        let _guard = RawGuard::new()?;
        let mut stdout = stdout();
        let mut printed_lines = 0;

        loop {
            let view = self.render_view();
            redraw_view(&mut stdout, &view, &mut printed_lines)?;

            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(None);
                }
                match self.handle_key(key) {
                    InteractiveAction::Submit(res) => return Ok(Some(res)),
                    InteractiveAction::Cancel => return Ok(None),
                    InteractiveAction::Continue => continue,
                }
            }
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi-line interactive text editor with line numbers.
pub struct TextArea {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub show_line_numbers: bool,
    pub placeholder: String,
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            show_line_numbers: true,
            placeholder: "Enter text (Esc to submit)...".to_string(),
        }
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InteractiveAction<String> {
        let cur_line = &mut self.lines[self.cursor_row];
        let cur_len = cur_line.chars().count();

        match key.code {
            KeyCode::Char(c) => {
                let mut chars: Vec<char> = cur_line.chars().collect();
                chars.insert(self.cursor_col, c);
                *cur_line = chars.into_iter().collect();
                self.cursor_col += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_col > 0 {
                    let mut chars: Vec<char> = cur_line.chars().collect();
                    chars.remove(self.cursor_col - 1);
                    *cur_line = chars.into_iter().collect();
                    self.cursor_col -= 1;
                } else if self.cursor_row > 0 {
                    let prev_row = self.cursor_row - 1;
                    let prev_len = self.lines[prev_row].chars().count();
                    let removed = self.lines.remove(self.cursor_row);
                    self.lines[prev_row].push_str(&removed);
                    self.cursor_row = prev_row;
                    self.cursor_col = prev_len;
                }
            }
            KeyCode::Enter => {
                let rest: String = cur_line.chars().skip(self.cursor_col).collect();
                *cur_line = cur_line.chars().take(self.cursor_col).collect();
                self.lines.insert(self.cursor_row + 1, rest);
                self.cursor_row += 1;
                self.cursor_col = 0;
            }
            KeyCode::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.cursor_col = self.lines[self.cursor_row].chars().count();
                }
            }
            KeyCode::Right => {
                if self.cursor_col < cur_len {
                    self.cursor_col += 1;
                } else if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.cursor_col = 0;
                }
            }
            KeyCode::Up => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].chars().count());
                }
            }
            KeyCode::Down => {
                if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].chars().count());
                }
            }
            KeyCode::Esc => {
                return InteractiveAction::Submit(self.lines.join("\n"));
            }
            _ => {}
        }

        InteractiveAction::Continue
    }

    pub fn run_headless(&mut self, keys: impl IntoIterator<Item = KeyEvent>) -> Option<String> {
        for key in keys {
            match self.handle_key(key) {
                InteractiveAction::Submit(res) => return Some(res),
                InteractiveAction::Cancel => return None,
                InteractiveAction::Continue => continue,
            }
        }
        None
    }

    pub fn render_view(&self) -> String {
        let mut out = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            if self.show_line_numbers {
                out.push_str(&format!("\x1b[2m{:3} │ \x1b[0m", i + 1));
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    pub fn run(&mut self) -> io::Result<Option<String>> {
        let _guard = RawGuard::new()?;
        let mut stdout = stdout();
        let mut printed_lines = 0;

        loop {
            let view = self.render_view();
            redraw_view(&mut stdout, &view, &mut printed_lines)?;

            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(None);
                }
                match self.handle_key(key) {
                    InteractiveAction::Submit(res) => return Ok(Some(res)),
                    InteractiveAction::Cancel => return Ok(None),
                    InteractiveAction::Continue => continue,
                }
            }
        }
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_typing() {
        let mut input = Input::new();
        let keys = vec![
            KeyEvent::from(KeyCode::Char('H')),
            KeyEvent::from(KeyCode::Char('i')),
            KeyEvent::from(KeyCode::Enter),
        ];

        let res = input.run_headless(keys);
        assert_eq!(res, Some("Hi".to_string()));
    }

    #[test]
    fn test_textarea_multiline() {
        let mut ta = TextArea::new();
        let keys = vec![
            KeyEvent::from(KeyCode::Char('A')),
            KeyEvent::from(KeyCode::Enter),
            KeyEvent::from(KeyCode::Char('B')),
            KeyEvent::from(KeyCode::Esc),
        ];

        let res = ta.run_headless(keys);
        assert_eq!(res, Some("A\nB".to_string()));
    }
}
