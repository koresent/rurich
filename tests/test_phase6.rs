use rurich::color::ColorSystem;
use rurich::console::Console;
use rurich::panel::Panel;
use rurich::prompt::{Confirm, Prompt};
use rurich::traceback::{StackFrame, Traceback};
use std::io::Cursor;

#[test]
fn test_prompt_validation_and_stream() {
    let prompt: Prompt<String> = Prompt::new("Select cloud")
        .choices(["aws", "gcp", "azure"])
        .default("aws".to_string());

    // Test default when pressing enter
    let mut input = Cursor::new(b"\n");
    let mut output = Vec::new();
    let res = prompt.ask_stream(&mut input, &mut output).unwrap();
    assert_eq!(res, "aws");

    // Test explicit choice
    let mut input_choice = Cursor::new(b"gcp\n");
    let mut output_choice = Vec::new();
    let res_choice = prompt
        .ask_stream(&mut input_choice, &mut output_choice)
        .unwrap();
    assert_eq!(res_choice, "gcp");
}

#[test]
fn test_confirm_stream_responses() {
    let confirm = Confirm::new("Deploy to production?");

    let mut in_yes = Cursor::new(b"yes\n");
    let mut out_yes = Vec::new();
    assert!(confirm.ask_stream(&mut in_yes, &mut out_yes).unwrap());

    let mut in_no = Cursor::new(b"no\n");
    let mut out_no = Vec::new();
    assert!(!confirm.ask_stream(&mut in_no, &mut out_no).unwrap());
}

#[test]
fn test_console_export_html_and_svg() {
    let console = Console::with_dimensions(50, 10).with_color_system(ColorSystem::TrueColor);
    let panel =
        Panel::from_markup("[bold green]Success![/]\n[cyan]All tasks completed[/]").title("Result");

    let html = console.export_html(&panel);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Result"));
    assert!(html.contains("Success!"));
    assert!(html.contains("All tasks completed"));

    let svg = console.export_svg(&panel, "Status Window");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Status Window"));
    assert!(svg.contains("Success!"));
}

#[test]
fn test_traceback_custom_frames() {
    let mut tb = Traceback::new("Unexpected error in worker loop");
    tb.add_frame(StackFrame {
        filename: "src/worker.rs".to_string(),
        lineno: 105,
        function: "process_queue".to_string(),
        code_context: Some(vec![
            (104, "let job = queue.pop();".to_string(), false),
            (105, "job.unwrap().execute();".to_string(), true),
            (106, "println!(\"Job done\");".to_string(), false),
        ]),
    });

    let console = Console::with_dimensions(70, 20).with_color_system(ColorSystem::NoColor);
    let out = console.capture(|c| {
        c.print(tb);
    });

    assert!(out.contains("Traceback (most recent call last)"));
    assert!(out.contains("src/worker.rs:105 in process_queue"));
    assert!(out.contains("> 105 │ job.unwrap().execute();"));
    assert!(out.contains("Unexpected error in worker loop"));
}

#[cfg(feature = "tracing")]
#[test]
fn test_tracing_console_layer() {
    use rurich::logging::ConsoleLayer;
    use tracing_subscriber::prelude::*;

    let console = Console::with_dimensions(80, 10).with_color_system(ColorSystem::NoColor);
    let subscriber = tracing_subscriber::registry().with(ConsoleLayer::new().with_console(console));

    tracing::subscriber::with_default(subscriber, || {
        tracing::info!("Tracing test message with value={}", 42);
        tracing::warn!("Warning occurred!");
    });
}
