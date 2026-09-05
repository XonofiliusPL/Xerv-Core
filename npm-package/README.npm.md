# xerv (npm package)

[![npm](https://img.shields.io/npm/v/xerv.svg)](https://www.npmjs.com/package/xerv)

**Modular terminal infrastructure for systems, agents, and automation.**

This is the official npm distribution of Xerv. It provides a small Node.js
launcher that runs the native Rust binary.

For the full project documentation, see the
[main README](https://github.com/XonofiliusPL/Xerv-Core#readme).

## Install

```bash
npm install -g xerv
```

No `sudo` needed. Installs to your user directory.

## Usage

```bash
xerv              # Launch the Xerv TUI
xerv help         # Show available commands
xerv version      # Print Xerv version
xerv update       # Update to the latest release
```

## Uninstall

```bash
npm uninstall -g xerv
```

## What's Inside

- `cli.js` — a minimal Node.js launcher that runs the native binary.
  Contains no Xerv logic; Rust is the sole implementation.
- `bin/xerv` — the native Xerv binary (Linux x86_64).

## Supported Platforms

- **Linux x86_64** — current focus

Future releases will add macOS and Windows via separate platform-specific packages.

## License

MIT OR Apache-2.0
