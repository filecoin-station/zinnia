use clap::{command, Parser, Subcommand};
use deno_runtime::{
  colors, deno_core::error::JsError, fmt_errors::format_js_error,
};

type AnyError = anyhow::Error;

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

pub fn main() {
  #[cfg(windows)]
  colors::enable_ansi(); // For Windows 10

  match main_impl() {
    Ok(_) => (),
    Err(err) => exit_with_error(err),
  }
}

fn main_impl() -> Result<(), AnyError> {
  let cli_args = CliArgs::parse_from(std::env::args());
  match cli_args.command {
    Commands::Run { file } => {
      println!("{} execute {file}", colors::green("TODO"));
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
