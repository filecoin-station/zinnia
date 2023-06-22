mod args;

use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use args::{CliArgs, Commands};
use clap::Parser;

use zinnia_runtime::anyhow::{Context, Error, Result};
use zinnia_runtime::deno_core::error::JsError;
use zinnia_runtime::fmt_errors::format_js_error;
use zinnia_runtime::{
    colors, lassie, resolve_path, run_js_module, BootstrapOptions, ConsoleReporter,
};

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

async fn main_impl() -> Result<()> {
    let cli_args = CliArgs::parse_from(std::env::args());
    match cli_args.command {
        Commands::Run { file } => {
            let main_module = resolve_path(
                &file,
                &std::env::current_dir().context("unable to get current working directory")?,
            )?;

            let lassie_daemon = Arc::new(
                lassie::Daemon::start(lassie::DaemonConfig {
                    // This configuration applies to `zinnia` CLI only. The `zinniad` daemon running
                    // inside Station uses a different temp_dir config based on the env var
                    // `CACHE_ROOT` provided by the Station.
                    //
                    // By default, Lassie stores its temporary files in the system temp directory.
                    // That's good enough for now. We can improve this later based on user feedback,
                    // for example:
                    // - we can honour CACHE_ROOT
                    // - we can default to something like
                    //   `~/.cache/zinnia/lassie` on Unix,
                    //   `%APPLOCALDATA%\zinnia\lassie' on Windows.
                    //
                    // Important: if we tell Lassie to use a specific temp dir that's not
                    // automatically cleaned by the operating system, we will need to clean any
                    // leftover files ourselves. See the GH issue for deleting leftover files
                    // when `zinniad` starts: https://github.com/filecoin-station/zinnia/issues/245
                    temp_dir: None,
                    port: 0,
                })
                .context("cannot initialize the IPFS retrieval client Lassie")?,
            );

            let runtime_config = BootstrapOptions {
                zinnia_version: env!("CARGO_PKG_VERSION"),
                ..BootstrapOptions::new(
                    format!("zinnia/{}", env!("CARGO_PKG_VERSION")),
                    Rc::new(ConsoleReporter::new(Duration::from_millis(500))),
                    lassie_daemon,
                    None,
                )
            };

            run_js_module(&main_module, &runtime_config).await?;
            Ok(())
        }
    }
}

fn exit_with_error(error: Error) {
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
