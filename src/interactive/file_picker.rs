use crate::interactive::{redraw_view, InteractiveAction, RawGuard};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::fs;
use std::io::{self, stdout};
use std::path::{Path, PathBuf};

/// Interactive file picker and directory tree navigator.
pub struct FilePicker {
    pub current_dir: PathBuf,
    pub entries: Vec<PathBuf>,
    pub cursor: usize,
    pub show_hidden: bool,
}

impl FilePicker {
    pub fn new(start_dir: impl AsRef<Path>) -> io::Result<Self> {
        let path = start_dir.as_ref().canonicalize()?;
        let mut picker = Self {
            current_dir: path,
            entries: Vec::new(),
            cursor: 0,
            show_hidden: false,
        };
        picker.refresh_entries()?;
        Ok(picker)
    }

    pub fn show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        let _ = self.refresh_entries();
        self
    }

    pub fn refresh_entries(&mut self) -> io::Result<()> {
        let mut entries = Vec::new();
        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            for entry in read_dir.flatten() {
                let p = entry.path();
                let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !self.show_hidden && file_name.starts_with('.') {
                    continue;
                }
                entries.push(p);
            }
        }
        // Sort: directories first, then alphabetically
        entries.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });
        self.entries = entries;
        self.cursor = self.cursor.min(self.entries.len().saturating_sub(1));
        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InteractiveAction<PathBuf> {
        let total = self.entries.len();

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if total > 0 {
                    if self.cursor == 0 {
                        self.cursor = total - 1;
                    } else {
                        self.cursor -= 1;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if total > 0 {
                    if self.cursor + 1 >= total {
                        self.cursor = 0;
                    } else {
                        self.cursor += 1;
                    }
                }
            }
            KeyCode::Backspace | KeyCode::Char('h') => {
                if let Some(parent) = self.current_dir.parent().map(|p| p.to_path_buf()) {
                    self.current_dir = parent;
                    let _ = self.refresh_entries();
                    self.cursor = 0;
                }
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(selected) = self.entries.get(self.cursor).cloned() {
                    if selected.is_dir() {
                        self.current_dir = selected;
                        let _ = self.refresh_entries();
                        self.cursor = 0;
                    } else {
                        return InteractiveAction::Submit(selected);
                    }
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                return InteractiveAction::Cancel;
            }
            _ => {}
        }
        InteractiveAction::Continue
    }

    pub fn run_headless(&mut self, keys: impl IntoIterator<Item = KeyEvent>) -> Option<PathBuf> {
        for key in keys {
            match self.handle_key(key) {
                InteractiveAction::Submit(path) => return Some(path),
                InteractiveAction::Cancel => return None,
                InteractiveAction::Continue => continue,
            }
        }
        None
    }

    pub fn render_view(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "\x1b[1;34m📁 {}\x1b[0m\n",
            self.current_dir.display()
        ));

        for (i, entry) in self.entries.iter().enumerate() {
            let is_cursor = i == self.cursor;
            let mark = if is_cursor { "> " } else { "  " };
            let file_name = entry
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            if entry.is_dir() {
                out.push_str(&format!("{mark}\x1b[1;34m📁 {file_name}/\x1b[0m\n"));
            } else {
                out.push_str(&format!("{mark}📄 {file_name}\n"));
            }
        }
        out
    }

    pub fn run(&mut self) -> io::Result<Option<PathBuf>> {
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
                    InteractiveAction::Submit(path) => return Ok(Some(path)),
                    InteractiveAction::Cancel => return Ok(None),
                    InteractiveAction::Continue => continue,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_picker_entries() {
        let picker = FilePicker::new(".").expect("valid current dir");
        assert!(!picker.entries.is_empty());
    }
}
