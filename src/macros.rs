#[macro_export]
macro_rules! rprint {
    ($($arg:tt)*) => {{
        let formatted = format!($($arg)*);
        $crate::console::get_console().print_markup(&formatted);
    }};
}

#[macro_export]
macro_rules! rprintln {
    () => {{
        $crate::console::get_console().println("");
    }};
    ($($arg:tt)*) => {{
        let formatted = format!($($arg)*);
        $crate::console::get_console().println_markup(&formatted);
    }};
}

#[macro_export]
macro_rules! reprint {
    ($($arg:tt)*) => {{
        let formatted = format!($($arg)*);
        // Stderr printing with markup
        let console = $crate::console::get_console();
        if let Ok(text) = $crate::markup::render(&formatted) {
            eprint!("{}", $crate::segment::Segment::render_ansi(&text.render_to_segments(), console.color_system));
        } else {
            eprint!("{formatted}");
        }
    }};
}

#[macro_export]
macro_rules! reprintln {
    () => {{
        eprintln!();
    }};
    ($($arg:tt)*) => {{
        $crate::reprint!($($arg)*);
        eprintln!();
    }};
}
