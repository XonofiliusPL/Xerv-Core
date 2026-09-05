//! CLI mode for `xerv` — modes running outside TUI.
//!
//! - `xerv` (no args)       → Core TUI
//! - `xerv install`          → interactive installer (user-space)
//! - `xerv uninstall`        → removes binary + symlink + data/config
//! - `xerv update`           → updates Xerv to latest release
//! - `xerv --version` / `xerv version` → version
//!
//! No external dependencies — we use `std::io::stdin` for prompts.
//! This module is called from `main.rs` as a side-effect (print + exit).

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use xerv_core::api::Error as ApiError;
use xerv_core::api::{ApiResult, CoreConfig, API_VERSION};

/// Default binary location (matching planned layout).
pub const DEFAULT_BIN_DIR: &str = ".local/share/xerv/bin";
pub const DEFAULT_SYMLINK: &str = ".local/bin/xerv";
pub const DEFAULT_CONFIG_DIR: &str = ".config/xerv";

/// Resolve HOME (without `dirs` crate — read `$HOME`).
fn home_dir() -> ApiResult<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| ApiError::Other("HOME not set".into()))
}

/// Check whether an active installation exists (symlink + binary).
fn detect_installation(home: &Path) -> Option<PathBuf> {
    let symlink = home.join(DEFAULT_SYMLINK);
    let bin = home.join(DEFAULT_BIN_DIR).join("xerv");
    if symlink.is_symlink() || bin.exists() {
        Some(bin)
    } else {
        None
    }
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
    println!("xerv {API_VERSION}");
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

/// `xerv install` — interactive installer (user-space, no sudo).
pub fn run_install() -> ApiResult<()> {
    let home = home_dir()?;
    let bin = home.join(DEFAULT_BIN_DIR).join("xerv");
    let symlink = home.join(DEFAULT_SYMLINK);
    let config_dir = home.join(DEFAULT_CONFIG_DIR);

    eprintln!("=== xerv install ===");
    eprintln!("Install location: {}", bin.display());

    let detected = detect_installation(home.as_path()).is_some();
    if detected {
        eprintln!("Detected existing installation — will upgrade in place.");
    }
    let prompt_msg = if detected {
        "Proceed with upgrade?"
    } else {
        "Proceed with installation?"
    };
    if !prompt_yes(prompt_msg) {
        eprintln!("Install cancelled.");
        return Ok(());
    }

    if !bin.exists() {
        eprintln!("ERROR: binary not found at {}", bin.display());
        eprintln!("Hint: run the install script from GitHub releases first:");
        eprintln!(
            "  curl -fsSL https://github.com/XonofiliusPL/Xerv-Core/releases/latest/download/xerv-install.sh | bash"
        );
        return Err(ApiError::Other("binary missing".into()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(&bin)
            .map(|m| m.permissions())
            .map_err(|e| ApiError::Other(format!("stat: {e}")))?;
        let mode = perms.mode();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(mode | 0o111))
            .map_err(|e| ApiError::Other(format!("chmod: {e}")))?;
    }

    if config_dir.exists() {
        eprintln!("Config dir already exists: {}", config_dir.display());
    } else {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| ApiError::Other(format!("mkdir config: {e}")))?;
        eprintln!("Created config dir: {}", config_dir.display());
    }

    let bin_parent = symlink.parent().unwrap();
    if bin_parent.exists() {
        if symlink.is_symlink() || symlink.exists() {
            std::fs::remove_file(&symlink)
                .map_err(|e| ApiError::Other(format!("remove old symlink: {e}")))?;
        }
        std::os::unix::fs::symlink(&bin, &symlink)
            .map_err(|e| ApiError::Other(format!("symlink: {e}")))?;
        eprintln!("Symlinked {} → {}", symlink.display(), bin.display());
    } else {
        eprintln!(
            "WARN: {} does not exist — add ~/.local/bin to your PATH",
            bin_parent.display()
        );
    }

    eprintln!("Done. Run 'xerv' to launch Xerv Core TUI.");
    Ok(())
}

/// `xerv uninstall` — removes binary + symlink + data + optionally config.
pub fn run_uninstall() -> ApiResult<()> {
    let home = home_dir()?;
    let bin = home.join(DEFAULT_BIN_DIR).join("xerv");
    let bin_dir = home.join(DEFAULT_BIN_DIR);
    let symlink = home.join(DEFAULT_SYMLINK);
    let config_dir = home.join(DEFAULT_CONFIG_DIR);

    eprintln!("=== xerv uninstall ===");

    if bin.exists() {
        std::fs::remove_file(&bin).map_err(|e| ApiError::Other(format!("remove binary: {e}")))?;
        eprintln!("Removed: {}", bin.display());
    }
    if bin_dir.exists() {
        let _ = std::fs::remove_dir(&bin_dir);
    }
    if symlink.is_symlink() || symlink.exists() {
        let _ = std::fs::remove_file(&symlink);
        eprintln!("Removed: {}", symlink.display());
    }
    let cfg = CoreConfig::default();
    if cfg.data_dir.exists() {
        std::fs::remove_dir_all(&cfg.data_dir)
            .map_err(|e| ApiError::Other(format!("remove data: {e}")))?;
        eprintln!("Removed: {}", cfg.data_dir.display());
    }

    if config_dir.exists() {
        if prompt_yes("Also remove config files (~/.config/xerv)?") {
            let _ = std::fs::remove_dir_all(&config_dir);
            eprintln!("Removed: {}", config_dir.display());
        } else {
            eprintln!("Kept: {}", config_dir.display());
        }
    }

    eprintln!("Done. Xerv Core has been uninstalled.");
    Ok(())
}
