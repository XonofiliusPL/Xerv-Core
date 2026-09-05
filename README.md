# Xerv

[![npm](https://img.shields.io/npm/v/xerv.svg)](https://www.npmjs.com/package/xerv)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-XonofiliusPL%2FXerv--Core-blue)](https://github.com/XonofiliusPL/Xerv-Core)

**Modular terminal infrastructure for systems, agents, and automation.**

---

## What is Xerv?

Xerv is a modern TUI-based platform designed to bring together system monitoring,
agent orchestration, and automation workflows into a single, keyboard-driven
terminal interface. Built in Rust for performance and reliability, Xerv provides
a foundation for building composable, interactive terminal tools.

## Why Xerv?

- **Terminal-native** — works in any terminal, no GUI required
- **Keyboard-driven navigation** — designed for power users who live in the terminal
- **Live system insights** — monitor processes, resources, and services in real-time
- **Agent workflow** — integrate AI agents and automation directly into your terminal session
- **Extensible architecture** — future plugin/addons system for custom functionality
- **Cross-platform** — built for Linux and macOS (Windows coming later)

## What Can Xerv Do Right Now?

**Xerv Core** is currently in **pre-alpha** development. Here is what the
current build provides:

- **Interactive TUI** — a screen-based terminal interface with smooth keyboard
  and mouse navigation
- **Core runtime** — lifecycle management, configuration, and persistent state
- **Built-in commands** — version info, help, and automatic update checking
- **Update system** — checks GitHub releases, verifies SHA256 checksums, and
  performs safe atomic binary replacement with rollback on failure

### Keyboard Shortcuts

| Key         | Action                            |
|-------------|-----------------------------------|
| `q` / `Esc` | Quit                              |
| `h`         | Open Help                         |
| `← ↑ → ↓`   | Navigate the menu                 |
| `Enter`     | Confirm / activate selection      |
| `Backspace` | Return to previous screen         |
| `U`         | Open Update screen (when available) |
| `y`         | Confirm update                    |
| `n`         | Cancel update                     |

---

## Installation

Xerv is distributed as an npm package, which provides the native Rust binary.

```bash
npm install -g xerv
```

This installs the `xerv` binary globally in user-space — **no `sudo` required**.

### System Requirements

- **Node.js** 18+ (for the npm wrapper)
- **Linux** (x86_64) — current supported platform
- macOS — supported (Apple Silicon and Intel)

---

## Getting Started

Launch Xerv:

```bash
xerv
```

This starts the interactive TUI. Use the arrow keys to navigate, `Enter` to
select, and `q` to quit.

### Commands

```bash
xerv              # Launch the TUI
xerv help         # Show available commands
xerv version      # Print the installed version
```

---

## Updating

Xerv checks for new releases automatically on startup. When an update is
available, an **Update** option appears in the main menu. You can also update
manually from the command line:

```bash
xerv update
```

Each update is verified with a SHA256 checksum and installed atomically, with
automatic rollback on failure.

After updating, restart Xerv to apply changes.

---

## Uninstall

Remove Xerv completely:

```bash
npm uninstall -g xerv
```

This removes the binary and all associated files.

---

## Screenshots

> Screenshots and animated GIFs will be added here as the TUI is finalized.

---

## Addons, Plugins, and Modules

Xerv is designed with a future **addons ecosystem** that will allow the
community to extend functionality with custom views, data sources, and
automation modules.

> **[Planned]** The addons/plugins/modules system does not yet exist.
> This section is reserved for future functionality and will be populated
> once the architecture is finalized.

If you are interested in contributing to or designing the addons API, please
[open an issue](https://github.com/XonofiliusPL/Xerv-Core/issues).

---

## Themes

Xerv follows a **dark terminal aesthetic** with a carefully designed color
palette:

| Role            | Color      | Purpose                          |
|-----------------|------------|----------------------------------|
| Primary         | Cyan       | Interactive elements, brand      |
| Secondary       | Magenta    | Selected info, accents           |
| Neutral         | White      | Data values                      |
| Muted           | Dark Gray  | Labels, separators, idle state   |
| Success         | Green      | Ready status                     |
| Danger          | Red        | Shutdown status                  |

> **[Planned]** Theme configuration and user-customizable color schemes.

---

## Roadmap

Xerv is in active pre-alpha development. Here is the short-term focus:

| Milestone | Status      | Description |
|-----------|-------------|-------------|
| Point 1   | ✅ Done     | Rust Core — errors, config, state, logging, lifecycle |
| Point 2   | ✅ Done     | Stable API module and contract |
| Point 3   | ✅ Done     | Agent Workspace in dashboard |
| Point 4   | ✅ Done     | TUI navigation and visual identity |
| Point 5   | ✅ Done     | GitHub release detection and safe update mechanism |
| npm distribution | ✅ Done | Official installation via `npm install -g xerv` |

**Future areas:**
- Addons/plugins/modules ecosystem
- Additional lifecycle and runtime capabilities
- Cross-platform binary packages
- Stronger API compatibility guarantees
- Themes and configuration customization

---

## Project Status

**● Pre-alpha — active development**

Xerv Core is under active development. APIs, architecture, and implementation
details may change as the project evolves.

---

## Links

- **GitHub**: [XonofiliusPL/Xerv-Core](https://github.com/XonofiliusPL/Xerv-Core)
- **npm**: [xerv](https://www.npmjs.com/package/xerv) — (not yet published)
- **Issues**: [GitHub Issues](https://github.com/XonofiliusPL/Xerv-Core/issues)

---

## License

Xerv Core is intended to be released under the **MIT OR Apache-2.0**
dual license.

---

*Made with ❤️ in Rust.*
