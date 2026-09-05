//! Xerv TUI — consumer of xerv-core.
//!
//! Contains:
//! - `app` — screen-based UI state model
//! - `event` — crossterm event → `Event` enum
//! - `ui` — ratatui render (single main area)
//! - `cli` — CLI mode (install / uninstall / version), same binary as TUI
//! - `update` — update system (network-free: check/comparison/select;
//!   network: `check_for_update`, `download_and_install`)

pub mod app;
pub mod cli;
pub mod event;
pub mod ui;
pub mod update;
