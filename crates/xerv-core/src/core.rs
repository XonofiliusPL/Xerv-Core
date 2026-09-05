use std::path::PathBuf;
use std::sync::Mutex;

use semver::Version;

use crate::config::CoreConfig;
use crate::state::CoreState;
use crate::{Error, Result};

/// The main Core instance. Holds configuration and state, exposes `shutdown`.
///
/// Lifecycle: `XervCore::new(...)` → usage → `shutdown()`.
#[derive(Debug)]
pub struct XervCore {
    config: CoreConfig,
    state: Mutex<CoreState>,
    state_path: PathBuf,
    shutdown_flag: Mutex<bool>,
}

impl XervCore {
    pub fn new(config: CoreConfig, state_path: PathBuf) -> Result<Self> {
        let state = CoreState::load(&state_path)?;
        Ok(Self {
            config,
            state: Mutex::new(state),
            state_path,
            shutdown_flag: Mutex::new(false),
        })
    }

    pub fn config(&self) -> &CoreConfig {
        &self.config
    }

    pub fn state(&self) -> CoreState {
        self.state.lock().unwrap().clone()
    }

    pub fn api_version(&self) -> Version {
        crate::version::api_version()
    }

    /// Returns `true` if `shutdown()` has already been called.
    pub fn is_shutdown(&self) -> bool {
        *self.shutdown_flag.lock().unwrap()
    }

    /// Saves state and locks the instance against further use.
    /// A second call returns `Error::Lifecycle`.
    pub fn shutdown(&self) -> Result<()> {
        let mut flag = self.shutdown_flag.lock().unwrap();
        if *flag {
            return Err(Error::Lifecycle("already shut down".into()));
        }
        let snapshot = self.state.lock().unwrap().clone();
        snapshot.save(&self.state_path)?;
        *flag = true;
        Ok(())
    }
}
