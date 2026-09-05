//! Xerv Core — the stable public API.
//!
//! This module is the **contract** between the Core and future layers
//! (TUI, CLI, modules, plugins, adapters). We guarantee:
//!
//! - type and function names listed in the module (aliases and re-exports);
//! - method semantics documented via `///` on each type;
//! - that `Cargo.toml` does not break the API without bumping
//!   `API_VERSION` in [`crate::API_VERSION`].
//!
//! **We do not guarantee** (out of scope for Point 2):
//! - any extension traits;
//! - any plugin registry;
//! - any event subscriptions;
//! - any consumer-specific API (TUI/CLI/modules).

/// Alias for stable [`crate::Error`].
pub type ApiError = crate::Error;

/// Alias for stable [`crate::Result`].
pub type ApiResult<T> = crate::Result<T>;

// Stable re-exports of Core types.
pub use crate::config::CoreConfig;
pub use crate::core::XervCore;
pub use crate::error::{Error, Result};
pub use crate::state::CoreState;
pub use crate::version::{api_version, API_VERSION};

// Re-export semver for consumers (TUI/CLI) — needed for version comparison.
pub use semver::Version;
