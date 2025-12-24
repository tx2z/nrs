# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-12-23

### Added

- Initial release of nrs (Node Run Scripts)
- Interactive TUI for selecting and running npm scripts
- Package manager auto-detection (npm, yarn, pnpm, bun)
  - Detection via `packageManager` field in package.json
  - Detection via lock files (package-lock.json, yarn.lock, pnpm-lock.yaml, bun.lockb)
- Fuzzy filtering for scripts
- Multiple sort modes: recent, alphabetical, category
- Script execution history tracking
- Rerun last script with `--last` flag
- Script descriptions from multiple sources:
  - `scripts-info` field in package.json
  - `ntl.descriptions` field in package.json
  - Comment format (`//script-name`)
- Configuration via TOML files
  - Project-level `.nrsrc.toml`
  - Global `~/.config/nrs/config.toml`
- Exclude patterns for filtering scripts
- Multi-select mode for running multiple scripts
- Dry-run mode to preview commands
- Debug output for troubleshooting
- Keyboard navigation with vim-style keys (hjkl)
- Quick selection with number keys (1-9)
- Shell completion generation (bash, zsh, fish)

### Technical

- Written in Rust for fast startup (~30ms)
- Uses ratatui for the terminal UI
- Comprehensive test suite with unit and integration tests
- Snapshot tests with insta

[Unreleased]: https://github.com/user/nrs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/user/nrs/releases/tag/v0.1.0
