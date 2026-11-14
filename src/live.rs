use crate::console::Console;
use crate::control::ControlCode;
use crate::protocol::Renderable;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn lock_writer(use_stderr: bool) -> Box<dyn Write> {
    if use_stderr {
        Box::new(io::stderr().lock())
    } else {
        Box::new(io::stdout().lock())
    }
}

fn render_step<T: Renderable>(
    renderable: &Arc<Mutex<T>>,
    console: &Console,
    last_lines: &Arc<Mutex<usize>>,
    use_stderr: bool,
) {
    let r = renderable.lock().unwrap();
    let out_str = console.capture(|c| {
        c.print(&*r);
    });
    drop(r);

    let trimmed_out = out_str.trim_end_matches('\n');
    let lines: Vec<&str> = if trimmed_out.is_empty() {
        Vec::new()
    } else {
        trimmed_out.lines().collect()
    };
    let lines_count = lines.len();

    let mut last = last_lines.lock().unwrap();
    let mut out = lock_writer(use_stderr);

    if *last > 0 {
        let _ = write!(out, "\r\x1b[2K");
        for _ in 1..*last {
            let _ = write!(out, "\x1b[1A\x1b[2K");
        }
    }

    if lines_count > 0 {
        for (idx, line) in lines.iter().enumerate() {
            if idx + 1 < lines_count {
                let _ = writeln!(out, "{line}");
            } else {
                let _ = write!(out, "{line}");
            }
        }
    }

    let _ = out.flush();
    *last = lines_count;
}

pub struct Live<T> {
    renderable: Arc<Mutex<T>>,
    console: Console,
    last_lines: Arc<Mutex<usize>>,
    running: Arc<Mutex<bool>>,
    refresh_rate: Duration,
    transient: bool,
    use_stderr: bool,
}

impl<T: Renderable + Send + 'static> Live<T> {
    pub fn new(renderable: T) -> Self {
        Self {
            renderable: Arc::new(Mutex::new(renderable)),
            console: Console::new(),
            last_lines: Arc::new(Mutex::new(0)),
            running: Arc::new(Mutex::new(false)),
            refresh_rate: Duration::from_millis(100),
            transient: false,
            use_stderr: false,
        }
    }

    pub fn stderr(mut self, stderr: bool) -> Self {
        self.use_stderr = stderr;
        self
    }

    pub fn transient(mut self, transient: bool) -> Self {
        self.transient = transient;
        self
    }

    pub fn refresh_rate(mut self, rate: Duration) -> Self {
        self.refresh_rate = rate;
        self
    }

    pub fn auto_refresh(self, _auto: bool) -> Self {
        self
    }

    pub fn update(&self, new_renderable: T) {
        let mut r = self.renderable.lock().unwrap();
        *r = new_renderable;
    }

    pub fn refresh(&self) {
        render_step(
            &self.renderable,
            &self.console,
            &self.last_lines,
            self.use_stderr,
        );
    }

    pub fn clear(&self) {
        let mut last = self.last_lines.lock().unwrap();
        if *last > 0 {
            let mut out = lock_writer(self.use_stderr);
            let _ = write!(out, "\r\x1b[2K");
            for _ in 1..*last {
                let _ = write!(out, "\x1b[1A\x1b[2K");
            }
            let _ = out.flush();
            *last = 0;
        }
    }

    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;

        let renderable = Arc::clone(&self.renderable);
        let last_lines = Arc::clone(&self.last_lines);
        let running_clone = Arc::clone(&self.running);
        let refresh_rate = self.refresh_rate;
        let use_stderr = self.use_stderr;
        let console = Console::new();

        let mut out = lock_writer(use_stderr);
        let _ = write!(out, "{}", ControlCode::HideCursor);
        let _ = out.flush();
        drop(out);

        thread::spawn(move || {
            while *running_clone.lock().unwrap() {
                render_step(&renderable, &console, &last_lines, use_stderr);
                thread::sleep(refresh_rate);
            }
        });
    }

    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return;
        }
        *running = false;

        let mut out = lock_writer(self.use_stderr);
        let mut last = self.last_lines.lock().unwrap();
        if self.transient {
            if *last > 0 {
                let _ = write!(out, "\r\x1b[2K");
                for _ in 1..*last {
                    let _ = write!(out, "\x1b[1A\x1b[2K");
                }
            }
        } else if *last > 0 {
            let _ = writeln!(out);
        }
        let _ = write!(out, "{}", ControlCode::ShowCursor);
        let _ = out.flush();
        *last = 0;
    }
}

impl<T> Drop for Live<T> {
    fn drop(&mut self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            *running = false;
            let mut out = lock_writer(self.use_stderr);
            let _ = write!(out, "{}", ControlCode::ShowCursor);
            let _ = out.flush();
        }
    }
}
