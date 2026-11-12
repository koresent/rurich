use rurich::ansi::AnsiDecoder;
use rurich::color::ColorSystem;
use rurich::console::Console;
use rurich::constrain::Constrain;
use rurich::emoji::Emoji;
use rurich::filesize::{binary as format_binary_size, decimal as format_decimal_size};
use rurich::group::Group;
use rurich::panel::Panel;
use rurich::progress::{
    DownloadColumn, FileSizeColumn, MofNCompleteColumn, Progress, TimeRemainingColumn,
    TotalFileSizeColumn, TransferSpeedColumn,
};
use rurich::rule::Rule;
use rurich::screen::Screen;
use rurich::styled::Styled;
use rurich::text::Text;

#[test]
fn test_group_nesting_in_panel() {
    let mut group = Group::new();
    group.add(Text::from_plain("Status: All green"));
    group.add(Rule::new("Sub-section"));
    group.add(Text::from_plain("Worker threads: 16"));

    let panel = Panel::new(group).title("System Overview");

    let console = Console::with_dimensions(50, 10).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| {
        c.print(panel);
    });

    assert!(out.contains("System Overview"));
    assert!(out.contains("Status: All green"));
    assert!(out.contains("Sub-section"));
    assert!(out.contains("Worker threads: 16"));
}

#[test]
fn test_constrain_renderable() {
    let text =
        Text::from_plain("This line is very long and should be constrained to 20 columns max");
    let constrained = Constrain::new(text, 20);

    let console = Console::with_dimensions(60, 5).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| {
        c.print(constrained);
    });

    for line in out.lines() {
        assert!(rurich::cells::cell_len(line) <= 20);
    }
}

#[test]
fn test_ansi_decoder_to_styled_text() {
    let raw_terminal_out = "\x1b[32mPASS\x1b[0m \x1b[1;34mtest_suite::core\x1b[0m";
    let mut decoder = AnsiDecoder::new();
    let text = decoder.decode(raw_terminal_out);

    assert_eq!(text.plain, "PASS test_suite::core");
    assert_eq!(text.spans.len(), 2);
}

#[test]
fn test_additional_progress_columns() {
    let progress = Progress::with_columns(vec![
        Box::new(rurich::progress::TextColumn),
        Box::new(FileSizeColumn::new()),
        Box::new(TotalFileSizeColumn::new()),
        Box::new(DownloadColumn::new()),
        Box::new(TransferSpeedColumn),
        Box::new(TimeRemainingColumn),
        Box::new(MofNCompleteColumn::new()),
    ]);

    let task_id = progress.add_task("Downloading dataset", 10_000_000);
    progress.advance(task_id, 2_500_000);

    let console = Console::with_dimensions(120, 5).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| {
        c.print(&progress);
    });

    assert!(out.contains("Downloading dataset"));
    assert!(out.contains("2.5 MB"));
    assert!(out.contains("10.0 MB"));
    assert!(out.contains("2.5 MB/10.0 MB"));
}

#[test]
fn test_screen_container() {
    let text = Text::from_plain("Fullscreen Content");
    let screen = Screen::new(text);

    let console = Console::with_dimensions(40, 10).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| {
        c.print(screen);
    });

    assert!(out.contains("Fullscreen Content"));
}

#[test]
fn test_emoji_and_filesize_utilities() {
    let replaced = Emoji::replace("Rocket: :rocket: | Crab: :crab: | Status: :check:");
    assert_eq!(replaced, "Rocket: 🚀 | Crab: 🦀 | Status: ✔");

    assert_eq!(format_decimal_size(1024), "1.0 kB");
    assert_eq!(format_decimal_size(50_000_000), "50.0 MB");
    assert_eq!(format_binary_size(1024), "1.0 KiB");
    assert_eq!(format_binary_size(1024 * 1024 * 64), "64.0 MiB");
}

#[test]
fn test_styled_wrapper() {
    let text = Text::from_plain("Styled Message");
    let styled = Styled::new(text, rurich::style::Style::parse("bold green").unwrap());

    let console = Console::with_dimensions(40, 5).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| {
        c.print(styled);
    });

    assert!(out.contains("Styled Message"));
}
