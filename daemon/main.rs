mod args;

use args::CliArgs;
use clap::Parser;

use log::{error, info};
use zinnia_runtime::anyhow::{anyhow, Error, Result};

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
            "zinniad does not yet support running more than one module."
        ));
    }

    info!("To be done: run {}", config.files[0]);
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
