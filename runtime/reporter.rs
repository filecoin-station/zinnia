use std::cell::RefCell;
use std::io::{stderr, stdout, Write};
use std::time::{Duration, Instant};

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

const JOBS_REPORT_DELAY: Duration = Duration::from_millis(200);

#[derive(Debug)]
struct JobCompletionTracker {
    delay: Duration,
    counter: u64,
    last_report: Option<(Instant, u64)>,
}

impl JobCompletionTracker {
    pub fn new(delay: Duration) -> Self {
        Self {
            delay,
            counter: 0,
            last_report: None,
        }
    }

    pub fn job_completed<F: FnOnce(u64)>(&mut self, log: F) {
        self.counter += 1;

        if let Some(last) = self.last_report {
            if last.0.elapsed() < self.delay {
                return;
            }
        }
        self.last_report.replace((Instant::now(), self.counter));

        log(self.counter);
    }

    pub fn flush<F: FnOnce(u64)>(&mut self, log: F) {
        match self.last_report {
            None => {
                // no jobs were completed, nothing to report
            }
            Some((_, last_total)) => {
                if last_total != self.counter {
                    // new jobs were completed since the last report
                    log(self.counter);
                }
            }
        }
    }
}

pub struct ConsoleReporter {
    tracker: RefCell<JobCompletionTracker>,
}

impl ConsoleReporter {
    pub fn new() -> Self {
        Self {
            tracker: RefCell::new(JobCompletionTracker::new(JOBS_REPORT_DELAY)),
        }
    }

    fn print_jobs_completed(&self, total: u64) {
        let msg = format!("Jobs completed: {total}");
        self.report("STATS", &msg);
    }

    fn report(&self, scope: &str, msg: &str) {
        // Important: activity messages do not include the final newline character
        let msg = format!("[{} {scope:5}] {msg}\n", now_str());
        // We are ignoring write errors because there isn't much to do in such case
        let _ = stdout().write_all(msg.as_bytes());
        let _ = stdout().flush();
    }
}

impl Default for ConsoleReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ConsoleReporter {
    fn drop(&mut self) {
        self.tracker
            .borrow_mut()
            .flush(|n| self.print_jobs_completed(n));
    }
}

impl Reporter for ConsoleReporter {
    fn debug_print(&self, msg: &str) {
        // Important: debug logs already contain the final newline character
        // We are ignoring write errors because there isn't much to do in such case
        let _ = stderr().write_all(msg.as_bytes());
        let _ = stderr().flush();
    }

    fn info_activity(&self, msg: &str) {
        self.report("INFO", msg)
    }

    fn error_activity(&self, msg: &str) {
        self.report("ERROR", msg)
    }

    fn job_completed(&self) {
        self.tracker
            .borrow_mut()
            .job_completed(|n| self.print_jobs_completed(n));
    }
}

fn now_str() -> impl std::fmt::Display {
    let now = chrono::Local::now();
    now.time().format("%H:%M:%S%.3f")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_prints_first_job_completion() {
        let mut reported = 0;
        let mut tracker = JobCompletionTracker::new(JOBS_REPORT_DELAY);
        tracker.job_completed(|x| reported = x);
        assert_eq!(reported, 1);
    }

    #[test]
    fn tracker_hides_next_job_completion() {
        let mut reported = 0;
        let mut tracker = JobCompletionTracker::new(JOBS_REPORT_DELAY);
        tracker.job_completed(|x| reported = x);
        tracker.job_completed(|x| reported = x);
        assert_eq!(reported, 1);
    }

    #[test]
    fn tracker_prints_new_jobs_after_delay() {
        let mut reported = 0;
        let mut tracker = JobCompletionTracker::new(Duration::from_millis(1));
        tracker.job_completed(|x| reported = x);
        std::thread::sleep(Duration::from_millis(2));
        tracker.job_completed(|x| reported = x);
        assert_eq!(reported, 2);
    }

    #[test]
    fn flush_prints_job_completion() {
        let mut reported = 0;
        let mut tracker = JobCompletionTracker::new(JOBS_REPORT_DELAY);
        tracker.job_completed(|_| ());
        tracker.job_completed(|_| ());
        tracker.flush(|x| reported = x);
        assert_eq!(reported, 2);
    }
}
