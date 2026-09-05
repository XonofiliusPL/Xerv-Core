//! Xerv Core — the minimal foundation of Xerv (Point 1).
//!
//! Scope: errors, API version, configuration, logging, state, lifecycle.
//! Out of scope: TUI, CLI, plugins, modules, agents, networking, integrations.

pub mod api;
pub mod config;
pub mod core;
pub mod error;
pub mod logging;
pub mod state;
pub mod version;
pub use config::CoreConfig;
pub use core::XervCore;
pub use error::{Error, Result};
pub use state::CoreState;
pub use version::{api_version, API_VERSION};
