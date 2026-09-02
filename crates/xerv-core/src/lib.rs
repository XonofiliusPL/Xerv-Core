//! Xerv Core — minimalny fundament Xerv (Punkt 1).
//!
//! Zakres: błędy, wersja API, konfiguracja, logowanie, stan, lifecycle.
//! Poza zakresem: TUI, CLI, pluginy, moduły, agenci, sieć, integracje.

pub mod error;
pub use error::{Error, Result};
