use crate::default_styles::DEFAULT_STYLES;
use crate::style::Style;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Theme {
    styles: HashMap<String, Style>,
    inherit: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}

impl Theme {
    pub fn new() -> Self {
        Self {
            styles: HashMap::new(),
            inherit: true,
        }
    }

    pub fn with_styles(styles: HashMap<String, Style>, inherit: bool) -> Self {
        Self { styles, inherit }
    }

    pub fn set(&mut self, name: impl Into<String>, style: Style) {
        self.styles.insert(name.into(), style);
    }

    pub fn get(&self, name: &str) -> Option<Style> {
        if let Some(style) = self.styles.get(name) {
            return Some(style.clone());
        }
        if self.inherit {
            return DEFAULT_STYLES.get(name).cloned();
        }
        None
    }
}
