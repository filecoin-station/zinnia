use std::env;

use clap::{command, Parser, Subcommand};

#[derive(Parser, PartialEq, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Address of Station's built-in wallet (required).
    #[arg(long, short = 'w', env)]
    pub wallet_address: String,

    /// Directory where to keep state files (optional). Defaults to a platform-specific location,
    /// e.g. $XDG_STATE_HOME/zinniad on Linux.
    #[arg(long, short = 'r', env, default_value_t = get_default_root_dir(env::var))]
    pub root_dir: String,

    /// List of modules to run, where each module is a single JS file. We don't make any assumptions
    /// about the directory layout of modules. Paths are resolved relatively to the current working
    /// directory.
    pub files: Vec<String>,
}

#[derive(Subcommand, PartialEq, Debug)]
pub enum Commands {
    Run {
        /// JavaScript file containing the Station Module to run
        file: String,
    },
}

// TODO: replace platform-specific code with https://crates.io/crates/directories
// We need to contribute support for `state_dir` on Windows & MacOS
// https://github.com/dirs-dev/directories-rs/issues/70

#[cfg(target_os = "macos")]
fn get_default_root_dir<'a, F>(get_env_var: F) -> String
where
    F: Fn(&'a str) -> Result<String, env::VarError>,
{
    let home = get_env_var("HOME").expect("HOME must be set");
    format!("{home}/Library/Application Support/app.filstation.zinniad")
}

#[cfg(target_os = "linux")]
fn get_default_root_dir<'a, F>(get_env_var: F) -> String
where
    F: Fn(&'a str) -> Result<String, env::VarError>,
{
    match get_env_var("XDG_STATE_HOME") {
        Ok(state_home) => format!("{state_home}/zinniad"),
        Err(_) => {
            let home = get_env_var("HOME").expect("HOME must be set");
            format!("{home}/.local/state/zinniad")
        }
    }
}

#[cfg(target_os = "windows")]
fn get_default_root_dir<'a, F>(get_env_var: F) -> String
where
    F: Fn(&'a str) -> Result<String, env::VarError>,
{
    let app_data = get_env_var("LOCALAPPDATA").expect("LOCALAPPDATA must be set");
    format!("{app_data}\\zinniad")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    #[test]
    #[cfg(target_os = "macos")]
    fn default_root_dir_on_macos() {
        let env = HashMap::from([("HOME", "/users/labber")]);
        let dir = get_default_root_dir(|key| {
            env.get(&key)
                .map(|val| String::from(*val))
                .ok_or(env::VarError::NotPresent)
        });

        assert_eq!(
            dir,
            "/users/labber/Library/Application Support/app.filstation.zinniad"
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn default_root_dir_on_linux() {
        let env = HashMap::from([("XDG_STATE_HOME", "/users/labber/.local-state")]);
        let dir = get_default_root_dir(|key| {
            env.get(&key)
                .map(|val| String::from(*val))
                .ok_or(env::VarError::NotPresent)
        });

        assert_eq!(dir, "/users/labber/.local-state/zinniad");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn default_root_dir_on_linux_without_xdg() {
        let env = HashMap::from([("HOME", "/users/labber")]);
        let dir = get_default_root_dir(|key| {
            env.get(&key)
                .map(|val| String::from(*val))
                .ok_or(env::VarError::NotPresent)
        });

        assert_eq!(dir, "/users/labber/.local/state/zinniad");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn default_root_dir_on_linux() {
        let env = HashMap::from([("LOCALAPPDATA", r"\Users\Jane Smith\AppData\Local")]);
        let dir = get_default_root_dir(|key| {
            env.get(&key)
                .map(|val| String::from(*val))
                .ok_or(env::VarError::NotPresent)
        });

        assert_eq!(dir, r"\Users\Jane Smith\AppData\Local\zinniad");
    }
}
