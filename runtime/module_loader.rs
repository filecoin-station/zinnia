use std::path::{Path, PathBuf};

use deno_core::anyhow::anyhow;
use deno_core::error::type_error;
use deno_core::futures::FutureExt;
use deno_core::{
    resolve_import, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleSpecifier, ModuleType,
    ResolutionKind,
};

use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub type AnyError = deno_core::anyhow::Error;
use deno_core::anyhow::{Context, Result};

/// Our custom module loader.
pub struct ZinniaModuleLoader {
    module_root: PathBuf,
}

impl ZinniaModuleLoader {
    pub fn new(main_js_module: ModuleSpecifier) -> Result<Self> {
        let module_root =
            ZinniaModuleLoader::get_module_root(&main_js_module).with_context(|| {
                format!(
                    "Cannot determine module root for the main file: {}",
                    main_js_module
                )
            })?;

        Ok(Self { module_root })
    }

    fn get_module_root(main_js_module: &ModuleSpecifier) -> Result<PathBuf> {
        Ok(main_js_module
            .to_file_path()
            .map_err(|_| anyhow!("Invalid main module specifier: not a local path."))?
            .parent()
            .ok_or_else(|| anyhow!("Invalid main module specifier: it has no parent directory!"))?
            // Resolve any symlinks inside the path to prevent modules from escaping our sandbox
            .canonicalize()?)
    }
}

impl ModuleLoader for ZinniaModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, AnyError> {
        if specifier == "zinnia:test" {
            return Ok(ModuleSpecifier::parse("ext:zinnia_runtime/test.js").unwrap());
        } else if specifier == "zinnia:assert" {
            return Ok(
                ModuleSpecifier::parse("ext:zinnia_runtime/vendored/asserts.bundle.js").unwrap(),
            );
        }

        let resolved = resolve_import(specifier, referrer)?;
        Ok(resolved)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        maybe_referrer: Option<&ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> std::pin::Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        let module_root = self.module_root.clone();
        let maybe_referrer = maybe_referrer.cloned();
        async move {
            let spec_str = module_specifier.as_str();

            let code = {
                let is_module_local = match module_specifier.to_file_path() {
                    Err(()) => false,
                    Ok(p) => p
                         // Resolve any symlinks inside the path to prevent modules from escaping our sandbox
                        .canonicalize()
                         // Check that the module path is inside the module root directory
                        .map(|p| p.starts_with(&module_root))
                        .unwrap_or(false),
                };
                if is_module_local {
                    read_file_to_string(module_specifier.to_file_path().unwrap()).await?
                } else if spec_str == "https://deno.land/std@0.177.0/testing/asserts.ts" || spec_str == "https://deno.land/std@0.181.0/testing/asserts.ts" {
                    return Err(anyhow!(
                        "Zinnia no longer bundles Deno asserts. Please vendor the module yourself and load it using a relative path.\nModule URL: {spec_str}\nImported from: {}",
                        maybe_referrer.map(|u| u.to_string()).unwrap_or("(none)".into())
                    ));
                } else {
                    let mut msg = if module_specifier.scheme() == "file" {
                         format!("Cannot import files outside of module root directory {}. ",  module_root.display())
                    } else {
                        "Zinnia supports importing from relative paths only. ".to_string()
                    };
                    msg.push_str(module_specifier.as_str());
                    if let Some(referrer) = &maybe_referrer {
                        msg.push_str(" imported from ");
                        msg.push_str(referrer.as_str());
                    }
                    return Err(anyhow!(msg));
                }
            };

            let module = ModuleSource::new(ModuleType::JavaScript, code.into(), &module_specifier);
            Ok(module)
        }.boxed_local()
    }
}

async fn read_file_to_string(path: impl AsRef<Path>) -> Result<String, AnyError> {
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
