# rurich

RuRich is a native Rust library for terminal formatting, styled text, tables, panels, markdown, syntax highlighting, progress displays, and diagnostic reporting. It provides terminal styling capabilities with complete visual parity to Python Rich, without requiring Python or external runtimes.

---

## Features

### Colors and Styles
- **TrueColor (24-bit RGB)**: Supports hexadecimal (`#ff00aa`) and RGB (`rgb(255, 0, 128)`) color definitions.
- **8-bit (ANSI 256) and Standard 16 ANSI colors**: Comprehensive palette with over 230 standard named colors.
- **Terminal capability detection**: Automatic detection and graceful downgrading based on environment variables (`NO_COLOR`, `COLORTERM`, `TERM`).
- **Text attributes**: `bold`, `dim`, `italic`, `underline`, `double_underline`, `strike`, `reverse`, `blink`, `overline`.
- **OSC 8 Hyperlinks**: Clickable hyperlinks in compatible terminal emulators.

### Inline Markup Engine
- Human-readable BBCode-style syntax with nesting and implicit tag closure:
  ```rust
  rprintln!("[bold red]Error:[/] Operation failed on [cyan]{}[/]", target);
  ```
- Bracket escaping via `\[tag]`.

### Tables
- Automated column width negotiation and constraint solving (`ratio_resolve`, `ratio_reduce`).
- Column headers, footers, and per-column text alignment (`Left`, `Center`, `Right`, `Full`).
- Row section dividers via `table.add_section()`.
- Borderless grid mode via `Table::grid()`.
- Automatic markup parsing inside string cells.

### Panels and Box Styles
- Bordered boxes enclosing any renderable widget.
- Centered, left-, or right-aligned titles and subtitles embedded directly into border lines.
- 13 standard box drawing sets: `ROUNDED`, `SQUARE`, `SQUARE_DOUBLE_HEAD`, `ASCII`, `ASCII2`, `HEAVY`, `HEAVY_HEAD`, `DOUBLE`, `DOUBLE_EDGE`, `MINIMAL`, `SIMPLE`, `HORIZONTALS`, `MARKDOWN`.
- Configurable inner padding and border colors.

### Layout and Containers
- **`Layout`**: Hierarchical split-screen grid engine (rows and columns with ratios and minimum dimensions).
- **`Group`**: Container to combine multiple heterogeneous renderables into a single renderable widget.
- **`Columns`**: Flow layout arranging items across the terminal width.
- **`Constrain`**: Restricts the maximum width of any inner renderable.
- **`Padding` & `Align`**: Spacing and alignment (horizontal and vertical) controls.

### Dynamic Live Displays and Progress Bars
- **`Live`**: Flicker-free, in-place re-rendering engine using terminal cursor manipulation (`CarriageReturn`, `CursorUp`, `EraseInLine`).
- **`Progress`**: Multi-task progress tracking engine with pluggable columns:
  - `SpinnerColumn`: Animated spinner.
  - `TextColumn`: Task description.
  - `BarColumn`: Configurable visual progress bar.
  - `TaskProgressColumn`: Percentage indicator.
  - `TimeElapsedColumn` & `TimeRemainingColumn`: Elapsed time and ETA estimation.
  - `FileSizeColumn` & `TotalFileSizeColumn`: Byte formatters.
  - `DownloadColumn`: Combined download progress (e.g. `12.5 / 100.0 MB` or `MiB`).
  - `TransferSpeedColumn`: Transfer rate formatting (e.g. `2.4 MB/s`).
  - `MofNCompleteColumn`: Step ratios (e.g. `10/100`).
- **`track`**: Iterator wrapper adding an automatic live progress bar to any Rust iterator.
- **`Status`**: Spinner indicator with dynamic status text.

### Content Renderers
- **`Tree`**: Hierarchical tree views with configurable branch guides (`DEFAULT_GUIDES`, `ASCII_GUIDES`, `BOLD_GUIDES`, `DOUBLE_GUIDES`).
- **`Markdown`**: CommonMark parser rendering headings with decorative rules, styled blockquotes, code blocks, and bullet/numbered lists.
- **`Json`**: JSON formatter with color-coded tokens, keys, values, and custom indentation.
- **`Syntax`**: Syntect-powered code syntax highlighting supporting themes, line numbers, line ranges, and line highlights.
- **`Pretty`**: Syntax-highlighted data structure inspector for types implementing `Debug`.

### Diagnostic & CLI Utilities
- **`Traceback`**: Stack trace formatter with source code context snippets, stack frame boxes, and panic hook installation (`Traceback::install()`).
- **`Prompt` & `Confirm`**: Interactive terminal prompts supporting default values, choices, and input validation.
- **`export_html` & `export_svg`**: Direct console export to standalone HTML or vector SVG terminal window images.
- **`AnsiDecoder`**: Translates raw terminal output containing ANSI escape sequences into styled `Text`.
- **`ConsoleLayer` / `RuRichLayer`**: `tracing-subscriber` layer formatting events with level badges, timestamps, and target locations.
- **`inspect`**: Inspects a Rust value, printing its type, memory size, and pretty debug representation.

---

## Usage Examples

### 1. Basic Markup and Console

```rust
use rurich::prelude::*;

fn main() {
    rprintln!("[bold green]Success![/] Configuration loaded from [cyan]/etc/app.conf[/]");
}
```

### 2. Table

```rust
use rurich::prelude::*;

fn main() {
    let mut table = Table::new().box_style(ROUNDED);
    table.add_column(Column::new().header("Service").style(Style::new().bold()));
    table.add_column(Column::new().header("Status").justify(JustifyMethod::Center));
    table.add_column(Column::new().header("Latency").justify(JustifyMethod::Right));

    table.add_row(["auth-api", "[bold green]ONLINE[/]", "12ms"]);
    table.add_row(["db-replica", "[bold yellow]SYNCING[/]", "45ms"]);
    table.add_row(["cache", "[bold red]OFFLINE[/]", "0ms"]);

    get_console().println(table);
}
```

### 3. Bordered Panel

```rust
use rurich::prelude::*;

fn main() {
    let panel = Panel::from_markup("[bold white]Node cluster synchronized.[/]\nAll 16 nodes healthy.")
        .box_style(ROUNDED)
        .title("[bold green] Cluster Status [/]")
        .subtitle("[dim] Region: us-east-1 [/]")
        .padding(1, 2);

    get_console().println(panel);
}
```

### 4. Hierarchical Tree

```rust
use rurich::prelude::*;

fn main() {
    let mut tree = Tree::from_markup("[bold green]Project[/]");
    let src = tree.add_markup("[bold yellow]src/[/]");
    src.add_markup("[cyan]lib.rs[/]");
    src.add_markup("[cyan]main.rs[/]");
    tree.add_markup("[dim]Cargo.toml[/]");

    get_console().println(tree);
}
```

### 5. Progress Tracking via Iterator

```rust
use rurich::prelude::*;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let items = vec![1, 2, 3, 4, 5];
    for item in track(items, "Processing items") {
        sleep(Duration::from_millis(100));
        let _ = item;
    }
}
```

### 6. Interactive Prompt

```rust
use rurich::prompt::{Confirm, Prompt};

fn main() {
    let username: String = Prompt::new("Username")
        .default("admin".to_string())
        .ask()
        .unwrap();

    let proceed: bool = Confirm::new("Deploy changes?")
        .default(false)
        .ask()
        .unwrap();

    println!("User: {}, Proceed: {}", username, proceed);
}
```

### 7. Exporting to SVG and HTML

```rust
use rurich::prelude::*;

fn main() {
    let console = Console::new();
    let panel = Panel::from_markup("[green]Success![/] All checks passed.")
        .title("Build Report");

    let html_content = console.export_html(&panel);
    let svg_content = console.export_svg(&panel, "Build Output");

    std::fs::write("output.html", html_content).unwrap();
    std::fs::write("output.svg", svg_content).unwrap();
}
```

---

## Running Examples & Tests

Run the demo showcase:
```bash
cargo run --all-features --example demo
```

Run the multi-task progress demo:
```bash
cargo run --example progress_demo
```

Run the automated test suite:
```bash
cargo test --all-features
```
