use rurich::ColorSystem;
use rurich::prelude::*;

#[test]
fn test_table_parity_with_rounded_box() {
    let console = Console::with_dimensions(40, 10).with_color_system(ColorSystem::NoColor);
    let mut table = Table::new().box_style(ROUNDED);
    table.add_column(Column::new().header("Planet"));
    table.add_column(
        Column::new()
            .header("Radius (km)")
            .justify(JustifyMethod::Right),
    );
    table.add_row(["Mercury", "2440"]);
    table.add_row(["Venus", "6052"]);
    table.add_row(["Earth", "6371"]);

    let output = console.capture(|c| {
        c.print(table);
    });

    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "╭─────────┬─────────────╮");
    assert_eq!(lines[1], "│ Planet  │ Radius (km) │");
    assert_eq!(lines[2], "├─────────┼─────────────┤");
    assert_eq!(lines[3], "│ Mercury │        2440 │");
    assert_eq!(lines[4], "│ Venus   │        6052 │");
    assert_eq!(lines[5], "│ Earth   │        6371 │");
    assert_eq!(lines[6], "╰─────────┴─────────────╯");
}

#[test]
fn test_table_parity_with_square_box() {
    let console = Console::with_dimensions(30, 8).with_color_system(ColorSystem::NoColor);
    let mut table = Table::new().box_style(SQUARE);
    table.add_column(Column::new().header("ID").justify(JustifyMethod::Center));
    table.add_column(Column::new().header("Item"));
    table.add_row(["1", "Apple"]);
    table.add_row(["2", "Banana"]);

    let output = console.capture(|c| {
        c.print(table);
    });

    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "┌────┬────────┐");
    assert_eq!(lines[1], "│ ID │ Item   │");
    assert_eq!(lines[2], "├────┼────────┤");
    assert_eq!(lines[3], "│ 1  │ Apple  │");
    assert_eq!(lines[4], "│ 2  │ Banana │");
    assert_eq!(lines[5], "└────┴────────┘");
}

#[test]
fn test_panel_parity_with_title_and_subtitle() {
    let console = Console::with_dimensions(30, 8).with_color_system(ColorSystem::NoColor);
    let panel = Panel::fit("Hello from RuRich!")
        .box_style(ROUNDED)
        .title("Greeting")
        .subtitle("v0.1")
        .padding(0, 1);

    let output = console.capture(|c| {
        c.print(panel);
    });

    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "╭───── Greeting ─────╮");
    assert_eq!(lines[1], "│ Hello from RuRich! │");
    assert_eq!(lines[2], "╰─────── v0.1 ───────╯");
}

#[test]
fn test_layout_split_grid() {
    let console = Console::with_dimensions(30, 4).with_color_system(ColorSystem::NoColor);
    let mut layout = Layout::new();
    layout.split_row(vec![
        Layout::with_renderable(Panel::new("L").box_style(ASCII).padding(0, 0)),
        Layout::with_renderable(Panel::new("R").box_style(ASCII).padding(0, 0)),
    ]);

    let output = console.capture(|c| {
        c.print(layout);
    });

    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(lines[0].starts_with("+-------------++-------------+"));
    assert!(lines[1].contains("|L"));
    assert!(lines[1].contains("|R"));
}
