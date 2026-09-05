use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::{Error, Result};

/// Initializes the global `tracing` subscriber with a filter from `RUST_LOG`
/// (fallback: `level`). Idempotent: a second init returns `Error::Other`
/// instead of panicking.
pub fn init(level: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let layer = fmt::layer().with_target(false);
    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .try_init()
        .map_err(|e| Error::Other(format!("logging init: {e}")))
}
