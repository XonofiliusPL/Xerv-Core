# Xerv (npm)

Native binary wrapper for Xerv Core distributed via npm.

## Installation

```bash
npm install -g xerv
```

## Usage

```bash
xerv            # Launch Xerv Core TUI
xerv --version  # Print version
xerv help       # Print help
```

## Architecture

The npm package is a pure distribution layer. It does NOT contain any Xerv
logic — the entire application (Core, TUI, CLI, update system) is implemented
in Rust.

- `cli.js` — Node.js wrapper that spawns the native `xerv` binary
- `bin/xerv` — the pre-built Rust binary for the current platform

The wrapper simply forwards all command-line arguments to the native
binary and propagates its exit code.

## Platform Support

- Linux x86_64 — current focus
- macOS (Apple Silicon and Intel) — planned
- Windows — planned

## Source Code

The source code is available at:
https://github.com/XonofiliusPL/Xerv-Core

## License

Xerv Core is intended to be released under the **MIT OR Apache-2.0**
dual license.
