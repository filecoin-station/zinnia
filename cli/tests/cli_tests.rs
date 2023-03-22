use std::process::Output;

use assert_cmd::Command;
use assert_fs::prelude::*;
use lazy_static::lazy_static;
use pretty_assertions::assert_eq;
use regex::Regex;

use zinnia_runtime::anyhow::Context;
use zinnia_runtime::resolve_path;

#[test]
fn run_js_module() -> Result<(), Box<dyn std::error::Error>> {
    let mod_js = assert_fs::NamedTempFile::new("hello-mod.js")?;
    mod_js.write_str(
        r#"
console.log("debug-info");
console.error("debug-error");
Zinnia.activity.info("information");
Zinnia.activity.error("problem");
Zinnia.jobCompleted();
"#,
    )?;

    let output = Command::cargo_bin("zinnia")?
        .args(["run", &mod_js.path().display().to_string()])
        .output()?;

    assert_eq!(
        CmdResult::from(&output).map_stdout(strip_timestamps),
        CmdResult {
            exit_ok: true,
            stdout: [
                "[TIMESTAMP  INFO] information\n",
                "[TIMESTAMP ERROR] problem\n",
                "[TIMESTAMP STATS] Jobs completed: 1\n",
            ]
            .join(""),
            stderr: "debug-info\ndebug-error\n".into(),
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
