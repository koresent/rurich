use crate::style::Style;
use std::collections::HashMap;
use std::sync::LazyLock;

pub static DEFAULT_STYLES: LazyLock<HashMap<&'static str, Style>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    let entries: &[(&'static str, &'static str)] = &[
        ("none", "none"),
        ("dim", "dim"),
        ("bright", "not dim"),
        ("bold", "bold"),
        ("b", "bold"),
        ("strong", "bold"),
        ("italic", "italic"),
        ("i", "italic"),
        ("emphasize", "italic"),
        ("underline", "underline"),
        ("u", "underline"),
        ("strike", "strike"),
        ("s", "strike"),
        ("reverse", "reverse"),
        ("blink", "blink"),
        ("black", "black"),
        ("red", "red"),
        ("green", "green"),
        ("yellow", "yellow"),
        ("blue", "blue"),
        ("magenta", "magenta"),
        ("cyan", "cyan"),
        ("white", "white"),
        // Semantic styles
        ("info", "dim cyan"),
        ("warning", "bold yellow"),
        ("danger", "bold red"),
        ("error", "bold red"),
        ("success", "bold green"),
        // Repr styles
        ("repr.str", "green"),
        ("repr.brace", "bold"),
        ("repr.tag_start", "bold"),
        ("repr.tag_name", "bright_magenta"),
        ("repr.tag_contents", "default"),
        ("repr.tag_end", "bold"),
        ("repr.attrib_name", "yellow italic"),
        ("repr.attrib_equal", "bold"),
        ("repr.attrib_value", "magenta"),
        ("repr.number", "cyan"),
        ("repr.number_complex", "cyan"),
        ("repr.bool_true", "italic green"),
        ("repr.bool_false", "italic red"),
        ("repr.none", "italic magenta"),
        ("repr.url", "underline not bold blue"),
        ("repr.uuid", "bright_yellow"),
        ("repr.call", "magenta bold"),
        ("repr.path", "magenta"),
        ("repr.filename", "bright_magenta"),
        // Rule & Table styles
        ("rule.line", "bright_green"),
        ("rule.text", "italic"),
        ("table.header", "bold"),
        ("table.footer", "bold"),
        ("table.cell", "none"),
        ("table.title", "bold italic"),
        ("table.caption", "italic dim"),
    ];

    for &(name, def) in entries {
        if let Ok(style) = Style::parse(def) {
            m.insert(name, style);
        }
    }
    m
});
