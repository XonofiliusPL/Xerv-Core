# xerv

[![npm](https://img.shields.io/npm/v/xerv.svg)](https://www.npmjs.com/package/xerv)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

**Modular terminal infrastructure for systems, agents, and automation.**

> This is the npm distribution package. It provides the native Rust binary
> for Xerv. For full documentation, see the
> [main README](https://github.com/XonofiliusPL/Xerv-Core).

## Installation

```bash
npm install -g xerv
```

This installs the `xerv` command globally — no `sudo` required.

## Usage

```bash
xerv              # Launch Xerv TUI
xerv help         # Show commands
xerv version      # Print version
xerv update       # Update to latest release
```

To uninstall:

```bash
npm uninstall -g xerv
```

## What This Package Contains

- `cli.js` — a minimal Node.js launcher that locates and runs the native
  binary. It contains **no Xerv logic** — Rust is the sole implementation.
- `bin/xerv` — the pre-built native Rust binary for Linux x86_64.

This wrapper simply forwards all command-line arguments to the native binary
and propagates its exit code.

## Supported Platforms

- **Linux x86_64** — current focus

Future releases will add per-platform npm packages for macOS and Windows.

## Source Code

The source code is available at:
**https://github.com/XonofiliusPL/Xerv-Core**

## License

Xerv Core is intended to be released under the **MIT OR Apache-2.0**
dual license.
