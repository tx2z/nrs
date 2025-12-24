# CLAUDE.md

This file provides guidance for Claude Code when working on this project.

## Project Overview

**nrs** (Node Run Scripts) is a fast, interactive TUI for running npm scripts. Written in Rust, it provides a visual interface for discovering and executing scripts from `package.json` files.

## Development Commands

```bash
# Build
cargo build              # Debug build
cargo build --release    # Release build

# Run
cargo run                # Run in current directory
cargo run -- --list      # List scripts non-interactively
cargo run -- --help      # Show help

# Test
cargo test               # Run all tests
cargo test --lib         # Run unit tests only
cargo insta test         # Run snapshot tests (update with --review)

# Quality
cargo fmt                # Format code
cargo clippy             # Run linter
cargo clippy -- -D warnings  # Strict linting (CI mode)

# Benchmarks
cargo bench              # Run benchmarks

# Documentation
cargo doc --open         # Generate and open docs
```

## Project Structure

```
src/
├── main.rs          # Entry point, CLI handling, TUI loop
├── lib.rs           # Library root with re-exports
├── cli.rs           # CLI argument definitions (clap)
├── error.rs         # Error types (thiserror)
├── config/          # Configuration system (TOML)
├── package/         # package.json parsing, script extraction
│   ├── manager.rs   # Package manager detection (npm/yarn/pnpm/bun)
│   ├── scripts.rs   # Script parsing
│   ├── descriptions.rs  # Description extraction
│   └── workspace.rs # Monorepo support
├── history/         # Execution history tracking
├── tui/             # Terminal UI (ratatui)
│   ├── app.rs       # Application state
│   ├── ui.rs        # UI rendering
│   ├── input.rs     # Keyboard handling
│   ├── layout.rs    # Layout calculations
│   ├── theme.rs     # Colors and styles
│   └── widgets/     # Individual UI components
├── runner/          # Script execution
├── filter/          # Fuzzy filtering
└── utils/           # Path and terminal utilities
```

## Key Files

- `Cargo.toml` - Dependencies and project config
- `docs/01-SPECS.md` - Full technical specification
- `docs/02-RUST-BEST-PRACTICES.md` - Coding guidelines
- `tests/` - Integration and unit tests
- `benches/` - Performance benchmarks

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` - no warnings allowed
- No `unwrap()` in production code - use proper error handling
- Add context to errors with `anyhow::Context`
- Write tests for new functionality

## Testing

Tests are in:
- `tests/integration/` - CLI and integration tests
- `tests/fixtures/` - Sample package.json files for testing
- Inline `#[cfg(test)]` modules for unit tests

Snapshot tests use `insta` crate. Update snapshots with:
```bash
cargo insta test --review
```

## Architecture Notes

- The TUI uses `ratatui` with `crossterm` backend
- Package manager is auto-detected from lock files or `packageManager` field
- History stored in `~/.config/nrs/history.json`
- Config loaded from `.nrsrc.toml` (project) or `~/.config/nrs/config.toml` (global)
