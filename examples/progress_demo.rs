use rurich::prelude::*;
use rurich::{Live, Progress, Status};
use std::thread;
use std::time::Duration;

fn main() {
    let console = Console::new();
    console.rule(Some("RuRich: Phase 4 Dynamic Terminal Updates"));

    // 1. Status Spinner Demo
    rprintln!("[bold cyan]1. Status Spinner Demo:[/]");
    let status = Status::new("Initializing services...");
    let live_status = Live::new(status.clone()).transient(true);
    live_status.start();

    thread::sleep(Duration::from_millis(600));
    status.update("Connecting to database cluster...");
    thread::sleep(Duration::from_millis(600));
    status.update("Loading schemas and configurations...");
    thread::sleep(Duration::from_millis(600));
    live_status.stop();
    rprintln!("[bold green]✓[/] Services initialized successfully!\n");

    // 2. Multi-Task Progress Bar Demo
    rprintln!("[bold cyan]2. Multi-Task Progress Bar Demo:[/]");
    let progress = Progress::new();
    let task_download = progress.add_task("[bold]Downloading assets[/]", 100);
    let task_compile = progress.add_task("[bold]Compiling modules[/]", 100);
    let task_optimize = progress.add_task("[bold]Optimizing binaries[/]", 100);

    let live_progress = Live::new(progress.clone());
    live_progress.start();

    for i in 1..=100 {
        progress.advance(task_download, 2);
        if i > 20 {
            progress.advance(task_compile, 1);
        }
        if i > 50 {
            progress.advance(task_optimize, 3);
        }
        thread::sleep(Duration::from_millis(30));
    }

    live_progress.stop();
    rprintln!("[bold green]✓[/] All tasks finished!");

    console.rule(Some("End of Phase 4 Demo"));
}
