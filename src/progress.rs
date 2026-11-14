use crate::console::Console;
use crate::markup::render as render_markup;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::spinner_bar::{ProgressBar, Spinner, SpinnerPreset};
use crate::style::Style;
use crate::table::{Column, Table};
use crate::text::Text;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
    pub finished_text: Option<String>,
    pub style: Option<Style>,
}

impl Default for SpinnerColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl SpinnerColumn {
    pub fn new() -> Self {
        Self {
            spinner: Spinner::new(vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"], 80)
                .style(Style::parse("bold cyan").unwrap_or_default()),
            finished_text: Some("[bold green]✓[/]".to_string()),
            style: Some(Style::parse("bold cyan").unwrap_or_default()),
        }
    }

    pub fn with_preset(preset: SpinnerPreset) -> Self {
        Self {
            spinner: Spinner::from_preset(preset)
                .style(Style::parse("bold cyan").unwrap_or_default()),
            finished_text: Some("[bold green]✓[/]".to_string()),
            style: Some(Style::parse("bold cyan").unwrap_or_default()),
        }
    }

    pub fn with_spinner(spinner: Spinner) -> Self {
        Self {
            spinner,
            finished_text: Some("[bold green]✓[/]".to_string()),
            style: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn finished_text(mut self, text: impl Into<String>) -> Self {
        self.finished_text = Some(text.into());
        self
    }
}

impl ProgressColumn for SpinnerColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        if task.is_finished() {
            let text = self.finished_text.as_deref().unwrap_or("✓");
            let renderable =
                render_markup(text).unwrap_or_else(|_| Text::from_plain(text.to_string()));
            Box::new(renderable)
        } else {
            let mut s = self.spinner.clone();
            if let Some(ref st) = self.style {
                s.style = Some(st.clone());
            }
            Box::new(s)
        }
    }
}

pub struct TextColumn;

impl ProgressColumn for TextColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        let text = render_markup(&task.description)
            .unwrap_or_else(|_| Text::from_plain(task.description.clone()));
        Box::new(text)
    }
}

pub struct BarColumn {
    pub width: usize,
    pub smooth: bool,
    pub pulse: bool,
    pub line_mode: bool,
    pub complete_char: Option<char>,
    pub remaining_char: Option<char>,
    pub complete_style: Option<Style>,
    pub finished_style: Option<Style>,
    pub remaining_style: Option<Style>,
}

impl Default for BarColumn {
    fn default() -> Self {
        Self::new(20)
    }
}

impl BarColumn {
    pub fn new(width: usize) -> Self {
        Self {
            width,
            smooth: true,
            pulse: false,
            line_mode: true,
            complete_char: None,
            remaining_char: None,
            complete_style: Some(Style::parse("color(#f92672)").unwrap_or_default()),
            finished_style: Some(Style::parse("color(#729c1f)").unwrap_or_default()),
            remaining_style: Some(Style::parse("color(#3a3a3a)").unwrap_or_default()),
        }
    }

    pub fn line(mut self, line: bool) -> Self {
        self.line_mode = line;
        self
    }

    pub fn smooth_blocks(mut self) -> Self {
        self.line_mode = false;
        self.smooth = true;
        self
    }

    pub fn blocks(mut self) -> Self {
        self.line_mode = false;
        self.smooth = false;
        self
    }

    pub fn smooth(mut self, smooth: bool) -> Self {
        self.smooth = smooth;
        self
    }

    pub fn pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }

    pub fn complete_char(mut self, ch: char) -> Self {
        self.complete_char = Some(ch);
        self.line_mode = false;
        self
    }

    pub fn remaining_char(mut self, ch: char) -> Self {
        self.remaining_char = Some(ch);
        self.line_mode = false;
        self
    }

    pub fn complete_style(mut self, style: Style) -> Self {
        self.complete_style = Some(style);
        self
    }

    pub fn finished_style(mut self, style: Style) -> Self {
        self.finished_style = Some(style);
        self
    }

    pub fn remaining_style(mut self, style: Style) -> Self {
        self.remaining_style = Some(style);
        self
    }
}

impl ProgressColumn for BarColumn {
    fn render(&self, task: &Task) -> Box<dyn Renderable> {
        let mut bar = ProgressBar::new(task.total)
            .width(self.width)
            .smooth(self.smooth)
            .pulse(self.pulse)
            .line(self.line_mode);

        if let Some(ch) = self.complete_char {
            bar = bar.complete_char(ch);
            bar.line_mode = false;
        }
        if let Some(ch) = self.remaining_char {
            bar = bar.remaining_char(ch);
            bar.line_mode = false;
        }

        bar.update(task.completed);

        if task.is_finished() {
            let finish_st = self
                .finished_style
                .clone()
                .or_else(|| self.complete_style.clone())
                .unwrap_or_else(|| {
                    if self.line_mode {
                        Style::parse("color(#729c1f)").unwrap_or_default()
                    } else {
                        Style::parse("bold green").unwrap_or_default()
                    }
                });
            bar = bar.complete_style(finish_st);
        } else {
            let comp_st = self.complete_style.clone().unwrap_or_else(|| {
                if self.line_mode {
                    Style::parse("color(#f92672)").unwrap_or_default()
                } else {
                    Style::parse("bold cyan").unwrap_or_default()
                }
            });
            bar = bar.complete_style(comp_st);
        }

        let rem_st = self.remaining_style.clone().unwrap_or_else(|| {
            if self.line_mode {
                Style::parse("color(#3a3a3a)").unwrap_or_default()
            } else {
                Style::parse("dim").unwrap_or_default()
            }
        });
        bar = bar.remaining_style(rem_st);

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
    live: Arc<Mutex<Option<crate::live::Live<Progress>>>>,
    transient: Arc<AtomicBool>,
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
            live: Arc::new(Mutex::new(None)),
            transient: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_columns(columns: Vec<Box<dyn ProgressColumn>>) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            columns: Arc::new(Mutex::new(columns)),
            next_id: Arc::new(AtomicUsize::new(1)),
            live: Arc::new(Mutex::new(None)),
            transient: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn transient(self, transient: bool) -> Self {
        self.transient.store(transient, Ordering::SeqCst);
        self
    }

    pub fn start(&self) {
        let mut live_lock = self.live.lock().unwrap();
        if live_lock.is_none() {
            let live = crate::live::Live::new(self.clone())
                .refresh_rate(std::time::Duration::from_millis(80))
                .transient(self.transient.load(Ordering::SeqCst));
            live.start();
            *live_lock = Some(live);
        }
    }

    pub fn stop(&self) {
        let mut live_lock = self.live.lock().unwrap();
        if let Some(live) = live_lock.take() {
            live.stop();
        }
    }

    pub fn refresh(&self) {
        let live_lock = self.live.lock().unwrap();
        if let Some(ref live) = *live_lock {
            live.refresh();
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
    progress.start();

    TrackIter {
        iter: into_iter,
        progress,
        task_id,
    }
}

pub struct TrackIter<I> {
    iter: I,
    progress: Arc<Progress>,
    task_id: TaskId,
}

impl<I: Iterator> Iterator for TrackIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next();
        if item.is_some() {
            self.progress.advance(self.task_id, 1);
            self.progress.refresh();
        } else {
            self.progress.stop();
        }
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for TrackIter<I> {}

impl<I> Drop for TrackIter<I> {
    fn drop(&mut self) {
        self.progress.stop();
    }
}

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
