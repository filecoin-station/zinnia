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
use deno_core::anyhow::Result;

/// Our custom module loader.
pub struct ZinniaModuleLoader {
    module_root: Option<PathBuf>,
}

impl ZinniaModuleLoader {
    pub fn build(module_root: Option<PathBuf>) -> Result<Self> {
        let module_root = match module_root {
            None => None,
            // We must canonicalize the module root path too. It's best to do it once at startup.
            Some(r) => Some(r.canonicalize()?),
        };

        Ok(Self { module_root })
    }
}

pub fn get_module_root(main_js_module: &ModuleSpecifier) -> Result<PathBuf> {
    Ok(main_js_module
        .to_file_path()
        .map_err(|_| anyhow!("Invalid main module specifier: not a local path."))?
        .parent()
        .ok_or_else(|| anyhow!("Invalid main module specifier: it has no parent directory!"))?
        // Resolve any symlinks inside the path to prevent modules from escaping our sandbox
        .canonicalize()?)
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

            let details = || {
                let mut msg = format!("\nModule URL: {spec_str}");
                if let Some(referrer) = &maybe_referrer {
                    msg.push_str("\nImported from: ");
                    msg.push_str(referrer.as_str());
                }
                msg
            };

            if spec_str == "https://deno.land/std@0.177.0/testing/asserts.ts" || spec_str == "https://deno.land/std@0.181.0/testing/asserts.ts" {
                return Err(anyhow!("Zinnia bundles Deno asserts as 'zinnia:assert`. Please update your imports accordingly.{}", details()));
            }

            if module_specifier.scheme() != "file" {
                return Err(anyhow!(
                    "Unsupported scheme: {}. Zinnia can import local modules only.{}",
                     module_specifier.scheme(),
                     details()
                ))
            }

            let module_path = module_specifier.to_file_path().map_err(|_|
               anyhow!("Module specifier cannot be converted to a filepath.{}", details())
            )?;

            // Check that the module path is inside the module root directory
            if let Some(canonical_root) = &module_root {
                // Resolve any symlinks inside the path to prevent modules from escaping our sandbox
                let canonical_module = module_path.canonicalize().map_err(|err| anyhow!(
                    "Cannot canonicalize module path: {err}.\nModule file path: {}{}",
                    module_path.display(),
                    details()
                ))?;

                if !canonical_module.starts_with(canonical_root) {
                    return Err(anyhow!(
                        "Cannot import files outside of the module root directory.\n\
                         Root directory (canonical): {}\n\
                         Module file path (canonical): {}\
                         {}",
                        canonical_root.display(),
                        canonical_module.display(),
                        details()
                    ));
                }
            };

            let code = read_file_to_string(module_path).await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use deno_core::anyhow::Context;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn allows_import_of_files_inside_sandbox() {
        let mut imported_file = get_js_dir();
        imported_file.push("99_main.js");

        let loader = ZinniaModuleLoader::build(Some(get_js_dir())).unwrap();
        let result = loader
            .load(
                &ModuleSpecifier::from_file_path(&imported_file).unwrap(),
                None,
                false,
            )
            .await
            .with_context(|| format!("cannot import {}", imported_file.display()))
            .unwrap();

        assert_eq!(result.module_type, ModuleType::JavaScript);
    }

    #[tokio::test]
    async fn rejects_import_of_files_outside_sandbox() {
        // project_root is `runtime/tests/js`
        let mut project_root = get_js_dir().parent().unwrap().to_path_buf();
        project_root.push("tests");
        project_root.push("js");

        // we are importing file `runtime/js/99_main.js` - it's outside for project_root
        let mut imported_file = get_js_dir();
        imported_file.push("99_main.js");

        let loader = ZinniaModuleLoader::build(Some(project_root)).unwrap();
        let result = loader
            .load(
                &ModuleSpecifier::from_file_path(&imported_file).unwrap(),
                None,
                false,
            )
            .await;

        match result {
            Ok(_) => {
                assert!(
                    result.is_err(),
                    "Expected import from '{}' to fail, it succeeded instead.",
                    imported_file.display()
                );
            }
            Err(err) => {
                let msg = format!("{err}");
                assert!(
                    msg.contains("Cannot import files outside of the module root directory"),
                    "Expected import to fail with the sandboxing error, it failed with a different error instead:\n{}",
                    msg,
                );
            }
        }
    }

    fn get_js_dir() -> PathBuf {
        let mut base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        base_dir.push("js");
        base_dir
    }
}
