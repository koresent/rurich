use rurich::prelude::*;
use rurich::traceback::StackFrame;

#[derive(Debug)]
#[allow(dead_code)]
struct ServerConfig {
    host: String,
    port: u16,
    max_connections: usize,
    tls_enabled: bool,
    endpoints: Vec<String>,
}

fn main() {
    let console = Console::new();

    console.rule(Some("RuRich: Native Rust Terminal Showcase"));

    rprintln!(
        "[bold green]Welcome to RuRich![/] [italic white]Pure native Rust terminal formatting.[/]"
    );
    rprintln!();

    // 1. Table Showcase
    rprintln!("[bold underline]1. Table with Column Negotiation & Alignment:[/]");
    let mut table = Table::new()
        .box_style(ROUNDED)
        .title("[bold cyan]System Resources Report[/]");
    table.add_column(Column::new().header("Service").style(Style::new().bold()));
    table.add_column(
        Column::new()
            .header("Status")
            .justify(JustifyMethod::Center),
    );
    table.add_column(Column::new().header("CPU %").justify(JustifyMethod::Right));
    table.add_column(Column::new().header("Memory").justify(JustifyMethod::Right));

    table.add_row(["web-server-01", "[bold green]HEALTHY[/]", "12.4%", "256 MB"]);
    table.add_row(["db-replica-02", "[bold green]HEALTHY[/]", "45.1%", "1.2 GB"]);
    table.add_row([
        "cache-redis-01",
        "[bold yellow]DEGRADED[/]",
        "89.7%",
        "4.8 GB",
    ]);
    table.add_row(["auth-service-04", "[bold red]OFFLINE[/]", "0.0%", "0 MB"]);
    console.println(table);
    rprintln!();

    // 2. Panel Showcase
    rprintln!("[bold underline]2. Bordered Panel with Custom Title & Subtitle:[/]");
    let panel = Panel::from_markup(
        "[bold white]Deployment v2.4.0 completed successfully![/]\n\
         All 16 cluster nodes responded within [green]12ms[/].\n\
         Zero downtime deployment achieved via rolling updates.",
    )
    .box_style(ROUNDED)
    .title("[bold green] Deployment Notice [/]")
    .subtitle("[dim] Region: eu-central-1 [/]")
    .border_style(Style::parse("bright_blue").unwrap())
    .padding(1, 2);

    console.println(panel);
    rprintln!();

    // 3. Tree Hierarchy
    rprintln!("[bold underline]3. Hierarchical Tree Rendering:[/]");
    let mut tree = Tree::from_markup("[bold green]Project Root[/]");
    let src = tree.add_markup("[bold yellow]src/[/]");
    src.add_markup("[cyan]lib.rs[/]");
    src.add_markup("[cyan]console.rs[/]");
    src.add_markup("[cyan]table.rs[/]");
    let tests = tree.add_markup("[bold yellow]tests/[/]");
    tests.add_markup("[cyan]test_phase1.rs[/]");
    tests.add_markup("[cyan]test_parity.rs[/]");
    tree.add_markup("[dim]Cargo.toml[/]");
    tree.add_markup("[dim]README.md[/]");
    console.println(tree);
    rprintln!();

    // 4. Markdown Rendering
    #[cfg(feature = "markdown")]
    {
        rprintln!("[bold underline]4. CommonMark Markdown Renderer:[/]");
        let md_text = r#"## RuRich Markdown Feature Set
RuRich supports **bold text**, *italics*, and `inline code`.

> Beautiful, modern, zero-python terminal formatting for Rust!

- First-class support for tables and trees
- High performance, pure native Rust
"#;
        console.println(Markdown::new(md_text));
        rprintln!();
    }

    // 5. JSON Syntax Highlighting
    #[cfg(feature = "json")]
    {
        rprintln!("[bold underline]5. Formatted & Colored JSON Output:[/]");
        let sample_json =
            r#"{"name": "rurich", "version": "0.1.0", "active": true, "stars": 1280}"#;
        if let Ok(json) = Json::from_str(sample_json) {
            console.println(json);
        }
        rprintln!();
    }

    // 6. Syntax-highlighted Source Code
    #[cfg(feature = "syntax")]
    {
        rprintln!("[bold underline]6. Syntax Highlighting with Line Numbers & Range:[/]");
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

    // 7. Pretty Debug Representation
    rprintln!("[bold underline]7. Pretty Data Structure Inspector:[/]");
    let config = ServerConfig {
        host: "0.0.0.0".into(),
        port: 8443,
        max_connections: 10_000,
        tls_enabled: true,
        endpoints: vec!["/api/v1/health".into(), "/api/v1/metrics".into()],
    };
    console.println(Pretty::new(&config));
    rprintln!();

    // 8. Traceback Diagnostic
    rprintln!("[bold underline]8. Traceback Error Diagnostic:[/]");
    let mut tb = Traceback::new("Failed to connect to primary database replica");
    tb.add_frame(StackFrame {
        filename: "src/db/connection.rs".to_string(),
        lineno: 88,
        function: "connect_with_retry".to_string(),
        code_context: Some(vec![
            (
                87,
                "let pool = create_pool(&config).await?;".to_string(),
                false,
            ),
            (
                88,
                "pool.acquire().await.expect(\"Connection timeout\");".to_string(),
                true,
            ),
            (89, "Ok(pool)".to_string(), false),
        ]),
    });
    console.println(tb);
    rprintln!();

    console.rule(Some("End of RuRich Showcase"));
}
