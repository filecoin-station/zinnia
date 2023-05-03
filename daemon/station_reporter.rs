use std::cell::RefCell;
use std::io::{stderr, stdout, Write};
use std::path::PathBuf;
use std::time::Duration;

use serde_json::{json, Map};
use zinnia_runtime::{JobCompletionTracker, LogLevel, Reporter};

use crate::state::State;

/// StationReporter reports activities to stdout as ND-JSON stream and all Console logs to stderr
pub struct StationReporter {
    tracker: RefCell<JobCompletionTracker>,
    module_name: String,
    log_target: String,
    state_file: PathBuf,
}

impl StationReporter {
    /// Create a new instance.
    ///
    /// `job_report_delay` specifies how often the information about new jobs is printed.
    pub fn new(state_file: PathBuf, job_report_delay: Duration, module_name: String) -> Self {
        let log_target = format!("module:{module_name}");
        let initial_job_count = State::load(&state_file)
            // NOTE(bajtos) We are intentionally calling unwrap() to crash the process in case
            // it's not possible to read the state file or parse the content.
            .unwrap()
            .total_jobs_completed;

        Self {
            tracker: RefCell::new(JobCompletionTracker::new(
                initial_job_count,
                job_report_delay,
            )),
            module_name,
            log_target,
            state_file,
        }
    }

    fn print_jobs_completed(&self, total: u64) {
        // TODO: print data from all modules
        // https://github.com/filecoin-station/zinnia/issues/144
        // modules: {"saturn": 100, "retrieval-checker": 23}}
        let mut modules = Map::new();
        modules.insert(self.module_name.clone(), json!(total));

        let event = json!({
            "type": "jobs-completed",
            "total": total,
            "modules": modules,
        });

        print_event(&event);
    }
}

fn print_event(data: &serde_json::Value) {
    writeln!(stdout(), "{data}")
        .and_then(|_| stdout().flush())
        .unwrap_or_else(|err| {
            // We are ignoring errors because there isn't much to do in such case
            log::debug!("Cannot print event {}: {}", data, err);
        });
}

pub fn log_info_activity(msg: &str) {
    let event = json!({
        "type": "activity:info",
        "module": serde_json::Value::Null,
        "message": msg,
    });
    print_event(&event);
}

#[allow(unused)]
pub fn log_error_activity(msg: &str) {
    let event = json!({
        "type": "activity:error",
        "module": serde_json::Value::Null,
        "message": msg,
    });
    print_event(&event);
}

impl Drop for StationReporter {
    fn drop(&mut self) {
        self.tracker
            .borrow_mut()
            .flush(|n| self.print_jobs_completed(n));
    }
}

impl Reporter for StationReporter {
    fn log(&self, level: LogLevel, msg: &str) {
        // Important: Console logs already contain the final newline character
        // We print all Console logs to stderr, because stdout is reserved for activity events
        // We are ignoring write errors because there isn't much to do in such case
        log::log!(target: &self.log_target, level.into(), "{}", msg.trim_end());
        // let _ = write!(stderr(), "[{level:>5}] {msg}");
        let _ = stderr().flush();
    }

    fn info_activity(&self, msg: &str) {
        let event = json!({
            "type": "activity:info",
            "module": self.module_name,
            "message": msg,
        });
        print_event(&event);
    }

    fn error_activity(&self, msg: &str) {
        let event = json!({
            "type": "activity:error",
            "module": self.module_name,
            "message": msg,
        });
        print_event(&event);
    }

    fn job_completed(&self) {
        let total_jobs_completed = self
            .tracker
            .borrow_mut()
            .job_completed(|n| self.print_jobs_completed(n));

        let state = State {
            total_jobs_completed,
        };
        state
            .store(&self.state_file)
            // NOTE(bajtos) We are intentionally calling unwrap() to crash the process in case
            // we cannot store the state into the file.
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;
    use zinnia_runtime::anyhow::Result;

    const NO_DELAY: Duration = Duration::from_millis(0);

    #[test]
    fn persists_job_counter() -> Result<()> {
        let state_dir = tempdir()?;
        let state_file = state_dir.path().join("state.json");
        let reporter = StationReporter::new(state_file.clone(), NO_DELAY, "test".into());
        assert_eq!(reporter.tracker.borrow().counter(), 0, "initial count");

        reporter.job_completed();
        assert_eq!(
            reporter.tracker.borrow().counter(),
            1,
            "count after a job was completed"
        );

        let reporter = StationReporter::new(state_file, NO_DELAY, "test".into());
        assert_eq!(
            reporter.tracker.borrow().counter(),
            1,
            "count after creating a new reporter"
        );

        Ok(())
    }
}
