# Xerv

[![npm](https://img.shields.io/npm/v/@xonofilius/xerv.svg)](https://www.npmjs.com/package/@xonofilius/xerv)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

**Your unified terminal — monitor, navigate, and automate from one place.**

---

## What is Xerv?

If you spend time in a terminal, you have probably felt this: switching between
`htop`, `dmesg`, `kubectl`, log files, and a dozen other tools just to get a
complete picture of what is happening. Each tool tells you one thing. None of
them talk to each other. And none of them adapt to what you need *right now*.

Xerv is different. It is a single terminal application that brings monitoring,
navigation, and automation together in one interactive interface. You launch
it once, and it becomes your central place for understanding and acting on
what is happening in your system.

Built in Rust, Xerv is fast, reliable, and runs anywhere you have a terminal —
on your laptop, a remote server over SSH, or an embedded device.

Xerv is currently in **pre-alpha**: the core is being built. What you see today
is the beginning — an interactive interface, a command system, and automatic
updates. You can run it, explore the first screens, and see where it is heading.

---

## Why Xerv?

Every day, terminal users waste time:

- **Searching** across tools to piece together what is going on.
- **Remembering** which command does what, in which tool.
- **Switching contexts** between viewing, diagnosing, and acting.

Xerv replaces all of that with one question: **What if your terminal adapted
to you, not the other way around?**

Instead of memorizing a dozen commands across a dozen tools, Xerv gives you a
single, consistent interface. A place that shows you what matters, lets you
act on it immediately, and remembers how you like to work.

### Who is Xerv for?

| You are... | Xerv helps you... |
|---|---|
| **A developer** | See your app, logs, and processes in one view — no tab-switching. |
| **A system administrator** | React to incidents fast, in a single screen, without leaving the terminal. |
| **An automation engineer** | Orchestrate, monitor, and update your tools from one control center. |
| **Anyone who works in a terminal** | A clean, keyboard-driven interface instead of a wall of commands. |

---

## What Can Xerv Do Today?

Xerv offers a solid foundation — and a clear path forward:

- **An interactive terminal UI** that runs directly in your terminal window.
  No GUI, no browser — just your terminal.
- **Keyboard-driven navigation** — move through the interface, open sections,
  and trigger actions using only your keyboard.
- **A built-in command system** — quick commands like `xerv --version`,
  `xerv help`, and `xerv update` for common tasks.
- **Automatic version tracking** — Xerv knows what version you are running
  and can update itself.
- **Self-updating with rollback** — when a new version is available, Xerv
  downloads it, verifies its checksum, and installs it atomically. If
  something goes wrong, it rolls back automatically.

What is **not** here yet: process monitoring, log streaming, dashboards,
custom data views. These are coming. Today is about the interface and the
foundation.

---

## Features

### Interactive Terminal Interface (TUI)

Xerv opens a full-screen interface inside your terminal. There is no separate
app to install, no browser tab, no background service. It runs where you work.

**Why it matters:** You see everything in one place. No more jumping between
tools — Xerv brings the information you need directly to your terminal window.

### Keyboard-Driven Navigation

Every part of the Xerv interface — menus, sections, commands — is controlled
by your keyboard. Arrow keys, Enter, and a few shortcuts are all you need.

**Why it matters:** You stay in flow. No reaching for the mouse, no hunting
through menus. Muscle memory takes over quickly.

### Command System

Beyond the TUI, Xerv exposes a simple command-line interface:

```bash
xerv              # Launch the interactive interface
xerv help         # See all available commands
xerv version      # Check your installed version
xerv update       # Download and install the latest release
```

**Why it matters:** Sometimes you just need one piece of information — and
you want it fast. Quick commands let you get answers without entering the
full interface.

### Automatic Updates with Rollback

Xerv checks for new releases, downloads them, verifies their integrity with
a SHA-256 checksum, and installs them atomically. If an update fails for any
reason, Xerv automatically restores the previous version.

**Why it matters:** You always get the latest fixes and features — safely.
A broken update should never leave you stranded. With Xerv, it does not.

---

## The Xerv Interface

When you launch `xerv`, a full-screen terminal interface opens. It is divided
into clear, purposeful sections:

- **Header** — shows the project name, version, and live status.
- **Sidebar** — navigable menu of available sections.
- **Main area** — displays the content of the selected section.
- **Command bar** — type `help`, `version`, or any supported command for
  quick access without leaving the interface.

### Navigation

| Key | Action |
|---|---|
| `↑ ↓ ← →` | Move through the interface |
| `Enter` | Open the selected section |
| `Backspace` | Go back |
| `q` or `Esc` | Quit |
| `h` | Open in-interface help |
| `U` | Open the update screen (when an update is available) |
| `y` | Confirm an action (e.g., update) |
| `n` | Cancel an action |

> 📸 **Screenshots and animated demos** are coming once the interface is
> finalized. This space is reserved for visual walkthroughs.

---

## Installation

Xerv is installed like any Node package — in your user space, no `sudo` needed:

```bash
npm install -g @xonofilius/xerv
```

Requires Node.js 18+ and a Linux or macOS system.

---

## Getting Started

After installing, launch Xerv:

```bash
xerv
```

That is it. The interactive interface opens immediately.

Want to check your version or see what commands are available?

```bash
xerv version     # Shows the installed Xerv version
xerv help        # Lists all available commands
```

---

## Updating

You can update Xerv in two ways:

1. **From inside the TUI** — when a new version is available, Xerv shows an
   update notification. Open the update screen with `U`, confirm with `y`.

2. **From the command line**:

```bash
xerv update
```

This downloads the latest release, verifies it, and installs it atomically
with automatic rollback on failure.

> The current update system checks GitHub for new releases, downloads the
> binary, and verifies a SHA-256 checksum. This is a working prototype —
> future versions will extend this to addons and modular components.

---

## Uninstall

Remove Xerv completely:

```bash
npm uninstall -g @xonofilius/xerv
```

This removes the binary and all associated files.

---

## Addons, Plugins, and Modules

Xerv is being designed to support a future **addons ecosystem** — a way to
extend functionality with custom views, data sources, and automation modules
without modifying the core.

> **[Planned]** The addons/plugins/modules system does not exist yet. This
> section is reserved for future functionality and will be populated once the
> architecture is finalized.

Interested in helping shape the addon API? [Open an issue](https://github.com/XonofiliusPL/Xerv-Core/issues).

---

## Screenshots / Demo

> Visual materials will be added once the interface is finalized. This
> section is reserved for screenshots and animated demonstrations.

---

## Roadmap

| Feature | Status |
|---|---|
| Rust Core — errors, config, state, logging, lifecycle | ✅ Done |
| Stable API and versioning | ✅ Done |
| Agent Workspace with state model and workspace tree | ✅ Done |
| Terminal UI — navigation, layout, visual identity | ✅ Done |
| Update system — GitHub releases, checksum, atomic install | ✅ Done |
| npm distribution — `npm install -g @xonofilius/xerv` | ✅ Done |
| Process and system monitoring views | 🔶 Planned |
| Log streaming and data integration in TUI | 🔷 Future |
| Addons / plugins / modules system | 🔷 Future |
| Themes and visual customization | 🔷 Future |

**Legend:** ✅ Done · 🔶 Planned · 🔷 Future

---

## Project Status

**Pre-alpha — active development**

Xerv is in early development. Today's release is a functional foundation — an
interactive interface, a command system, and automatic, safe updates. APIs,
architecture, and behavior will evolve as Xerv grows.

---

## Links

- **GitHub**: [XonofiliusPL/Xerv-Core](https://github.com/XonofiliusPL/Xerv-Core)
- **npm**: [@xonofilius/xerv](https://www.npmjs.com/package/@xonofilius/xerv)
- **Report an issue**: [GitHub Issues](https://github.com/XonofiliusPL/Xerv-Core/issues)
- **View changes**: [Commit history](https://github.com/XonofiliusPL/Xerv-Core/commits/main)

---

## License

Xerv Core is intended to be released under the **MIT OR Apache-2.0** dual
license.

---

*Xerv — built in Rust. For everyone who works in a terminal.*
