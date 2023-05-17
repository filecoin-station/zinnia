use std::cell::RefCell;
use std::fmt::Display;

use serde_repr::Deserialize_repr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize_repr)]
#[repr(u8)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        };
        f.write_str(str)
    }
}

impl From<LogLevel> for log::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
        }
    }
}

// Report events, activities and messages from the running module
pub trait Reporter {
    /// Print a debug log message. This is typically triggered by Console APIs like `console.log`.
    /// Important: these messages include the final newline, the reporter SHOULD NOT add EOL.
    fn log(&self, level: LogLevel, msg: &str);

    /// Record an activity log entry with level "info".
    /// Important: this message DOES NOT include the final newline. The reporter should add EOL.
    fn info_activity(&self, msg: &str);

    /// Record an activity log entry with level "error".
    /// Important: this message DOES NOT include the final newline. The reporter should add EOL.
    fn error_activity(&self, msg: &str);

    /// Report that module completed another job.
    fn job_completed(&self);
}

/// Reporter that collects all recorded events, useful for testing.
pub struct RecordingReporter {
    pub events: RefCell<Vec<String>>,
}

impl RecordingReporter {
    pub fn new() -> Self {
        Self {
            events: RefCell::new(Vec::new()),
        }
    }

    fn record(&self, event: String) {
        self.events.borrow_mut().push(event)
    }
}

impl Default for RecordingReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for RecordingReporter {
    fn log(&self, level: LogLevel, msg: &str) {
        print!("{msg}");
        self.record(format!("console.{level}: {msg}"));
    }

    fn info_activity(&self, msg: &str) {
        println!("INFO: {msg}");
        self.record(format!("INFO: {msg}"));
    }

    fn error_activity(&self, msg: &str) {
        println!("ERROR: {msg}");
        self.record(format!("ERROR: {msg}"));
    }

    fn job_completed(&self) {
        println!("JOB-COMPLETED");
        self.record("JOB-COMPLETED".into());
    }
}
