mod runtime;
mod utils;

use std::path::Path;

use clap::{command, Parser, Subcommand};
use deno_runtime::{
  colors, deno_core::error::JsError, fmt_errors::format_js_error,
};

use runtime::AnyError;

use crate::runtime::run_js_module;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
  Run {
    /// JavaScript file containing the Station Module to run
    file: String,
  },
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
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
      println!("{} {file}", colors::green("EXECUTE"));
      run_js_module(Path::new(&file)).await?;
      Ok(())
    }
  }
}

fn exit_with_error(error: AnyError) {
  // Inspired by unwrap_or_exit<T> from https://github.com/denoland/deno/blob/main/cli/main.rs
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
