use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoreConfig {
    pub data_dir: PathBuf,
    pub log_level: String,
    pub state_filename: String,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            data_dir: directories::ProjectDirs::from("xerv", "xerv", "xerv")
                .map(|d| d.data_dir().to_path_buf())
                .unwrap_or_else(|| PathBuf::from("./.xerv")),
            log_level: "info".into(),
            state_filename: "state.json".into(),
        }
    }
}

impl CoreConfig {
    /// Loads configuration from `path`. When `path` is `None` or the file
    /// does not exist, returns `Default`.
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(p) = path else {
            return Ok(Self::default());
        };
        if !p.exists() {
            return Ok(Self::default());
        }
        let txt = std::fs::read_to_string(p)?;
        let cfg: CoreConfig = toml::from_str(&txt)?;
        Ok(cfg)
    }
}
