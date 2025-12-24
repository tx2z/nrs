# nrs Project Documentation

## Quick Start Guide

Welcome! This folder contains everything you need to develop **nrs** (Node Run Scripts), a fast Rust TUI for running npm scripts.

---

## 📁 Documents Overview

| File | Description |
|------|-------------|
| `01-SPECS.md` | Complete technical specification - features, TUI design, all behaviors |
| `02-RUST-BEST-PRACTICES.md` | Rust coding guidelines for AI-assisted development |
| `03-RUST-INSTALLATION.md` | How to install Rust and set up your dev environment |
| `04-CLAUDE-CODE-PROMPTS.md` | Copy-paste prompts for Claude Code to build the project |

---

## 🚀 Getting Started

### Step 1: Install Rust

Follow `03-RUST-INSTALLATION.md` to install Rust on your machine.

Quick version (macOS):
```bash
brew install rustup-init
rustup-init
source $HOME/.cargo/env
rustup component add rustfmt clippy rust-analyzer
```

### Step 2: Start Development with Claude Code

1. Open Claude Code
2. Share these files with the AI:
   - `01-SPECS.md` (specification)
   - `02-RUST-BEST-PRACTICES.md` (coding guidelines)
3. Copy the first prompt from `04-CLAUDE-CODE-PROMPTS.md` (Phase 1.1)
4. Let Claude Code create the initial project

### Step 3: Build & Run

```bash
# Build
cargo build

# Run
cargo run

# Test
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

---

## 🎯 Development Phases

The project is split into 7 phases in the prompts document:

| Phase | Focus | Estimated Time |
|-------|-------|----------------|
| 1 | Project setup, CLI, package.json parsing | 2-3 hours |
| 2 | Configuration & history systems | 1-2 hours |
| 3 | TUI foundation (state, layout, widgets) | 3-4 hours |
| 4 | Script execution, fuzzy filter, main loop | 2-3 hours |
| 5 | `--last` rerun feature | 30 minutes |
| 6 | Polish, testing, documentation | 2-3 hours |
| 7 | Advanced features (optional) | As needed |

**Tip:** Complete phases 1-5 for a working MVP, then polish with phase 6.

---

## 📋 Key Commands

After the project is created:

```bash
# Development workflow
cargo watch -x "clippy -- -D warnings"  # Watch for changes

# Build release
cargo build --release

# Run with args
cargo run -- --list
cargo run -- --help
cargo run -- --script dev
cargo run -- --dry-run --script build

# Install locally
cargo install --path .
```

---

## 🎨 TUI Preview

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  📦 my-project                                                    pnpm ▪ ?   │
├──────────────────────────────────────────────────────────────────────────────┤
│  🔍 _                                                                        │
├──────────────────────────────────────────────────────────────────────────────┤
│  SCRIPTS                                                                     │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  1 ❯dev          2  build        3  test         4  lint              │  │
│  │  5  format       6  typecheck    7  serve        8  deploy            │  │
│  │  9  clean           docs            release         publish           │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────┤
│  DESCRIPTION                                                                 │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  Start the development server with hot reload                          │  │
│  │  ─────────────────────────────────────────────────────────────────────│  │
│  │  $ vite --mode development                                             │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────┤
│  ↑↓←→ move  ⏎ run  1-9 quick  / filter  s sort  a args  m multi  ? help    │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## ⚙️ Configuration

The app supports configuration files in TOML format:

**Location priority:**
1. `.nrsrc.toml` in project directory
2. `~/.config/nrs/config.toml` (global)

**Example config:**
```toml
[general]
default_sort = "recent"
column_direction = "horizontal"

[exclude]
patterns = ["pre*", "post*"]

[appearance]
icons = true
compact = false
```

---

## 🐛 Troubleshooting

### Rust not found
```bash
source $HOME/.cargo/env
```

### Build errors
```bash
cargo clean
cargo build
```

### Clippy errors
Claude Code should fix these - share the error output with it.

---

## 📚 Learning Resources

If you're new to Rust:

1. **The Rust Book**: `rustup doc --book`
2. **Rust by Example**: https://doc.rust-lang.org/rust-by-example/
3. **Rustlings**: https://github.com/rust-lang/rustlings

---

## 🤝 Support

If Claude Code gets stuck:
1. Share the error message
2. Reference the specific section in the spec
3. Ask it to explain what it's trying to do
4. Provide the Rust best practices document again if needed

Good luck building nrs! 🚀
