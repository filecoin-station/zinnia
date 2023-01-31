// TODO: extract this into a standalone crate

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use deno_runtime::deno_core::anyhow::anyhow;
use deno_runtime::deno_core::futures::FutureExt;
use deno_runtime::deno_core::op;
use deno_runtime::deno_core::url::Url;
use deno_runtime::deno_core::ByteString;
use deno_runtime::deno_core::Extension;
use deno_runtime::deno_core::JsRuntime;
use deno_runtime::deno_core::ModuleLoader;
use deno_runtime::deno_core::ModuleSource;
use deno_runtime::deno_core::ModuleSourceFuture;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::deno_core::ModuleType;
use deno_runtime::deno_core::OpState;
use deno_runtime::deno_core::ResolutionKind;
use deno_runtime::deno_core::RuntimeOptions;

use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

pub type AnyError = deno_runtime::deno_core::anyhow::Error;

use crate::utils::canonicalize_path;

pub async fn run_js_module(path: &Path) -> Result<(), AnyError> {
  // Initialize a runtime instance
  let mut runtime = JsRuntime::new(RuntimeOptions {
    extensions: vec![
      // Web Platform APIs implemented by Deno
      deno_runtime::deno_console::init(),
      // Zinnia-specific APIs
      // (to be done)
    ],
    will_snapshot: false,
    inspector: false,
    module_loader: Some(Rc::new(ZinniaModuleLoader {
      main_js_module: String::from(path.to_str().unwrap()),
    })),
    ..Default::default()
  });

  // Enable Async Ops
  runtime.execute_script(
    "internal://enable-async-ops.js",
    "Deno.core.initializeAsyncOps()",
  )?;

  // Load and run the module
  let abs_path = canonicalize_path(Path::new(path))?;
  // This could fail only when the abs_path is not absolute
  let url = ModuleSpecifier::from_file_path(abs_path).unwrap();
  let main_module_id = runtime.load_main_module(&url, None).await?;
  let res = runtime.mod_evaluate(main_module_id);
  runtime.run_event_loop(false).await?;
  res.await??;

  Ok(())
}

/// Our custom module loader.
pub struct ZinniaModuleLoader {
  main_js_module: String,
}

impl ModuleLoader for ZinniaModuleLoader {
  fn resolve(
    &self,
    specifier: &str,
    _referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier, AnyError> {
    println!("Resolving specifier: {:#?}", specifier);
    println!("Main js module: {:#?}", self.main_js_module);
    match specifier {
      str if str.eq(self.main_js_module.as_str()) => {
        let abs_path = canonicalize_path(Path::new(str))?;
        // This could fail only when the abs_path is not absolute
        let url = ModuleSpecifier::from_file_path(abs_path).unwrap();
        Ok(url)
      }

      _ => Err(anyhow!(
        "Zinnia does not support module resolution (while loading {})",
        specifier
      )),
    }
  }

  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    _maybe_referrer: Option<ModuleSpecifier>,
    is_dyn_import: bool,
  ) -> std::pin::Pin<Box<ModuleSourceFuture>> {
    let specifier = module_specifier.clone();
    async move {
      if is_dyn_import {
        return Err(anyhow!(
          "Zinnia does not support dynamic imports. (URL: {})",
          specifier
        ));
      }

      let code = read_file_to_string(specifier.to_file_path().unwrap()).await?;

      let module = ModuleSource {
        code: Box::from(code.as_bytes()),
        module_type: ModuleType::JavaScript,
        module_url_specified: specifier.to_string(),
        module_url_found: specifier.to_string(),
      };
      Ok(module)
    }
    .boxed_local()
  }
}

async fn read_file_to_string(
  path: impl AsRef<Path>,
) -> Result<String, AnyError> {
  let mut f = File::open(path).await?;
  let mut buffer = Vec::new();

  // read the whole file
  f.read_to_end(&mut buffer).await?;

  Ok(String::from_utf8_lossy(&buffer).to_string())
}
