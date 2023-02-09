use assert_fs::prelude::*;
use zinnia_runtime::resolve_path;
use zinnia_runtime::run_js_module;

#[tokio::test]
async fn has_global_window() -> Result<(), Box<dyn std::error::Error>> {
  let mod_js = assert_fs::NamedTempFile::new("global_window_test.js")?;
  mod_js.write_str(
    r#"
if (typeof window !== 'object')
    throw new Error(`Expected \`window\` to have type "object" but found "${typeof window}"`);
if (window != globalThis)
    throw new Error('Expected `window` to be `globalThis`');
"#,
  )?;

  run_js_module(
    &resolve_path(&mod_js.path().to_string_lossy())?,
    &Default::default(),
  )
  .await?;

  Ok(())
}

#[tokio::test]
async fn has_global_self() -> Result<(), Box<dyn std::error::Error>> {
  let mod_js = assert_fs::NamedTempFile::new("global_self_test.js")?;
  mod_js.write_str(
    r#"
if (typeof self !== 'object')
    throw new Error(`Expected \`self\` to have type "object" but found "${typeof self}"`);
if (self != globalThis)
    throw new Error('Expected `self` to be `globalThis`');
"#,
  )?;

  run_js_module(
    &resolve_path(&mod_js.path().to_string_lossy())?,
    &Default::default(),
  )
  .await?;

  Ok(())
}
