use crossterm::event::{KeyCode, KeyEvent};
use rurich::badge::{Badge, BadgeStyle, Breadcrumbs};
use rurich::callout::Callout;
use rurich::color::ColorSystem;
use rurich::interactive::{
    Choose, ConfirmDialog, FilePicker, Filter, Input, MultiChoose, Pager, TextArea,
};
use rurich::join::{AlignJoin, join_horizontal, join_vertical};
use rurich::prelude::*;
use rurich::sparkline::Sparkline;
use rurich::spinner_bar::{ProgressBar, Spinner};

#[test]
fn test_badge_variants() {
    let console = Console::with_dimensions(40, 5).with_color_system(ColorSystem::NoColor);
    let b1 = Badge::success("ACTIVE").badge_style(BadgeStyle::Square);
    let b2 = Badge::error("FAILED").badge_style(BadgeStyle::Rounded);
    let b3 = Badge::warning("PENDING").badge_style(BadgeStyle::Minimal);

    assert_eq!(console.capture(|c| c.print(b1)).trim(), "[ ACTIVE ]");
    assert_eq!(console.capture(|c| c.print(b2)).trim(), "( FAILED )");
    assert_eq!(console.capture(|c| c.print(b3)).trim(), "PENDING");
}

#[test]
fn test_breadcrumbs_rendering() {
    let console = Console::with_dimensions(50, 5).with_color_system(ColorSystem::NoColor);
    let bc = Breadcrumbs::new(vec!["workspace", "project", "src"]).separator(" ❯ ");
    let out = console.capture(|c| c.print(bc));
    assert_eq!(out.trim(), "workspace ❯ project ❯ src");
}

#[test]
fn test_callout_admonitions() {
    let console = Console::with_dimensions(40, 5).with_color_system(ColorSystem::NoColor);
    let callout = Callout::warning("Be careful with this operation");
    let out = console.capture(|c| c.print(callout));
    assert!(out.contains("Warning"));
    assert!(out.contains("Be careful with this operation"));
    assert!(out.contains("▌"));
}

#[test]
fn test_sparkline_variations() {
    let spark = Sparkline::new(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    assert_eq!(spark.render_string(), "\u{2581}▂▃▄▅▆▇█");

    let spark_ints = Sparkline::from_integers(vec![10, 20, 30])
        .min(10.0)
        .max(30.0);
    assert_eq!(spark_ints.render_string(), "\u{2581}▅█");
}

#[test]
fn test_join_blocks() {
    let console = Console::with_dimensions(40, 10).with_color_system(ColorSystem::NoColor);
    let p1 = Panel::new("Left").padding(0, 1);
    let p2 = Panel::new("Right").padding(0, 1);

    let joined_h = join_horizontal(AlignJoin::Top, &[&p1, &p2]);
    let out_h = console.capture(|c| c.print(joined_h));
    assert!(out_h.contains("Left") && out_h.contains("Right"));

    let joined_v = join_vertical(AlignJoin::Center, &[&p1, &p2]);
    let out_v = console.capture(|c| c.print(joined_v));
    assert!(out_v.contains("Left"));
    assert!(out_v.contains("Right"));
}

#[test]
fn test_panel_sides_and_margin() {
    let console = Console::with_dimensions(30, 8).with_color_system(ColorSystem::NoColor);
    let panel = Panel::new("Notice")
        .horizontal_only()
        .margin(1, 2)
        .padding(0, 1);

    let out = console.capture(|c| c.print(panel));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], ""); // vertical margin
    assert!(lines[1].starts_with("  ─")); // horizontal margin + border
}

#[test]
fn test_spinner_preset_lookup() {
    for name in &["dots", "arc", "clock", "moon", "grow_vertical", "globe"] {
        assert!(Spinner::by_name(name).is_some());
    }
}

#[test]
fn test_progress_bar_pulse_and_smooth() {
    let pb_pulse = ProgressBar::new(100).pulse(true).width(20);
    let console = Console::with_dimensions(30, 3).with_color_system(ColorSystem::NoColor);
    let out_pulse = console.capture(|c| c.print(&pb_pulse));
    assert_eq!(out_pulse.chars().count(), 20);
    assert!(!out_pulse.contains('─'));
    assert!(out_pulse.contains('━'));

    let mut pb_line = ProgressBar::new(100).as_line().width(20);
    pb_line.update(25);
    let out_line = console.capture(|c| c.print(&pb_line));
    assert_eq!(out_line.chars().count(), 20);
    assert!(out_line.contains('━'));
    assert!(!out_line.contains('─'));

    let mut pb_smooth = ProgressBar::new(80).smooth(true);
    pb_smooth.width = 10;
    pb_smooth.update(10); // 1 cell plus remainder
    let out_smooth = console.capture(|c| c.print(&pb_smooth));
    assert!(!out_smooth.is_empty());
}

#[test]
fn test_interactive_choose_and_multichoose() {
    let mut choose = Choose::new(vec!["Dev", "Staging", "Prod"]);
    let res = choose.run_headless(vec![
        KeyEvent::from(KeyCode::Down),
        KeyEvent::from(KeyCode::Enter),
    ]);
    assert_eq!(res, Some(vec!["Staging"]));

    let mut multi = MultiChoose::new(vec!["App", "Db", "Cache"]).multi(true);
    let res_multi = multi.run_headless(vec![
        KeyEvent::from(KeyCode::Char(' ')),
        KeyEvent::from(KeyCode::Down),
        KeyEvent::from(KeyCode::Down),
        KeyEvent::from(KeyCode::Char(' ')),
        KeyEvent::from(KeyCode::Enter),
    ]);
    assert_eq!(res_multi, Some(vec!["App", "Cache"]));
}

#[test]
fn test_interactive_filter_fuzzy() {
    let mut filter = Filter::new(vec![
        "authentication",
        "authorization",
        "database",
        "analytics",
    ]);
    let res = filter.run_headless(vec![
        KeyEvent::from(KeyCode::Char('a')),
        KeyEvent::from(KeyCode::Char('u')),
        KeyEvent::from(KeyCode::Char('t')),
        KeyEvent::from(KeyCode::Char('h')),
        KeyEvent::from(KeyCode::Char('z')),
        KeyEvent::from(KeyCode::Enter),
    ]);
    assert_eq!(res, Some("authorization"));
}

#[test]
fn test_interactive_confirm_dialog() {
    let mut dialog = ConfirmDialog::new("Deploy to production?");
    let res = dialog.run_headless(vec![
        KeyEvent::from(KeyCode::Left),
        KeyEvent::from(KeyCode::Enter),
    ]);
    assert_eq!(res, Some(false));
}

#[test]
fn test_interactive_input_and_textarea() {
    let mut input = Input::new().placeholder("commit message");
    let res = input.run_headless(vec![
        KeyEvent::from(KeyCode::Char('i')),
        KeyEvent::from(KeyCode::Char('n')),
        KeyEvent::from(KeyCode::Char('i')),
        KeyEvent::from(KeyCode::Char('t')),
        KeyEvent::from(KeyCode::Enter),
    ]);
    assert_eq!(res, Some("init".to_string()));

    let mut ta = TextArea::new();
    let res_ta = ta.run_headless(vec![
        KeyEvent::from(KeyCode::Char('H')),
        KeyEvent::from(KeyCode::Char('e')),
        KeyEvent::from(KeyCode::Char('y')),
        KeyEvent::from(KeyCode::Enter),
        KeyEvent::from(KeyCode::Char('!')),
        KeyEvent::from(KeyCode::Esc),
    ]);
    assert_eq!(res_ta, Some("Hey\n!".to_string()));
}

#[test]
fn test_interactive_pager_navigation() {
    let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
    let mut pager = Pager::new(content).height(3);
    pager.run_headless(vec![
        KeyEvent::from(KeyCode::Down),
        KeyEvent::from(KeyCode::Char('q')),
    ]);
    assert_eq!(pager.top_line, 1);
}

#[test]
fn test_interactive_file_picker_navigation() {
    let mut picker = FilePicker::new(".").expect("valid cwd");
    let res = picker.run_headless(vec![KeyEvent::from(KeyCode::Down)]);
    assert_eq!(res, None); // didn't press enter on file, just moved cursor
}

#[test]
fn test_spinner_column_and_progress_lifecycle() {
    let progress = Progress::new();
    let t1 = progress.add_task("[bold]Test Task[/]", 100);
    progress.advance(t1, 100);

    let console = Console::with_dimensions(60, 4).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| c.print(&progress));
    assert!(out.contains("Test Task"));
    assert!(out.contains("100%"));
    assert!(out.contains("✓")); // checkmark upon completion

    // Progress start/stop without panic
    progress.start();
    progress.stop();
}
