//! CLI mode dla `xerv` — tryby uruchamiania poza TUI.
//!
//! - `xerv` (brak arg)         → Core TUI
//! - `xerv install`            → interaktywny installer (user-space)
//! - `xerv uninstall`          → usuwa binarkę + symlink + data/config
//! - `xerv --version` / `xerv version` → wersja
//!
//! Brak zewnętrznych zależności — używamy `std::io::stdin` dla promptów.
//! Ten moduł jest wywoływany z `main.rs` jako side-effect (print + exit).

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use xerv_core::api::Error as ApiError;
use xerv_core::api::{CoreConfig, API_VERSION};

/// Domyślna lokalizacja binarki (zgodna z projektowanym layoutem).
pub const DEFAULT_BIN_DIR: &str = ".local/share/xerv/bin";
pub const DEFAULT_SYMLINK: &str = ".local/bin/xerv";
pub const DEFAULT_CONFIG_DIR: &str = ".config/xerv";

/// Rozwiązuje HOME (bez `dirs` crate — czytamy `$HOME`).
fn home_dir() -> xerv_core::api::ApiResult<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| ApiError::Other("HOME not set".into()))
}

/// Czy istnieje aktywna instalacja (sprawdzamy symlink + binarka)?
fn detect_installation(home: &Path) -> Option<PathBuf> {
    let symlink = home.join(DEFAULT_SYMLINK);
    let bin = home.join(DEFAULT_BIN_DIR).join("xerv");
    if symlink.is_symlink() || bin.exists() {
        Some(bin)
    } else {
        None
    }
}

/// Interaktywne pytanie Tak/Nie — czysty `stdin`, domyślnie `Yes`.
fn prompt_yes(message: &str) -> bool {
    let mut input = String::new();
    // stderr = logi; stdout = output. Prompt → stderr.
    eprintln!("{message} [Y/n] ",);
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

/// `xerv install` — interaktywny installer (user-space, brak sudo).
pub fn run_install() -> xerv_core::api::ApiResult<()> {
    let home = home_dir()?;
    let bin = home.join(DEFAULT_BIN_DIR).join("xerv");
    let symlink = home.join(DEFAULT_SYMLINK);
    let config_dir = home.join(DEFAULT_CONFIG_DIR);

    eprintln!("=== xerv install ===");
    eprintln!("Install location: {}", bin.display());

    // Wykrywanie istniejącej instalacji.
    if let Some(existing) = detect_installation(home.as_path()) {
        eprintln!("Detected existing installation at: {}", existing.display());
        if !prompt_yes("Upgrade existing installation?") {
            eprintln!("Install cancelled.");
            return Ok(());
        }
        // Proceed to overwrite — no early return.
    } else if !prompt_yes("Proceed with installation?") {
        eprintln!("Install cancelled.");
        return Ok(());
    }

    // Upewnij się, że binarka istnnieje — bootstrap powinien ją już pobrać.
    if !bin.exists() {
        eprintln!("ERROR: binary not found at {}", bin.display());
        eprintln!("Hint: run the install script from GitHub releases first:");
        eprintln!("  curl -fsSL https://github.com/XonofiliusPL/Xerv-Core/releases/latest/download/xerv-install.sh | bash");
        return Err(ApiError::Other("binary missing".into()));
    }
    // chmod +x
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

    // Symlink ~/.local/bin/xerv → binarka (jeśli ~/.local/bin istnieje).
    if config_dir.exists() {
        eprintln!("Config dir already exists: {}", config_dir.display());
    } else {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| ApiError::Other(format!("mkdir config: {e}")))?;
        eprintln!("Created config dir: {}", config_dir.display());
    }

    // Symlink — tylko jeśli ~/.local/bin istnieje w PATH.
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

/// `xerv uninstall` — usuwa binarkę + symlink + data + opcjonalnie config.
pub fn run_uninstall() -> xerv_core::api::ApiResult<()> {
    let home = home_dir()?;
    let bin = home.join(DEFAULT_BIN_DIR).join("xerv");
    let bin_dir = home.join(DEFAULT_BIN_DIR);
    let symlink = home.join(DEFAULT_SYMLINK);
    let config_dir = home.join(DEFAULT_CONFIG_DIR);

    eprintln!("=== xerv uninstall ===");

    // Usuń binarkę
    if bin.exists() {
        std::fs::remove_file(&bin).map_err(|e| ApiError::Other(format!("remove binary: {e}")))?;
        eprintln!("Removed: {}", bin.display());
    }
    // Usuń pusty bin dir jeśli pusty
    if bin_dir.exists() {
        let _ = std::fs::remove_dir(&bin_dir);
    }
    // Usuń symlink
    if symlink.is_symlink() || symlink.exists() {
        let _ = std::fs::remove_file(&symlink);
        eprintln!("Removed: {}", symlink.display());
    }
    // Data dir (domyślnie)
    let cfg = CoreConfig::default();
    if cfg.data_dir.exists() {
        std::fs::remove_dir_all(&cfg.data_dir)
            .map_err(|e| ApiError::Other(format!("remove data: {e}")))?;
        eprintln!("Removed: {}", cfg.data_dir.display());
    }

    // Config — pytaj
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
