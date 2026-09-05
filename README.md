# Xerv

[![Xerv Core](https://img.shields.io/badge/Xerv_Core-0.1.0-blue)]()
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue)]()
[![GitHub](https://img.shields.io/badge/GitHub-XonofiliusPL%2FXerv--Core-blue)]()

**Xerv** is a modular infrastructure for systems, agents, and automation.

**Xerv Core** is the foundational Rust library providing the runtime,
configuration, state management, logging, versioning, and lifecycle
primitives upon which the wider Xerv ecosystem is built.

---

## Overview

Xerv Core is the foundational runtime and API layer of the Xerv ecosystem,
providing lifecycle management, configuration, persistent state, logging,
versioning, and a stable interface for higher-level components.

The Core is intentionally kept independent from higher-level modules and
plugins. Components built around Xerv should communicate through defined
APIs, events, and integration points rather than depending directly on
internal implementation details.

## Xerv Core

The Rust foundation of Xerv. Provides:

- **Core lifecycle management** — initialization and graceful shutdown
- **Persistent application state** — JSON-backed, atomic writes
- **TOML-based configuration** — extensible configuration loader
- **JSON state persistence** — structured state with schema versioning
- **Atomic state writes** — safe concurrent state access via `Mutex`
- **Structured logging** via `tracing` / `tracing-subscriber`
- **API versioning** — stable `API_VERSION` contract
- **Public API module** and re-exports
- **Error handling** through a dedicated Core error type
- **Rust-native library architecture**

## Xerv TUI

A terminal user interface built with `ratatui` + `crossterm`. Features a
screen-based navigation model with keyboard and mouse interaction support.

## CLI Mode

The `xerv` binary also supports CLI commands for installation,
uninstallation, updating, and version printing — all operating in
user-space without requiring `sudo`.

## Current Status

**● Pre-alpha — active development**

Xerv Core is under active development. APIs, architecture, and
implementation details may change as the project evolves.

## Architecture

Xerv Core is designed as a small foundational library rather than a
monolithic application.

```text
Higher-level components
        │
        ▼
   Xerv API layer
        │
        ▼
    Xerv Core
    ├── Lifecycle
    ├── Configuration
    ├── State
    ├── Logging
    └── Versioning
```

The Core does not depend on modules or plugins. Higher-level functionality
is expected to build on top of the Core through its public interfaces.

## Current Capabilities

### Rust Core (`xerv-core`)

- `XervCore` — main runtime instance with lifecycle management
- `CoreConfig` — TOML-based configuration with sensible defaults
- `CoreState` — persistent state with schema versioning and atomic writes
- `Error` enum — structured error handling with `thiserror`
- `API_VERSION` — stable semantic versioning contract (`0.1.0`)
- `tracing` subscriber initialization with `RUST_LOG` support

### TUI (`xerv-tui`)

- Screen-based navigation model (Main, Help, Settings, UpdateConfirm)
- Keyboard navigation: arrow keys, Enter, Backspace, `h`, `q`, `U`
- Mouse interaction: hover highlighting and click-to-activate
- Help screen with keyboard shortcuts and source link
- Update confirmation screen with version comparison

### CLI (`xerv` binary)

- `xerv` — launch Core TUI
- `xerv install` — interactive user-space installer
- `xerv uninstall` — remove installation (binary, symlink, data, config)
- `xerv update` — update to latest GitHub release (with SHA256 verification)
- `xerv version` / `xerv --version` — print current version
- `xerv help` / `xerv -h` / `xerv --help` — print usage information

### Update System

- GitHub Releases API integration for version detection
- SHA256 checksum verification for safe downloads
- Atomic installation with backup and rollback on failure
- Supports `.tar.gz` and `.tar.zst` artifacts

## Supported Environment

The project currently targets **Linux** (`x86_64-unknown-linux-gnu`,
`aarch64-unknown-linux-gnu`) and **macOS** (`x86_64-apple-darwin`,
`aarch64-apple-darwin`). Other targets may work but are not actively
tested.

## Getting Started

### Prerequisites

- **Rust** 1.74 or later
- A Unix-like environment (Linux or macOS)

### Installation

```bash
xerv install
```

This runs the interactive installer, which sets up the binary, symlink,
and configuration directory in user-space.

### Running Xerv

```bash
xerv          # Launch Xerv Core TUI
xerv version  # Print version information
xerv help     # Show command help
```

### Updating Xerv

```bash
xerv update   # Fetch and install the latest release
```

## Repository Structure

```
Xerv-Core/
├── Cargo.toml              # Workspace manifest
├── Cargo.lock              # Locked dependency versions
├── README.md               # This file
├── DESIGN.md               # Visual design specification
├── crates/
│   ├── xerv-core/          # Core library (errors, config, state, logging, lifecycle)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs      # Public API, module declarations
│   │       ├── api.rs      # Stable public API re-exports and aliases
│   │       ├── error.rs    # Error type definitions
│   │       ├── config.rs   # Core configuration
│   │       ├── state.rs    # Persistent state management
│   │       ├── logging.rs  # Tracing subscriber initialization
│   │       ├── core.rs     # XervCore runtime instance
│   │       ├── version.rs  # API version constants
│   │       └── tests/      # Core tests (error, config, state, lifecycle, ...)
│   └── xerv-tui/           # Terminal UI and CLI
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs     # Entry point, CLI dispatch, TUI loop
│           ├── app.rs      # Application state model (screens, navigation)
│           ├── cli.rs      # CLI commands (install, uninstall, update, version)
│           ├── event.rs    # Input event handling (keyboard, mouse)
│           ├── ui.rs       # ratatui rendering (header, footer, screens)
│           ├── update.rs   # Update system (GitHub API, download, install)
│           ├── lib.rs      # Module declarations
│           └── tests/      # TUI tests (app, dashboard, header, CLI dispatch)
├── .gitignore
└── .claude/                # Claude Code settings (local, not committed)
```

## Development

Xerv Core is written in Rust.

Basic validation currently includes:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps --workspace
```

## Build & Test

All workspace components build and test together:

```bash
cargo build               # Build the workspace
cargo test --workspace    # Run all tests
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
cargo doc --no-deps --workspace
```

Current validation results: build ✓, tests ✓, clippy ✓, fmt ✓, doc ✓

## For Developers

### Contributing

Contributions, feedback, and technical discussion are welcome as the
project evolves.

Please use GitHub Issues for bugs, proposals, and technical discussion
related to Xerv Core.

### License

Xerv Core is intended to be released under the **MIT OR Apache-2.0**
dual license.

### Source Code

The source code is available at:
https://github.com/XonofiliusPL/Xerv-Core

## Roadmap

Xerv Core will evolve incrementally as the wider Xerv architecture develops.

**Current (Points 1–5):**
- Point 1: Rust Core — errors, configuration, state, logging, lifecycle
- Point 2: Stable API module and contract
- Point 3: Agent Workspace in dashboard
- Point 4: TUI navigation and visual identity
- Point 5: GitHub release detection and safe update mechanism

**Planned areas:**
- Expanding the Core API surface
- Additional lifecycle and runtime capabilities
- Stronger compatibility guarantees as the API matures
- Foundations for higher-level Xerv components

The roadmap will be updated as features become formally planned and
implemented.

## License

Xerv Core is intended to be released under the **MIT OR Apache-2.0**
dual license.

---

**Xerv** — Modular infrastructure for systems, agents and automation.
