use rurich::prelude::*;
use rurich::spinner_bar::{ProgressBar, Spinner};

#[test]
fn test_spinner() {
    let spinner = Spinner::new(vec!["|", "/", "-", "\\"], 100);
    assert_eq!(spinner.get_frame(), "|");
}

#[test]
fn test_progress_bar() {
    let mut bar = ProgressBar::new(100);
    bar.update(50);

    let console = Console::with_dimensions(20, 1).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| {
        c.print(bar);
    });

    // Width 20, 50% = 10 filled, 10 empty
    assert_eq!(out, "██████████░░░░░░░░░░");
}
