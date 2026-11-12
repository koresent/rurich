use crate::console::get_console;
use crate::panel::Panel;
use crate::pretty::Pretty;
use crate::style::Style;
use crate::text::Text;
use std::any::type_name;
use std::fmt::Debug;
use std::mem::size_of;

/// Inspects a Rust value and prints its type name, memory size, and pretty debug representation.
pub fn inspect<T: Debug>(value: &T) {
    let t_name = type_name::<T>();
    let size = size_of::<T>();

    let mut header = Text::new();
    header.append(
        "Type: ",
        Some(Style::parse("bold cyan").unwrap_or_default()),
    );
    header.append(
        t_name,
        Some(Style::parse("bold yellow").unwrap_or_default()),
    );
    header.append(
        &format!(" ({} bytes)\n\n", size),
        Some(Style::parse("dim").unwrap_or_default()),
    );

    let mut body = Text::new();
    body.append_text(&header);

    let pretty = Pretty::new(value);
    body.append_text(&pretty.text);

    let panel = Panel::new(body)
        .box_style(crate::box_styles::ROUNDED)
        .title("[bold green] Inspect [/]")
        .border_style(Style::parse("cyan").unwrap_or_default());

    get_console().println(panel);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Example {
        x: i32,
        y: String,
    }

    #[test]
    fn test_inspect_call() {
        let ex = Example {
            x: 10,
            y: "test".into(),
        };
        // Verify inspect runs without panic
        let console = get_console();
        let _out = console.capture(|_| {
            inspect(&ex);
        });
    }
}
