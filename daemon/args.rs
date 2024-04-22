use std::env;

use clap::{command, Parser, Subcommand};

#[derive(Parser, PartialEq, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Address of Station's built-in Filecoin wallet (required).
    #[arg(long, short = 'w', env = "FIL_WALLET_ADDRESS", name = "FIL ADDRESS")]
    pub wallet_address: String,

    /// Unique identifier of the Filecoin Station (required).
    #[arg(long, env = "STATION_ID", name = "STATION ID")]
    pub station_id: String,

    /// The type of Station Core deployment. Either 'cli', 'docker', or 'station-desktop' (required).
    #[arg(long, short = 'd', env = "DEPLOYMENT_TYPE", name = "DEPLOYMENT TYPE")]
    pub deployment_type: String,

    /// Directory where to keep state files.
    #[arg(long, env, default_value_t = get_default_state_dir(env::var), name = "LOCAL STATE DIR PATH")]
    pub state_root: String,

    /// Directory where to keep temporary files like cached data.
    #[arg(long, env, default_value_t = get_default_cache_dir(env::var), name = "CACHE DIR PATH")]
    pub cache_root: String,

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
fn get_default_state_dir<'a, F>(get_env_var: F) -> String
where
    F: Fn(&'a str) -> Result<String, env::VarError>,
{
    let home = get_env_var("HOME").expect("HOME must be set");
    format!("{home}/Library/Application Support/app.filstation.zinniad")
}

#[cfg(target_os = "macos")]
fn get_default_cache_dir<'a, F>(get_env_var: F) -> String
where
    F: Fn(&'a str) -> Result<String, env::VarError>,
{
    let home = get_env_var("HOME").expect("HOME must be set");
    format!("{home}/Library/Caches/app.filstation.zinniad")
}

#[cfg(target_os = "linux")]
fn get_default_state_dir<'a, F>(get_env_var: F) -> String
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

#[cfg(target_os = "linux")]
fn get_default_cache_dir<'a, F>(get_env_var: F) -> String
where
    F: Fn(&'a str) -> Result<String, env::VarError>,
{
    match get_env_var("XDG_CACHE_HOME") {
        Ok(state_home) => format!("{state_home}/zinniad"),
        Err(_) => {
            let home = get_env_var("HOME").expect("HOME must be set");
            format!("{home}/.cache/zinniad")
        }
    }
}

#[cfg(target_os = "windows")]
fn get_default_state_dir<'a, F>(get_env_var: F) -> String
where
    F: Fn(&'a str) -> Result<String, env::VarError>,
{
    // LOCALAPPDATA is usually C:\Users\{username}\AppData\Local
    let app_data = get_env_var("LOCALAPPDATA").expect("LOCALAPPDATA must be set");
    format!("{app_data}\\zinniad")
}

#[cfg(target_os = "windows")]
fn get_default_cache_dir<'a, F>(get_env_var: F) -> String
where
    F: Fn(&'a str) -> Result<String, env::VarError>,
{
    // TEMP or TMP is usually C:\Users\{Username}\AppData\Local\Temp
    let temp = get_env_var("TEMP").expect("TEMP must be set");
    format!("{temp}\\zinniad")
}

#[cfg(test)]
mod tests {
    mod state_root {
        use super::super::*;
        use pretty_assertions::assert_eq;
        use std::collections::HashMap;

        #[test]
        #[cfg(target_os = "macos")]
        fn default_state_dir_on_macos() {
            let env = HashMap::from([("HOME", "/users/labber")]);
            let dir = get_default_state_dir(|key| {
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
        fn default_state_dir_on_linux() {
            let env = HashMap::from([("XDG_STATE_HOME", "/users/labber/.local-state")]);
            let dir = get_default_state_dir(|key| {
                env.get(&key)
                    .map(|val| String::from(*val))
                    .ok_or(env::VarError::NotPresent)
            });

            assert_eq!(dir, "/users/labber/.local-state/zinniad");
        }

        #[test]
        #[cfg(target_os = "linux")]
        fn default_state_dir_on_linux_without_xdg() {
            let env = HashMap::from([("HOME", "/users/labber")]);
            let dir = get_default_state_dir(|key| {
                env.get(&key)
                    .map(|val| String::from(*val))
                    .ok_or(env::VarError::NotPresent)
            });

            assert_eq!(dir, "/users/labber/.local/state/zinniad");
        }

        #[test]
        #[cfg(target_os = "windows")]
        fn default_state_dir_on_linux() {
            let env = HashMap::from([("LOCALAPPDATA", r"\Users\Jane Smith\AppData\Local")]);
            let dir = get_default_state_dir(|key| {
                env.get(&key)
                    .map(|val| String::from(*val))
                    .ok_or(env::VarError::NotPresent)
            });

            assert_eq!(dir, r"\Users\Jane Smith\AppData\Local\zinniad");
        }
    }

    mod cache_root {
        use super::super::*;
        use pretty_assertions::assert_eq;
        use std::collections::HashMap;

        #[test]
        #[cfg(target_os = "macos")]
        fn default_cache_dir_on_macos() {
            let env = HashMap::from([("HOME", "/users/labber")]);
            let dir = get_default_cache_dir(|key| {
                env.get(&key)
                    .map(|val| String::from(*val))
                    .ok_or(env::VarError::NotPresent)
            });

            assert_eq!(dir, "/users/labber/Library/Caches/app.filstation.zinniad");
        }

        #[test]
        #[cfg(target_os = "linux")]
        fn default_cache_dir_on_linux() {
            let env = HashMap::from([("XDG_CACHE_HOME", "/users/labber/.temp")]);
            let dir = get_default_cache_dir(|key| {
                env.get(&key)
                    .map(|val| String::from(*val))
                    .ok_or(env::VarError::NotPresent)
            });

            assert_eq!(dir, "/users/labber/.temp/zinniad");
        }

        #[test]
        #[cfg(target_os = "linux")]
        fn default_cache_dir_on_linux_without_xdg() {
            let env = HashMap::from([("HOME", "/users/labber")]);
            let dir = get_default_cache_dir(|key| {
                env.get(&key)
                    .map(|val| String::from(*val))
                    .ok_or(env::VarError::NotPresent)
            });

            assert_eq!(dir, "/users/labber/.cache/zinniad");
        }

        #[test]
        #[cfg(target_os = "windows")]
        fn default_cache_dir_on_linux() {
            let env = HashMap::from([("TEMP", r"\Users\Jane Smith\AppData\Local\Temp")]);
            let dir = get_default_cache_dir(|key| {
                env.get(&key)
                    .map(|val| String::from(*val))
                    .ok_or(env::VarError::NotPresent)
            });

            assert_eq!(dir, r"\Users\Jane Smith\AppData\Local\Temp\zinniad");
        }
    }
}
