use crate::segment::Segment;

pub const CONSOLE_HTML_FORMAT: &str = r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<style>
body {
    color: #c0c0c0;
    background-color: #1e1e1e;
    margin: 0;
    padding: 16px;
}
pre {
    font-family: Menlo, 'DejaVu Sans Mono', Consolas, 'Courier New', monospace;
    font-size: 14px;
    line-height: 1.25;
}
code {
    font-family: inherit;
}
</style>
</head>
<body>
    <pre><code>{code}</code></pre>
</body>
</html>"#;

pub const CONSOLE_SVG_FORMAT: &str = r##"<svg class="terminal-window" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">
    <style>
    .terminal-bg { fill: #1e1e1e; rx: 8px; }
    .terminal-bar { fill: #2d2d2d; }
    .terminal-title { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; font-size: 12px; fill: #888888; text-anchor: middle; }
    .terminal-text { font-family: 'Fira Code', Menlo, Consolas, Monaco, monospace; font-size: 13px; fill: #d4d4d4; }
    </style>
    <rect width="{width}" height="{height}" class="terminal-bg" />
    <path d="M 0 8 A 8 8 0 0 1 8 0 L {w_minus_8} 0 A 8 8 0 0 1 {width} 8 L {width} 30 L 0 30 Z" class="terminal-bar" />
    <circle cx="16" cy="15" r="5" fill="#ff5f56" />
    <circle cx="32" cy="15" r="5" fill="#ffbd2e" />
    <circle cx="48" cy="15" r="5" fill="#27c93f" />
    <text x="{mid_x}" y="19" class="terminal-title">{title}</text>
    <g transform="translate(16, 48)" class="terminal-text">
{text_lines}
    </g>
</svg>"##;

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Export lines of segments to standalone HTML.
pub fn export_html(lines: &[Vec<Segment>]) -> String {
    let mut code = String::new();

    for (line_idx, line) in lines.iter().enumerate() {
        if line_idx > 0 {
            code.push('\n');
        }
        for seg in line {
            let escaped = escape_html(&seg.text);
            if let Some(ref style) = seg.style {
                let mut css_rules = Vec::new();
                if let Some(ref c) = style.color {
                    let triplet = c.get_truecolor();
                    css_rules.push(format!("color: {}", triplet.hex()));
                }
                if let Some(ref bg) = style.bgcolor {
                    let triplet = bg.get_truecolor();
                    css_rules.push(format!("background-color: {}", triplet.hex()));
                }
                if style
                    .attributes
                    .contains(crate::style::StyleAttributes::BOLD)
                {
                    css_rules.push("font-weight: bold".to_string());
                }
                if style
                    .attributes
                    .contains(crate::style::StyleAttributes::ITALIC)
                {
                    css_rules.push("font-style: italic".to_string());
                }
                if style
                    .attributes
                    .contains(crate::style::StyleAttributes::UNDERLINE)
                {
                    css_rules.push("text-decoration: underline".to_string());
                }

                if css_rules.is_empty() {
                    code.push_str(&escaped);
                } else {
                    let style_attr = css_rules.join("; ");
                    code.push_str(&format!(r#"<span style="{style_attr}">{escaped}</span>"#));
                }
            } else {
                code.push_str(&escaped);
            }
        }
    }

    CONSOLE_HTML_FORMAT.replace("{code}", &code)
}

/// Export lines of segments to a stylized SVG terminal window image.
pub fn export_svg(lines: &[Vec<Segment>], title: &str) -> String {
    let char_width = 8.2;
    let line_height = 18.0;

    let mut max_cols = 10;
    for line in lines {
        let cols: usize = line.iter().map(|s| s.cell_length()).sum();
        max_cols = max_cols.max(cols);
    }

    let pad_x = 32.0;
    let pad_top = 48.0;
    let pad_bottom = 20.0;

    let width = ((max_cols as f64 * char_width) + pad_x).max(240.0).round() as usize;
    let height = ((lines.len() as f64 * line_height) + pad_top + pad_bottom).round() as usize;

    let mut text_lines = String::new();
    for (i, line) in lines.iter().enumerate() {
        let y = (i as f64 * line_height) + 14.0;
        text_lines.push_str(&format!(r#"      <text y="{y:.1}" xml:space="preserve">"#));

        for seg in line {
            let escaped = escape_xml(&seg.text);
            if let Some(ref style) = seg.style {
                let mut attrs = Vec::new();
                if let Some(ref c) = style.color {
                    let triplet = c.get_truecolor();
                    attrs.push(format!("fill=\"{}\"", triplet.hex()));
                }
                if style
                    .attributes
                    .contains(crate::style::StyleAttributes::BOLD)
                {
                    attrs.push(r#"font-weight="bold""#.to_string());
                }
                if style
                    .attributes
                    .contains(crate::style::StyleAttributes::ITALIC)
                {
                    attrs.push(r#"font-style="italic""#.to_string());
                }
                if attrs.is_empty() {
                    text_lines.push_str(&escaped);
                } else {
                    let attr_str = attrs.join(" ");
                    text_lines.push_str(&format!(r#"<tspan {attr_str}>{escaped}</tspan>"#));
                }
            } else {
                text_lines.push_str(&escaped);
            }
        }
        text_lines.push_str("</text>\n");
    }

    CONSOLE_SVG_FORMAT
        .replace("{width}", &width.to_string())
        .replace("{height}", &height.to_string())
        .replace("{w_minus_8}", &(width - 8).to_string())
        .replace("{mid_x}", &(width / 2).to_string())
        .replace("{title}", &escape_xml(title))
        .replace("{text_lines}", text_lines.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::style::Style;
    use std::borrow::Cow;

    #[test]
    fn test_export_html() {
        let lines = vec![vec![
            Segment {
                text: Cow::Borrowed("Hello "),
                style: Some(Style::new().color(Color::from_rgb(255, 0, 0))),
                control: None,
            },
            Segment {
                text: Cow::Borrowed("<world>"),
                style: Some(Style::new().bold()),
                control: None,
            },
        ]];

        let html = export_html(&lines);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("color: #ff0000"));
        assert!(html.contains("&lt;world&gt;"));
    }

    #[test]
    fn test_export_svg() {
        let lines = vec![vec![Segment {
            text: Cow::Borrowed("cargo test"),
            style: Some(Style::new().color(Color::from_rgb(0, 255, 0))),
            control: None,
        }]];

        let svg = export_svg(&lines, "Terminal - Tests");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Terminal - Tests"));
        assert!(svg.contains("fill=\"#00ff00\""));
        assert!(svg.contains("cargo test"));
    }
}
