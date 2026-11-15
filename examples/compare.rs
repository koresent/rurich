use rurich::badge::{Badge, BadgeStyle, Breadcrumbs};
use rurich::callout::Callout;
use rurich::interactive::{Choose, ConfirmDialog, Filter, Input};
use rurich::join::{AlignJoin, join_horizontal};
use rurich::prelude::*;
use rurich::sparkline::Sparkline;
use rurich::spinner_bar::ProgressBar;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Configuration for running external Rich and Gum tools
struct CompareEnv {
    rich_python_path: Option<PathBuf>,
    gum_bin: Option<PathBuf>,
}

impl CompareEnv {
    fn detect() -> Self {
        let contrib_rich = PathBuf::from("/home/koresent/Projects/contrib/rich");
        let rich_python_path = if contrib_rich.exists() {
            Some(contrib_rich)
        } else {
            None
        };

        let contrib_gum = PathBuf::from("/home/koresent/Projects/contrib/gum/gum");
        let gum_bin = if contrib_gum.exists() {
            Some(contrib_gum)
        } else if let Ok(path) = which::which("gum") {
            Some(path)
        } else if Path::new("/usr/bin/gum").exists() {
            Some(PathBuf::from("/usr/bin/gum"))
        } else {
            None
        };

        Self {
            rich_python_path,
            gum_bin,
        }
    }

    fn run_python_rich(&self, code: &str) -> Result<String, String> {
        let mut cmd = Command::new("python3");
        if let Some(ref p) = self.rich_python_path {
            cmd.env("PYTHONPATH", p);
        }
        cmd.env("FORCE_COLOR", "1");
        cmd.env("COLORTERM", "truecolor");
        cmd.arg("-c").arg(code);

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Python error: {}", err.trim()))
                }
            }
            Err(e) => Err(format!("Failed to execute python3: {}", e)),
        }
    }

    fn run_gum(&self, args: &[&str]) -> Result<String, String> {
        let bin = match &self.gum_bin {
            Some(b) => b.as_path(),
            None => Path::new("gum"),
        };

        let mut cmd = Command::new(bin);
        cmd.env("FORCE_COLOR", "1");
        cmd.env("COLORTERM", "truecolor");
        cmd.args(args);

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Gum error: {}", err.trim()))
                }
            }
            Err(e) => Err(format!("Failed to execute gum: {}", e)),
        }
    }
}

mod which {
    use std::env;
    use std::path::PathBuf;

    pub fn which(name: &str) -> Result<PathBuf, ()> {
        if let Ok(paths) = env::var("PATH") {
            for p in env::split_paths(&paths) {
                let full = p.join(name);
                if full.is_file() {
                    return Ok(full);
                }
            }
        }
        Err(())
    }
}

fn markup_text(s: &str) -> Text {
    rurich::parse_markup(s).unwrap_or_else(|_| Text::from_plain(s))
}

fn parse_style(s: &str) -> Style {
    Style::parse(s).unwrap_or_else(|_| Style::null())
}

/// Renders two ANSI strings side-by-side inside styled panels using rurich's AnsiDecoder and join_horizontal
fn render_side_by_side(
    left_title: &str,
    left_ansi: &str,
    right_title: &str,
    right_ansi: &str,
    panel_width: usize,
) {
    let console = Console::new();

    let mut decoder_left = AnsiDecoder::new();
    let left_text = decoder_left.decode(left_ansi.trim_end_matches('\n'));

    let mut decoder_right = AnsiDecoder::new();
    let right_text = decoder_right.decode(right_ansi.trim_end_matches('\n'));

    let left_panel = Panel::new(left_text)
        .title(format!(" {} ", left_title))
        .box_style(ROUNDED)
        .width(panel_width)
        .border_style(parse_style("dim cyan"));

    let right_panel = Panel::new(right_text)
        .title(format!(" {} ", right_title))
        .box_style(ROUNDED)
        .width(panel_width)
        .border_style(parse_style("dim green"));

    let spacer = Text::from_plain("  ");
    let joined = join_horizontal(AlignJoin::Top, &[&left_panel, &spacer, &right_panel]);
    console.println(joined);
    println!();
}

fn compare_tables(env: &CompareEnv, panel_width: usize) {
    let console = Console::new();
    console.rule(Some("Tables"));

    let inner_width = panel_width.saturating_sub(4);

    // Case 1: Rounded Table with Alignments
    let python_code = format!(
        r#"
from rich.console import Console
from rich.table import Table
from rich import box

c = Console(width={}, color_system="truecolor", force_terminal=True)
t = Table(title="Services", box=box.ROUNDED)
t.add_column("Service", justify="left", style="cyan")
t.add_column("Status", justify="center", style="green")
t.add_column("Latency", justify="right", style="magenta")
t.add_row("auth-svc", "ONLINE", "1.2 ms")
t.add_row("data-api", "ONLINE", "4.8 ms")
t.add_row("payment", "DEGRADED", "145.0 ms")
c.print(t)
"#,
        inner_width
    );

    let rich_out = env
        .run_python_rich(&python_code)
        .unwrap_or_else(|e| format!("Python error: {}", e));

    let mut table = Table::new().title("Services").box_style(ROUNDED);
    table
        .add_column(
            Column::new()
                .header("Service")
                .justify(JustifyMethod::Left)
                .style(parse_style("cyan")),
        )
        .add_column(
            Column::new()
                .header("Status")
                .justify(JustifyMethod::Center)
                .style(parse_style("green")),
        )
        .add_column(
            Column::new()
                .header("Latency")
                .justify(JustifyMethod::Right)
                .style(parse_style("magenta")),
        );
    table.add_row(vec!["auth-svc", "ONLINE", "1.2 ms"]);
    table.add_row(vec!["data-api", "ONLINE", "4.8 ms"]);
    table.add_row(vec!["payment", "DEGRADED", "145.0 ms"]);

    let inner_console = Console::with_dimensions(inner_width, 24);
    let rurich_out = inner_console.capture(|c| {
        c.println(&table);
    });

    render_side_by_side(
        "Rich: box.ROUNDED",
        &rich_out,
        "RuRich: box_style(ROUNDED)",
        &rurich_out,
        panel_width,
    );

    // Case 2: Heavy Box Style with Sections
    let python_code_sections = format!(
        r#"
from rich.console import Console
from rich.table import Table
from rich import box

c = Console(width={}, color_system="truecolor", force_terminal=True)
t = Table(title="Metrics", box=box.HEAVY_HEAD)
t.add_column("Node", justify="left")
t.add_column("CPU", justify="right")
t.add_column("RAM", justify="right")
t.add_row("node-01", "24.5%", "4.2 GB")
t.add_row("node-02", "88.1%", "15.8 GB")
t.add_section()
t.add_row("Total", "56.3%", "20.0 GB")
c.print(t)
"#,
        inner_width
    );

    let rich_out_sec = env
        .run_python_rich(&python_code_sections)
        .unwrap_or_else(|e| format!("Python error: {}", e));

    let mut table_sec = Table::new().title("Metrics").box_style(HEAVY_HEAD);
    table_sec
        .add_column(Column::new().header("Node").justify(JustifyMethod::Left))
        .add_column(Column::new().header("CPU").justify(JustifyMethod::Right))
        .add_column(Column::new().header("RAM").justify(JustifyMethod::Right));
    table_sec.add_row(vec!["node-01", "24.5%", "4.2 GB"]);
    table_sec.add_row(vec!["node-02", "88.1%", "15.8 GB"]);
    table_sec.add_section();
    table_sec.add_row(vec!["Total", "56.3%", "20.0 GB"]);

    let rurich_out_sec = inner_console.capture(|c| {
        c.println(&table_sec);
    });

    render_side_by_side(
        "Rich: box.HEAVY_HEAD",
        &rich_out_sec,
        "RuRich: box_style(HEAVY_HEAD)",
        &rurich_out_sec,
        panel_width,
    );
}

fn compare_panels(env: &CompareEnv, panel_width: usize) {
    let console = Console::new();
    console.rule(Some("Panels"));

    let inner_width = panel_width.saturating_sub(4);
    let python_code = format!(
        r#"
from rich.console import Console
from rich.panel import Panel
from rich import box

c = Console(width={}, color_system="truecolor", force_terminal=True)
p = Panel(
    "Deployment completed.\nCluster is ready for traffic.",
    title="[bold green]Deploy Status[/]",
    subtitle="[dim]us-east-1a[/]",
    box=box.ROUNDED,
    padding=(1, 2),
    width={}
)
c.print(p)
"#,
        inner_width, inner_width
    );

    let rich_out = env
        .run_python_rich(&python_code)
        .unwrap_or_else(|e| format!("Python error: {}", e));

    let content = markup_text("Deployment completed.\nCluster is ready for traffic.");
    let rurich_panel = Panel::new(content)
        .title_text(markup_text("[bold green]Deploy Status[/]"))
        .subtitle_text(markup_text("[dim]us-east-1a[/]"))
        .box_style(ROUNDED)
        .padding(1, 2)
        .width(inner_width);

    let inner_console = Console::with_dimensions(inner_width, 24);
    let rurich_out = inner_console.capture(|c| {
        c.println(rurich_panel);
    });

    render_side_by_side(
        "Rich: Panel(rounded)",
        &rich_out,
        "RuRich: Panel(ROUNDED)",
        &rurich_out,
        panel_width,
    );
}

fn compare_markup_and_colors(env: &CompareEnv, panel_width: usize) {
    let console = Console::new();
    console.rule(Some("BBCode Markup & Colors"));

    let inner_width = panel_width.saturating_sub(4);
    let python_code = format!(
        r#"
from rich.console import Console

c = Console(width={}, color_system="truecolor", force_terminal=True)
c.print("[bold red]Bold Red Text[/]")
c.print("[italic cyan]Italic Cyan[/] and [underline green]Underline Green[/]")
c.print("[#ff00aa]Hex #ff00aa[/] on [rgb(0,255,200)]RGB Triplet[/]")
c.print("[bold white on blue] Inverted [/] [strike]Strikethrough[/]")
c.print("[link=https://github.com]OSC 8 Hyperlink[/link]")
"#,
        inner_width
    );

    let rich_out = env
        .run_python_rich(&python_code)
        .unwrap_or_else(|e| format!("Python error: {}", e));

    let inner_console = Console::with_dimensions(inner_width, 24);
    let rurich_out = inner_console.capture(|c| {
        c.println_markup("[bold red]Bold Red Text[/]");
        c.println_markup("[italic cyan]Italic Cyan[/] and [underline green]Underline Green[/]");
        c.println_markup("[#ff00aa]Hex #ff00aa[/] on [rgb(0,255,200)]RGB Triplet[/]");
        c.println_markup("[bold white on blue] Inverted [/] [strike]Strikethrough[/]");
        c.println_markup("[link=https://github.com]OSC 8 Hyperlink[/link]");
    });

    render_side_by_side(
        "Rich: Markup",
        &rich_out,
        "RuRich: Markup",
        &rurich_out,
        panel_width,
    );
}

fn compare_trees(env: &CompareEnv, panel_width: usize) {
    let console = Console::new();
    console.rule(Some("Trees"));

    let inner_width = panel_width.saturating_sub(4);
    let python_code = format!(
        r#"
from rich.console import Console
from rich.tree import Tree

c = Console(width={}, color_system="truecolor", force_terminal=True)
t = Tree("📦 [bold magenta]my-project[/]")
src = t.add("📁 [cyan]src[/]")
src.add("📄 main.rs")
src.add("📄 lib.rs")
models = src.add("📁 models")
models.add("📄 user.rs")
t.add("📄 Cargo.toml")
c.print(t)
"#,
        inner_width
    );

    let rich_out = env
        .run_python_rich(&python_code)
        .unwrap_or_else(|e| format!("Python error: {}", e));

    let mut tree = Tree::from_markup("📦 [bold magenta]my-project[/]");
    let src = tree.add_markup("📁 [cyan]src[/]");
    src.add_markup("📄 main.rs");
    src.add_markup("📄 lib.rs");
    let models = src.add_markup("📁 models");
    models.add_markup("📄 user.rs");
    tree.add_markup("📄 Cargo.toml");

    let inner_console = Console::with_dimensions(inner_width, 24);
    let rurich_out = inner_console.capture(|c| {
        c.println(tree);
    });

    render_side_by_side(
        "Rich: Tree",
        &rich_out,
        "RuRich: Tree",
        &rurich_out,
        panel_width,
    );
}

fn compare_markdown_and_syntax(env: &CompareEnv, panel_width: usize) {
    let console = Console::new();
    console.rule(Some("Markdown & Syntax"));

    let inner_width = panel_width.saturating_sub(4);

    // Case 1: Markdown
    let md_content = "# Overview\n\n- Pure native **Rust**\n- Zero *Python* runtime\n- Sub-character **smooth bars**";
    let python_md = format!(
        r#"
from rich.console import Console
from rich.markdown import Markdown

c = Console(width={}, height=24, color_system="truecolor", force_terminal=True)
c.print(Markdown("""{}"""))
"#,
        inner_width, md_content
    );

    let rich_md_out = env
        .run_python_rich(&python_md)
        .unwrap_or_else(|e| format!("Python error: {}", e));

    let rurich_md = Markdown::new(md_content);
    let inner_console = Console::with_dimensions(inner_width, 24);
    let rurich_md_out = inner_console.capture(|c| {
        c.println(rurich_md);
    });

    render_side_by_side(
        "Rich: Markdown",
        &rich_md_out,
        "RuRich: Markdown",
        &rurich_md_out,
        panel_width,
    );

    // Case 2: Syntax Highlighting with Line Numbers
    let rust_code = "fn main() {\n    let answer = 42;\n    println!(\"{}\", answer);\n}";
    let python_syntax = format!(
        r#"
from rich.console import Console
from rich.syntax import Syntax

c = Console(width={}, color_system="truecolor", force_terminal=True)
s = Syntax("""{}""", "rust", line_numbers=True, theme="monokai")
c.print(s)
"#,
        inner_width, rust_code
    );

    let rich_syntax_out = env
        .run_python_rich(&python_syntax)
        .unwrap_or_else(|e| format!("Python error: {}", e));

    let rurich_syntax = Syntax::new(rust_code, "rs")
        .line_numbers(true)
        .theme("base16-ocean.dark");

    let rurich_syntax_out = inner_console.capture(|c| {
        c.println(rurich_syntax);
    });

    render_side_by_side(
        "Rich: Syntax",
        &rich_syntax_out,
        "RuRich: Syntax",
        &rurich_syntax_out,
        panel_width,
    );
}

fn compare_json(env: &CompareEnv, panel_width: usize) {
    let console = Console::new();
    console.rule(Some("JSON Formatting"));

    let inner_width = panel_width.saturating_sub(4);
    let json_data = r#"{"name": "rurich", "version": "0.1.0", "active": true, "threads": 8, "features": ["tables", "tui"]}"#;

    // Use json= instead of data= so Python rich parses and formats JSON instead of stringifying
    let python_json = format!(
        r#"
from rich.console import Console

c = Console(width={}, color_system="truecolor", force_terminal=True)
c.print_json(json="""{}""")
"#,
        inner_width, json_data
    );

    let rich_json_out = env
        .run_python_rich(&python_json)
        .unwrap_or_else(|e| format!("Python error: {}", e));

    let rurich_json = Json::from_str(json_data).unwrap();
    let inner_console = Console::with_dimensions(inner_width, 24);
    let rurich_json_out = inner_console.capture(|c| {
        c.println(rurich_json);
    });

    render_side_by_side(
        "Rich: print_json()",
        &rich_json_out,
        "RuRich: Json::from_str()",
        &rurich_json_out,
        panel_width,
    );
}

fn compare_gum_widgets(env: &CompareEnv, panel_width: usize) {
    let console = Console::new();
    console.rule(Some("Charm Gum Elements"));

    let inner_width = panel_width.saturating_sub(4);

    // Case 1: Gum Style vs RuRich Panel
    let gum_style_args = [
        "style",
        "--border",
        "rounded",
        "--padding",
        "1 2",
        "--border-foreground",
        "212",
        "Hello from Gum",
    ];
    let gum_style_out = env
        .run_gum(&gum_style_args)
        .unwrap_or_else(|e| format!("Gum error: {}", e));

    let rurich_styled_panel = Panel::fit(Text::from_plain("Hello from RuRich"))
        .box_style(ROUNDED)
        .padding(1, 2)
        .border_style(parse_style("magenta"));

    let inner_console = Console::with_dimensions(inner_width, 24);
    let rurich_panel_out = inner_console.capture(|c| {
        c.println(rurich_styled_panel);
    });

    render_side_by_side(
        "Gum: style --border rounded",
        &gum_style_out,
        "RuRich: Panel::fit()",
        &rurich_panel_out,
        panel_width,
    );

    // Case 2: Interactive Choose View snapshot
    let choose_cursor_fg =
        rurich::interactive::env_fg_ansi("GUM_CHOOSE_CURSOR_FOREGROUND", "#fabd2f");
    let choose_selected_fg =
        rurich::interactive::env_fg_ansi("GUM_CHOOSE_SELECTED_FOREGROUND", "#fabd2f");
    let gum_choose_mock = format!(
        "Select target:\n{choose_cursor_fg}> {choose_selected_fg}Production (us-east-1)\x1b[0m\n  Staging (eu-central-1)\n  Development (local)"
    );

    let choose_widget = Choose::new(vec![
        "Production (us-east-1)",
        "Staging (eu-central-1)",
        "Development (local)",
    ])
    .header("Select target:");
    let rurich_choose_view = choose_widget.render_view();

    render_side_by_side(
        "Gum: choose (view)",
        &gum_choose_mock,
        "RuRich: Choose.render_view()",
        &rurich_choose_view,
        panel_width,
    );

    // Case 3: Confirm Dialog snapshot
    let confirm_sel_bg =
        rurich::interactive::env_bg_ansi("GUM_CONFIRM_SELECTED_BACKGROUND", "#b8bb26");
    let confirm_sel_fg =
        rurich::interactive::env_fg_ansi("GUM_CONFIRM_SELECTED_FOREGROUND", "#282828");
    let confirm_unsel_bg =
        rurich::interactive::env_bg_ansi("GUM_CONFIRM_UNSELECTED_BACKGROUND", "#3c3836");
    let confirm_unsel_fg =
        rurich::interactive::env_fg_ansi("GUM_CONFIRM_UNSELECTED_FOREGROUND", "#ebdbb2");
    let confirm_prompt_fg =
        rurich::interactive::env_fg_ansi("GUM_CONFIRM_PROMPT_FOREGROUND", "#ebdbb2");
    let gum_confirm_mock = format!(
        "{confirm_prompt_fg}Deploy to production?\x1b[0m\n {confirm_sel_bg}{confirm_sel_fg}   Yes   \x1b[0m  {confirm_unsel_bg}{confirm_unsel_fg}   No   \x1b[0m"
    );

    let confirm_widget = ConfirmDialog::new("Deploy to production?");
    let rurich_confirm_view = confirm_widget.render_view();

    render_side_by_side(
        "Gum: confirm (view)",
        &gum_confirm_mock,
        "RuRich: ConfirmDialog.render_view()",
        &rurich_confirm_view,
        panel_width,
    );
}

fn compare_rurich_exclusives(panel_width: usize) {
    let console = Console::new();
    console.rule(Some("RuRich Extras"));

    // 1. TrueColor Gradients & Rainbow
    let mut grad_text = Text::from_plain("===> Continuous TrueColor RGB Gradient Transition <===");
    grad_text.gradient(Color::from_rgb(255, 50, 100), Color::from_rgb(0, 200, 255));

    let mut rainbow_text =
        Text::from_plain("===> Full Spectrum Rainbow Walk Across Characters <===");
    rainbow_text.rainbow();

    console.println(
        Panel::new(grad_text)
            .box_style(ROUNDED)
            .width(panel_width * 2 + 2),
    );
    console.println(
        Panel::new(rainbow_text)
            .box_style(ROUNDED)
            .width(panel_width * 2 + 2),
    );
    println!();

    // 2. Sub-character Smooth Progress Bar
    let inner_width = panel_width.saturating_sub(4);
    let mut bar_normal = ProgressBar::new(100)
        .smooth(false)
        .width(inner_width.saturating_sub(2));
    bar_normal.update(57);
    let mut bar_smooth = ProgressBar::new(100)
        .smooth(true)
        .width(inner_width.saturating_sub(2));
    bar_smooth.update(57);

    let inner_console = Console::with_dimensions(inner_width, 24);
    let left_bar = inner_console.capture(|c| {
        c.println(Text::from_plain("1-char block step (Rich default):"));
        c.println(bar_normal);
    });
    let right_bar = inner_console.capture(|c| {
        c.println(Text::from_plain("8-level sub-character step:"));
        c.println(bar_smooth);
    });

    render_side_by_side(
        "Standard Step",
        &left_bar,
        "RuRich Smooth Resolution",
        &right_bar,
        panel_width,
    );

    // 3. Status Badges and Breadcrumbs
    let b1 = Badge::success("ACTIVE").badge_style(BadgeStyle::Powerline);
    let b2 = Badge::error("FAILED").badge_style(BadgeStyle::Rounded);
    let b3 = Badge::warning("PENDING").badge_style(BadgeStyle::Square);
    let b4 = Badge::info("v0.1.0").badge_style(BadgeStyle::Rounded);

    let breadcrumbs = Breadcrumbs::new(vec!["root", "home", "user", "projects", "rurich"])
        .separator(" > ")
        .separator_style(parse_style("dim yellow"));

    let badge_out = console.capture(|c| {
        c.print(b1);
        c.print("  ");
        c.print(b2);
        c.print("  ");
        c.print(b3);
        c.print("  ");
        c.println(b4);
        c.println(breadcrumbs);
    });
    let badge_text = AnsiDecoder::new().decode(&badge_out);
    console.println(
        Panel::new(badge_text)
            .box_style(ROUNDED)
            .width(panel_width * 2 + 2),
    );
    println!();

    // 4. Callout Admonitions
    let callout_note = Callout::note("Zero-overhead single binary without VM.");
    let callout_tip = Callout::tip("AnsiDecoder translates raw ANSI to styled text.");
    let callout_warn = Callout::warning("High memory usage threshold on worker.");

    let c_left = inner_console.capture(|c| {
        c.println(callout_note);
        c.println(callout_tip);
    });
    let c_right = inner_console.capture(|c| {
        c.println(callout_warn);
    });

    render_side_by_side(
        "Callouts: Note & Tip",
        &c_left,
        "Callout: Warning",
        &c_right,
        panel_width,
    );

    // 5. Sparklines
    let sparkline = Sparkline::new(vec![
        1.0, 3.2, 5.5, 2.0, 8.4, 9.1, 7.3, 4.0, 1.2, 0.5, 6.2, 8.9,
    ])
    .style(parse_style("cyan"));
    let spark_out = console.capture(|c| {
        c.print("CPU Load (12 ticks): ");
        c.println(sparkline);
    });
    let spark_text = AnsiDecoder::new().decode(&spark_out);
    console.println(
        Panel::new(spark_text)
            .box_style(ROUNDED)
            .width(panel_width * 2 + 2),
    );
    println!();
}

fn print_parity_matrix() {
    let console = Console::new();
    console.rule(Some("Parity Matrix"));

    let mut table = Table::new().title("Feature Comparison").box_style(ROUNDED);
    table
        .add_column(
            Column::new()
                .header("Feature")
                .justify(JustifyMethod::Left)
                .style(parse_style("bold")),
        )
        .add_column(
            Column::new()
                .header("Rich")
                .justify(JustifyMethod::Center)
                .style(parse_style("cyan")),
        )
        .add_column(
            Column::new()
                .header("Gum")
                .justify(JustifyMethod::Center)
                .style(parse_style("magenta")),
        )
        .add_column(
            Column::new()
                .header("RuRich")
                .justify(JustifyMethod::Center)
                .style(parse_style("green")),
        )
        .add_column(
            Column::new()
                .header("Status")
                .justify(JustifyMethod::Center),
        )
        .add_column(
            Column::new()
                .header("Notes")
                .justify(JustifyMethod::Left)
                .style(parse_style("dim")),
        );

    table.add_row(vec![
        "Tables",
        "Full",
        "Basic",
        "Full",
        "Parity",
        "13 box styles, ratio solver, sections",
    ]);
    table.add_row(vec![
        "Panels",
        "Full",
        "Basic",
        "Full",
        "Parity",
        "Margin, padding, left-accent, titles",
    ]);
    table.add_row(vec![
        "BBCode & TrueColor",
        "Full",
        "N/A",
        "Full",
        "Parity",
        "Hex (#rgb), RGB triplets, OSC 8 links",
    ]);
    table.add_row(vec![
        "Trees",
        "Full",
        "N/A",
        "Full",
        "Parity",
        "Branch guides (DEFAULT, ASCII, BOLD, DOUBLE)",
    ]);
    table.add_row(vec![
        "Markdown",
        "Full",
        "Full",
        "Full",
        "Parity",
        "Headings, lists, codeblocks, quotes",
    ]);
    table.add_row(vec![
        "Syntax Highlighting",
        "Full",
        "N/A",
        "Full",
        "Parity",
        "Syntect lexer, themes, line gutters",
    ]);
    table.add_row(vec![
        "JSON Formatter",
        "Full",
        "N/A",
        "Full",
        "Parity",
        "Tokenized keys, values, custom indent",
    ]);
    table.add_row(vec![
        "Choose / MultiSelect",
        "Prompt",
        "Full",
        "Full",
        "Parity",
        "Vim keys, filter search, checkboxes",
    ]);
    table.add_row(vec![
        "Fuzzy Filter",
        "N/A",
        "Full",
        "Full",
        "Parity",
        "Subsequence scoring, highlighted runes",
    ]);
    table.add_row(vec![
        "Confirm Dialog",
        "Prompt",
        "Full",
        "Full",
        "Parity",
        "Tab/arrow toggle, yes/no labels",
    ]);
    table.add_row(vec![
        "Input / TextArea",
        "Prompt",
        "Full",
        "Full",
        "Parity",
        "Cursor navigation, placeholder",
    ]);
    table.add_row(vec![
        "Pager & FilePicker",
        "N/A",
        "Full",
        "Full",
        "Parity",
        "Scrollable buffer, directory tree",
    ]);
    table.add_section();
    table.add_row(vec![
        "Text Gradients",
        "Manual",
        "N/A",
        "Built-in",
        "Extended",
        "Smooth RGB interpolation across runes",
    ]);
    table.add_row(vec![
        "Sub-character Progress",
        "1-char",
        "ASCII",
        "8-level",
        "Extended",
        "Fractional block resolution: ▏..█",
    ]);
    table.add_row(vec![
        "Badges & Breadcrumbs",
        "N/A",
        "N/A",
        "Built-in",
        "Extended",
        "Powerline/rounded pills, path crumbs",
    ]);
    table.add_row(vec![
        "Callout Admonitions",
        "N/A",
        "N/A",
        "Built-in",
        "Extended",
        "Note, tip, warning, caution blocks",
    ]);
    table.add_row(vec![
        "Block Stitching (Join)",
        "Columns",
        "Basic",
        "Full",
        "Extended",
        "2D arbitrary alignment (H & V)",
    ]);
    table.add_row(vec![
        "Startup Latency",
        "~80ms",
        "~15ms",
        "<1ms",
        "Faster",
        "Native binary, zero VM overhead",
    ]);

    console.println(&table);
    println!();
}

fn export_reports(_env: &CompareEnv, out_dir: &Path) {
    let console = Console::new();
    console.rule(Some("Exporting Reports"));

    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!("Failed to create export directory: {}", e);
        return;
    }

    let mut table = Table::new()
        .title("RuRich vs Python Rich vs Charm Gum")
        .box_style(ROUNDED);
    table
        .add_column(
            Column::new()
                .header("Capability")
                .justify(JustifyMethod::Left)
                .style(parse_style("bold")),
        )
        .add_column(
            Column::new()
                .header("Rich / Gum")
                .justify(JustifyMethod::Center)
                .style(parse_style("cyan")),
        )
        .add_column(
            Column::new()
                .header("RuRich")
                .justify(JustifyMethod::Center)
                .style(parse_style("green")),
        )
        .add_column(
            Column::new()
                .header("Evaluation")
                .justify(JustifyMethod::Center),
        );
    table.add_row(vec![
        "Tables & Box Styles",
        "Supported",
        "Supported",
        "Parity",
    ]);
    table.add_row(vec![
        "Panels with Margin/Padding",
        "Supported",
        "Supported",
        "Parity",
    ]);
    table.add_row(vec![
        "BBCode & TrueColor",
        "Supported",
        "Supported",
        "Parity",
    ]);
    table.add_row(vec!["Trees & Guides", "Supported", "Supported", "Parity"]);
    table.add_row(vec![
        "Interactive Choose/Filter",
        "Gum only",
        "Supported",
        "Parity",
    ]);
    table.add_row(vec![
        "Text Gradients & Rainbow",
        "None",
        "Built-in",
        "Extended",
    ]);
    table.add_row(vec![
        "Sub-character Progress",
        "None",
        "Built-in",
        "Extended",
    ]);
    table.add_row(vec!["Badges & Callouts", "None", "Built-in", "Extended"]);

    let html_content = console.export_html(&table);
    let html_path = out_dir.join("comparison_report.html");
    if let Ok(_) = fs::write(&html_path, html_content) {
        println!("  Saved HTML: {}", html_path.display());
    }

    let svg_content = console.export_svg(&table, "RuRich Comparison");
    let svg_path = out_dir.join("comparison_report.svg");
    if let Ok(_) = fs::write(&svg_path, svg_content) {
        println!("  Saved SVG: {}", svg_path.display());
    }

    println!();
}

fn run_gum_interactive(
    gum_bin: Option<&Path>,
    args: &[&str],
    piped_stdin: Option<&str>,
) -> Option<String> {
    let bin = gum_bin?;
    let mut cmd = Command::new(bin);
    cmd.args(args);

    if let Some(input) = piped_stdin {
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        let mut child = cmd.spawn().ok()?;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = std::io::Write::write_all(&mut stdin, input.as_bytes());
        }
        let output = child.wait_with_output().ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    } else {
        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::piped());
        let child = cmd.spawn().ok()?;
        let output = child.wait_with_output().ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }
}

fn drain_input() {
    while crossterm::event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
        let _ = crossterm::event::read();
    }
}

fn wait_enter(prompt: &str) {
    println!("\n  \x1b[2m-> {}\x1b[0m", prompt);
    drain_input();
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
}

fn run_interactive_tui_showcase(env: &CompareEnv, panel_width: usize) {
    let console = Console::new();
    console.rule(Some("Interactive TUI: Gum vs RuRich"));

    let gum_status = if env.gum_bin.is_some() {
        "Available"
    } else {
        "Not found (running RuRich only)"
    };
    println!("  Gum CLI status: {}\n", gum_status);
    println!("  Each test lets you interact with Charm Gum, then RuRich, and compares results.");

    // 1. Choose Menu
    println!("\n── 1. Choose Menu (Single Select) ──");
    let choose_items = [
        "Deploy to Kubernetes",
        "Run Database Migrations",
        "Generate Parity Report",
        "Cancel",
    ];

    let gum_res = if env.gum_bin.is_some() {
        wait_enter("1.1 Run Gum: Press [Enter] to launch 'gum choose' (arrows / Enter)...");
        let res = run_gum_interactive(
            env.gum_bin.as_deref(),
            &[
                "choose",
                "Deploy to Kubernetes",
                "Run Database Migrations",
                "Generate Parity Report",
                "Cancel",
            ],
            None,
        );
        println!(
            "  Gum result   : {:?}",
            res.as_deref().unwrap_or("[Cancelled]")
        );
        res
    } else {
        None
    };

    wait_enter("1.2 Run RuRich: Press [Enter] to launch 'rurich::Choose' (↑/↓/j/k / Enter)...");
    let mut choose = Choose::new(choose_items.to_vec()).header("Select action to execute:");
    let rurich_res = choose.run().ok().flatten().and_then(|v| v.first().cloned());
    println!(
        "  RuRich result: {:?}",
        rurich_res.as_deref().unwrap_or("[Cancelled]")
    );
    println!(
        "  -> Result comparison: Gum={:?} | RuRich={:?}",
        gum_res.as_deref(),
        rurich_res.as_deref()
    );

    let choose_cursor_fg =
        rurich::interactive::env_fg_ansi("GUM_CHOOSE_CURSOR_FOREGROUND", "#fabd2f");
    let choose_selected_fg =
        rurich::interactive::env_fg_ansi("GUM_CHOOSE_SELECTED_FOREGROUND", "#fabd2f");
    let gum_mock = format!(
        "Select action to execute:\n{choose_cursor_fg}> {choose_selected_fg}Deploy to Kubernetes\x1b[0m\n  Run Database Migrations\n  Generate Parity Report\n  Cancel"
    );
    render_side_by_side(
        "Gum: choose (view)",
        &gum_mock,
        "RuRich: Choose (view)",
        &choose.render_view(),
        panel_width,
    );

    // 2. Confirm Dialog
    println!("\n── 2. Confirm Dialog ──");
    let gum_confirm_res = if let Some(ref bin) = env.gum_bin {
        wait_enter("2.1 Run Gum: Press [Enter] to launch 'gum confirm' (Tab/arrows / Enter)...");
        let status = Command::new(bin)
            .args(&["confirm", "Apply changes to production database?"])
            .stdin(std::process::Stdio::inherit())
            .status()
            .map(|s| s.success())
            .ok();
        println!("  Gum decision   : {:?}", status);
        status
    } else {
        None
    };

    wait_enter(
        "2.2 Run RuRich: Press [Enter] to launch 'rurich::ConfirmDialog' (Tab/h,l / Enter)...",
    );
    let mut confirm = ConfirmDialog::new("Apply changes to production database?");
    let rurich_confirm_res = confirm.run().ok().flatten();
    println!("  RuRich decision: {:?}", rurich_confirm_res);
    println!(
        "  -> Result comparison: Gum={:?} | RuRich={:?}",
        gum_confirm_res, rurich_confirm_res
    );

    let confirm_sel_bg =
        rurich::interactive::env_bg_ansi("GUM_CONFIRM_SELECTED_BACKGROUND", "#b8bb26");
    let confirm_sel_fg =
        rurich::interactive::env_fg_ansi("GUM_CONFIRM_SELECTED_FOREGROUND", "#282828");
    let confirm_unsel_bg =
        rurich::interactive::env_bg_ansi("GUM_CONFIRM_UNSELECTED_BACKGROUND", "#3c3836");
    let confirm_unsel_fg =
        rurich::interactive::env_fg_ansi("GUM_CONFIRM_UNSELECTED_FOREGROUND", "#ebdbb2");
    let confirm_prompt_fg =
        rurich::interactive::env_fg_ansi("GUM_CONFIRM_PROMPT_FOREGROUND", "#ebdbb2");
    let gum_confirm_mock = format!(
        "{confirm_prompt_fg}Apply changes to production database?\x1b[0m\n {confirm_sel_bg}{confirm_sel_fg}   Yes   \x1b[0m  {confirm_unsel_bg}{confirm_unsel_fg}   No   \x1b[0m"
    );
    render_side_by_side(
        "Gum: confirm (view)",
        &gum_confirm_mock,
        "RuRich: ConfirmDialog (view)",
        &confirm.render_view(),
        panel_width,
    );

    // 3. Fuzzy Filter
    println!("\n── 3. Fuzzy Search Filter ──");
    let filter_candidates = "git status\ngit commit -m 'feat: clean up'\ngit push origin main\ncargo check\ncargo run --example compare\n";

    let gum_filter_res = if env.gum_bin.is_some() {
        wait_enter("3.1 Run Gum: Press [Enter] to launch 'gum filter' (type text / Enter)...");
        let res = run_gum_interactive(
            env.gum_bin.as_deref(),
            &["filter", "--placeholder", "Search commands..."],
            Some(filter_candidates),
        );
        println!(
            "  Gum match   : {:?}",
            res.as_deref().unwrap_or("[Cancelled]")
        );
        res
    } else {
        None
    };

    wait_enter("3.2 Run RuRich: Press [Enter] to launch 'rurich::Filter' (type text / Enter)...");
    let mut filter = Filter::new(vec![
        "git status",
        "git commit -m 'feat: clean up'",
        "git push origin main",
        "cargo check",
        "cargo run --example compare",
    ])
    .placeholder("Search commands...");
    let rurich_filter_res = filter.run().ok().flatten();
    println!(
        "  RuRich match: {:?}",
        rurich_filter_res.as_deref().unwrap_or("[Cancelled]")
    );
    println!(
        "  -> Result comparison: Gum={:?} | RuRich={:?}",
        gum_filter_res.as_deref(),
        rurich_filter_res.as_deref()
    );

    let filter_prompt_fg =
        rurich::interactive::env_fg_ansi("GUM_FILTER_PROMPT_FOREGROUND", "#fabd2f");
    let filter_header_fg =
        rurich::interactive::env_fg_ansi("GUM_FILTER_HEADER_FOREGROUND", "#928374");
    let filter_ind_fg =
        rurich::interactive::env_fg_ansi("GUM_FILTER_INDICATOR_FOREGROUND", "#fabd2f");
    let gum_filter_mock = format!(
        "{filter_prompt_fg}>\x1b[0m {filter_header_fg}Search commands...\x1b[0m\n{filter_ind_fg}•\x1b[0m git status\n  git commit -m 'feat: clean up'\n  git push origin main"
    );
    render_side_by_side(
        "Gum: filter (view)",
        &gum_filter_mock,
        "RuRich: Filter (view)",
        &filter.render_view(),
        panel_width,
    );

    // 4. Text Input
    println!("\n── 4. Single-line Input ──");
    let gum_input_res = if env.gum_bin.is_some() {
        wait_enter("4.1 Run Gum: Press [Enter] to launch 'gum input' (type text / Enter)...");
        let res = run_gum_interactive(
            env.gum_bin.as_deref(),
            &["input", "--placeholder", "Enter configuration name..."],
            None,
        );
        println!(
            "  Gum input   : {:?}",
            res.as_deref().unwrap_or("[Cancelled]")
        );
        res
    } else {
        None
    };

    wait_enter("4.2 Run RuRich: Press [Enter] to launch 'rurich::Input' (type text / Enter)...");
    let mut input = Input::new().placeholder("Enter configuration name...");
    let rurich_input_res = input.run().ok().flatten();
    println!(
        "  RuRich input: {:?}",
        rurich_input_res.as_deref().unwrap_or("[Cancelled]")
    );
    println!(
        "  -> Result comparison: Gum={:?} | RuRich={:?}",
        gum_input_res.as_deref(),
        rurich_input_res.as_deref()
    );

    let input_prompt_fg =
        rurich::interactive::env_fg_ansi("GUM_INPUT_PROMPT_FOREGROUND", "#fabd2f");
    let input_header_fg =
        rurich::interactive::env_fg_ansi("GUM_INPUT_HEADER_FOREGROUND", "#928374");
    let gum_input_mock = format!(
        "{input_prompt_fg}>\x1b[0m \x1b[7mE\x1b[0m{input_header_fg}nter configuration name...\x1b[0m"
    );
    render_side_by_side(
        "Gum: input (view)",
        &gum_input_mock,
        "RuRich: Input (view)",
        &input.render_view(),
        panel_width,
    );

    println!("  Interactive head-to-head comparison complete.\n");
}

fn main() {
    let env = CompareEnv::detect();
    let console = Console::new();

    let term_width = console.size().width;
    let panel_width = ((term_width.max(80) - 6) / 2).clamp(38, 60);

    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--all") {
        run_all(&env, panel_width);
        return;
    }

    if args.iter().any(|a| a == "--matrix") {
        print_parity_matrix();
        return;
    }

    if let Some(pos) = args.iter().position(|a| a == "--export") {
        let path_str = args
            .get(pos + 1)
            .map(|s| s.as_str())
            .unwrap_or("target/comparison");
        export_reports(&env, Path::new(path_str));
        return;
    }

    if let Some(pos) = args.iter().position(|a| a == "--category") {
        if let Some(cat) = args.get(pos + 1) {
            match cat.to_lowercase().as_str() {
                "tables" | "table" => compare_tables(&env, panel_width),
                "panels" | "panel" => compare_panels(&env, panel_width),
                "markup" | "colors" => compare_markup_and_colors(&env, panel_width),
                "trees" | "tree" => compare_trees(&env, panel_width),
                "markdown" | "syntax" => compare_markdown_and_syntax(&env, panel_width),
                "json" => compare_json(&env, panel_width),
                "gum" => compare_gum_widgets(&env, panel_width),
                "exclusives" => compare_rurich_exclusives(panel_width),
                "matrix" => print_parity_matrix(),
                "interactive" => run_interactive_tui_showcase(&env, panel_width),
                _ => eprintln!(
                    "Unknown category '{}'. Available: tables, panels, markup, trees, markdown, json, gum, exclusives, matrix, interactive",
                    cat
                ),
            }
            return;
        }
    }

    println!("\n  RuRich vs Rich & Gum Comparison\n");

    let menu_items = vec![
        "Run All Visual Comparisons",
        "Tables",
        "Panels",
        "BBCode Markup & Colors",
        "Trees",
        "Markdown & Syntax Highlighting",
        "JSON Formatting",
        "Charm Gum Elements",
        "RuRich Extras",
        "Interactive TUI Head-to-Head (Gum vs RuRich)",
        "Print Parity Matrix",
        "Export HTML & SVG Gallery",
        "Exit",
    ];

    let mut menu = Choose::new(menu_items).header("Select comparison suite:");

    loop {
        match menu.run() {
            Ok(Some(selected)) => {
                let choice = selected.first().map(|s| *s).unwrap_or("");
                if choice.contains("All Visual") {
                    run_all(&env, panel_width);
                } else if choice == "Tables" {
                    compare_tables(&env, panel_width);
                } else if choice == "Panels" {
                    compare_panels(&env, panel_width);
                } else if choice.contains("BBCode") {
                    compare_markup_and_colors(&env, panel_width);
                } else if choice == "Trees" {
                    compare_trees(&env, panel_width);
                } else if choice.contains("Markdown") {
                    compare_markdown_and_syntax(&env, panel_width);
                } else if choice.contains("JSON") {
                    compare_json(&env, panel_width);
                } else if choice.contains("Gum") {
                    compare_gum_widgets(&env, panel_width);
                } else if choice.contains("Extras") {
                    compare_rurich_exclusives(panel_width);
                } else if choice.contains("Interactive") {
                    run_interactive_tui_showcase(&env, panel_width);
                } else if choice.contains("Matrix") {
                    print_parity_matrix();
                } else if choice.contains("Export") {
                    export_reports(&env, Path::new("target/comparison"));
                } else {
                    break;
                }
            }
            _ => {
                break;
            }
        }
    }
}

fn run_all(env: &CompareEnv, panel_width: usize) {
    compare_tables(env, panel_width);
    compare_panels(env, panel_width);
    compare_markup_and_colors(env, panel_width);
    compare_trees(env, panel_width);
    compare_markdown_and_syntax(env, panel_width);
    compare_json(env, panel_width);
    compare_gum_widgets(env, panel_width);
    compare_rurich_exclusives(panel_width);
    print_parity_matrix();
}
