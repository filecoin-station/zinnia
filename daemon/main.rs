mod args;
mod state;
mod station_reporter;

use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use args::CliArgs;
use clap::Parser;

use log::{error, info};
use zinnia_runtime::anyhow::{anyhow, Context, Error, Result};
use zinnia_runtime::{resolve_path, run_js_module, BootstrapOptions};

use crate::station_reporter::{log_info_activity, StationReporter};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    setup_logger();
    let cli_args = CliArgs::parse_from(std::env::args());

    match run(cli_args).await {
        Ok(_) => (),
        Err(err) => exit_with_error(err),
    }
}

async fn run(config: CliArgs) -> Result<()> {
    info!("Starting zinniad with config {config:?}");

    if config.files.is_empty() {
        return Err(anyhow!("You must provide at least one module to run."));
    }
    if config.files.len() > 1 {
        return Err(anyhow!(
            "We do not yet support running more than one module."
        ));
    }

    let state_file = PathBuf::from(config.state_root).join("state.json");
    log::debug!("Using state file: {}", state_file.display());

    log_info_activity("Module Runtime started.");

    let file = &config.files[0];

    // TODO: configurable module name and version
    // https://github.com/filecoin-station/zinnia/issues/147
    let module_name = file.trim_end_matches(".js");
    let module_version = "unknown";

    let main_module = resolve_path(
        file,
        &std::env::current_dir().context("unable to get current working directory")?,
    )?;
    let config = BootstrapOptions {
        agent_version: format!(
            "zinniad/{} {module_name}/{module_version}",
            env!("CARGO_PKG_VERSION")
        ),
        wallet_address: config.wallet_address,
        reporter: Rc::new(StationReporter::new(
            state_file,
            Duration::from_millis(200),
            module_name.into(),
        )),
        ..Default::default()
    };

    // TODO: handle module exit and restart it
    // https://github.com/filecoin-station/zinnia/issues/146
    log::info!("Starting module {main_module}");
    run_js_module(&main_module, &config).await?;

    Ok(())
}

fn setup_logger() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.parse_default_env();
    builder.init();
}

fn exit_with_error(error: Error) {
    let error_string = format!("{error:?}");
    let error_code = 1;

    error!("{}", error_string.trim_start_matches("error: "));
    std::process::exit(error_code);
}
