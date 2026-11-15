use crate::interactive::{InteractiveAction, RawGuard, redraw_view};
use crate::style::Style;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::fmt::Display;
use std::io::{self, stdout};

pub fn fuzzy_match(pattern: &str, text: &str) -> Option<(i64, Vec<usize>)> {
    if pattern.is_empty() {
        return Some((0, Vec::new()));
    }

    let pattern_chars: Vec<char> = pattern.to_lowercase().chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    let text_lower: Vec<char> = text.to_lowercase().chars().collect();

    let mut pattern_idx = 0;
    let mut matched_indices = Vec::new();
    let mut score = 0i64;
    let mut prev_matched_idx = None;

    for (i, &ch) in text_lower.iter().enumerate() {
        if pattern_idx < pattern_chars.len() && ch == pattern_chars[pattern_idx] {
            matched_indices.push(i);
            score += 10;

            // Consecutive bonus
            if let Some(prev) = prev_matched_idx
                && prev + 1 == i
            {
                score += 15;
            }

            // Word boundary bonus
            if i == 0 || matches!(text_chars[i - 1], ' ' | '_' | '-' | '/' | '.' | ':') {
                score += 20;
            }

            prev_matched_idx = Some(i);
            pattern_idx += 1;
        }
    }

    if pattern_idx == pattern_chars.len() {
        Some((score, matched_indices))
    } else {
        None
    }
}

pub struct Filter<T = String> {
    pub items: Vec<T>,
    pub query: String,
    pub cursor: usize,
    pub prompt: String,
    pub placeholder: String,
    pub match_style: Style,
}

impl<T: Display + Clone> Filter<T> {
    pub fn new(items: impl IntoIterator<Item = T>) -> Self {
        Self {
            items: items.into_iter().collect(),
            query: String::new(),
            cursor: 0,
            prompt: "> ".to_string(),
            placeholder: "Type to filter...".to_string(),
            match_style: Style::parse("bold cyan underline").unwrap_or_default(),
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

    pub fn get_matches(&self) -> Vec<(usize, i64, Vec<usize>)> {
        let mut matches = Vec::new();
        for (idx, item) in self.items.iter().enumerate() {
            let item_str = item.to_string();
            if let Some((score, indices)) = fuzzy_match(&self.query, &item_str) {
                matches.push((idx, score, indices));
            }
        }
        // Sort descending by score
        matches.sort_by_key(|a| std::cmp::Reverse(a.1));
        matches
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InteractiveAction<T> {
        if key.kind == KeyEventKind::Release {
            return InteractiveAction::Continue;
        }

        let matches = self.get_matches();
        let match_count = matches.len();

        match key.code {
            KeyCode::Char(c) => {
                self.query.push(c);
                self.cursor = 0;
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.cursor = 0;
            }
            KeyCode::Up => {
                if match_count > 0 {
                    if self.cursor == 0 {
                        self.cursor = match_count - 1;
                    } else {
                        self.cursor -= 1;
                    }
                }
            }
            KeyCode::Down => {
                if match_count > 0 {
                    if self.cursor + 1 >= match_count {
                        self.cursor = 0;
                    } else {
                        self.cursor += 1;
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(&(orig_idx, _, _)) = matches.get(self.cursor) {
                    return InteractiveAction::Submit(self.items[orig_idx].clone());
                } else {
                    return InteractiveAction::Cancel;
                }
            }
            KeyCode::Esc => {
                return InteractiveAction::Cancel;
            }
            _ => {}
        }

        InteractiveAction::Continue
    }

    pub fn run_headless(&mut self, keys: impl IntoIterator<Item = KeyEvent>) -> Option<T> {
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
        let prompt_fg = crate::interactive::env_fg_ansi("GUM_FILTER_PROMPT_FOREGROUND", "#fabd2f");
        let placeholder_fg = crate::interactive::env_fg_ansi("GUM_FILTER_HEADER_FOREGROUND", "#928374");
        let indicator_fg = crate::interactive::env_fg_ansi("GUM_FILTER_INDICATOR_FOREGROUND", "#fabd2f");
        let match_fg = crate::interactive::env_fg_ansi("GUM_FILTER_MATCH_FOREGROUND", "#fb4934");

        let prompt_symbol = self.prompt.trim_end();
        out.push_str(&format!("{prompt_fg}{prompt_symbol}\x1b[0m "));
        if self.query.is_empty() {
            out.push_str(&format!("{placeholder_fg}{}\x1b[0m", self.placeholder));
        } else {
            out.push_str(&self.query);
            out.push_str("\x1b[7m \x1b[0m");
        }
        out.push('\n');

        let matches = self.get_matches();
        for (i, &(orig_idx, _, ref matched_indices)) in matches.iter().enumerate() {
            let is_cursor = i == self.cursor;
            let mark = if is_cursor { format!("{indicator_fg}•\x1b[0m ") } else { "  ".to_string() };
            out.push_str(&mark);

            let item_str = self.items[orig_idx].to_string();
            let chars: Vec<char> = item_str.chars().collect();
            for (char_idx, &ch) in chars.iter().enumerate() {
                if matched_indices.contains(&char_idx) {
                    out.push_str(&format!("{match_fg}{ch}\x1b[0m"));
                } else {
                    out.push(ch);
                }
            }
            out.push('\n');
        }

        out
    }

    pub fn run(&mut self) -> io::Result<Option<T>> {
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
    fn test_fuzzy_match() {
        let (score, indices) = fuzzy_match("fb", "foobar").expect("should match");
        assert!(score > 0);
        assert_eq!(indices, vec![0, 3]);
    }

    #[test]
    fn test_filter_headless() {
        let mut filter = Filter::new(vec!["cargo", "git", "crossterm", "curl"]);
        let keys = vec![
            KeyEvent::from(KeyCode::Char('c')),
            KeyEvent::from(KeyCode::Char('u')),
            KeyEvent::from(KeyCode::Enter),
        ];

        let res = filter.run_headless(keys);
        assert_eq!(res, Some("curl"));
    }
}
