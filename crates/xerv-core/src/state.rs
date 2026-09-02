use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoreState {
    pub schema_version: u32,
    pub started_at_unix: u64,
    pub boot_count: u64,
}

impl CoreState {
    pub fn fresh() -> Self {
        Self {
            schema_version: 1,
            started_at_unix: now_unix(),
            boot_count: 1,
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::fresh());
        }
        let txt = std::fs::read_to_string(path)?;
        let s: CoreState = serde_json::from_str(&txt)?;
        Ok(s)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = tmp_path(path);
        let txt = serde_json::to_string_pretty(self)?;
        std::fs::write(&tmp, txt)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

fn tmp_path(p: &Path) -> PathBuf {
    let mut s = p.as_os_str().to_os_string();
    s.push(".tmp");
    PathBuf::from(s)
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
