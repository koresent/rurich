use crate::interactive::{InteractiveAction, RawGuard, redraw_view};
use crate::style::Style;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io::{self, stdout};

/// Visual confirmation dialog with styled buttons.
pub struct ConfirmDialog {
    pub message: String,
    pub yes_label: String,
    pub no_label: String,
    pub active: bool, // true for Yes, false for No
    pub active_style: Style,
    pub inactive_style: Style,
}

impl ConfirmDialog {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            yes_label: "Yes".to_string(),
            no_label: "No".to_string(),
            active: true,
            active_style: Style::parse("bold white on green").unwrap_or_default(),
            inactive_style: Style::parse("dim").unwrap_or_default(),
        }
    }

    pub fn default_value(mut self, default: bool) -> Self {
        self.active = default;
        self
    }

    pub fn yes_label(mut self, label: impl Into<String>) -> Self {
        self.yes_label = label.into();
        self
    }

    pub fn no_label(mut self, label: impl Into<String>) -> Self {
        self.no_label = label.into();
        self
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InteractiveAction<bool> {
        if key.kind == KeyEventKind::Release {
            return InteractiveAction::Continue;
        }

        match key.code {
            KeyCode::Left
            | KeyCode::Right
            | KeyCode::Tab
            | KeyCode::Char('h')
            | KeyCode::Char('l') => {
                self.active = !self.active;
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                return InteractiveAction::Submit(true);
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                return InteractiveAction::Submit(false);
            }
            KeyCode::Enter => {
                return InteractiveAction::Submit(self.active);
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                return InteractiveAction::Cancel;
            }
            _ => {}
        }
        InteractiveAction::Continue
    }

    pub fn run_headless(&mut self, keys: impl IntoIterator<Item = KeyEvent>) -> Option<bool> {
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
        let prompt_fg = crate::interactive::env_fg_ansi("GUM_CONFIRM_PROMPT_FOREGROUND", "#ebdbb2");
        out.push_str(&format!("{prompt_fg}{}\x1b[0m\n", self.message));

        let sel_bg = crate::interactive::env_bg_ansi("GUM_CONFIRM_SELECTED_BACKGROUND", "#b8bb26");
        let sel_fg = crate::interactive::env_fg_ansi("GUM_CONFIRM_SELECTED_FOREGROUND", "#282828");
        let unsel_bg =
            crate::interactive::env_bg_ansi("GUM_CONFIRM_UNSELECTED_BACKGROUND", "#3c3836");
        let unsel_fg =
            crate::interactive::env_fg_ansi("GUM_CONFIRM_UNSELECTED_FOREGROUND", "#ebdbb2");

        let selected_style = format!("{sel_bg}{sel_fg}");
        let unselected_style = format!("{unsel_bg}{unsel_fg}");

        let (yes_btn, no_btn) = if self.active {
            (
                format!("{selected_style}   {}   \x1b[0m", self.yes_label),
                format!("{unselected_style}   {}   \x1b[0m", self.no_label),
            )
        } else {
            (
                format!("{unselected_style}   {}   \x1b[0m", self.yes_label),
                format!("{selected_style}   {}   \x1b[0m", self.no_label),
            )
        };

        out.push_str(&format!(" {yes_btn}  {no_btn}\n"));
        out
    }

    pub fn run(&mut self) -> io::Result<Option<bool>> {
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
                    InteractiveAction::Submit(res) => return Ok(Some(res)),
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
    fn test_confirm_dialog_toggle() {
        let mut dialog = ConfirmDialog::new("Proceed?");
        assert!(dialog.active);

        // Tab toggles to No, Enter confirms
        let keys = vec![KeyEvent::from(KeyCode::Tab), KeyEvent::from(KeyCode::Enter)];

        let res = dialog.run_headless(keys);
        assert_eq!(res, Some(false));
    }
}
