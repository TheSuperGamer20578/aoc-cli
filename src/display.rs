use std::collections::HashMap;
use std::fmt::{Debug, Display, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Duration;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use anyhow::{anyhow, Result};
use console::{Alignment, pad_str, Style};
use crossbeam::atomic::AtomicCell;
use lazy_static::lazy_static;
use pyo3::{PyErr, PyResult, Python};
use tracing::{Event, Id, Level, Metadata, Subscriber, subscriber};
use tracing::field::{Field, Visit};
use tracing::metadata::LevelFilter;
use tracing::span::{Attributes, Record};
use crate::value_enum;

const INDENT: u8 = 12;

lazy_static! {
    static ref PROGRESS: MultiProgress = MultiProgress::new();
}

macro_rules! eprintln_safe {
    ($($arg:tt)*) => {
        $crate::display::PROGRESS.suspend(|| {
            eprintln!($($arg)*);
        });
    };
}

pub fn progress_bar(action: String, action_type: ActionType, len: u64) -> Result<ProgressBar> {
    let bar = PROGRESS.add(ProgressBar::new(len))
        .with_style(ProgressStyle::default_bar()
            .progress_chars("=>·")
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template(&"{prefix:>INDENT} [{elapsed}] (eta {eta}) {spinner:.green} [{bar:40.green/red.dim}] {pos:>4}/{len:4} {msg}".replace("INDENT", &INDENT.to_string()))?
        );
    bar.set_prefix(action_type.value().apply_to(action).to_string());
    bar.enable_steady_tick(Duration::from_millis(50));
    Ok(bar)
}

pub fn spinner(action: String, action_type: ActionType, message: String) -> Result<ProgressBar> {
    let bar = PROGRESS.add(ProgressBar::new_spinner())
        .with_style(ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template(&"{prefix:>INDENT} {spinner:.green} {msg} [{elapsed}]".replace("INDENT", &INDENT.to_string()))?
        );
    bar.set_prefix(action_type.value().apply_to(action).to_string());
    bar.set_message(message);
    bar.enable_steady_tick(Duration::from_millis(50));
    Ok(bar)
}

pub fn confirm(prompt: impl Into<String>) -> Result<bool> {
    Ok(PROGRESS.suspend(|| dialoguer::Confirm::new()
        .with_prompt(prompt)
        .interact())?)
}

value_enum! {
    #[derive(Copy, Clone, Debug)]
    pub enum ActionType: Style {
        Success = Style::new()
            .green()
            .bright()
            .bold(),
        Failure = Style::new()
            .red()
            .bright()
            .bold(),
        Error = Style::new()
            .red()
            .bold(),
        Warning = Style::new()
            .yellow()
            .bold(),
        Info = Style::new()
            .blue()
            .bold(),
        Debug = Style::new()
            .magenta(),
        Trace = Style::new()
            .cyan(),
        Progress = Style::new()
            .green()
            .bold(),
        Prepare = Style::new()
            .green(),
    }
}

impl From<Level> for ActionType {
    fn from(level: Level) -> Self {
        match level {
            Level::ERROR => Self::Error,
            Level::WARN => Self::Warning,
            Level::INFO => Self::Info,
            Level::DEBUG => Self::Debug,
            Level::TRACE => Self::Trace,
        }
    }
}

pub fn println(action: impl Display, action_type: ActionType, msg: impl Display) {
    eprintln_safe!("{} {msg}", action_type.value().apply_to(pad_str(&action.to_string(), INDENT as usize, Alignment::Right, None)));
}

fn format_debug_info(field: impl Display, value: impl Display) -> String {
    let leading = pad_str("└──", INDENT as usize + 1, Alignment::Right, None);
    let field = format!("{field}");
    let field = ActionType::Debug.value().apply_to(pad_str(&field, 10, Alignment::Right, None));
    format!("{leading} {field} {value}")
}

fn format_debug_kv(field: impl Display, value: impl Debug) -> String {
    format_debug_info(field, format!("{} {value:#?}", Style::new().bright().bold().apply_to("=")))
}

struct Visitor<'a>(&'a mut String);

impl<'a> Visit for Visitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            writeln!(self.0, "{value:?}").ok();
        } else if cfg!(debug_assertions) {
            writeln!(self.0, "{}", format_debug_kv(field.name(), value)).ok();
        }
    }
}

struct LoggerSpan {
    name: String,
    spinners: AtomicCell<Option<ProgressBar>>,
    #[cfg(debug_assertions)] parent: Option<Id>,
}

impl LoggerSpan {
    pub fn new(name: String, #[cfg(debug_assertions)] parent: Option<Id>) -> Self {
        Self {
            name,
            spinners: AtomicCell::new(None),
            #[cfg(debug_assertions)] parent,
        }
    }
}

pub struct Logger {
    pub level: LevelFilter,
    next_id: AtomicU64,
    spans: RwLock<HashMap<Id, LoggerSpan>>,
}

impl Logger {
    pub fn new(level: LevelFilter) -> Self {
        Self {
            level,
            next_id: AtomicU64::new(1),
            spans: RwLock::new(HashMap::new()),
        }
    }

    pub fn init(self) -> Result<()> {
        subscriber::set_global_default(self)?;
        Ok(())
    }
}

impl Subscriber for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        *metadata.level() <= self.level
    }

    fn new_span(&self, span: &Attributes<'_>) -> Id {
        let id = Id::from_u64(self.next_id.fetch_add(1, Ordering::Relaxed));
        let name = span.metadata().name().to_string();
        self.spans.write().unwrap().insert(id.clone(), LoggerSpan::new(name, #[cfg(debug_assertions)] span.parent().cloned()));
        id
    }

    fn record(&self, _span: &Id, _values: &Record<'_>) {}

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut message = String::new();
        event.record(&mut Visitor(&mut message));
        println(
            event.metadata().level(),
            (*event.metadata().level()).into(),
            message,
        );
        #[cfg(debug_assertions)]
        PROGRESS.suspend(|| {
            if let Some(module) = event.metadata().module_path() {
                eprintln!("{}", format_debug_info("Module", module));
            }
            else if let Some(file) = event.metadata().file() {
                eprintln!("{}", format_debug_info("File", file));
            }
            else if let Some(line) = event.metadata().line() {
                eprintln!("{}", format_debug_info("Line", line));
            }
            if let (Some(file), Some(line)) = (event.metadata().file(), event.metadata().line()) {
                eprintln!("{}", format_debug_info("At", format!("{file}:{line}")));
            }
            let spans = self.spans.read().unwrap();
            let mut parent = event.parent();
            while let Some(span) = parent {
                let span = spans.get(span).unwrap();
                eprintln!("{}", format_debug_info("In", &span.name));
                parent = span.parent.as_ref();
            }
            eprintln!();
        });
    }

    fn enter(&self, span: &Id) {
        let spans = self.spans.read().unwrap();
        let spinner = spinner(
            "Running".to_string(),
            ActionType::Progress,
            spans.get(span).unwrap().name.clone(),
        ).unwrap();
        spans.get(span).unwrap().spinners.store(Some(spinner));
    }

    fn exit(&self, span: &Id) {
        let spans = self.spans.read().unwrap();
        spans.get(span).unwrap().spinners.swap(None).unwrap().finish();
    }
}

pub trait FormatTraceBack<T> {
    fn tb(self) -> Result<T>;
}

fn format_trace_back(error: PyErr) -> String {
    // TODO: Add fancy formatting
    Python::with_gil(|py| {
        py.import("traceback").unwrap()
            .getattr("format_exception").unwrap()
            .call1((error,)).unwrap()
            .extract::<Vec<String>>().unwrap()
            .join("")
    })
}

impl<T> FormatTraceBack<T> for PyResult<T> {
    fn tb(self) -> Result<T> {
        match self {
            Ok(result) => Ok(result),
            Err(error) => Err(anyhow!("Uncaught python exception:\n\n{}", format_trace_back(error))),
        }
    }
}
