//! CLI mode for `xerv` — modes running outside TUI.
//!
//! - `xerv` (no args)              → Core TUI
//! - `xerv update`                → updates Xerv to latest release
//! - `xerv --version` / `xerv version` → version
//!
//! No external dependencies — we use `std::io::stdin` for prompts.
//! This module is called from `main.rs` as a side-effect (print + exit).

use std::io::{self, Write};
use std::path::PathBuf;

use xerv_core::api::Error as ApiError;
use xerv_core::api::{ApiResult, API_VERSION};

/// Default binary location (matching planned layout).
pub const DEFAULT_BIN_DIR: &str = ".local/share/xerv/bin";

/// Resolve HOME (without `dirs` crate — read `$HOME`).
fn home_dir() -> ApiResult<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| ApiError::Other("HOME not set".into()))
}

/// Interactive Yes/No prompt — plain `stdin`, default `Yes`.
fn prompt_yes(message: &str) -> bool {
    let mut input = String::new();
    eprintln!("{message} [Y/n] ");
    let _ = io::stdout().flush();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    let t = input.trim().to_ascii_lowercase();
    t.is_empty() || t == "y" || t == "yes"
}

/// `xerv --version`
pub fn version() {
    println!("xerv {}", API_VERSION);
}

/// `xerv update` — fetch and install the latest release.
/// Uses the same mechanism as UpdateConfirm in TUI.
pub fn run_update() -> ApiResult<()> {
    eprintln!("=== xerv update ===");
    let home = home_dir()?;
    let bin = home.join(DEFAULT_BIN_DIR).join("xerv");

    let current = xerv_core::api::api_version();
    let info = crate::update::check_for_update(&current)?;
    let rel = match info {
        Some(r) => r,
        None => {
            eprintln!("xerv: already on latest version ({current})");
            return Ok(());
        }
    };

    eprintln!("Current: {current}");
    eprintln!("Latest:  {}", rel.version);
    if rel.is_prerelease {
        eprintln!("WARNING: this is a pre-release version.");
    }
    if !prompt_yes("Proceed with update?") {
        eprintln!("Update cancelled.");
        return Ok(());
    }

    eprintln!("Downloading and installing...");
    crate::update::download_and_install(&rel, &bin)?;
    eprintln!("Done. Restart 'xerv' to apply the update.");
    Ok(())
}
