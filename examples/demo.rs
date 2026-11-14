use rurich::badge::{Badge, BadgeStyle, Breadcrumbs};
use rurich::callout::Callout;
use rurich::interactive::{
    Choose, ConfirmDialog, Filter, Input, MultiChoose, RawGuard, redraw_view,
};
use rurich::join::{AlignJoin, join_horizontal, join_vertical};
use rurich::prelude::*;
use rurich::sparkline::Sparkline;
use rurich::spinner_bar::{ProgressBar, Spinner, SpinnerPreset};
use rurich::traceback::StackFrame;
use std::io::IsTerminal;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
#[allow(dead_code)]
struct ClusterNode {
    id: String,
    role: String,
    cores: usize,
    memory_gb: f64,
    tags: Vec<String>,
}

fn main() {
    let console = Console::new();

    // 1. Welcome Banner with TrueColor Rainbow Gradient
    let mut banner = Text::from_plain("RuRich: Complete Native Rust Terminal Showcase");
    banner.rainbow();
    console.println(
        Panel::new(banner)
            .box_style(ROUNDED)
            .margin(1, 2)
            .padding(1, 4),
    );

    rprintln!(
        "[bold green]Welcome![/] [italic white]Comprehensive showcase of all formatting, layout, visual sugar, and TUI capabilities.[/]\n"
    );

    // 2. Inline Markup Engine & Styles
    console.rule(Some("1. Inline BBCode Markup & Styles"));
    rprintln!("[bold]Attributes:[/] [bold]bold[/], [dim]dim[/], [italic]italic[/], [underline]underline[/], [double_underline]double[/], [strike]strike[/], [reverse]reverse[/]");
    rprintln!("[bold]TrueColor RGB:[/] [#ff5500]Hex colors[/], [rgb(0,200,255)]RGB triplets[/], [bold white on blue] Backgrounds [/]");
    rprintln!("[bold]OSC 8 Hyperlinks:[/] [link=https://github.com]Clickable Link[/]");

    let mut grad_sample = Text::from_plain("Smooth gradient text interpolation across TrueColor spectrum");
    grad_sample.gradient(Color::from_rgb(255, 50, 100), Color::from_rgb(50, 200, 255));
    console.println(grad_sample);
    rprintln!();

    // 3. Tables with Auto-Negotiation & Alignment
    console.rule(Some("2. Tables with Column Negotiation & Alignment"));
    let mut table = Table::new()
        .box_style(ROUNDED)
        .title("[bold cyan]Microservices Cluster Metrics[/]");
    table.add_column(Column::new().header("Service").style(Style::new().bold()));
    table.add_column(Column::new().header("Status").justify(JustifyMethod::Center));
    table.add_column(Column::new().header("CPU %").justify(JustifyMethod::Right));
    table.add_column(Column::new().header("Memory").justify(JustifyMethod::Right));
    table.add_column(Column::new().header("Latency").justify(JustifyMethod::Right));

    table.add_row(["auth-api", "[bold green]ONLINE[/]", "14.2%", "340 MB", "12ms"]);
    table.add_row(["db-primary", "[bold green]ONLINE[/]", "48.7%", "2.4 GB", "2ms"]);
    table.add_row(["cache-redis", "[bold yellow]DEGRADED[/]", "88.1%", "6.1 GB", "45ms"]);
    table.add_row(["indexer-01", "[bold red]OFFLINE[/]", "0.0%", "0 MB", "—"]);
    console.println(table);
    rprintln!();

    // 4. Panels, Box Styles & Per-Edge Borders
    console.rule(Some("3. Panels, Box Styles & Per-Edge Borders"));
    let panel = Panel::from_markup(
        "[bold white]Deployment v3.0.0 completed successfully![/]\n\
         All 24 cluster nodes responded within [green]15ms[/].\n\
         Rolling update executed with zero dropped requests.",
    )
    .box_style(ROUNDED)
    .title("[bold green] Production Release [/]")
    .subtitle("[dim] Region: eu-central-1 [/]")
    .border_style(Style::parse("bright_blue").unwrap())
    .padding(1, 2);
    console.println(panel);

    // Left accent quote panel
    let quote = Panel::from_markup("[italic cyan]\"Pure native Rust terminal formatting with complete layout controls.\"[/]")
        .box_style(HEAVY)
        .left_accent()
        .border_style(Style::parse("bold cyan").unwrap())
        .padding(0, 1);
    console.println(quote);
    rprintln!();

    // 5. Layout Joining (Horizontal & Vertical)
    console.rule(Some("4. Layout Composition & Joining"));
    let p_left = Panel::new("CPU: 32 Cores\nRAM: 128 GB\nDisk: 2 TB NVMe")
        .box_style(SQUARE)
        .title("Hardware")
        .padding(0, 1);
    let p_right = Panel::new("OS: Linux 6.8\nArch: x86_64\nRust: 1.84.0")
        .box_style(SQUARE)
        .title("Environment")
        .padding(0, 1);
    let joined = join_horizontal(AlignJoin::Top, &[&p_left, &p_right]);
    let p_net = Panel::new("Network: 10 Gbps Ethernet (Bonded dual-port)")
        .box_style(ROUNDED)
        .padding(0, 1);
    let joined_all = join_vertical(AlignJoin::Left, &[&joined, &p_net]);
    console.println(joined_all);
    rprintln!();

    // 6. Badges and Breadcrumbs
    console.rule(Some("5. Status Badges & Navigation Breadcrumbs"));
    let b_ready = Badge::success("HEALTHY").badge_style(BadgeStyle::Square);
    let b_alert = Badge::error("CRITICAL").badge_style(BadgeStyle::Rounded);
    let b_info = Badge::info("SYNCED").badge_style(BadgeStyle::Minimal);
    let badges = join_horizontal(AlignJoin::Center, &[&b_ready, &b_alert, &b_info]);
    console.println(badges);

    let bc = Breadcrumbs::new(vec!["infrastructure", "europe", "production", "primary-db"]);
    console.println(bc);
    rprintln!();

    // 7. Callout Admonitions
    console.rule(Some("6. Callout Admonition Blocks"));
    let tip = Callout::tip("Use release builds with LTO enabled for maximum throughput.");
    let warn = Callout::warning("Cluster memory pressure above 85% on node-04.");
    console.println(tip);
    console.println(warn);
    rprintln!();

    // 8. Sparkline Mini-Charts
    console.rule(Some("7. High-Resolution Sparkline Mini-Charts"));
    let spark = Sparkline::new(vec![1.0, 3.2, 5.0, 8.4, 9.8, 6.1, 4.0, 7.5, 10.0])
        .style(Style::parse("bold cyan").unwrap());
    console.print_markup("[bold]Real-Time Traffic (req/s): [/]");
    console.println(spark);
    rprintln!();

    // 9. Hierarchical Tree Rendering
    console.rule(Some("8. Hierarchical Tree Structure"));
    let mut tree = Tree::from_markup("[bold green]Project Workspace[/]");
    let src = tree.add_markup("[bold yellow]src/[/]");
    src.add_markup("[cyan]lib.rs[/]");
    src.add_markup("[cyan]console.rs[/]");
    src.add_markup("[cyan]interactive/[/]");
    let tests = tree.add_markup("[bold yellow]tests/[/]");
    tests.add_markup("[cyan]test_visual_sugar.rs[/]");
    tree.add_markup("[dim]Cargo.toml[/]");
    tree.add_markup("[dim]README.md[/]");
    console.println(tree);
    rprintln!();

    // 10. CommonMark Markdown
    #[cfg(feature = "markdown")]
    {
        console.rule(Some("9. CommonMark Markdown Renderer"));
        let md_text = r#"## Native Markdown Rendering
Supports **bold**, *italics*, and `inline code`.

> High performance formatting directly in the terminal!

- Tables and trees
- Progress bars and status spinners
"#;
        console.println(Markdown::new(md_text));
        rprintln!();
    }

    // 11. Formatted & Colored JSON
    #[cfg(feature = "json")]
    {
        console.rule(Some("10. Formatted & Highlighted JSON"));
        let sample_json = r#"{"service": "rurich", "version": "0.1.0", "active": true, "nodes": 16}"#;
        if let Ok(json) = Json::from_str(sample_json) {
            console.println(json);
        }
        rprintln!();
    }

    // 12. Syntax Highlighting
    #[cfg(feature = "syntax")]
    {
        console.rule(Some("11. Source Code Syntax Highlighting"));
        let rust_code = r#"fn compute_hash(seed: u64, salt: &[u8]) -> u64 {
    let mut state = seed;
    for &byte in salt {
        state = state.wrapping_mul(31).wrapping_add(byte as u64);
    }
    state
}"#;
        let syntax = Syntax::new(rust_code, "rust")
            .line_numbers(true)
            .highlight_line(3);
        console.println(syntax);
        rprintln!();
    }

    // 13. Pretty Debug Inspector
    console.rule(Some("12. Pretty Debug Data Structure Inspector"));
    let node = ClusterNode {
        id: "node-fra-01".into(),
        role: "primary-coordinator".into(),
        cores: 64,
        memory_gb: 256.0,
        tags: vec!["production".into(), "nvme".into(), "tier-1".into()],
    };
    console.println(Pretty::new(&node));
    rprintln!();

    // 14. Diagnostic Traceback
    console.rule(Some("13. Diagnostic Traceback Error Formatter"));
    let mut tb = Traceback::new("Failed to connect to primary replica");
    tb.add_frame(StackFrame {
        filename: "src/cluster/connection.rs".to_string(),
        lineno: 88,
        function: "connect_with_retry".to_string(),
        code_context: Some(vec![
            (87, "let pool = create_pool(&config).await?;".to_string(), false),
            (88, "pool.acquire().await.expect(\"Connection timeout\");".to_string(), true),
            (89, "Ok(pool)".to_string(), false),
        ]),
    });
    console.println(tb);
    rprintln!();

    // 14. Live Progress Bar Styles, Glyphs & Variations Gallery
    console.rule(Some("14. Live Progress Bar Styles, Glyphs & Variations Gallery"));
    let is_tty = std::io::stdin().is_terminal();

    let mut pb_line = ProgressBar::new(100).as_line().width(34);
    let mut pb_smooth = ProgressBar::new(100).smooth(true).width(34)
        .complete_style(Style::parse("bold cyan").unwrap())
        .remaining_style(Style::parse("dim").unwrap());
    let mut pb_emerald = ProgressBar::new(100).smooth(true).width(34)
        .complete_style(Style::parse("bold green").unwrap())
        .remaining_style(Style::parse("dim green").unwrap());
    let mut pb_discrete = ProgressBar::new(100).smooth(false).width(34)
        .complete_style(Style::parse("bold bright_blue").unwrap())
        .remaining_style(Style::parse("dim").unwrap());
    let mut pb_squares = ProgressBar::new(100).width(34)
        .complete_char('■')
        .remaining_char('□')
        .complete_style(Style::parse("bold magenta").unwrap())
        .remaining_style(Style::parse("dim").unwrap());
    let mut pb_circles = ProgressBar::new(100).width(34)
        .complete_char('●')
        .remaining_char('○')
        .complete_style(Style::parse("bold bright_cyan").unwrap())
        .remaining_style(Style::parse("dim").unwrap());
    let mut pb_stars = ProgressBar::new(100).width(34)
        .complete_char('★')
        .remaining_char('☆')
        .complete_style(Style::parse("bold yellow").unwrap())
        .remaining_style(Style::parse("dim").unwrap());
    let mut pb_ascii = ProgressBar::new(100).width(34)
        .complete_char('=')
        .remaining_char('-')
        .complete_style(Style::parse("bold yellow").unwrap())
        .remaining_style(Style::parse("dim").unwrap());
    let mut pb_hashes = ProgressBar::new(100).width(34)
        .complete_char('#')
        .remaining_char('.')
        .complete_style(Style::parse("bold bright_red").unwrap())
        .remaining_style(Style::parse("dim").unwrap());
    let mut pb_matrix = ProgressBar::new(100).width(34)
        .complete_char('/')
        .remaining_char('░')
        .complete_style(Style::parse("bold green").unwrap())
        .remaining_style(Style::parse("dim green").unwrap());
    let mut pb_compact = ProgressBar::new(100).smooth(true).width(14)
        .complete_style(Style::parse("bold white on red").unwrap())
        .remaining_style(Style::parse("dim").unwrap());
    let mut pb_wide = ProgressBar::new(100).smooth(true).width(44)
        .complete_style(Style::parse("bold bright_magenta").unwrap())
        .remaining_style(Style::parse("dim").unwrap());
    let pb_pulse = ProgressBar::new(100).pulse(true).width(34)
        .pulse_style(Style::parse("color(#f92672)").unwrap());
    let async_spinner = Spinner::from_preset(SpinnerPreset::Dots);

    let render_all_bars = |pb_line: &ProgressBar,
                           pb_smooth: &ProgressBar,
                           pb_emerald: &ProgressBar,
                           pb_discrete: &ProgressBar,
                           pb_squares: &ProgressBar,
                           pb_circles: &ProgressBar,
                           pb_stars: &ProgressBar,
                           pb_ascii: &ProgressBar,
                           pb_hashes: &ProgressBar,
                           pb_matrix: &ProgressBar,
                           pb_compact: &ProgressBar,
                           pb_wide: &ProgressBar,
                           pb_pulse: &ProgressBar,
                           async_spinner: &Spinner| -> String {
        let frame_console = Console::new();
        frame_console.capture(|c| {
            c.print_markup("  [bold]Signature Thin Line:[/]        ");
            c.print(pb_line);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_line.completed));

            c.print_markup("  [bold]Smooth 1/8 Fractional:[/]      ");
            c.print(pb_smooth);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_smooth.completed));

            c.print_markup("  [bold]Smooth Emerald:[/]             ");
            c.print(pb_emerald);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_emerald.completed));

            c.print_markup("  [bold]Classic Full Blocks:[/]        ");
            c.print(pb_discrete);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_discrete.completed));

            c.print_markup("  [bold]Geometric Squares (■/□):[/]    ");
            c.print(pb_squares);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_squares.completed));

            c.print_markup("  [bold]Bullet Circles (●/○):[/]       ");
            c.print(pb_circles);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_circles.completed));

            c.print_markup("  [bold]Rating Stars (★/☆):[/]         ");
            c.print(pb_stars);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_stars.completed));

            c.print_markup("  [bold]ASCII Custom (=/-):[/]         ");
            c.print(pb_ascii);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_ascii.completed));

            c.print_markup("  [bold]ASCII Hashes (#/.):[/]         ");
            c.print(pb_hashes);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_hashes.completed));

            c.print_markup("  [bold]Matrix Stream (/ / ░):[/]      ");
            c.print(pb_matrix);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_matrix.completed));

            c.print_markup("  [bold]Compact Micro-Bar (w:14):[/]   ");
            c.print(pb_compact);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_compact.completed));

            c.print_markup("  [bold]Panoramic Wide Bar (w:44):[/]  ");
            c.print(pb_wide);
            c.print_markup(&format!(" [bold]{:>3}%[/]\n", pb_wide.completed));

            c.print_markup("  [bold]Pulse Shimmer Wave:[/]         ");
            c.println(pb_pulse);

            let s_frame = async_spinner.get_frame();
            c.print_markup(&format!("  [bold]Async DB Lock (Waiting):[/]     [bold cyan]{s_frame}[/] [dim](0% - Spinner runs independently)[/]\n\n"));

            if is_tty {
                c.print_markup("  [bold cyan]● ALL 14 PROGRESS BAR STYLES ANIMATING LIVE[/] (12.5 FPS)\n  [bold green]Press [Space] or [Enter] to continue to live spinners (or 'q' to skip)...[/]");
            }
        })
    };

    if is_tty {
        let _guard = RawGuard::new().ok();
        let mut stdout = std::io::stdout();
        let mut printed_lines = 0;
        let mut step = 0;

        loop {
            pb_line.completed = (step * 2) % 101;
            pb_smooth.completed = (step + 20) % 101;
            pb_emerald.completed = (step + 50) % 101;
            pb_discrete.completed = (step + 10) % 101;
            pb_squares.completed = (step + 30) % 101;
            pb_circles.completed = (step + 65) % 101;
            pb_stars.completed = (step + 80) % 101;
            pb_ascii.completed = (step + 15) % 101;
            pb_hashes.completed = (step + 40) % 101;
            pb_matrix.completed = (step + 5) % 101;
            pb_compact.completed = (step * 3) % 101;
            pb_wide.completed = (step + 25) % 101;

            let view = render_all_bars(
                &pb_line,
                &pb_smooth,
                &pb_emerald,
                &pb_discrete,
                &pb_squares,
                &pb_circles,
                &pb_stars,
                &pb_ascii,
                &pb_hashes,
                &pb_matrix,
                &pb_compact,
                &pb_wide,
                &pb_pulse,
                &async_spinner,
            );
            let _ = redraw_view(&mut stdout, &view, &mut printed_lines);

            step += 1;

            if crossterm::event::poll(Duration::from_millis(80)).unwrap_or(false)
                && let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read()
            {
                match key.code {
                    crossterm::event::KeyCode::Char(' ')
                    | crossterm::event::KeyCode::Enter
                    | crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Char('Q')
                    | crossterm::event::KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    } else {
        pb_line.completed = 60;
        pb_smooth.completed = 72;
        pb_emerald.completed = 88;
        pb_discrete.completed = 55;
        pb_squares.completed = 68;
        pb_circles.completed = 45;
        pb_stars.completed = 80;
        pb_ascii.completed = 50;
        pb_hashes.completed = 75;
        pb_matrix.completed = 35;
        pb_compact.completed = 92;
        pb_wide.completed = 64;
        let view = render_all_bars(
            &pb_line,
            &pb_smooth,
            &pb_emerald,
            &pb_discrete,
            &pb_squares,
            &pb_circles,
            &pb_stars,
            &pb_ascii,
            &pb_hashes,
            &pb_matrix,
            &pb_compact,
            &pb_wide,
            &pb_pulse,
            &async_spinner,
        );
        print!("{view}");
    }
    rprintln!();

    // 15. Live Named Spinners Catalog
    console.rule(Some("15. Live Named Spinners Catalog (18 Presets)"));
    let spinners: Vec<(&'static str, Spinner)> = vec![
        ("dots", Spinner::from_preset(SpinnerPreset::Dots)),
        ("dots2", Spinner::from_preset(SpinnerPreset::Dots2)),
        ("line", Spinner::from_preset(SpinnerPreset::Line)),
        ("moon", Spinner::from_preset(SpinnerPreset::Moon)),
        ("clock", Spinner::from_preset(SpinnerPreset::Clock)),
        ("globe", Spinner::from_preset(SpinnerPreset::Globe)),
        ("arc", Spinner::from_preset(SpinnerPreset::Arc)),
        ("pulse", Spinner::from_preset(SpinnerPreset::Pulse)),
        ("points", Spinner::from_preset(SpinnerPreset::Points)),
        ("jump", Spinner::from_preset(SpinnerPreset::Jump)),
        ("bouncing_bar", Spinner::from_preset(SpinnerPreset::BouncingBar)),
        ("grow_vertical", Spinner::from_preset(SpinnerPreset::GrowVertical)),
        ("hamburger", Spinner::from_preset(SpinnerPreset::Hamburger)),
        ("arrow", Spinner::from_preset(SpinnerPreset::Arrow)),
        ("monkey", Spinner::from_preset(SpinnerPreset::Monkey)),
        ("square_corners", Spinner::from_preset(SpinnerPreset::SquareCorners)),
        ("circle_halves", Spinner::from_preset(SpinnerPreset::CircleHalves)),
        ("triangle", Spinner::from_preset(SpinnerPreset::Triangle)),
    ];

    let render_spinners_table = |spinners: &[(&'static str, Spinner)]| -> String {
        let frame_console = Console::new();
        frame_console.capture(|c| {
            let mut spinner_table = Table::new().box_style(ROUNDED);
            spinner_table.add_column(Column::new().header("Preset").style(Style::new().bold()));
            spinner_table.add_column(Column::new().header("Live Frame").justify(JustifyMethod::Center));
            spinner_table.add_column(Column::new().header("Preset").style(Style::new().bold()));
            spinner_table.add_column(Column::new().header("Live Frame").justify(JustifyMethod::Center));
            spinner_table.add_column(Column::new().header("Preset").style(Style::new().bold()));
            spinner_table.add_column(Column::new().header("Live Frame").justify(JustifyMethod::Center));

            for chunk in spinners.chunks(3) {
                let r0_name = chunk[0].0;
                let r0_frame = chunk[0].1.get_frame();
                let (r1_name, r1_frame) = chunk.get(1).map(|c| (c.0, c.1.get_frame())).unwrap_or(("", ""));
                let (r2_name, r2_frame) = chunk.get(2).map(|c| (c.0, c.1.get_frame())).unwrap_or(("", ""));
                spinner_table.add_row([r0_name, r0_frame, r1_name, r1_frame, r2_name, r2_frame]);
            }
            c.println(spinner_table);
            if is_tty {
                c.print_markup("  [bold cyan]● ALL 18 SPINNERS ROTATING LIVE[/] (12.5 FPS)\n  [bold green]Press [Space] or [Enter] to continue to interactive TUI playground (or 'q' to quit)...[/]");
            }
        })
    };

    if is_tty {
        let _guard = RawGuard::new().ok();
        let mut stdout = std::io::stdout();
        let mut printed_lines = 0;

        loop {
            let view = render_spinners_table(&spinners);
            let _ = redraw_view(&mut stdout, &view, &mut printed_lines);

            if crossterm::event::poll(Duration::from_millis(80)).unwrap_or(false)
                && let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read()
            {
                match key.code {
                    crossterm::event::KeyCode::Char(' ')
                    | crossterm::event::KeyCode::Enter
                    | crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Char('Q')
                    | crossterm::event::KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    } else {
        let view = render_spinners_table(&spinners);
        print!("{view}");
    }
    rprintln!();

    // 16. Dynamic Multi-Task Background Progress
    console.rule(Some("16. Multi-Task Background Progress Engine (`Progress::start`)"));
    let batch_progress = Progress::new();
    let task_core = batch_progress.add_task("[bold cyan]Downloading binary package[/]", 100);
    let task_compile = batch_progress.add_task("[bold yellow]Compiling crate modules[/]", 100);
    let task_lock = batch_progress.add_task(
        "[bold magenta]Waiting for replica lock (async spinner)[/]",
        100,
    );

    batch_progress.start();

    for i in 1..=25 {
        batch_progress.advance(task_core, 4);
        if i > 5 {
            batch_progress.advance(task_compile, 4);
        }
        if i > 16 {
            batch_progress.advance(task_lock, 12);
        }
        thread::sleep(Duration::from_millis(30));
    }
    batch_progress.stop();
    rprintln!("[bold green]✓[/] Multi-task background progress completed!\n");

    // 17. Automated Iterator Progress Tracking
    console.rule(Some("17. Automated Iterator Progress Tracking (`track`)"));
    for _ in track(0..10, "[bold green]Processing data chunks[/]") {
        thread::sleep(Duration::from_millis(25));
    }
    rprintln!();

    // 18. Interactive Micro-TUI Playground
    console.rule(Some("18. Live Interactive TUI Playground"));
    if std::io::stdin().is_terminal() {
        let mut confirm = ConfirmDialog::new("Would you like to test the interactive TUI components live in this terminal?")
            .yes_label("Yes, launch TUI")
            .no_label("Skip");

        if let Ok(Some(true)) = confirm.run() {
            rprintln!("\n[bold cyan]1. Single-Selection Menu (Choose):[/] (Use Up/Down arrows or j/k, Enter to select)");
            let mut choose = Choose::new(vec![
                "Production Cluster (eu-central-1)",
                "Staging Environment (us-east-1)",
                "Local Development Node",
            ])
            .header("Select deployment target environment:");

            if let Ok(Some(choice)) = choose.run() {
                rprintln!("[bold green]Selected:[/] {}\n", choice[0]);
            }

            rprintln!("[bold cyan]2. Multi-Selection Checkboxes (MultiChoose):[/] (Use Space to toggle, Enter to confirm)");
            let mut multi = MultiChoose::new(vec![
                "TLS 1.3 / HTTPS Encryption",
                "Prometheus Metrics Exporter",
                "OpenTelemetry Distributed Tracing",
                "HTTP/3 QUIC Protocol",
            ])
            .multi(true)
            .header("Select platform features to enable:");

            if let Ok(Some(features)) = multi.run() {
                rprintln!("[bold green]Enabled features:[/] {:?}\n", features);
            }

            rprintln!("[bold cyan]3. Live Fuzzy Finder (Filter):[/] (Type letters to filter with real-time highlighting)");
            let mut filter = Filter::new(vec![
                "auth-service",
                "billing-gateway",
                "user-profile-api",
                "notification-worker",
                "search-indexer",
                "metrics-collector",
                "cache-manager",
            ])
            .prompt("Search microservice: ");

            if let Ok(Some(selected_service)) = filter.run() {
                rprintln!("[bold green]Found service:[/] {}\n", selected_service);
            }

            rprintln!("[bold cyan]4. Interactive Text Input (Input):[/] (Type with live cursor, Enter to submit)");
            let mut input = Input::new()
                .prompt("Enter deployment release tag: ")
                .placeholder("e.g. v3.1.0");

            if let Ok(Some(tag_name)) = input.run() {
                let badge = Badge::success(format!("RELEASED: {tag_name}"));
                console.println(badge);
            }
        } else {
            rprintln!("[dim]Interactive TUI demo skipped.[/]");
        }
    } else {
        rprintln!("[dim]Running in non-interactive environment. Interactive TUI widgets (Choose, Filter, ConfirmDialog, Input, TextArea, Pager, FilePicker) can be run in a real TTY terminal.[/]");
    }

    rprintln!();
    console.rule(Some("End of Complete RuRich Showcase"));
}
