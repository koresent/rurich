use crate::console::Console;
use crate::control::ControlCode;
use crate::protocol::Renderable;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
        let r = self.renderable.lock().unwrap();
        let out = self.console.capture(|c| {
            c.print(&*r);
        });
        drop(r);

        let trimmed_out = out.trim_end_matches('\n');
        let lines_count = if trimmed_out.is_empty() {
            0
        } else {
            trimmed_out.lines().count()
        };
        let mut last = self.last_lines.lock().unwrap();

        let mut out: Box<dyn Write> = if self.use_stderr {
            Box::new(io::stderr().lock())
        } else {
            Box::new(io::stdout().lock())
        };
        if *last > 0 {
            let _ = write!(out, "\r\x1b[{}A\x1b[0J", *last);
        }

        let _ = writeln!(out, "{trimmed_out}");
        let _ = out.flush();
        *last = lines_count;
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

        let mut out: Box<dyn Write> = if use_stderr {
            Box::new(io::stderr().lock())
        } else {
            Box::new(io::stdout().lock())
        };
        let _ = write!(out, "{}", ControlCode::HideCursor);
        let _ = out.flush();
        drop(out);

        thread::spawn(move || {
            while *running_clone.lock().unwrap() {
                let r = renderable.lock().unwrap();
                let out_str = console.capture(|c| {
                    c.print(&*r);
                });
                drop(r);

                let trimmed_out = out_str.trim_end_matches('\n');
                let lines_count = if trimmed_out.is_empty() {
                    0
                } else {
                    trimmed_out.lines().count()
                };
                let mut last = last_lines.lock().unwrap();

                let mut out: Box<dyn Write> = if use_stderr {
                    Box::new(io::stderr().lock())
                } else {
                    Box::new(io::stdout().lock())
                };
                if *last > 0 {
                    let _ = write!(out, "\r\x1b[{}A\x1b[0J", *last);
                }
                let _ = writeln!(out, "{trimmed_out}");
                let _ = out.flush();
                *last = lines_count;
                drop(out);
                drop(last);

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

        let mut out: Box<dyn Write> = if self.use_stderr {
            Box::new(io::stderr().lock())
        } else {
            Box::new(io::stdout().lock())
        };
        let _ = write!(out, "{}", ControlCode::ShowCursor);
        if self.transient {
            let last = *self.last_lines.lock().unwrap();
            if last > 0 {
                let _ = write!(out, "\r\x1b[{}A\x1b[0J", last);
            }
        }
        let _ = out.flush();
    }
}

impl<T> Drop for Live<T> {
    fn drop(&mut self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            *running = false;
            let mut out: Box<dyn Write> = if self.use_stderr {
                Box::new(io::stderr().lock())
            } else {
                Box::new(io::stdout().lock())
            };
            let _ = write!(out, "{}", ControlCode::ShowCursor);
            let _ = out.flush();
        }
    }
}
