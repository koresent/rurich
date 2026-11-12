use std::collections::HashMap;
use std::sync::LazyLock;

static EMOJI_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("smile", "😄");
    m.insert("grin", "😁");
    m.insert("laughing", "😆");
    m.insert("rofl", "🤣");
    m.insert("joy", "😂");
    m.insert("heart", "❤️");
    m.insert("sparkles", "✨");
    m.insert("star", "⭐");
    m.insert("fire", "🔥");
    m.insert("rocket", "🚀");
    m.insert("tada", "🎉");
    m.insert("check", "✔");
    m.insert("cross", "❌");
    m.insert("warning", "⚠");
    m.insert("thumbs_up", "👍");
    m.insert("thumbs_down", "👎");
    m.insert("clap", "👏");
    m.insert("package", "📦");
    m.insert("gear", "⚙️");
    m.insert("wrench", "🔧");
    m.insert("bug", "🐛");
    m.insert("zap", "⚡");
    m.insert("bulb", "💡");
    m.insert("lock", "🔒");
    m.insert("key", "🔑");
    m.insert("bell", "🔔");
    m.insert("memo", "📝");
    m.insert("book", "📖");
    m.insert("link", "🔗");
    m.insert("globe", "🌐");
    m.insert("crab", "🦀");
    m.insert("snake", "🐍");
    m
});

pub struct Emoji;

impl Emoji {
    /// Look up an emoji character by code name (e.g. "rocket" -> "🚀").
    pub fn get(name: &str) -> Option<&'static str> {
        EMOJI_MAP.get(name).copied()
    }

    /// Replace all `:emoji_name:` occurrences in text with their unicode equivalent.
    pub fn replace(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == ':' {
                let mut name = String::new();
                let mut found_close = false;
                for nc in chars.by_ref() {
                    if nc == ':' {
                        found_close = true;
                        break;
                    }
                    if nc.is_alphanumeric() || nc == '_' {
                        name.push(nc);
                    } else {
                        // Not a valid emoji name
                        result.push(':');
                        result.push_str(&name);
                        result.push(nc);
                        found_close = false;
                        break;
                    }
                }

                if found_close {
                    if let Some(emoji_char) = Self::get(&name) {
                        result.push_str(emoji_char);
                    } else {
                        result.push(':');
                        result.push_str(&name);
                        result.push(':');
                    }
                }
            } else {
                result.push(c);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji_replacement() {
        let text = "Hello :crab: and :rocket:! Ignored: :unknown_xyz: and normal : text.";
        let replaced = Emoji::replace(text);
        assert_eq!(
            replaced,
            "Hello 🦀 and 🚀! Ignored: :unknown_xyz: and normal : text."
        );
    }
}
