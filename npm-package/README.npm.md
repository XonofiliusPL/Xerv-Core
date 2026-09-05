# xerv (npm package)

[![npm](https://img.shields.io/npm/v/xerv.svg)](https://www.npmjs.com/package/xerv)

**Your unified terminal — monitor, navigate, and automate from one place.**

This is the official npm distribution of Xerv. It installs a small Node.js
launcher that runs the native Xerv binary. All of Xerv's functionality is
implemented in Rust — the launcher simply finds and runs the binary.

For the full project documentation, see the
[main README](https://github.com/XonofiliusPL/Xerv-Core#readme).

## Install

```bash
npm install -g xerv
```

No `sudo` required. Installs to your user directory.

Requires Node.js 18+ on Linux.

## Usage

```bash
xerv              # Launch the Xerv interactive terminal interface
xerv help         # Show available commands
xerv version      # Print the installed Xerv version
xerv update       # Download and install the latest release
```

## Uninstall

```bash
npm uninstall -g xerv
```

## What's Inside the Package

- `cli.js` — a minimal Node.js launcher that locates and runs the native binary.
  Contains no Xerv logic.
- `bin/xerv` — the native Xerv binary for Linux x86_64.

## Supported Platforms

- **Linux (x86_64)** — current focus

Future releases will add macOS and Windows via separate platform-specific packages.

## License

MIT OR Apache-2.0
