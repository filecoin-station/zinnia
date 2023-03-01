use std::process::Output;

use assert_cmd::Command;
use assert_fs::prelude::*;
use pretty_assertions::assert_eq;

#[test]
fn run_js_module() -> Result<(), Box<dyn std::error::Error>> {
    let mod_js = assert_fs::NamedTempFile::new("hello-mod.js")?;
    mod_js.write_str("console.log('Hello world!')")?;

    let output = Command::cargo_bin("zinnia")?
        .args(["run", &mod_js.path().display().to_string()])
        .output()?;

    assert_eq!(
        CmdResult::from(&output),
        CmdResult {
            exit_ok: true,
            stdout: "Hello world!\n".into(),
            stderr: "".into(),
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

    let expected_stderr = format!(
        r#"
  error: Uncaught Error: boom!
  throw new Error("boom!");
        ^
    at fail (file://{mod_js_str}:5:9)
    at file://{mod_js_str}:2:1
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
