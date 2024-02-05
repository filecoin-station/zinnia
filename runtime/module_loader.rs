use std::path::{Path, PathBuf};

use deno_core::anyhow::anyhow;
use deno_core::error::type_error;
use deno_core::futures::FutureExt;
use deno_core::{
    resolve_import, ModuleLoader, ModuleSource, ModuleSourceCode, ModuleSpecifier, ModuleType,
    RequestedModuleType, ResolutionKind,
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
        // TODO(bajtos): support loading JSON files as ES modules. See
        // https://github.com/denoland/deno/commit/bc8d00c880756a46b0ce4c8bf0c89407a2de669c
        _requested_module_type: RequestedModuleType,
    ) -> deno_core::ModuleLoadResponse {
        let module_specifier = module_specifier.clone();
        let module_root = self.module_root.clone();
        let maybe_referrer = maybe_referrer.cloned();
        deno_core::ModuleLoadResponse::Async(async move {
            let spec_str = module_specifier.as_str();

            let code = {
                let is_module_local = match module_specifier.to_file_path() {
                    Err(()) => false,
                    Ok(module_path) => {
                        match &module_root {
                            None => true,
                            Some(root) => module_path
                                // Resolve any symlinks inside the path to prevent modules from escaping our sandbox
                                .canonicalize()
                                // Check that the module path is inside the module root directory
                                .map(|p| p.starts_with(root))
                                .unwrap_or(false),
                        }
                    }
                };
                if is_module_local {
                    read_file_to_string(module_specifier.to_file_path().unwrap()).await?
                } else if spec_str == "https://deno.land/std@0.177.0/testing/asserts.ts"
                    || spec_str == "https://deno.land/std@0.181.0/testing/asserts.ts"
                {
                    return Err(anyhow!(
                        "Zinnia bundles Deno asserts as 'zinnia:assert`. Please update your imports accordingly.\nModule URL: {spec_str}\nImported from: {}",
                        maybe_referrer.map(|u| u.to_string()).unwrap_or("(none)".into())
                    ));
                } else {
                    let mut msg = if module_specifier.scheme() == "file" && module_root.is_some() {
                        format!(
                            "Cannot import files outside of module root directory {}. ",
                            module_root.unwrap().display()
                        )
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

            let module = ModuleSource::new(
                ModuleType::JavaScript,
                ModuleSourceCode::String(code.into()),
                &module_specifier
            );
            Ok(module)
        }.boxed_local())
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
    use deno_core::{anyhow::Context, ModuleLoadResponse};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn allows_import_of_files_inside_sandbox() {
        let mut imported_file = get_js_dir();
        imported_file.push("99_main.js");

        let loader = ZinniaModuleLoader::build(Some(get_js_dir())).unwrap();
        let result = loader.load(
            &ModuleSpecifier::from_file_path(&imported_file).unwrap(),
            None,
            false,
            RequestedModuleType::None,
        );
        let result = match result {
            ModuleLoadResponse::Async(result) => result.await,
            ModuleLoadResponse::Sync(result) => result,
        };
        let result = result
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
        let result = loader.load(
            &ModuleSpecifier::from_file_path(&imported_file).unwrap(),
            None,
            false,
            RequestedModuleType::None,
        );
        let result = match result {
            ModuleLoadResponse::Async(result) => result.await,
            ModuleLoadResponse::Sync(result) => result,
        };

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
                    msg.contains("Cannot import files outside of module root directory"),
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
