// Runs tests defined in files inside the `js` directory
// The test files are valid Zinnia modules, so you can run them without Rust:
//   ./target/debug/zinnia run runtime/tests/js/timers_tests.js
// Most of the tests should pass on Deno too!
//   deno run runtime/tests/js/timers_tests.js
use std::path::{Path, PathBuf};
use std::rc::Rc;

use zinnia_runtime::RecordingReporter;
use zinnia_runtime::{anyhow::Context, deno_core, run_js_module, AnyError, BootstrapOptions};

use pretty_assertions::assert_eq;

macro_rules! js_tests(
  ( $name:ident ) => {
    #[tokio::test]
    async fn $name() -> Result<(), AnyError> {
      run_js_test_file(&format!("{}.js", stringify!($name)), None).await
  }
  };

  ( $name:ident check_activity) => {
    #[tokio::test]
    async fn $name() -> Result<(), AnyError> {
      run_js_test_file(
        &format!("{}.js", stringify!($name)),
        Some(&format!("{}.activity.txt", stringify!($name))),
      ).await
  }
  };
);

js_tests!(globals_tests);
js_tests!(timers_tests);
js_tests!(webapis_tests);
js_tests!(webcrypto_tests);
js_tests!(libp2p_tests);
js_tests!(station_apis_tests check_activity);
js_tests!(module_loader_tests);

// Run all tests in a single JS file
async fn run_js_test_file(name: &str, activity_log: Option<&str>) -> Result<(), AnyError> {
    let mut base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_dir.push("tests");
    base_dir.push("js");

    let mut full_path = base_dir.clone();
    full_path.push(name);

    let main_module = deno_core::resolve_path(
        &full_path.to_string_lossy(),
        &std::env::current_dir().context("unable to get current working directory")?,
    )?;
    let reporter = Rc::new(RecordingReporter::new());
    let config = BootstrapOptions {
        agent_version: format!("zinnia_runtime_tests/{}", env!("CARGO_PKG_VERSION")),
        reporter: reporter.clone(),
        ..Default::default()
    };
    run_js_module(&main_module, &config).await?;

    if let Some(log_file) = activity_log {
        let mut activity_path = base_dir.clone();
        activity_path.push(log_file);
        assert_activity_log(reporter.events.borrow(), &activity_path);
    }

    Ok(())
}

fn assert_activity_log(events: std::cell::Ref<Vec<String>>, activity_path: &Path) {
    let expected_text = std::fs::read_to_string(activity_path)
        .unwrap_or_else(|err| panic!("cannot read {}: {}", activity_path.display(), err))
        // normalize line endings to Unix style (LF only)
        .replace("\r\n", "\n");

    let actual_output = events
        .iter()
        .map(|e| format!("{}\n", e.trim_end()))
        .collect::<Vec<String>>()
        .join("");

    assert_eq!(actual_output, expected_text,);
}
