use std::time::Instant;

use assert_fs::prelude::*;
use zinnia_runtime::resolve_path;
use zinnia_runtime::run_js_module;

#[tokio::test]
async fn set_timeout_works() -> Result<(), Box<dyn std::error::Error>> {
  let mod_js = assert_fs::NamedTempFile::new("timers_tests.js")?;
  mod_js.write_str(
    r#"
setTimeout(() => {}, 50);
"#,
  )?;

  let now = Instant::now();
  run_js_module(
    &resolve_path(&mod_js.path().to_string_lossy())?,
    &Default::default(),
  )
  .await?;
  let elapsed = now.elapsed().as_millis();

  assert!(
    elapsed > 40 && elapsed < 100,
    "setTimeout(50) should take between 40 to 100 ms to execute, but took {elapsed} ms instead",
  );

  Ok(())
}
