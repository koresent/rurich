use rurich::color::ColorSystem;
use rurich::console::Console;
use rurich::json::Json;
use rurich::markdown::Markdown;
use rurich::pretty::Pretty;
use rurich::syntax::Syntax;
use rurich::tree::{ASCII_GUIDES, BOLD_GUIDES, DOUBLE_GUIDES, Tree};

#[test]
fn test_tree_nested_parity() {
    let console = Console::with_dimensions(40, 10).with_color_system(ColorSystem::NoColor);
    let mut tree = Tree::new("Root");
    tree.add("Child 1");
    let sub = tree.add("Child 2");
    sub.add("Subchild 2.1");
    tree.add("Child 3");

    let out = console.capture(|c| {
        c.print(tree);
    });

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "Root");
    assert_eq!(lines[1], "├── Child 1");
    assert_eq!(lines[2], "├── Child 2");
    assert_eq!(lines[3], "│   └── Subchild 2.1");
    assert_eq!(lines[4], "└── Child 3");
}

#[test]
fn test_tree_guides_styles() {
    let console = Console::with_dimensions(40, 10).with_color_system(ColorSystem::NoColor);

    // ASCII guides
    let mut tree_ascii = Tree::new("Root").guides(ASCII_GUIDES);
    tree_ascii.add("Leaf A");
    tree_ascii.add("Leaf B");

    let out_ascii = console.capture(|c| {
        c.print(tree_ascii);
    });
    let lines_ascii: Vec<&str> = out_ascii.lines().collect();
    assert_eq!(lines_ascii[0], "Root");
    assert_eq!(lines_ascii[1], "+-- Leaf A");
    assert_eq!(lines_ascii[2], "`-- Leaf B");

    // Bold guides
    let mut tree_bold = Tree::new("Root").guides(BOLD_GUIDES);
    tree_bold.add("Leaf 1");
    let out_bold = console.capture(|c| {
        c.print(tree_bold);
    });
    assert!(out_bold.contains("┗━━ Leaf 1"));

    // Double guides
    let mut tree_double = Tree::new("Root").guides(DOUBLE_GUIDES);
    tree_double.add("Leaf 1");
    let out_double = console.capture(|c| {
        c.print(tree_double);
    });
    assert!(out_double.contains("╚══ Leaf 1"));
}

#[test]
fn test_markdown_full_features() {
    let md_source = r#"# Main Header

A paragraph with **bold text**, *italic text*, and `code inline`.

> A wise blockquote
> with two lines.

## Subsection

1. First item
2. Second item
"#;

    let md = Markdown::new(md_source);
    let console = Console::with_dimensions(60, 25).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| {
        c.print(md);
    });

    assert!(out.contains("Main Header"));
    assert!(out.contains("bold text"));
    assert!(out.contains("italic text"));
    assert!(out.contains("`code inline`"));
    assert!(out.contains("▌ "));
    assert!(out.contains("A wise blockquote"));
    assert!(out.contains("Subsection"));
    assert!(out.contains("1. First item"));
    assert!(out.contains("2. Second item"));
}

#[test]
fn test_json_formatting_and_indentation() {
    let raw = r#"{"project": "rurich", "version": [0, 1, 0], "features": {"markdown": true, "syntax": true}}"#;
    let json = Json::from_str(raw).unwrap().indent(4);

    let console = Console::with_dimensions(50, 15).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| {
        c.print(json);
    });

    assert!(out.contains(r#""project": "rurich""#));
    assert!(out.contains(r#"    "version": ["#));
    assert!(out.contains("        0,"));
    assert!(out.contains(r#""markdown": true"#));
}

#[derive(Debug)]
#[allow(dead_code)]
struct Config {
    debug: bool,
    port: u16,
    hosts: Vec<String>,
}

#[test]
fn test_pretty_repr_debug() {
    let cfg = Config {
        debug: true,
        port: 8080,
        hosts: vec!["localhost".into(), "127.0.0.1".into()],
    };

    let pretty = Pretty::new(&cfg);
    let console = Console::with_dimensions(50, 10).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| {
        c.print(pretty);
    });

    assert!(out.contains("Config"));
    assert!(out.contains("debug: true"));
    assert!(out.contains("port: 8080"));
    assert!(out.contains("\"localhost\""));
}

#[test]
fn test_syntax_highlighting_line_numbers_and_ranges() {
    let code = r#"def calculate_sum(a, b):
    # Sum two numbers
    result = a + b
    return result
"#;

    let syn = Syntax::new(code, "python")
        .line_numbers(true)
        .start_line(10)
        .line_range(11, 13)
        .highlight_line(12);

    let console = Console::with_dimensions(50, 10).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| {
        c.print(syn);
    });

    // Line 10 should be skipped by line_range
    assert!(!out.contains("calculate_sum"));
    // Lines 11, 12, 13 should be present
    assert!(out.contains("11 │"));
    assert!(out.contains("12 │"));
    assert!(out.contains("13 │"));
    assert!(out.contains("Sum two numbers"));
    assert!(out.contains("result = a + b"));
    assert!(out.contains("return result"));
}
