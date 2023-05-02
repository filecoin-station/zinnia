use std::process::Output;

use assert_cmd::Command;
use assert_fs::prelude::*;
use lazy_static::lazy_static;
use pretty_assertions::assert_eq;
use regex::Regex;
use rexpect;

use rexpect::process::wait::WaitStatus;
use zinnia_runtime::anyhow::Context;
use zinnia_runtime::resolve_path;

#[test]
fn run_js_module() -> Result<(), Box<dyn std::error::Error>> {
    let mod_js = assert_fs::NamedTempFile::new("hello-mod.js")?;
    mod_js.write_str(
        r#"
console.log("console.log");
console.error("console.error");
Zinnia.activity.info("activity.info");
Zinnia.activity.error("activity.error");
Zinnia.jobCompleted();
"#,
    )?;

    let output = Command::cargo_bin("zinnia")?
        .env("NO_COLOR", "1")
        .args(["run", &mod_js.path().display().to_string()])
        .output()?;

    assert_eq!(
        CmdResult::from(&output).map_stdout(strip_timestamps),
        CmdResult {
            exit_ok: true,
            stdout: [
                "console.log\n",
                "[TIMESTAMP  INFO] activity.info\n",
                "[TIMESTAMP ERROR] activity.error\n",
                "[TIMESTAMP STATS] Jobs completed: 1\n",
            ]
            .join(""),
            stderr: "console.error\n".into(),
        }
    );

    Ok(())
}

#[test]
fn report_js_stack_trace() -> Result<(), Box<dyn std::error::Error>> {
    let mod_js = assert_fs::NamedTempFile::new("error-mod.js")?;
    mod_js.write_str(
        r#"
fail();

function fail() {
  throw new Error("boom!");
}
"#,
    )?;

    let mod_js_str = mod_js.path().display().to_string();

    let output = Command::cargo_bin("zinnia")?
        .env("NO_COLOR", "1")
        .args(["run", &mod_js_str])
        .output()?;

    let mod_url = resolve_path(
        &mod_js_str,
        &std::env::current_dir().context("unable to get current working directory")?,
    )?;

    let expected_stderr = format!(
        r#"
  error: Uncaught Error: boom!
  throw new Error("boom!");
        ^
    at fail ({mod_url}:5:9)
    at {mod_url}:2:1
"#
    )
    .trim_start()
    .to_string();

    assert_eq!(
        CmdResult::from(&output),
        CmdResult {
            exit_ok: false,
            stdout: "".into(),
            stderr: expected_stderr,
        }
    );

    Ok(())
}

#[test]
fn exits_gracefully() -> Result<(), Box<dyn std::error::Error>> {
    let mod_js = assert_fs::NamedTempFile::new("exit.js")?;
    mod_js.write_str(
        r#"
tick();
function tick() {
    console.log('tick');
    setTimeout(tick, 100);
}
"#,
    )?;

    let mod_js_str = mod_js.path().display().to_string();

    let mut cmd = std::process::Command::new(assert_cmd::cargo::cargo_bin("zinnia"));
    cmd.env("NO_COLOR", "1")
        .env("RUST_LOG", "debug/shutting")
        .args(["run", &mod_js_str]);

    // Wait until the module starts and prints the first log line
    let mut p = rexpect::session::spawn_command(cmd, Some(1000 /* milliseconds */))?;
    p.exp_regex("tick")?;

    // Kill the process via Ctrl+C
    p.send_control('c')?;
    // Read the rest of stdout
    let output = p.exp_eof()?;
    let output = Regex::new("^(?s).*DEBUG zinnia]")?.replace_all(&output, "");
    let output = output.trim();
    assert_eq!(output, "Shutting down...");

    // Check the exit code
    let result = p.process.wait()?;
    assert_eq!(result, WaitStatus::Exited(p.process.child_pid, 0));

    Ok(())
}

// HELPERS

#[derive(PartialEq)]
struct CmdResult {
    pub exit_ok: bool,
    pub stdout: String,
    pub stderr: String,
}

impl CmdResult {
    pub fn from(cmd_output: &Output) -> Self {
        Self {
            exit_ok: cmd_output.status.success(),
            stdout: String::from_utf8_lossy(&cmd_output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&cmd_output.stderr).to_string(),
        }
    }

    pub fn map_stdout<F>(self, f: F) -> Self
    where
        F: FnOnce(&str) -> String,
    {
        Self {
            stdout: f(&self.stdout),
            ..self
        }
    }

    #[allow(unused)]
    pub fn map_stderr<F>(self, f: F) -> Self
    where
        F: FnOnce(&str) -> String,
    {
        Self {
            stderr: f(&self.stdout),
            ..self
        }
    }
}

impl std::fmt::Debug for CmdResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Zinnia execution result\n")?;
        f.write_fmt(format_args!(
            "Exit code indicates {}\n",
            if self.exit_ok { "success" } else { "failure" }
        ))?;
        f.write_fmt(format_args!("==STDOUT==\n{}\n", self.stdout))?;
        f.write_fmt(format_args!("==STDERR==\n{}\n", self.stderr))?;
        f.write_str("==END==")?;
        Ok(())
    }
}

fn strip_timestamps(text: &str) -> String {
    lazy_static! {
        static ref TIME_PATTERN: Regex = Regex::new(r"(?m)^\[\d\d:\d\d:\d\d.\d\d\d ").unwrap();
    }

    TIME_PATTERN.replace_all(text, "[TIMESTAMP ").to_string()
}
