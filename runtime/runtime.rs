use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use deno_core::anyhow::anyhow;
use deno_core::error::type_error;
use deno_core::futures::FutureExt;
use deno_core::{
    located_script_name, resolve_import, serde_json, JsRuntime, ModuleLoader, ModuleSource,
    ModuleSourceFuture, ModuleSpecifier, ModuleType, ResolutionKind, RuntimeOptions,
};

use deno_web::BlobStore;

use crate::{colors, ConsoleReporter, Reporter};

use crate::ext::ZinniaPermissions;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

use zinnia_libp2p;

pub type AnyError = deno_core::anyhow::Error;

/// Common bootstrap options for MainWorker & WebWorker
#[derive(Clone)]
pub struct BootstrapOptions {
    pub no_color: bool,
    pub is_tty: bool,

    /// The user agent version string to use for Fetch API requests and libp2p Identify protocol
    pub agent_version: String,

    /// Seed value for initializing the random number generator
    pub rng_seed: Option<u64>,

    /// Filecoin wallet address - typically the built-in wallet in Filecoin Station
    pub wallet_address: String,

    /// Report activities
    pub reporter: Rc<dyn Reporter>,
}

impl Default for BootstrapOptions {
    fn default() -> Self {
        Self::new(Rc::new(ConsoleReporter::new(Duration::from_millis(500))))
    }
}

impl BootstrapOptions {
    fn new(reporter: Rc<dyn Reporter>) -> Self {
        Self {
            no_color: !colors::use_color(),
            is_tty: colors::is_tty(),
            agent_version: format!("zinnia_runtime/{}", env!("CARGO_PKG_VERSION")),
            rng_seed: None,
            // See https://lotus.filecoin.io/lotus/manage/manage-fil/#public-key-address
            wallet_address: String::from("t1abjxfbp274xpdqcpuaykwkfb43omjotacm2p3za"),
            reporter,
        }
    }

    pub fn as_json(&self) -> String {
        let payload = serde_json::json!({
          "noColor": self.no_color,
          "isTty": self.is_tty,
          "walletAddress": self.wallet_address,
        });
        serde_json::to_string_pretty(&payload).unwrap()
    }
}

pub async fn run_js_module(
    module_specifier: &ModuleSpecifier,
    bootstrap_options: &BootstrapOptions,
) -> Result<(), AnyError> {
    let blob_store = BlobStore::default();
    let reporter = Rc::clone(&bootstrap_options.reporter);

    // Initialize a runtime instance
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![
            // Web Platform APIs implemented by Deno
            deno_console::deno_console::init_ops_and_esm(),
            deno_webidl::deno_webidl::init_ops_and_esm(),
            deno_url::deno_url::init_ops_and_esm(),
            deno_web::deno_web::init_ops_and_esm::<ZinniaPermissions>(
                blob_store,
                Some(module_specifier.clone()),
            ),
            deno_fetch::deno_fetch::init_ops_and_esm::<ZinniaPermissions>(Default::default()),
            deno_crypto::deno_crypto::init_ops_and_esm(bootstrap_options.rng_seed),
            // Zinnia-specific APIs
            zinnia_libp2p::zinnia_libp2p::init_ops_and_esm(zinnia_libp2p::PeerNodeConfig {
                agent_version: bootstrap_options.agent_version.clone(),
                ..Default::default()
            }),
            crate::ext::zinnia_runtime::init_ops_and_esm(reporter),
        ],
        will_snapshot: false,
        inspector: false,
        module_loader: Some(Rc::new(ZinniaModuleLoader {
            main_js_module: module_specifier.clone(),
        })),
        ..Default::default()
    });

    let script = format!("bootstrap.mainRuntime({})", bootstrap_options.as_json());
    runtime.execute_script(located_script_name!(), script.into())?;

    // Load and run the module
    let main_module_id = runtime.load_main_module(module_specifier, None).await?;
    let res = runtime.mod_evaluate(main_module_id);
    runtime.run_event_loop(false).await?;
    res.await??;

    // TODO: it would be nicer to have this exposed as another Deno op
    // and call it from the JavaScript side as part of the regular runtime shutdown
    zinnia_libp2p::shutdown(runtime.op_state()).await?;

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
        maybe_referrer: Option<&ModuleSpecifier>,
        is_dyn_import: bool,
    ) -> std::pin::Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        let main_js_module = self.main_js_module.clone();
        let maybe_referrer = maybe_referrer.cloned();
        async move {
            if is_dyn_import {
                return Err(anyhow!(
                    "Zinnia does not support dynamic imports. (URL: {})",
                    module_specifier
                ));
            }

            let spec_str = module_specifier.as_str();

            let code = {
                if spec_str.eq_ignore_ascii_case(main_js_module.as_str()) {
                    read_file_to_string(module_specifier.to_file_path().unwrap()).await?
                } else if spec_str == "https://deno.land/std@0.177.0/testing/asserts.ts" {
                    return Err(anyhow!(
                        "The vendored version of deno asserts was upgraded to 0.181.0. Please update your imports.\nModule URL: {spec_str}\nImported from: {}",
                        maybe_referrer.map(|u| u.to_string()).unwrap_or("(none)".into())
                    ));
                } else if spec_str == "https://deno.land/std@0.181.0/testing/asserts.ts" {
                    // Temporary workaround until we implement ES Modules
                    // https://github.com/filecoin-station/zinnia/issues/43
                    include_str!("./vendored/asserts.bundle.js").to_string()
                } else {
                    let mut msg =
                        "Zinnia does not support importing from other modules yet. ".to_string();
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
