//! Xerv Core — stabilne publiczne API.
//!
//! Ten moduł jest **kontraktem** między Rdzeniem a przyszłymi warstwami
//! (TUI, CLI, moduły, pluginy, adaptery). Gwarantujemy:
//!
//! - nazwy typów i funkcji wymienione w module (aliasy i reexporty);
//! - semantykę metod udokumentowaną w `///` przy każdym typie;
//! - że `Cargo.toml` nie zmienia się w sposób łamiący API bez bumpu
//!   `API_VERSION` w [`crate::API_VERSION`].
//!
//! **Nie gwarantujemy** (są poza zakresem Punktu 2):
//! - żadnych traitów rozszerzeń;
//! - żadnego rejestru pluginów;
//! - żadnych subskrypcji zdarzeń;
//! - żadnego API specyficznego dla konsumenta (TUI/CLI/moduły).

/// Alias stabilnego [`crate::Error`].
pub type ApiError = crate::Error;

/// Alias stabilnego [`crate::Result`].
pub type ApiResult<T> = crate::Result<T>;

// Reexporty stabilnych typów Rdzenia.
pub use crate::config::CoreConfig;
pub use crate::core::XervCore;
pub use crate::error::{Error, Result};
pub use crate::state::CoreState;
pub use crate::version::{api_version, API_VERSION};
