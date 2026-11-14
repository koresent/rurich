# rurich

RuRich is a native Rust library for terminal formatting, styled text, tables, panels, layout composition, markdown, syntax highlighting, progress displays, interactive micro-TUI widgets, and diagnostic reporting.

---

## Features

### Colors, Styles and Gradients
- **TrueColor (24-bit RGB)**: Supports hexadecimal (`#ff00aa`) and RGB (`rgb(255, 0, 128)`) color definitions.
- **8-bit (ANSI 256) and Standard 16 ANSI colors**: Comprehensive palette with over 230 standard named colors.
- **Terminal capability detection**: Automatic detection and graceful downgrading based on environment variables (`NO_COLOR`, `COLORTERM`, `TERM`).
- **Text attributes**: `bold`, `dim`, `italic`, `underline`, `double_underline`, `strike`, `reverse`, `blink`, `overline`.
- **RGB Gradients & Rainbows**: `Text::gradient(from, to)` and `Text::rainbow()` interpolating smooth TrueColor along text.
- **OSC 8 Hyperlinks**: Clickable hyperlinks in compatible terminal emulators.

### Inline Markup Engine
- Human-readable BBCode-style syntax with nesting and implicit tag closure:
  ```rust
  rprintln!("[bold red]Error:[/] Operation failed on [cyan]{}[/]", target);
  ```
- Bracket escaping via `\[tag]`.

### Interactive TUI Components
- **`Choose` & `MultiChoose`**: Interactive selection menus with arrow/Vim key navigation, customizable cursor (`> `), multi-select checkboxes (`[✓]`), and search filtering.
- **`Filter`**: Interactive fuzzy-finder over any dataset with live keystroke filtering and highlighted matching runes.
- **`ConfirmDialog`**: Visual button-based confirmation (`[ Yes ]  [ No ]`) with arrow/Tab focus switching.
- **`Input` & `TextArea`**: Single-line input with placeholder and password masking (`*`), and multi-line editor with line numbers and cursor navigation.
- **`Pager`**: Interactive scrollable viewport for viewing documents, tables, or markdown with keyboard navigation (`j`/`k`, `PageUp`/`PageDown`).
- **`FilePicker`**: Interactive directory browser and file selector with directory icons and hierarchy traversal.

### Visual Badges, Breadcrumbs and Callouts
- **`Badge`**: Status pills with square, rounded, or powerline borders (`Badge::success("PASS")`, `Badge::error("FAIL")`).
- **`Breadcrumbs`**: Navigation paths with custom chevron or slash separators (`Breadcrumbs::new(vec!["root", "user", "src"])`).
- **`Callout`**: Styled admonition blocks with left accent bars and icons (`Callout::note`, `Callout::tip`, `Callout::warning`, `Callout::caution`, `Callout::important`).

### Layout, Borders and Composition
- **`Join`**: Free-form horizontal and vertical block stitching with alignment controls (`join_horizontal`, `join_vertical`).
- **`Panel`**: Bordered boxes enclosing any renderable widget, supporting external margins (`margin`), per-edge borders (`left_accent`, `horizontal_only`), and aligned titles/subtitles.
- **13 Box Drawing Sets**: `ROUNDED`, `SQUARE`, `SQUARE_DOUBLE_HEAD`, `ASCII`, `ASCII2`, `HEAVY`, `HEAVY_HEAD`, `DOUBLE`, `DOUBLE_EDGE`, `MINIMAL`, `SIMPLE`, `HORIZONTALS`, `MARKDOWN`.
- **`Layout`**: Hierarchical split-screen grid engine (rows and columns with ratios and minimum dimensions).
- **`Group`**: Container combining multiple heterogeneous renderables into a single widget.
- **`Columns`**: Flow layout arranging items across the terminal width.
- **`Constrain`, `Padding` & `Align`**: Spacing, alignment, and maximum width constraints.

### Data Visualization, Progress Bars and Animations
- **`Sparkline`**: High-resolution mini-charts using 8 Unicode block levels (` ▂▃▅▆▇█`) from numeric sequences.
- **Smooth `ProgressBar`**: Sub-character resolution using 8-level fractional Unicode blocks (`▏` through `█`) for smooth progression.
- **Pulse Animation**: Shimmer wave for indeterminate progress indicators.
- **Spinner Catalog**: 20+ presets (`SpinnerPreset::Dots`, `Dots2`, `Line`, `Arc`, `Moon`, `Clock`, `BouncingBar`, `Globe`, etc.) accessible via enum or by name (`Spinner::by_name("moon")`).
- **`Progress`**: Multi-task progress tracking engine with pluggable columns (`SpinnerColumn`, `TextColumn`, `BarColumn`, `TaskProgressColumn`, `TimeElapsedColumn`, `DownloadColumn`, `TransferSpeedColumn`, `MofNCompleteColumn`).
- **`track`**: Automatic live progress indicator for any Rust iterator.
- **`Status`**: Animated spinner indicator with dynamic status text.

### Tables
- Automated column width negotiation and constraint solving (`ratio_resolve`, `ratio_reduce`).
- Column headers, footers, and per-column text alignment (`Left`, `Center`, `Right`, `Full`).
- Row section dividers via `table.add_section()`.
- Borderless grid mode via `Table::grid()`.
- Automatic markup parsing inside string cells.

### Content Renderers
- **`Tree`**: Hierarchical tree views with configurable branch guides (`DEFAULT_GUIDES`, `ASCII_GUIDES`, `BOLD_GUIDES`, `DOUBLE_GUIDES`).
- **`Markdown`**: CommonMark parser rendering headings with decorative rules, styled blockquotes, code blocks, and bullet/numbered lists.
- **`Json`**: JSON formatter with color-coded tokens, keys, values, and custom indentation.
- **`Syntax`**: Syntect-powered code syntax highlighting supporting themes, line numbers, line ranges, and line highlights.
- **`Pretty`**: Syntax-highlighted data structure inspector for types implementing `Debug`.

### Diagnostic & CLI Utilities
- **`Traceback`**: Stack trace formatter with source code context snippets, stack frame boxes, and panic hook installation (`Traceback::install()`).
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

### 2. Interactive Selection (`Choose`)

```rust
use rurich::interactive::Choose;

fn main() {
    let mut choose = Choose::new(vec!["Option A", "Option B", "Option C"])
        .header("Select target environment:");
    if let Ok(Some(selected)) = choose.run() {
        println!("Selected: {:?}", selected);
    }
}
```

### 3. Visual Callouts and Badges

```rust
use rurich::prelude::*;

fn main() {
    let badge = Badge::success("ACTIVE");
    let callout = Callout::tip("Use --release for optimized compilation performance.");

    get_console().println(badge);
    get_console().println(callout);
}
```

### 4. Sparkline Mini-Charts

```rust
use rurich::prelude::*;

fn main() {
    let sparkline = Sparkline::new(vec![1.0, 3.5, 2.0, 7.5, 9.0, 4.0, 8.5])
        .style(Style::parse("bold cyan").unwrap());

    get_console().println(sparkline);
}
```

### 5. Table with Alignment

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

### 6. Bordered Panel with Custom Sides and Margins

```rust
use rurich::prelude::*;

fn main() {
    let panel = Panel::from_markup("[bold white]Node cluster synchronized.[/]\nAll 16 nodes healthy.")
        .box_style(ROUNDED)
        .title("[bold green] Cluster Status [/]")
        .margin(1, 2)
        .padding(1, 2);

    get_console().println(panel);
}
```

### 7. Progress Tracking via Iterator

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

---

## Complete Feature Showcase

Run the all-in-one demonstration covering all library features:

```bash
cargo run --all-features --example demo
```

---

## Running Tests

```bash
cargo test --all-features
```
