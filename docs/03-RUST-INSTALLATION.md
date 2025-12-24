# Rust Development Environment Setup

## Complete Guide for macOS and Linux

This guide will help you set up a complete Rust development environment for the `nrs` project.

---

## Table of Contents

1. [Install Rust](#1-install-rust)
2. [Configure Your Shell](#2-configure-your-shell)
3. [Essential Tools](#3-essential-tools)
4. [IDE Setup](#4-ide-setup)
5. [Project Setup](#5-project-setup)
6. [Common Commands](#6-common-commands)
7. [Troubleshooting](#7-troubleshooting)

---

## 1. Install Rust

### 1.1 macOS with Homebrew (Recommended for macOS users)

The cleanest way to install Rust on macOS is via Homebrew + rustup:

```bash
# Install rustup via Homebrew
brew install rustup-init

# Run the initializer
rustup-init
```

During installation, choose option `1` for the default installation.

After installation completes:

```bash
# Add Rust to your PATH
source $HOME/.cargo/env
```

**Why this approach?**
- Homebrew manages the rustup binary itself
- rustup manages Rust versions, toolchains, and components
- Best of both worlds!

### 1.2 Linux / Alternative Method

If you're on Linux or prefer not to use Homebrew:

```bash
# Install rustup directly
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

During installation, choose option `1` for the default installation.

### 1.2 Verify Installation

After installation, restart your terminal or run:

```bash
source $HOME/.cargo/env
```

Then verify:

```bash
# Check Rust version
rustc --version
# Should output something like: rustc 1.75.0 (82e1608df 2023-12-21)

# Check Cargo version
cargo --version
# Should output something like: cargo 1.75.0 (1d8b05cdd 2023-11-20)

# Check rustup version
rustup --version
# Should output something like: rustup 1.26.0 (5af9b9484 2023-04-05)
```

### 1.3 Update Rust

Keep Rust updated:

```bash
# Update to latest stable
rustup update stable

# Update rustup itself
rustup self update
```

---

## 2. Configure Your Shell

### 2.1 Add to PATH

The installer should add this automatically, but verify your shell config has:

**For Bash (~/.bashrc or ~/.bash_profile):**
```bash
# Rust
export PATH="$HOME/.cargo/bin:$PATH"
```

**For Zsh (~/.zshrc):**
```bash
# Rust
export PATH="$HOME/.cargo/bin:$PATH"
```

**For Fish (~/.config/fish/config.fish):**
```fish
# Rust
set -gx PATH $HOME/.cargo/bin $PATH
```

### 2.2 Enable Cargo Completions

**Bash:**
```bash
# Add to ~/.bashrc
source "$(rustc --print sysroot)/etc/bash_completion.d/cargo"
```

**Zsh:**
```bash
# Add to ~/.zshrc
autoload -U compinit
compinit
fpath=(~/.zfunc $fpath)
rustup completions zsh > ~/.zfunc/_rustup
rustup completions zsh cargo > ~/.zfunc/_cargo
```

**Fish:**
```fish
# Run once
mkdir -p ~/.config/fish/completions
rustup completions fish > ~/.config/fish/completions/rustup.fish
rustup completions fish cargo > ~/.config/fish/completions/cargo.fish
```

---

## 3. Essential Tools

### 3.1 Install Required Components

```bash
# Code formatting
rustup component add rustfmt

# Linting
rustup component add clippy

# Documentation
rustup component add rust-docs

# Rust Language Server (for IDE support)
rustup component add rust-analyzer
```

### 3.2 Install Useful Cargo Extensions

```bash
# Fast file watcher for development
cargo install cargo-watch

# Check all features compile
cargo install cargo-hack

# Show dependency tree
cargo install cargo-tree

# Audit dependencies for vulnerabilities
cargo install cargo-audit

# Generate code coverage
cargo install cargo-tarpaulin

# Profile performance (optional)
cargo install flamegraph

# Faster linking (Linux only - recommended)
# First install lld: sudo apt install lld (Ubuntu) or sudo dnf install lld (Fedora)

# Faster linking (macOS - use default or zld)
# brew install michaeleisel/zld/zld
```

### 3.3 Configure Faster Linking (Recommended)

Create or edit `~/.cargo/config.toml`:

```toml
# Faster linking configuration

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.aarch64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

# Enable incremental compilation (default in dev, but be explicit)
[build]
incremental = true

# Use all CPU cores
[build]
jobs = 0  # 0 means auto-detect
```

### 3.4 System Dependencies

**macOS:**
```bash
# Install Xcode command line tools (if not installed)
xcode-select --install

# Optional: Install lld via Homebrew for faster linking
brew install llvm
```

**Ubuntu/Debian:**
```bash
# Build essentials
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Faster linker (recommended)
sudo apt install lld clang
```

**Fedora:**
```bash
# Build essentials
sudo dnf groupinstall "Development Tools"
sudo dnf install openssl-devel

# Faster linker (recommended)
sudo dnf install lld clang
```

**Arch Linux:**
```bash
# Build essentials
sudo pacman -S base-devel openssl

# Faster linker (recommended)
sudo pacman -S lld clang
```

---

## 4. IDE Setup

### 4.1 VS Code (Recommended)

**Install Extensions:**

1. **rust-analyzer** (essential) - Rust language support
   - Extension ID: `rust-lang.rust-analyzer`

2. **Even Better TOML** - TOML file support
   - Extension ID: `tamasfe.even-better-toml`

3. **crates** - Dependency version hints
   - Extension ID: `serayuzgur.crates`

4. **Error Lens** - Inline error display
   - Extension ID: `usernamehw.errorlens`

**VS Code Settings (.vscode/settings.json):**

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.tabSize": 4
  }
}
```

### 4.2 JetBrains (RustRover or IntelliJ + Plugin)

1. Download RustRover: https://www.jetbrains.com/rust/
2. Or install the Rust plugin in IntelliJ IDEA

### 4.3 Neovim

If using Neovim with LSP:

```lua
-- In your LSP config
require('lspconfig').rust_analyzer.setup({
  settings = {
    ['rust-analyzer'] = {
      checkOnSave = {
        command = "clippy"
      },
      cargo = {
        features = "all"
      }
    }
  }
})
```

### 4.4 Terminal-Only Development

If you prefer terminal:

```bash
# Watch and rebuild on changes
cargo watch -x "check" -x "clippy" -x "test"

# Or just check
cargo watch -x check

# Run with auto-reload
cargo watch -x run
```

---

## 5. Project Setup

### 5.1 Clone and Setup

```bash
# Clone the repository (when created)
git clone https://github.com/yourusername/nrs.git
cd nrs

# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run

# Run with arguments
cargo run -- --help
cargo run -- --list
```

### 5.2 Development Workflow

```bash
# Terminal 1: Watch for changes and check
cargo watch -x "clippy -- -D warnings"

# Terminal 2: Run the app when needed
cargo run

# Before committing
cargo fmt
cargo clippy -- -D warnings
cargo test
```

### 5.3 Create Project from Scratch

If starting fresh:

```bash
# Create new project
cargo new nrs
cd nrs

# Add dependencies
cargo add clap --features derive
cargo add ratatui
cargo add crossterm
cargo add serde --features derive
cargo add serde_json
cargo add toml
cargo add dirs
cargo add fuzzy-matcher
cargo add chrono --features serde
cargo add anyhow
cargo add thiserror

# Add dev dependencies
cargo add --dev criterion
cargo add --dev tempfile
cargo add --dev assert_cmd
cargo add --dev predicates
cargo add --dev insta
```

---

## 6. Common Commands

### 6.1 Building

```bash
# Debug build (fast compile, slow runtime)
cargo build

# Release build (slow compile, fast runtime)
cargo build --release

# Check without building (fastest)
cargo check

# Build documentation
cargo doc --open
```

### 6.2 Running

```bash
# Run debug build
cargo run

# Run with arguments
cargo run -- arg1 arg2

# Run release build
cargo run --release

# Run specific binary
cargo run --bin nrs
cargo run --bin ns
```

### 6.3 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Run tests in release mode
cargo test --release
```

### 6.4 Linting & Formatting

```bash
# Format code
cargo fmt

# Check formatting without changing
cargo fmt -- --check

# Lint with clippy
cargo clippy

# Lint with warnings as errors
cargo clippy -- -D warnings

# Fix clippy warnings automatically (when possible)
cargo clippy --fix
```

### 6.5 Dependencies

```bash
# Add a dependency
cargo add package_name

# Add with features
cargo add serde --features derive

# Add dev dependency
cargo add --dev package_name

# Remove dependency
cargo remove package_name

# Update dependencies
cargo update

# Show dependency tree
cargo tree

# Audit for vulnerabilities
cargo audit
```

### 6.6 Other Useful Commands

```bash
# Clean build artifacts
cargo clean

# Show what would be compiled
cargo build --dry-run

# Generate and view docs
cargo doc --open

# Show project info
cargo metadata

# Run benchmarks (if defined)
cargo bench
```

---

## 7. Troubleshooting

### 7.1 Common Issues

**"cargo: command not found"**
```bash
# Add to PATH
source $HOME/.cargo/env
# Or restart your terminal
```

**"linker 'cc' not found"**
```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt install build-essential

# Fedora
sudo dnf groupinstall "Development Tools"
```

**"failed to run custom build command for openssl-sys"**
```bash
# Ubuntu/Debian
sudo apt install pkg-config libssl-dev

# Fedora
sudo dnf install openssl-devel

# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
```

**Slow compilation**
```bash
# Use faster linker (see section 3.3)
# Or use mold linker (fastest on Linux)
cargo install mold
# Then in ~/.cargo/config.toml:
# [target.x86_64-unknown-linux-gnu]
# linker = "clang"
# rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

**rust-analyzer not working in VS Code**
```bash
# Restart rust-analyzer
# VS Code: Ctrl+Shift+P -> "rust-analyzer: Restart Server"

# Check rust-analyzer is installed
rustup component add rust-analyzer

# Make sure you opened the folder containing Cargo.toml
```

### 7.2 Reset Everything

If things are broken:

```bash
# Remove rustup completely
rustup self uninstall

# Remove cargo cache
rm -rf ~/.cargo

# Reinstall
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 7.3 Getting Help

```bash
# Rust documentation
rustup doc

# Book (The Rust Programming Language)
rustup doc --book

# Standard library docs
rustup doc --std

# Cargo book
rustup doc --cargo
```

**Online Resources:**
- The Rust Book: https://doc.rust-lang.org/book/
- Rust by Example: https://doc.rust-lang.org/rust-by-example/
- Rustlings (exercises): https://github.com/rust-lang/rustlings
- Rust Cookbook: https://rust-lang-nursery.github.io/rust-cookbook/

---

## Quick Start Checklist

```bash
# 1. Install Rust (macOS with Homebrew)
brew install rustup-init
rustup-init
source $HOME/.cargo/env

# -- OR on Linux --
# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# source $HOME/.cargo/env

# 2. Verify
rustc --version
cargo --version

# 3. Add components
rustup component add rustfmt clippy rust-analyzer

# 4. Install cargo-watch
cargo install cargo-watch

# 5. Clone project
git clone <repo-url>
cd nrs

# 6. Build and run
cargo build
cargo run

# You're ready! 🎉
```
