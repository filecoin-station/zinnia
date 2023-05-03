use atomicwrites::{AtomicFile, OverwriteBehavior};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub total_jobs_completed: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            total_jobs_completed: 0,
        }
    }
}

impl State {
    pub fn load(state_file: &Path) -> Self {
        log::debug!("Loading initial state from {}", state_file.display());
        match std::fs::read_to_string(state_file) {
            Err(err) => {
                if err.kind() != std::io::ErrorKind::NotFound {
                    log::warn!(
                        "Cannot read initial state from {}: {}",
                        state_file.display(),
                        err
                    );
                }
                State::default()
            }
            Ok(data) => match serde_json::from_str::<State>(&data) {
                Err(err) => {
                    log::warn!(
                        "Cannot parse initial state from {}: {}",
                        state_file.display(),
                        err
                    );
                    State::default()
                }
                Ok(state) => {
                    log::debug!("Initial state: {:?}", state);
                    state
                }
            },
        }
    }

    pub fn store(&self, state_file: &Path) {
        if let Some(parent) = state_file.parent() {
            if let Err(err) = std::fs::create_dir_all(&parent) {
                log::warn!(
                    "Cannot create state directory {}: {}",
                    parent.display(),
                    err
                );
                return;
            }
        }

        let payload = match serde_json::to_string_pretty(self) {
            Err(err) => {
                log::warn!("Cannot serialize state: {}", err);
                return;
            }

            Ok(payload) => payload,
        };

        let write_result = AtomicFile::new(state_file, OverwriteBehavior::AllowOverwrite)
            .write(|f| f.write_all(payload.as_bytes()));

        match write_result {
            Err(err) => log::warn!("Cannot write state to {}: {}", state_file.display(), err),
            Ok(()) => log::debug!("State stored in {}", state_file.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile;
    use zinnia_runtime::anyhow::Result;

    #[test]
    fn creates_missing_directories() -> Result<()> {
        let state_dir = tempfile::tempdir()?;
        let state_file = state_dir.path().join("subdir").join("state.json");
        let state = State {
            total_jobs_completed: 1,
        };
        state.store(&state_file);
        let loaded = State::load(&state_file);
        assert_eq!(loaded.total_jobs_completed, 1);
        Ok(())
    }
}
