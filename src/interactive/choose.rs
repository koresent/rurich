use crate::interactive::{InteractiveAction, RawGuard, redraw_view};
use crate::style::Style;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::fmt::Display;
use std::io::{self, stdout};

/// Interactive selection list from a set of choices.
pub struct Choose<T = String> {
    pub items: Vec<T>,
    pub cursor: usize,
    pub selected: Vec<bool>,
    pub multi: bool,
    pub cursor_prefix: String,
    pub selected_prefix: String,
    pub unselected_prefix: String,
    pub header: Option<String>,
    pub cursor_style: Style,
    pub item_style: Style,
    pub selected_style: Style,
}

impl<T: Display + Clone> Choose<T> {
    pub fn new(items: impl IntoIterator<Item = T>) -> Self {
        let items: Vec<T> = items.into_iter().collect();
        let len = items.len();
        Self {
            items,
            cursor: 0,
            selected: vec![false; len],
            multi: false,
            cursor_prefix: "> ".to_string(),
            selected_prefix: "[x] ".to_string(),
            unselected_prefix: "[ ] ".to_string(),
            header: None,
            cursor_style: Style::parse("bold 212").unwrap_or_default(),
            item_style: Style::new(),
            selected_style: Style::parse("bold 212").unwrap_or_default(),
        }
    }

    pub fn multi(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }

    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn cursor_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.cursor_prefix = prefix.into();
        self
    }

    /// Process a key event. Returns an InteractiveAction.
    pub fn handle_key(&mut self, key: KeyEvent) -> InteractiveAction<Vec<T>> {
        if key.kind == KeyEventKind::Release {
            return InteractiveAction::Continue;
        }

        if self.items.is_empty() {
            return InteractiveAction::Submit(Vec::new());
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor == 0 {
                    self.cursor = self.items.len().saturating_sub(1);
                } else {
                    self.cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor + 1 >= self.items.len() {
                    self.cursor = 0;
                } else {
                    self.cursor += 1;
                }
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.cursor = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.cursor = self.items.len().saturating_sub(1);
            }
            KeyCode::Char(' ') if self.multi => {
                if self.cursor < self.selected.len() {
                    self.selected[self.cursor] = !self.selected[self.cursor];
                }
            }
            KeyCode::Enter => {
                if self.multi {
                    let mut res = Vec::new();
                    for (i, item) in self.items.iter().enumerate() {
                        if self.selected[i] {
                            res.push(item.clone());
                        }
                    }
                    if res.is_empty() && self.cursor < self.items.len() {
                        res.push(self.items[self.cursor].clone());
                    }
                    return InteractiveAction::Submit(res);
                } else {
                    return InteractiveAction::Submit(vec![self.items[self.cursor].clone()]);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                return InteractiveAction::Cancel;
            }
            _ => {}
        }

        InteractiveAction::Continue
    }

    /// Run the interactive selector through a sequence of simulated keys (for tests).
    pub fn run_headless(&mut self, keys: impl IntoIterator<Item = KeyEvent>) -> Option<Vec<T>> {
        for key in keys {
            match self.handle_key(key) {
                InteractiveAction::Submit(res) => return Some(res),
                InteractiveAction::Cancel => return None,
                InteractiveAction::Continue => continue,
            }
        }
        None
    }

    /// Render the current view to a multi-line string.
    pub fn render_view(&self) -> String {
        let mut out = String::new();
        if let Some(ref h) = self.header {
            let header_fg = crate::interactive::env_fg_ansi("GUM_CHOOSE_HEADER_FOREGROUND", "#928374");
            out.push_str(&format!("{header_fg}{h}\x1b[0m\n"));
        }

        let cursor_fg = crate::interactive::env_fg_ansi("GUM_CHOOSE_CURSOR_FOREGROUND", "#fabd2f");
        let selected_fg = crate::interactive::env_fg_ansi("GUM_CHOOSE_SELECTED_FOREGROUND", "#fabd2f");

        for (i, item) in self.items.iter().enumerate() {
            let is_cursor = i == self.cursor;

            if is_cursor {
                out.push_str(&cursor_fg);
                out.push_str(&self.cursor_prefix);

                if self.multi {
                    let sel_mark = if self.selected.get(i).copied().unwrap_or(false) {
                        &self.selected_prefix
                    } else {
                        &self.unselected_prefix
                    };
                    out.push_str(sel_mark);
                }

                out.push_str(&selected_fg);
                out.push_str(&item.to_string());
                out.push_str("\x1b[0m\n");
            } else {
                out.push_str("  ");

                if self.multi {
                    let sel_mark = if self.selected.get(i).copied().unwrap_or(false) {
                        &self.selected_prefix
                    } else {
                        &self.unselected_prefix
                    };
                    out.push_str(sel_mark);
                }

                out.push_str(&item.to_string());
                out.push('\n');
            }
        }

        out
    }

    /// Run the interactive selection loop on the current terminal.
    pub fn run(&mut self) -> io::Result<Option<Vec<T>>> {
        while event::poll(std::time::Duration::from_millis(10))? {
            let _ = event::read()?;
        }
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
                    InteractiveAction::Submit(result) => return Ok(Some(result)),
                    InteractiveAction::Cancel => return Ok(None),
                    InteractiveAction::Continue => continue,
                }
            }
        }
    }
}

pub type MultiChoose<T> = Choose<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choose_single_navigation() {
        let mut choose = Choose::new(vec!["Apple", "Banana", "Cherry"]);
        let keys = vec![
            KeyEvent::from(KeyCode::Down),
            KeyEvent::from(KeyCode::Enter),
        ];

        let result = choose.run_headless(keys);
        assert_eq!(result, Some(vec!["Banana"]));
    }

    #[test]
    fn test_choose_multi_selection() {
        let mut choose = Choose::new(vec!["One", "Two", "Three"]).multi(true);
        let keys = vec![
            KeyEvent::from(KeyCode::Char(' ')), // select One
            KeyEvent::from(KeyCode::Down),
            KeyEvent::from(KeyCode::Down),
            KeyEvent::from(KeyCode::Char(' ')), // select Three
            KeyEvent::from(KeyCode::Enter),
        ];

        let result = choose.run_headless(keys);
        assert_eq!(result, Some(vec!["One", "Three"]));
    }
}
