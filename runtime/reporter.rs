use std::cell::RefCell;

// Report events, activities and messages from the running module
pub trait Reporter {
    /// Print a debug log message. This is typically triggered by Console APIs like `console.log`.
    /// Important: these messages include the final newline, the reporter SHOULD NOT add EOL.
    fn debug_print(&self, msg: &str);

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
    fn debug_print(&self, msg: &str) {
        self.record(format!("DEBUG: {msg}"));
    }

    fn info_activity(&self, msg: &str) {
        self.record(format!("INFO: {msg}"));
    }

    fn error_activity(&self, msg: &str) {
        self.record(format!("ERROR: {msg}"));
    }

    fn job_completed(&self) {
        self.record("JOB-COMPLETED".into());
    }
}
