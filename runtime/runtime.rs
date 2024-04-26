use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use deno_core::{located_script_name, serde_json, JsRuntime, ModuleSpecifier, RuntimeOptions};

use deno_web::BlobStore;

use crate::module_loader::ZinniaModuleLoader;
use crate::{colors, Reporter};

use crate::ext::ZinniaPermissions;

use zinnia_libp2p;

pub type AnyError = deno_core::anyhow::Error;
use deno_core::anyhow::Result;

/// Common bootstrap options for MainWorker & WebWorker
#[derive(Clone)]
pub struct BootstrapOptions {
    pub no_color: bool,
    pub is_tty: bool,

    /// The user agent version string to use for Fetch API requests and libp2p Identify protocol
    pub agent_version: String,

    /// Seed value for initializing the random number generator
    pub rng_seed: Option<u64>,

    /// Module root if you want to sandbox `import` of ES modules
    pub module_root: Option<PathBuf>,

    /// Filecoin wallet address - typically the built-in wallet in Filecoin Station
    pub wallet_address: String,

    /// Station ID - the unique identifier of the Filecoin Station
    pub station_id: String,

    /// Report activities
    pub reporter: Rc<dyn Reporter>,

    /// Lassie daemon to use as the IPFS retrieval client. We must use Arc here to allow sharing of
    /// the singleton Lassie instance between multiple threads spawned by Rust's test runner.
    pub lassie_daemon: Arc<lassie::Daemon>,

    /// Zinnia version reported by `Zinnia.versions.zinnia` API.
    /// Embedders can customize this value.
    pub zinnia_version: &'static str,
}

impl BootstrapOptions {
    pub fn new(
        agent_version: String,
        reporter: Rc<dyn Reporter>,
        lassie_daemon: Arc<lassie::Daemon>,
        module_root: Option<PathBuf>,
    ) -> Self {
        Self {
            no_color: !colors::use_color(),
            is_tty: colors::is_tty(),
            agent_version,
            rng_seed: None,
            module_root,
            // See https://lotus.filecoin.io/lotus/manage/manage-fil/#public-key-address
            wallet_address: String::from("t1abjxfbp274xpdqcpuaykwkfb43omjotacm2p3za"),
            // Station ID must look like a public key - 88 hexadecimal characters.
            // Let's use all-zeroes value to make it easy to distinguish data reported
            // from non-production systems (dev, CI).
            station_id: "0".repeat(88),
            reporter,
            lassie_daemon,
            zinnia_version: env!("CARGO_PKG_VERSION"),
        }
    }

    pub fn as_json(&self) -> String {
        let payload = serde_json::json!({
          "noColor": self.no_color,
          "isTty": self.is_tty,
          "walletAddress": self.wallet_address,
          "stationId": self.station_id,
          "lassieUrl": format!("http://127.0.0.1:{}/", self.lassie_daemon.port()),
          "lassieAuth": match self.lassie_daemon.access_token() {
            Some(token) => serde_json::Value::String(format!("Bearer {token}")),
            None => serde_json::Value::Null,
          },
          "zinniaVersion": self.zinnia_version,
          "v8Version": deno_core::v8_version(),
        });
        serde_json::to_string_pretty(&payload).unwrap()
    }
}

pub async fn run_js_module(
    module_specifier: &ModuleSpecifier,
    bootstrap_options: &BootstrapOptions,
) -> Result<(), AnyError> {
    let blob_store = Arc::new(BlobStore::default());
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
            deno_fetch::deno_fetch::init_ops_and_esm::<ZinniaPermissions>(deno_fetch::Options {
                user_agent: bootstrap_options.agent_version.clone(),
                ..Default::default()
            }),
            deno_crypto::deno_crypto::init_ops_and_esm(bootstrap_options.rng_seed),
            // Zinnia-specific APIs
            zinnia_libp2p::zinnia_libp2p::init_ops_and_esm(zinnia_libp2p::PeerNodeConfig {
                agent_version: bootstrap_options.agent_version.clone(),
                ..Default::default()
            }),
            crate::ext::zinnia_runtime::init_ops_and_esm(reporter),
        ],
        inspector: false,
        module_loader: Some(Rc::new(ZinniaModuleLoader::build(
            bootstrap_options.module_root.clone(),
        )?)),
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

use deno_crypto::rand::{self, distributions::Alphanumeric, Rng};

const ONE_DAY: Duration = Duration::from_secs(24 * 3600);

/// A baseline configuration for Lassie shared by `zinnia` and `zinniad` CLIs.
pub fn lassie_config() -> lassie::DaemonConfig {
    let access_token = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();

    lassie::DaemonConfig {
        port: 0,
        access_token: Some(access_token),
        provider_timeout: Some(ONE_DAY),
        global_timeout: Some(ONE_DAY),
        ..Default::default()
    }
}
