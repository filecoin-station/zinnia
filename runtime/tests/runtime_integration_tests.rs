// Runs tests defined in files inside the `js` directory
// The test files are valid Zinnia modules, so you can run them without Rust:
//   ./target/debug/zinnia run runtime/tests/js/timers_tests.js
// Most of the tests should pass on Deno too!
//   deno run runtime/tests/js/timers_tests.js
use std::path::PathBuf;

use zinnia_runtime::{deno_core, run_js_module, AnyError};

macro_rules! js_tests(
  ( $name:ident ) => {
    #[tokio::test]
    async fn $name() -> Result<(), AnyError> {
      run_js_test_file(&format!("{}.js", stringify!($name))).await
  }
  };
);

js_tests!(globals_tests);
js_tests!(timers_tests);
js_tests!(webapis_tests);
js_tests!(libp2p_tests);

// Run all tests in a single JS file
async fn run_js_test_file(name: &str) -> Result<(), AnyError> {
  let mut full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  full_path.push("tests");
  full_path.push("js");
  full_path.push(name);

  let main_module = deno_core::resolve_path(&full_path.to_string_lossy())?;
  run_js_module(&main_module, &Default::default()).await?;

  Ok(())
}
