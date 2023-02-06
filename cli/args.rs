use clap::{command, Parser, Subcommand};

#[derive(Parser, PartialEq, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand, PartialEq, Debug)]
pub enum Commands {
  Run {
    /// JavaScript file containing the Station Module to run
    file: String,
  },
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn run_js() {
    let args = CliArgs::parse_from(["zinnia", "run", "mod.js"]);
    assert_eq!(
      args,
      CliArgs {
        command: Commands::Run {
          file: "mod.js".to_string()
        }
      },
    );
  }
}
