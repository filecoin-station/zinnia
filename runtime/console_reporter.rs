use std::cell::RefCell;
use std::io::{stderr, stdout, Write};
use std::time::{Duration, Instant};

use termcolor::{Ansi, Color, ColorSpec, WriteColor};

use crate::anyhow::Result;
use crate::colors::use_color;
use crate::{LogLevel, Reporter};

#[derive(Debug)]
pub struct JobCompletionTracker {
    delay: Duration,
    counter: u64,
    last_report: Option<(Instant, u64)>,
}

impl JobCompletionTracker {
    pub fn new(initial_value: u64, delay: Duration) -> Self {
        Self {
            delay,
            counter: initial_value,
            last_report: None,
        }
    }

    pub fn counter(&self) -> u64 {
        self.counter
    }

    pub fn job_completed<F: FnOnce(u64)>(&mut self, log: F) -> u64 {
        self.counter += 1;

        if let Some(last) = self.last_report {
            if last.0.elapsed() < self.delay {
                return self.counter;
            }
        }
        self.last_report.replace((Instant::now(), self.counter));

        log(self.counter);

        self.counter
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

/// ConsoleReporter logs activities to stdout and debug logs to stderr
pub struct ConsoleReporter {
    tracker: RefCell<JobCompletionTracker>,
}

impl ConsoleReporter {
    /// Create a new instance.
    ///
    /// `job_report_delay` specifies how often the information about new jobs is printed.
    pub fn new(job_report_delay: Duration) -> Self {
        Self {
            tracker: RefCell::new(JobCompletionTracker::new(0, job_report_delay)),
        }
    }

    fn print_jobs_completed(&self, total: u64) {
        let msg = format!("Jobs completed: {total}");
        self.report("STATS", &msg, Color::Yellow);
    }

    fn report(&self, scope: &str, msg: &str, color: Color) {
        print_report(scope, msg, color).unwrap_or_else(|err| {
            // We are ignoring errors because there isn't much to do in such case
            log::debug!(
                "Cannot report event [scope:{scope} color:{color:?}] {msg:?}: {}",
                err
            )
        })
    }
}

fn print_report(scope: &str, msg: &str, color: Color) -> Result<()> {
    if use_color() {
        let mut spec = ColorSpec::new();
        // spec.set_fg(Some(color)).set_bold(true);
        spec.set_fg(Some(color));
        let mut ansi_writer = Ansi::new(stdout());
        ansi_writer.set_color(&spec)?;
        print_raw_report(&mut ansi_writer, scope, msg)?;
        ansi_writer.reset()?;
    } else {
        print_raw_report(&mut stdout(), scope, msg)?;
    }
    stdout().flush()?;
    Ok(())
}

fn print_raw_report<W: Write>(w: &mut W, scope: &str, msg: &str) -> std::io::Result<()> {
    // Important: activity messages do not include the final newline character
    writeln!(w, "[{} {scope:>5}] {msg}", now_str())
}

impl Drop for ConsoleReporter {
    fn drop(&mut self) {
        self.tracker
            .borrow_mut()
            .flush(|n| self.print_jobs_completed(n));
    }
}

impl Reporter for ConsoleReporter {
    fn log(&self, level: LogLevel, msg: &str) {
        // Important: Console logs already contain the final newline character
        if level <= LogLevel::Info {
            // We are ignoring write errors because there isn't much to do in such case
            let _ = stdout().write_all(msg.as_bytes());
            let _ = stdout().flush();
        } else {
            // We are ignoring write errors because there isn't much to do in such case
            let _ = stderr().write_all(msg.as_bytes());
            let _ = stderr().flush();
        }
    }

    fn info_activity(&self, msg: &str) {
        self.report("INFO", msg, Color::Green);
    }

    fn error_activity(&self, msg: &str) {
        self.report("ERROR", msg, Color::Red);
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

    impl Default for JobCompletionTracker {
        fn default() -> Self {
            Self::new(0, Duration::from_millis(1000))
        }
    }

    #[test]
    fn tracker_prints_first_job_completion() {
        let mut reported = 0;
        let mut tracker = JobCompletionTracker::default();
        tracker.job_completed(|x| reported = x);
        assert_eq!(reported, 1);
    }

    #[test]
    fn tracker_hides_next_job_completion() {
        let mut reported = 0;
        let mut tracker = JobCompletionTracker::default();
        tracker.job_completed(|x| reported = x);
        tracker.job_completed(|x| reported = x);
        assert_eq!(reported, 1);
    }

    #[test]
    fn tracker_prints_new_jobs_after_delay() {
        let mut reported = 0;
        let mut tracker = JobCompletionTracker::new(0, Duration::from_millis(1));
        tracker.job_completed(|x| reported = x);
        std::thread::sleep(Duration::from_millis(2));
        tracker.job_completed(|x| reported = x);
        assert_eq!(reported, 2);
    }

    #[test]
    fn flush_prints_job_completion() {
        let mut reported = 0;
        let mut tracker = JobCompletionTracker::default();
        tracker.job_completed(|_| ());
        tracker.job_completed(|_| ());
        tracker.flush(|x| reported = x);
        assert_eq!(reported, 2);
    }
}
