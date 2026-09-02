//! Xerv Core — minimalny fundament Xerv (Punkt 1).
//!
//! Zakres: błędy, wersja API, konfiguracja, logowanie, stan, lifecycle.
//! Poza zakresem: TUI, CLI, pluginy, moduły, agenci, sieć, integracje.

pub mod config;
pub mod error;
pub mod version;
pub use config::CoreConfig;
pub use error::{Error, Result};
pub use version::{api_version, API_VERSION};
