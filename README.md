# Xerv Core

Core runtime and stable API layer for the Xerv modular infrastructure.

## Status

**● Pre-alpha — active development**

Xerv Core is under active development. APIs, architecture and implementation details may change as the project evolves.

## Overview

Xerv Core is the foundational runtime and API layer of the Xerv ecosystem, providing lifecycle management, configuration, persistent state, logging, versioning and a stable interface for higher-level components.

The Core is intentionally kept independent from higher-level modules and plugins. Components built around Xerv should communicate through defined APIs, events and integration points rather than depending directly on internal implementation details.

## Current Features

- Core lifecycle management
- Persistent application state
- TOML-based configuration
- JSON state persistence
- Atomic state writes
- Structured logging with `tracing`
- API versioning
- Public API module and re-exports
- Error handling through a dedicated Core error type
- Rust-native library architecture

## Architecture

Xerv Core is designed as a small foundational library rather than a monolithic application.

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

The Core does not depend on modules or plugins. Higher-level functionality is expected to build on top of the Core through its public interfaces.

## API

Current API version:

```text
0.1.0
```

The public API is exposed through `xerv_core::api`.

Xerv Core is currently pre-alpha, so API compatibility guarantees are limited and breaking changes may occur during development.

## Development

Xerv Core is written in Rust.

Basic validation currently includes:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps
```

## Roadmap

Xerv Core will evolve incrementally as the wider Xerv architecture develops.

Planned areas include:

- expanding the Core API
- additional lifecycle and runtime capabilities
- stronger compatibility guarantees as the API matures
- foundations for higher-level Xerv components

The roadmap will be updated as features become formally planned and implemented.

## Contributing

Contributions, feedback and technical discussion are welcome as the project evolves.

Please use GitHub Issues for bugs, proposals and technical discussion related to Xerv Core.

## License

Xerv Core is intended to be released under the **MIT OR Apache-2.0** dual license.

---

**Xerv** — Modular infrastructure for systems, agents and automation.
