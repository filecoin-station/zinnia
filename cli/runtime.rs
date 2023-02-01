// TODO: extract this into a standalone crate

use std::path::Path;
use std::rc::Rc;

use deno_runtime::colors;
use deno_runtime::deno_core::anyhow::anyhow;
use deno_runtime::deno_core::error::type_error;
use deno_runtime::deno_core::futures::FutureExt;
use deno_runtime::deno_core::include_js_files;
use deno_runtime::deno_core::located_script_name;
use deno_runtime::deno_core::resolve_import;
use deno_runtime::deno_core::Extension;
use deno_runtime::deno_core::JsRuntime;
use deno_runtime::deno_core::ModuleLoader;
use deno_runtime::deno_core::ModuleSource;
use deno_runtime::deno_core::ModuleSourceFuture;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::deno_core::ModuleType;
use deno_runtime::deno_core::ResolutionKind;
use deno_runtime::deno_core::RuntimeOptions;

use deno_runtime::deno_core::serde_json;
use deno_runtime::deno_core::serde_json::json;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub type AnyError = deno_runtime::deno_core::anyhow::Error;

/// Common bootstrap options for MainWorker & WebWorker
#[derive(Clone)]
pub struct BootstrapOptions {
  pub no_color: bool,
  pub is_tty: bool,
}

impl Default for BootstrapOptions {
  fn default() -> Self {
    Self {
      no_color: !colors::use_color(),
      is_tty: colors::is_tty(),
    }
  }
}

impl BootstrapOptions {
  pub fn as_json(&self) -> String {
    let payload = json!({
      "noColor": self.no_color,
      "isTty": self.is_tty,
    });
    serde_json::to_string_pretty(&payload).unwrap()
  }
}

pub async fn run_js_module(
  module_specifier: &ModuleSpecifier,
  bootstrap_options: &BootstrapOptions,
) -> Result<(), AnyError> {
  // Initialize a runtime instance
  let mut runtime = JsRuntime::new(RuntimeOptions {
    extensions_with_js: vec![
      // Web Platform APIs implemented by Deno
      deno_runtime::deno_console::init(),
      // Zinnia-specific APIs
      // (to be done)
      Extension::builder("zinnia_runtime")
        .js(include_js_files!(
          prefix "zinnia:runtime",
          "runtime_js/98_global_scope.js",
          "runtime_js/99_main.js",
        ))
        .build(),
    ],
    will_snapshot: false,
    inspector: false,
    module_loader: Some(Rc::new(ZinniaModuleLoader {
      main_js_module: module_specifier.clone(),
    })),
    ..Default::default()
  });

  let script =
    format!("bootstrap.mainRuntime({})", bootstrap_options.as_json());
  runtime.execute_script(&located_script_name!(), &script)?;

  // Load and run the module
  let main_module_id = runtime.load_main_module(module_specifier, None).await?;
  let res = runtime.mod_evaluate(main_module_id);
  runtime.run_event_loop(false).await?;
  res.await??;

  Ok(())
}

/// Our custom module loader.
pub struct ZinniaModuleLoader {
  main_js_module: ModuleSpecifier,
}

impl ModuleLoader for ZinniaModuleLoader {
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier, AnyError> {
    let resolved = resolve_import(specifier, referrer)?;
    Ok(resolved)
  }

  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    maybe_referrer: Option<ModuleSpecifier>,
    is_dyn_import: bool,
  ) -> std::pin::Pin<Box<ModuleSourceFuture>> {
    let specifier = module_specifier.clone();
    let main_js_module = self.main_js_module.clone();
    async move {
      if is_dyn_import {
        return Err(anyhow!(
          "Zinnia does not support dynamic imports. (URL: {})",
          specifier
        ));
      }

      if !specifier
        .as_str()
        .eq_ignore_ascii_case(main_js_module.as_str())
      {
        let mut msg =
          "Zinnia does not support importing from other modules yet. "
            .to_string();
        msg.push_str(&specifier.as_str());
        if let Some(referrer) = &maybe_referrer {
          msg.push_str(" imported from ");
          msg.push_str(referrer.as_str());
        }
        return Err(anyhow!(msg));
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
  let mut f = File::open(&path).await.map_err(|err| {
    type_error(format!(
      "Module not found: {}. {}",
      err,
      path.as_ref().display()
    ))
  })?;

  // read the whole file
  let mut buffer = Vec::new();
  f.read_to_end(&mut buffer).await?;

  Ok(String::from_utf8_lossy(&buffer).to_string())
}
