use crate::console::Console;
use crate::interactive::{InteractiveAction, RawGuard, redraw_view};
use crate::protocol::Renderable;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::io;

/// Interactive scrollable text and document pager.
pub struct Pager {
    pub lines: Vec<String>,
    pub top_line: usize,
    pub height: usize,
    pub show_line_numbers: bool,
}

impl Pager {
    pub fn new(content: &str) -> Self {
        Self {
            lines: content.lines().map(|s| s.to_string()).collect(),
            top_line: 0,
            height: 20,
            show_line_numbers: false,
        }
    }

    pub fn from_renderable(renderable: &dyn Renderable, console: &Console) -> Self {
        let options = console.options();
        let rendered_lines = console.render_lines_owned(renderable, Some(&options), false);
        let mut lines = Vec::new();
        for line in rendered_lines {
            let mut line_str = String::new();
            for seg in line {
                line_str.push_str(&seg.text);
            }
            lines.push(line_str);
        }
        Self {
            lines,
            top_line: 0,
            height: 20,
            show_line_numbers: false,
        }
    }

    pub fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InteractiveAction<()> {
        let total = self.lines.len();
        let max_top = total.saturating_sub(self.height);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.top_line > 0 {
                    self.top_line -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.top_line < max_top {
                    self.top_line += 1;
                }
            }
            KeyCode::PageUp | KeyCode::Char('b') => {
                self.top_line = self.top_line.saturating_sub(self.height);
            }
            KeyCode::PageDown | KeyCode::Char(' ') | KeyCode::Char('f') => {
                self.top_line = (self.top_line + self.height).min(max_top);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.top_line = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.top_line = max_top;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                return InteractiveAction::Submit(()); // exit pager
            }
            _ => {}
        }
        InteractiveAction::Continue
    }

    pub fn run_headless(&mut self, keys: impl IntoIterator<Item = KeyEvent>) {
        for key in keys {
            if let InteractiveAction::Submit(()) | InteractiveAction::Cancel = self.handle_key(key)
            {
                break;
            }
        }
    }

    pub fn render_view(&self) -> String {
        let mut out = String::new();
        let total = self.lines.len();
        let end_line = (self.top_line + self.height).min(total);

        for i in self.top_line..end_line {
            if self.show_line_numbers {
                out.push_str(&format!("\x1b[2m{:4} │ \x1b[0m", i + 1));
            }
            out.push_str(&self.lines[i]);
            out.push('\n');
        }

        // Status bar
        let percent = if total > 0 {
            ((end_line as f64 / total as f64) * 100.0).round() as usize
        } else {
            100
        };
        out.push_str(&format!(
            "\x1b[7m Lines {}-{}/{} ({}%) [q: Quit, j/k: Scroll] \x1b[0m\n",
            self.top_line + 1,
            end_line,
            total,
            percent
        ));

        out
    }

    pub fn run(&mut self) -> io::Result<()> {
        let _guard = RawGuard::new()?;
        let mut term_out = crate::interactive::terminal_writer();
        let mut printed_lines = 0;

        loop {
            let view = self.render_view();
            redraw_view(&mut term_out, &view, &mut printed_lines)?;

            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(());
                }
                match self.handle_key(key) {
                    InteractiveAction::Submit(()) | InteractiveAction::Cancel => return Ok(()),
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
    fn test_pager_scroll() {
        let content = (1..=50)
            .map(|i| format!("Line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut pager = Pager::new(&content).height(10);
        assert_eq!(pager.top_line, 0);

        let keys = vec![KeyEvent::from(KeyCode::Down), KeyEvent::from(KeyCode::Down)];
        pager.run_headless(keys);
        assert_eq!(pager.top_line, 2);
    }
}
