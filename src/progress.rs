use crate::console::Console;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::spinner_bar::{ProgressBar, Spinner};
use crate::style::Style;
use crate::table::{Column, Table};
use crate::text::Text;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub type TaskId = usize;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub description: String,
    pub total: usize,
    pub completed: usize,
    pub visible: bool,
    pub start_time: Instant,
}

impl Task {
    pub fn new(id: TaskId, description: impl Into<String>, total: usize) -> Self {
        Self {
            id,
            description: description.into(),
            total,
            completed: 0,
            visible: true,
            start_time: Instant::now(),
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.completed as f64 / self.total as f64 * 100.0).min(100.0)
        }
    }

    pub fn is_finished(&self) -> bool {
        self.completed >= self.total
    }
}

pub trait ProgressColumn: Send + Sync {
    fn render(&self, task: &Task) -> Box<dyn Renderable>;
}

pub struct SpinnerColumn {
    pub spinner: Spinner,
}

impl Default for SpinnerColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl SpinnerColumn {
    pub fn new() -> Self {
        Self {
            spinner: Spinner::new(vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"], 80),
        }
    }
}

impl ProgressColumn for SpinnerColumn {
    fn render(&self, _task: &Task) -> Box<dyn Renderable> {
        Box::new(self.spinner.clone())
    }
}

pub struct TextColumn;

impl ProgressColumn for TextColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        Box::new(Text::from_plain(task.description.clone()))
    }
}

pub struct BarColumn {
    pub width: usize,
}

impl Default for BarColumn {
    fn default() -> Self {
        Self::new(20)
    }
}

impl BarColumn {
    pub fn new(width: usize) -> Self {
        Self { width }
    }
}

impl ProgressColumn for BarColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        let mut bar = ProgressBar::new(task.total);
        bar.width = self.width;
        bar.update(task.completed);
        bar.style = if task.is_finished() {
            Style::parse("green").unwrap_or_default()
        } else {
            Style::parse("cyan").unwrap_or_default()
        };
        Box::new(bar)
    }
}

pub struct TaskProgressColumn;

impl ProgressColumn for TaskProgressColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        let pct = format!("{:>3.0}%", task.percentage());
        Box::new(Text::styled(pct, Style::parse("bold").unwrap_or_default()))
    }
}

pub struct TimeElapsedColumn;

impl ProgressColumn for TimeElapsedColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        let secs = task.start_time.elapsed().as_secs();
        let m = secs / 60;
        let s = secs % 60;
        let text = format!("{:02}:{:02}", m, s);
        Box::new(Text::styled(text, Style::parse("dim").unwrap_or_default()))
    }
}

pub struct FileSizeColumn {
    pub binary: bool,
}

impl FileSizeColumn {
    pub fn new() -> Self {
        Self { binary: false }
    }
    pub fn binary(mut self, binary: bool) -> Self {
        self.binary = binary;
        self
    }
}

impl Default for FileSizeColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for FileSizeColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        let text = if self.binary {
            crate::filesize::binary(task.completed as u64)
        } else {
            crate::filesize::decimal(task.completed as u64)
        };
        Box::new(Text::styled(
            text,
            Style::parse("green").unwrap_or_default(),
        ))
    }
}

pub struct TotalFileSizeColumn {
    pub binary: bool,
}

impl TotalFileSizeColumn {
    pub fn new() -> Self {
        Self { binary: false }
    }
    pub fn binary(mut self, binary: bool) -> Self {
        self.binary = binary;
        self
    }
}

impl Default for TotalFileSizeColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for TotalFileSizeColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        let text = if self.binary {
            crate::filesize::binary(task.total as u64)
        } else {
            crate::filesize::decimal(task.total as u64)
        };
        Box::new(Text::styled(text, Style::parse("dim").unwrap_or_default()))
    }
}

pub struct DownloadColumn {
    pub binary: bool,
}

impl DownloadColumn {
    pub fn new() -> Self {
        Self { binary: false }
    }
    pub fn binary(mut self, binary: bool) -> Self {
        self.binary = binary;
        self
    }
}

impl Default for DownloadColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for DownloadColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        let (comp, tot) = if self.binary {
            (
                crate::filesize::binary(task.completed as u64),
                crate::filesize::binary(task.total as u64),
            )
        } else {
            (
                crate::filesize::decimal(task.completed as u64),
                crate::filesize::decimal(task.total as u64),
            )
        };
        let text = format!("{comp}/{tot}");
        Box::new(Text::styled(text, Style::parse("cyan").unwrap_or_default()))
    }
}

pub struct TransferSpeedColumn;

impl ProgressColumn for TransferSpeedColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        let elapsed = task.start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.05 {
            task.completed as f64 / elapsed
        } else {
            0.0
        };
        let text = format!("{}/s", crate::filesize::decimal(speed as u64));
        Box::new(Text::styled(
            text,
            Style::parse("yellow").unwrap_or_default(),
        ))
    }
}

pub struct TimeRemainingColumn;

impl ProgressColumn for TimeRemainingColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        if task.completed == 0 || task.completed >= task.total {
            return Box::new(Text::styled(
                "--:--",
                Style::parse("dim").unwrap_or_default(),
            ));
        }
        let elapsed = task.start_time.elapsed().as_secs_f64();
        let rate = task.completed as f64 / elapsed;
        let remaining_secs =
            (task.total.saturating_sub(task.completed) as f64 / rate).max(0.0) as u64;
        let m = remaining_secs / 60;
        let s = remaining_secs % 60;
        let text = format!("{:02}:{:02}", m, s);
        Box::new(Text::styled(text, Style::parse("dim").unwrap_or_default()))
    }
}

pub struct MofNCompleteColumn {
    pub separator: &'static str,
}

impl MofNCompleteColumn {
    pub fn new() -> Self {
        Self { separator: "/" }
    }
    pub fn separator(mut self, sep: &'static str) -> Self {
        self.separator = sep;
        self
    }
}

impl Default for MofNCompleteColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for MofNCompleteColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        let total_digits = format!("{}", task.total).len();
        let text = format!(
            "{:>width$}{}{}",
            task.completed,
            self.separator,
            task.total,
            width = total_digits
        );
        Box::new(Text::styled(text, Style::parse("bold").unwrap_or_default()))
    }
}

#[derive(Clone)]
pub struct Progress {
    tasks: Arc<Mutex<Vec<Task>>>,
    columns: Arc<Mutex<Vec<Box<dyn ProgressColumn>>>>,
    next_id: Arc<AtomicUsize>,
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl Progress {
    pub fn new() -> Self {
        let columns: Vec<Box<dyn ProgressColumn>> = vec![
            Box::new(SpinnerColumn::new()),
            Box::new(TextColumn),
            Box::new(BarColumn::new(20)),
            Box::new(TaskProgressColumn),
            Box::new(TimeElapsedColumn),
        ];
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            columns: Arc::new(Mutex::new(columns)),
            next_id: Arc::new(AtomicUsize::new(1)),
        }
    }

    pub fn with_columns(columns: Vec<Box<dyn ProgressColumn>>) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            columns: Arc::new(Mutex::new(columns)),
            next_id: Arc::new(AtomicUsize::new(1)),
        }
    }

    pub fn add_column(&self, column: Box<dyn ProgressColumn>) {
        let mut cols = self.columns.lock().unwrap();
        cols.push(column);
    }

    pub fn add_task(&self, description: impl Into<String>, total: usize) -> TaskId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let task = Task::new(id, description, total);
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);
        id
    }

    pub fn update(&self, id: TaskId, completed: usize) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.completed = completed;
        }
    }

    pub fn advance(&self, id: TaskId, step: usize) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.completed = task.completed.saturating_add(step);
        }
    }

    pub fn tasks(&self) -> Vec<Task> {
        self.tasks.lock().unwrap().clone()
    }

    pub fn make_table(&self) -> Table {
        let mut table = Table::grid();
        table.padding = (0, 1);

        let cols = self.columns.lock().unwrap();
        for _ in 0..cols.len() {
            table.add_column(Column::new());
        }

        let tasks = self.tasks.lock().unwrap();
        for task in tasks.iter() {
            if task.visible {
                let row_cells: Vec<Box<dyn Renderable>> =
                    cols.iter().map(|col| col.render(task)).collect();
                table.rows.push(crate::table::Row {
                    cells: row_cells,
                    style: None,
                    end_section: false,
                });
            }
        }

        table
    }
}

impl Renderable for Progress {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let table = self.make_table();
        let segments = table.render(console, options);
        for seg in segments {
            sink.push(seg.into_segment());
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let table = self.make_table();
        table.measure(console, options)
    }
}

/// Wrap an iterator with an automated progress bar display.
pub fn track<I: IntoIterator>(iter: I, description: impl Into<String>) -> TrackIter<I::IntoIter> {
    let into_iter = iter.into_iter();
    let total = match into_iter.size_hint() {
        (_, Some(high)) => high,
        (low, None) => low,
    };

    let progress = Arc::new(Progress::new());
    let task_id = progress.add_task(description, total);
    let live = Arc::new(Mutex::new(
        crate::live::Live::new(progress.clone()).auto_refresh(false),
    ));
    if let Ok(l) = live.lock() {
        l.start();
        l.refresh();
    }

    TrackIter {
        iter: into_iter,
        progress,
        task_id,
        live,
    }
}

pub struct TrackIter<I> {
    iter: I,
    progress: Arc<Progress>,
    task_id: TaskId,
    live: Arc<Mutex<crate::live::Live<Arc<Progress>>>>,
}

impl<I: Iterator> Iterator for TrackIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next();
        if item.is_some() {
            self.progress.advance(self.task_id, 1);
            if let Ok(l) = self.live.lock() {
                l.refresh();
            }
        } else if let Ok(l) = self.live.lock() {
            l.stop();
        }
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for TrackIter<I> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_progress_rendering() {
        let progress = Progress::new();
        let t1 = progress.add_task("Download 1", 100);
        let t2 = progress.add_task("Download 2", 200);

        progress.advance(t1, 50);
        progress.advance(t2, 200);

        let console = Console::with_dimensions(80, 5).with_color_system(ColorSystem::NoColor);
        let out = console.capture(|c| {
            c.print(&progress);
        });

        assert!(out.contains("Download 1"));
        assert!(out.contains("Download 2"));
        assert!(out.contains("50%"));
        assert!(out.contains("100%"));
    }
}
