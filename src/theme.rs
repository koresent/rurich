use crate::default_styles::DEFAULT_STYLES;
use crate::style::Style;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

static GLOBAL_THEME: LazyLock<RwLock<Theme>> = LazyLock::new(|| RwLock::new(Theme::default()));

pub fn get_theme_style(name: &str) -> Option<Style> {
    if let Ok(theme) = GLOBAL_THEME.read()
        && let Some(s) = theme.get(name)
    {
        return Some(s);
    }
    DEFAULT_STYLES.get(name).cloned()
}

pub fn set_global_theme(theme: Theme) {
    if let Ok(mut g) = GLOBAL_THEME.write() {
        *g = theme;
    }
}

pub fn get_global_theme() -> Theme {
    GLOBAL_THEME
        .read()
        .map(|g| g.clone())
        .unwrap_or_else(|_| Theme::default())
}

pub fn update_global_theme<F>(f: F)
where
    F: FnOnce(&mut Theme),
{
    if let Ok(mut g) = GLOBAL_THEME.write() {
        f(&mut g);
    }
}

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
