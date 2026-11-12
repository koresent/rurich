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
        }
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

        let lines: Vec<&str> = out.lines().collect();
        let mut last = self.last_lines.lock().unwrap();

        let mut stdout = io::stdout().lock();
        if *last > 0 {
            let _ = write!(stdout, "{}", ControlCode::CarriageReturn);
            for _ in 0..*last {
                let _ = write!(
                    stdout,
                    "{}{}",
                    ControlCode::CursorUp(1),
                    ControlCode::EraseInLine(2)
                );
            }
        }

        let _ = writeln!(stdout, "{out}");
        let _ = stdout.flush();
        *last = lines.len();
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
        let console = Console::new();

        let mut stdout = io::stdout().lock();
        let _ = write!(stdout, "{}", ControlCode::HideCursor);
        let _ = stdout.flush();
        drop(stdout);

        thread::spawn(move || {
            while *running_clone.lock().unwrap() {
                let r = renderable.lock().unwrap();
                let out = console.capture(|c| {
                    c.print(&*r);
                });
                drop(r);

                let lines: Vec<&str> = out.lines().collect();
                let mut last = last_lines.lock().unwrap();

                let mut stdout = io::stdout().lock();
                if *last > 0 {
                    let _ = write!(stdout, "{}", ControlCode::CarriageReturn);
                    for _ in 0..*last {
                        let _ = write!(
                            stdout,
                            "{}{}",
                            ControlCode::CursorUp(1),
                            ControlCode::EraseInLine(2)
                        );
                    }
                }
                let _ = writeln!(stdout, "{out}");
                let _ = stdout.flush();
                *last = lines.len();

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

        let mut stdout = io::stdout().lock();
        let _ = write!(stdout, "{}", ControlCode::ShowCursor);
        if self.transient {
            let last = *self.last_lines.lock().unwrap();
            if last > 0 {
                let _ = write!(stdout, "{}", ControlCode::CarriageReturn);
                for _ in 0..last {
                    let _ = write!(
                        stdout,
                        "{}{}",
                        ControlCode::CursorUp(1),
                        ControlCode::EraseInLine(2)
                    );
                }
            }
        }
        let _ = stdout.flush();
    }
}

impl<T> Drop for Live<T> {
    fn drop(&mut self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            *running = false;
            let mut stdout = io::stdout().lock();
            let _ = write!(stdout, "{}", ControlCode::ShowCursor);
            let _ = stdout.flush();
        }
    }
}
