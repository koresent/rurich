use rurich::prelude::*;
use rurich::{Color, ColorSystem, Style, escape, parse_markup};

#[test]
fn test_markup_rendering_and_ansi() {
    let console = Console::with_dimensions(80, 24).with_color_system(ColorSystem::Standard);
    let output = console.capture(|c| {
        c.println_markup("[bold red]ERROR:[/] [yellow]Database connection failed[/yellow]");
    });

    assert_eq!(
        output,
        "\x1b[1;31mERROR:\x1b[0m \x1b[33mDatabase connection failed\x1b[0m\n"
    );
}

#[test]
fn test_markup_escaped_brackets() {
    let console = Console::with_dimensions(80, 24).with_color_system(ColorSystem::NoColor);
    let output = console.capture(|c| {
        c.println_markup(r"Matrix: \[0, 1, 2] and [bold]val[/bold]");
    });

    assert_eq!(output, "Matrix: [0, 1, 2] and val\n");
}

#[test]
fn test_markup_nested_tags() {
    let console = Console::with_dimensions(80, 24).with_color_system(ColorSystem::Standard);
    let output = console.capture(|c| {
        c.println_markup("[bold]Level 1 [italic]Level 2[/italic] Back to 1[/bold]");
    });

    // In Level 2, both bold and italic are active
    assert_eq!(
        output,
        "\x1b[1mLevel 1 \x1b[0m\x1b[1;3mLevel 2\x1b[0m\x1b[1m Back to 1\x1b[0m\n"
    );
}

#[test]
fn test_style_builder_and_text_composition() {
    let console = Console::with_dimensions(40, 10).with_color_system(ColorSystem::TrueColor);
    let mut text = Text::new();
    text.append(
        "Rust",
        Some(Style::new().bold().color(Color::from_rgb(222, 165, 132))),
    );
    text.append(" is ", None);
    text.append(
        "fast!",
        Some(Style::new().italic().color(Color::from_rgb(0, 255, 0))),
    );

    let output = console.capture(|c| {
        c.println(text);
    });

    assert!(output.contains("\x1b[1;38;2;222;165;132mRust\x1b[0m"));
    assert!(output.contains("\x1b[3;38;2;0;255;0mfast!\x1b[0m"));
}

#[test]
fn test_escape_function() {
    assert_eq!(escape("item[0]"), "item[0]");
    assert_eq!(escape("item[bold]"), "item\\[bold]");
    assert_eq!(escape(r"C:\"), r"C:\\");
}

#[test]
fn test_rule_formatting() {
    let console = Console::with_dimensions(30, 5).with_color_system(ColorSystem::NoColor);
    let output = console.capture(|c| {
        c.rule(Some("System"));
    });

    // Width is 30, "System" is 6, padding is 2 -> 30 - 6 - 2 = 22 -> 11 on left, 11 on right
    assert_eq!(output, "─────────── System ───────────\n");
}

#[test]
fn test_word_wrapping_with_spans() {
    let console = Console::with_dimensions(15, 10).with_color_system(ColorSystem::NoColor);
    let text = parse_markup("[bold]The quick brown[/] fox jumps over the lazy dog").unwrap();

    let output = console.capture(|c| {
        c.println(text);
    });

    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "The quick brown");
    assert_eq!(lines[1], "fox jumps over");
    assert_eq!(lines[2], "the lazy dog");
}
