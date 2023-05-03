use atomicwrites::{AtomicFile, OverwriteBehavior};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use zinnia_runtime::anyhow::{self, Context, Result};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct State {
    pub total_jobs_completed: u64,
}

impl State {
    pub fn load(state_file: &Path) -> Result<Self> {
        log::debug!("Loading initial state from {}", state_file.display());
        match std::fs::read_to_string(state_file) {
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    let state = State::default();
                    log::debug!("State file not found, returning {state:?}");
                    Ok(state)
                }
                _ => Err(anyhow::Error::new(err).context(format!(
                    "Cannot load initial state from {}",
                    state_file.display()
                ))),
            },
            Ok(data) => {
                let state = serde_json::from_str::<State>(&data).with_context(|| {
                    format!("Cannot parse initial state from {}", state_file.display())
                })?;
                log::debug!("Loaded initial state: {state:?}");
                Ok(state)
            }
        }
    }

    pub fn store(&self, state_file: &Path) -> Result<()> {
        let payload = serde_json::to_string_pretty(self).context("Cannot serialize state")?;

        let mut write_result = AtomicFile::new(state_file, OverwriteBehavior::AllowOverwrite)
            .write(|f| f.write_all(payload.as_bytes()));

        if let Err(atomicwrites::Error::Internal(err)) = &write_result {
            if err.kind() == std::io::ErrorKind::NotFound {
                if let Some(parent) = state_file.parent() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("Cannot create state directory {}", parent.display(),)
                    })?;
                    write_result = AtomicFile::new(state_file, OverwriteBehavior::AllowOverwrite)
                        .write(|f| f.write_all(payload.as_bytes()));
                }
            }
        }

        write_result.with_context(|| format!("Cannot write state to {}", state_file.display()))?;
        log::debug!("State stored in {}", state_file.display());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;
    use zinnia_runtime::anyhow::Result;

    #[test]
    fn loads_empty_state() -> Result<()> {
        let state_dir = tempdir()?;
        let state_file = state_dir.path().join("state.json");
        let loaded = State::load(&state_file)?;
        assert_eq!(loaded.total_jobs_completed, 0, "total_jobs_completed");
        Ok(())
    }

    #[test]
    fn creates_missing_directories() -> Result<()> {
        let state_dir = tempdir()?;
        let state_file = state_dir.path().join("subdir").join("state.json");
        let state = State {
            total_jobs_completed: 1,
        };
        state.store(&state_file)?;
        let loaded = State::load(&state_file)?;
        assert_eq!(loaded.total_jobs_completed, 1);
        Ok(())
    }
}
