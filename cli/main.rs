mod args;

use args::{CliArgs, Commands};
use clap::Parser;

use zinnia_runtime::deno_core;
use zinnia_runtime::fmt_errors::format_js_error;
use zinnia_runtime::{colors, BootstrapOptions};
use zinnia_runtime::{run_js_module, AnyError};

use deno_core::error::JsError;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    #[cfg(windows)]
    colors::enable_ansi(); // For Windows 10

    match main_impl().await {
        Ok(_) => (),
        Err(err) => exit_with_error(err),
    }
}

async fn main_impl() -> Result<(), AnyError> {
    let cli_args = CliArgs::parse_from(std::env::args());
    match cli_args.command {
        Commands::Run { file } => {
            let main_module = deno_core::resolve_path(&file)?;
            let config = BootstrapOptions {
                agent_version: format!("zinnia/{}", env!("CARGO_PKG_VERSION")),
                ..Default::default()
            };
            run_js_module(&main_module, &config).await?;
            Ok(())
        }
    }
}

fn exit_with_error(error: AnyError) {
    // Inspired by unwrap_or_exit<T> from Deno's `cli/main.rs`
    // https://github.com/denoland/deno/blob/34bfa2cb2c1f0f74a94ced8fc164e81cc91cb9f4/cli/main.rs
    let mut error_string = format!("{error:?}");
    let error_code = 1;

    if let Some(e) = error.downcast_ref::<JsError>() {
        error_string = format_js_error(e);
    }

    eprintln!(
        "{}: {}",
        colors::red_bold("error"),
        error_string.trim_start_matches("error: ")
    );
    std::process::exit(error_code);
}
