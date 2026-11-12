use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

/// Get the cell length of a single character in terminal columns (0, 1, or 2).
#[inline]
pub fn get_character_cell_size(c: char) -> usize {
    let codepoint = c as u32;
    // Control characters (except zero-width or unprintable)
    if codepoint < 32 || (0x7F..=0x9F).contains(&codepoint) {
        return 0;
    }
    UnicodeWidthChar::width(c).unwrap_or(0)
}

/// Get the cell length of a string as it appears in a terminal.
pub fn cell_len(text: &str) -> usize {
    let mut total = 0;
    for c in text.chars() {
        total += get_character_cell_size(c);
    }
    total
}

/// Pad or truncate text to exact cell size.
pub fn set_cell_size(text: &str, total: usize) -> String {
    let current_len = cell_len(text);
    if current_len == total {
        return text.to_string();
    }
    if current_len < total {
        let mut res = String::with_capacity(text.len() + (total - current_len));
        res.push_str(text);
        res.push_str(&" ".repeat(total - current_len));
        return res;
    }

    // Truncation required
    let mut result = String::new();
    let mut width = 0;

    for g in text.graphemes(true) {
        let g_width: usize = g.chars().map(get_character_cell_size).sum();
        if width + g_width > total {
            // If the grapheme doesn't fit, pad remaining with spaces
            let remaining = total - width;
            result.push_str(&" ".repeat(remaining));
            width += remaining;
            break;
        }
        result.push_str(g);
        width += g_width;
        if width == total {
            break;
        }
    }

    if width < total {
        result.push_str(&" ".repeat(total - width));
    }

    result
}

/// Split text into two parts at a given cell boundary.
pub fn split_cells(text: &str, cut: usize) -> (String, String) {
    let mut left = String::new();
    let mut right = String::new();
    let mut width = 0;
    let mut cut_reached = false;

    for g in text.graphemes(true) {
        let g_width: usize = g.chars().map(get_character_cell_size).sum();
        if !cut_reached {
            if width + g_width <= cut {
                left.push_str(g);
                width += g_width;
                if width == cut {
                    cut_reached = true;
                }
            } else {
                // Grapheme crosses the cut! Pad left with space, right starts with space or grapheme
                let pad = cut - width;
                left.push_str(&" ".repeat(pad));
                let right_pad = g_width - pad;
                right.push_str(&" ".repeat(right_pad));
                cut_reached = true;
            }
        } else {
            right.push_str(g);
        }
    }

    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_len() {
        assert_eq!(cell_len("hello"), 5);
        assert_eq!(cell_len("Hello!"), 6);
        assert_eq!(cell_len("你好"), 4); // CJK characters are 2 cells wide
        assert_eq!(cell_len(""), 0);
    }

    #[test]
    fn test_set_cell_size() {
        assert_eq!(set_cell_size("hello", 10), "hello     ");
        assert_eq!(set_cell_size("hello world", 5), "hello");
        assert_eq!(set_cell_size("你好世界", 4), "你好");
        // Crossing boundary with double width character
        assert_eq!(set_cell_size("你好世界", 3), "你 ");
    }

    #[test]
    fn test_split_cells() {
        let (left, right) = split_cells("hello world", 5);
        assert_eq!(left, "hello");
        assert_eq!(right, " world");

        let (left, right) = split_cells("你好世界", 4);
        assert_eq!(left, "你好");
        assert_eq!(right, "世界");

        let (left, right) = split_cells("你好世界", 3);
        assert_eq!(left, "你 ");
        assert_eq!(right, " 世界");
    }
}
