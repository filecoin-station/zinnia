use std::cell::RefCell;
use std::io::{stderr, stdout, Write};
use std::path::PathBuf;
use std::time::Duration;

use serde_json::{json, Map};
use zinnia_runtime::anyhow::Result;
use zinnia_runtime::{JobCompletionTracker, LogLevel, Reporter};

use crate::state::{self, State};

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
        let initial_job_count = state::load(&state_file).total_jobs_completed;

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

        let _ = print_event(&event);
        // ^^^ We are ignoring errors because there isn't much to do in such case
    }
}

fn print_event(data: &serde_json::Value) -> Result<()> {
    writeln!(stdout(), "{data}")?;
    stdout().flush()?;
    Ok(())
}

pub fn log_info_activity(msg: &str) {
    let event = json!({
        "type": "activity:info",
        "module": serde_json::Value::Null,
        "message": msg,
    });
    let _ = print_event(&event);
    // ^^^ We are ignoring errors because there isn't much to do in such case
}

#[allow(unused)]
pub fn log_error_activity(msg: &str) {
    let event = json!({
        "type": "activity:error",
        "module": serde_json::Value::Null,
        "message": msg,
    });
    let _ = print_event(&event);
    // ^^^ We are ignoring errors because there isn't much to do in such case
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
        let _ = print_event(&event);
        // ^^^ We are ignoring errors because there isn't much to do in such case
    }

    fn error_activity(&self, msg: &str) {
        let event = json!({
            "type": "activity:error",
            "module": self.module_name,
            "message": msg,
        });
        let _ = print_event(&event);
        // ^^^ We are ignoring errors because there isn't much to do in such case
    }

    fn job_completed(&self) {
        let total_jobs_completed = self
            .tracker
            .borrow_mut()
            .job_completed(|n| self.print_jobs_completed(n));

        let state = State {
            total_jobs_completed,
        };
        state::store(&self.state_file, &state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile;

    const NO_DELAY: Duration = Duration::from_millis(0);

    #[test]
    fn persists_job_counter() -> Result<()> {
        let state_dir = tempfile::tempdir()?;
        let state_file = state_dir.path().join("state.json");
        let reporter = StationReporter::new(state_file.clone(), NO_DELAY, "test".into());
        assert_eq!(reporter.tracker.borrow().counter(), 0, "initial count");

        reporter.job_completed();
        println!("{:?}", reporter.tracker);
        assert_eq!(
            reporter.tracker.borrow().counter(),
            1,
            "count after a job was completed"
        );

        let reporter = StationReporter::new(state_file, NO_DELAY, "test".into());
        println!("{:?}", reporter.tracker);
        assert_eq!(
            reporter.tracker.borrow().counter(),
            1,
            "count after creating a new reporter"
        );

        Ok(())
    }
}
