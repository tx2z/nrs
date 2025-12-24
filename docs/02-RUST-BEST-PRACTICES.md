# Rust Development Guidelines for nrs

## Best Practices & Conventions for AI-Assisted Development

This document provides guidelines that the AI agent (Claude Code) must follow to ensure high-quality, idiomatic Rust code.

---

## Table of Contents

1. [Project Setup & Structure](#1-project-setup--structure)
2. [Code Style & Formatting](#2-code-style--formatting)
3. [Error Handling](#3-error-handling)
4. [Memory & Performance](#4-memory--performance)
5. [Type System Best Practices](#5-type-system-best-practices)
6. [Module Organization](#6-module-organization)
7. [Testing Strategy](#7-testing-strategy)
8. [Documentation](#8-documentation)
9. [Common Pitfalls to Avoid](#9-common-pitfalls-to-avoid)
10. [Dependency Guidelines](#10-dependency-guidelines)
11. [TUI-Specific Guidelines](#11-tui-specific-guidelines)
12. [Checklist Before Commits](#12-checklist-before-commits)

---

## 1. Project Setup & Structure

### 1.1 Cargo.toml Best Practices

```toml
[package]
name = "nrs"
version = "0.1.0"
edition = "2021"  # Always use latest stable edition
rust-version = "1.70"  # Minimum supported Rust version
authors = ["Your Name <email@example.com>"]
description = "Fast interactive TUI for running npm scripts"
license = "MIT"
repository = "https://github.com/username/nrs"
keywords = ["cli", "tui", "npm", "scripts", "terminal"]
categories = ["command-line-utilities", "development-tools"]

[dependencies]
# Pin major versions, allow minor updates
clap = { version = "4", features = ["derive"] }
ratatui = "0.26"
crossterm = "0.27"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
dirs = "5"
fuzzy-matcher = "0.3"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "1"

[dev-dependencies]
criterion = "0.5"
tempfile = "3"
assert_cmd = "2"
predicates = "3"
insta = "1"

[profile.release]
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
panic = "abort"     # Smaller binary
strip = true        # Strip symbols

[[bin]]
name = "nrs"
path = "src/main.rs"
```

### 1.2 Directory Structure Rules

```
src/
├── main.rs          # ONLY entry point logic, keep minimal
├── lib.rs           # Re-export public API
├── cli.rs           # CLI definitions only
└── <module>/
    ├── mod.rs       # Module exports, no logic
    ├── types.rs     # Type definitions
    └── impl.rs      # Implementations
```

**Rules:**
- Each module in its own directory if it has >1 file
- `mod.rs` only contains `pub mod` and `pub use` statements
- No business logic in `main.rs` - delegate to library

---

## 2. Code Style & Formatting

### 2.1 Mandatory Tools

```bash
# Always run before committing
cargo fmt            # Format code
cargo clippy -- -D warnings  # Lint with warnings as errors
cargo test           # Run tests
```

### 2.2 Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Crates | snake_case | `fuzzy_matcher` |
| Modules | snake_case | `package_manager` |
| Types (struct, enum) | PascalCase | `ScriptInfo` |
| Traits | PascalCase | `Executable` |
| Functions | snake_case | `detect_runner` |
| Methods | snake_case | `get_scripts` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_HISTORY_SIZE` |
| Variables | snake_case | `script_name` |
| Type parameters | Single uppercase or PascalCase | `T`, `Item` |
| Lifetimes | Short lowercase | `'a`, `'ctx` |

### 2.3 Import Organization

```rust
// 1. Standard library
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// 2. External crates (blank line after std)
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// 3. Crate modules (blank line after external)
use crate::config::Config;
use crate::package::Script;

// 4. Super/self (if needed)
use super::types::ScriptInfo;
```

### 2.4 Code Formatting Rules

```rust
// ✅ Good: Descriptive, specific types
fn parse_scripts(content: &str) -> Result<Vec<Script>> {
    // ...
}

// ❌ Bad: Generic names, unclear types
fn parse(s: &str) -> Result<Vec<S>> {
    // ...
}

// ✅ Good: Early returns for error cases
fn get_script(name: &str) -> Option<&Script> {
    if name.is_empty() {
        return None;
    }
    self.scripts.get(name)
}

// ❌ Bad: Deeply nested conditions
fn get_script(name: &str) -> Option<&Script> {
    if !name.is_empty() {
        if let Some(script) = self.scripts.get(name) {
            Some(script)
        } else {
            None
        }
    } else {
        None
    }
}
```

### 2.5 Line Length & Wrapping

- Maximum line length: 100 characters
- Break long chains thoughtfully:

```rust
// ✅ Good: Logical breaks
let scripts = package_json
    .scripts
    .iter()
    .filter(|(name, _)| !is_lifecycle_script(name))
    .map(|(name, cmd)| Script::new(name, cmd))
    .collect();

// ❌ Bad: Everything on one line
let scripts = package_json.scripts.iter().filter(|(name, _)| !is_lifecycle_script(name)).map(|(name, cmd)| Script::new(name, cmd)).collect();
```

---

## 3. Error Handling

### 3.1 Error Type Strategy

```rust
// Use thiserror for library errors
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NrsError {
    #[error("No package.json found in {path}")]
    NoPackageJson { path: PathBuf },
    
    #[error("Failed to parse package.json")]
    ParseError(#[from] serde_json::Error),
    
    #[error("No scripts defined in package.json")]
    NoScripts,
    
    #[error("Script '{name}' not found")]
    ScriptNotFound { name: String },
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

// Use anyhow for application-level errors
use anyhow::{Context, Result};

fn load_package() -> Result<Package> {
    let content = std::fs::read_to_string("package.json")
        .context("Failed to read package.json")?;
    
    let package: Package = serde_json::from_str(&content)
        .context("Failed to parse package.json")?;
    
    Ok(package)
}
```

### 3.2 Error Handling Rules

```rust
// ✅ Good: Add context to errors
fn find_package_json(start: &Path) -> Result<PathBuf> {
    find_upward(start, "package.json")
        .context(format!("No package.json found starting from {}", start.display()))
}

// ❌ Bad: Lose context with plain ?
fn find_package_json(start: &Path) -> Result<PathBuf> {
    find_upward(start, "package.json")? // Error message will be unclear
}

// ✅ Good: Use expect() only for programmer errors (impossible states)
let home = dirs::home_dir().expect("Home directory must exist");

// ❌ Bad: Using unwrap() in production code
let config = load_config().unwrap(); // Will panic!

// ✅ Good: Handle Option with meaningful defaults
let runner = config.runner.unwrap_or_else(|| detect_runner(&project_dir));
```

### 3.3 Never Use These in Production

```rust
// ❌ NEVER use these except in tests
.unwrap()           // Use .context()? or .unwrap_or()
.expect("...")      // Only for impossible states
panic!()            // Only for unrecoverable programmer errors
unreachable!()      // Only when truly unreachable
todo!()             // Remove before release
unimplemented!()    // Remove before release
```

---

## 4. Memory & Performance

### 4.1 Ownership Best Practices

```rust
// ✅ Good: Take ownership only when needed
fn run_script(script: &Script) -> Result<()> {
    // We only read, don't take ownership
}

// ✅ Good: Return owned data when caller needs it
fn get_scripts(&self) -> Vec<Script> {
    self.scripts.clone()
}

// ✅ Good: Use Cow for flexible ownership
use std::borrow::Cow;

fn normalize_name(name: &str) -> Cow<'_, str> {
    if name.contains(':') {
        Cow::Owned(name.replace(':', "_"))
    } else {
        Cow::Borrowed(name)
    }
}
```

### 4.2 Avoid Unnecessary Allocations

```rust
// ✅ Good: Use &str when possible
fn is_lifecycle_script(name: &str) -> bool {
    matches!(name, "preinstall" | "install" | "postinstall")
}

// ❌ Bad: Creating String unnecessarily
fn is_lifecycle_script(name: String) -> bool {
    name == "preinstall" || name == "install"
}

// ✅ Good: Use iterators instead of collecting
fn count_scripts(&self) -> usize {
    self.scripts.iter().filter(|s| s.is_visible()).count()
}

// ❌ Bad: Collecting just to count
fn count_scripts(&self) -> usize {
    self.scripts.iter().filter(|s| s.is_visible()).collect::<Vec<_>>().len()
}

// ✅ Good: Pre-allocate when size is known
let mut scripts = Vec::with_capacity(raw_scripts.len());
for (name, cmd) in raw_scripts {
    scripts.push(Script::new(name, cmd));
}
```

### 4.3 Performance-Critical Sections

For TUI rendering (called 60 times/sec):

```rust
// ✅ Good: Reuse buffers
struct App {
    filter_buffer: String,      // Reused for filtering
    visible_scripts: Vec<usize>, // Reused indices
}

impl App {
    fn update_filter(&mut self, query: &str) {
        self.filter_buffer.clear();
        self.filter_buffer.push_str(query);
        
        self.visible_scripts.clear();
        for (i, script) in self.scripts.iter().enumerate() {
            if self.matches_filter(script) {
                self.visible_scripts.push(i);
            }
        }
    }
}

// ❌ Bad: Allocating every frame
fn get_visible_scripts(&self, query: &str) -> Vec<&Script> {
    self.scripts
        .iter()
        .filter(|s| s.name.contains(query)) // New String every call
        .collect() // New Vec every call
}
```

---

## 5. Type System Best Practices

### 5.1 Use Newtypes for Type Safety

```rust
// ✅ Good: Distinct types for different IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScriptName(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectPath(PathBuf);

// Now these can't be confused:
fn run_script(name: ScriptName, project: ProjectPath) -> Result<()>;

// ❌ Bad: Easy to swap arguments
fn run_script(name: String, project: PathBuf) -> Result<()>;
```

### 5.2 Use Enums for State Machines

```rust
// ✅ Good: Explicit states
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Filter { query: String },
    MultiSelect { selected: Vec<usize> },
    Help,
    Error { message: String },
}

impl App {
    fn handle_input(&mut self, key: KeyEvent) {
        match &self.mode {
            AppMode::Normal => self.handle_normal_input(key),
            AppMode::Filter { .. } => self.handle_filter_input(key),
            AppMode::MultiSelect { .. } => self.handle_multiselect_input(key),
            AppMode::Help => self.handle_help_input(key),
            AppMode::Error { .. } => self.handle_error_input(key),
        }
    }
}

// ❌ Bad: Boolean flags
struct App {
    is_filtering: bool,
    is_multiselect: bool,
    is_help_open: bool,
    // What if multiple are true?
}
```

### 5.3 Builder Pattern for Complex Structs

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub runner: Option<Runner>,
    pub sort_mode: SortMode,
    pub theme: Theme,
    // ... many fields
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    runner: Option<Runner>,
    sort_mode: Option<SortMode>,
    theme: Option<Theme>,
}

impl ConfigBuilder {
    pub fn runner(mut self, runner: Runner) -> Self {
        self.runner = Some(runner);
        self
    }
    
    pub fn sort_mode(mut self, mode: SortMode) -> Self {
        self.sort_mode = Some(mode);
        self
    }
    
    pub fn build(self) -> Config {
        Config {
            runner: self.runner,
            sort_mode: self.sort_mode.unwrap_or_default(),
            theme: self.theme.unwrap_or_default(),
        }
    }
}

// Usage:
let config = Config::builder()
    .runner(Runner::Pnpm)
    .sort_mode(SortMode::Recent)
    .build();
```

---

## 6. Module Organization

### 6.1 Module File Pattern

```rust
// src/package/mod.rs - Module root
mod manager;
mod scripts;
mod descriptions;
mod workspace;

pub use manager::{PackageManager, Runner, detect_runner};
pub use scripts::{Script, Scripts, parse_scripts};
pub use descriptions::get_description;
pub use workspace::{Workspace, detect_workspaces};

// src/package/manager.rs - Implementation
use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runner {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

pub fn detect_runner(project_dir: &Path) -> Runner {
    // Implementation
}
```

### 6.2 Visibility Rules

```rust
// ✅ Good: Minimal public API
pub struct Script {
    name: String,        // Private field
    command: String,     // Private field
}

impl Script {
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn command(&self) -> &str {
        &self.command
    }
}

// ❌ Bad: Everything public
pub struct Script {
    pub name: String,
    pub command: String,
}
```

### 6.3 Prelude Pattern (Optional)

```rust
// src/prelude.rs - Common imports for internal use
pub use crate::config::Config;
pub use crate::error::{NrsError, Result};
pub use crate::package::{Script, Runner};

// In other modules:
use crate::prelude::*;
```

---

## 7. Testing Strategy

### 7.1 Test Organization

```rust
// Unit tests: Same file as implementation
// src/package/scripts.rs

pub fn parse_scripts(json: &str) -> Result<Vec<Script>> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_empty_scripts() {
        let json = r#"{"scripts": {}}"#;
        let scripts = parse_scripts(json).unwrap();
        assert!(scripts.is_empty());
    }
    
    #[test]
    fn test_parse_basic_scripts() {
        let json = r#"{"scripts": {"dev": "vite", "build": "vite build"}}"#;
        let scripts = parse_scripts(json).unwrap();
        assert_eq!(scripts.len(), 2);
    }
}
```

### 7.2 Integration Tests

```rust
// tests/integration/cli_test.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_no_package_json() {
    let temp = TempDir::new().unwrap();
    
    Command::cargo_bin("nrs")
        .unwrap()
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No package.json found"));
}

#[test]
fn test_list_scripts() {
    let temp = create_test_project();
    
    Command::cargo_bin("nrs")
        .unwrap()
        .current_dir(temp.path())
        .arg("--list")
        .assert()
        .success()
        .stdout(predicate::str::contains("dev"))
        .stdout(predicate::str::contains("build"));
}
```

### 7.3 Test Helpers

```rust
// tests/common/mod.rs
use std::fs;
use tempfile::TempDir;

pub fn create_test_project() -> TempDir {
    let temp = TempDir::new().unwrap();
    
    let package_json = r#"{
        "name": "test-project",
        "scripts": {
            "dev": "vite",
            "build": "vite build",
            "test": "vitest"
        }
    }"#;
    
    fs::write(temp.path().join("package.json"), package_json).unwrap();
    temp
}

pub fn create_project_with_scripts(scripts: &[(&str, &str)]) -> TempDir {
    let temp = TempDir::new().unwrap();
    
    let scripts_obj: serde_json::Map<_, _> = scripts
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
        .collect();
    
    let package = serde_json::json!({
        "name": "test-project",
        "scripts": scripts_obj
    });
    
    fs::write(
        temp.path().join("package.json"),
        serde_json::to_string_pretty(&package).unwrap()
    ).unwrap();
    
    temp
}
```

---

## 8. Documentation

### 8.1 Documentation Rules

```rust
//! Module-level documentation (at top of file)
//! 
//! This module handles package.json parsing and script extraction.

/// Type/function documentation
/// 
/// # Examples
/// 
/// ```
/// use nrs::package::Script;
/// 
/// let script = Script::new("dev", "vite");
/// assert_eq!(script.name(), "dev");
/// ```
/// 
/// # Errors
/// 
/// Returns `Err` if the JSON is invalid.
pub fn parse_scripts(json: &str) -> Result<Vec<Script>> {
    // ...
}

/// Short description for simple items.
pub const MAX_HISTORY_SIZE: usize = 100;
```

### 8.2 When to Document

- **Always**: Public API (functions, types, modules)
- **Always**: Complex algorithms
- **Always**: Non-obvious behavior
- **Optional**: Private implementation details
- **Never**: Self-explanatory code

---

## 9. Common Pitfalls to Avoid

### 9.1 Lifetime Issues

```rust
// ❌ Bad: Returning reference to local
fn get_name(&self) -> &str {
    let name = self.name.clone(); // Local
    &name // Dangling reference!
}

// ✅ Good: Return reference to owned data
fn get_name(&self) -> &str {
    &self.name
}

// ✅ Good: Return owned if needed
fn get_name_owned(&self) -> String {
    self.name.clone()
}
```

### 9.2 Borrow Checker Issues

```rust
// ❌ Bad: Borrowing while mutating
fn update(&mut self) {
    let item = &self.items[0]; // Immutable borrow
    self.items.push(item.clone()); // Mutable borrow - ERROR!
}

// ✅ Good: Clone first
fn update(&mut self) {
    let item = self.items[0].clone(); // Clone, no borrow
    self.items.push(item); // Now OK
}

// ✅ Good: Use indices
fn update(&mut self) {
    let item = self.items[0].clone();
    self.items.push(item);
}
```

### 9.3 Async Pitfalls (if using async)

```rust
// ❌ Bad: Blocking in async
async fn load_file() -> Result<String> {
    std::fs::read_to_string("file.txt") // Blocks thread!
}

// ✅ Good: Use async fs
async fn load_file() -> Result<String> {
    tokio::fs::read_to_string("file.txt").await
}
```

### 9.4 String Handling

```rust
// ❌ Bad: Unnecessary allocation
fn check_prefix(s: &str) -> bool {
    s.to_string().starts_with("pre") // Allocates new String
}

// ✅ Good: Work with &str
fn check_prefix(s: &str) -> bool {
    s.starts_with("pre") // No allocation
}

// ✅ Good: Accept impl AsRef<str> for flexibility
fn check_prefix(s: impl AsRef<str>) -> bool {
    s.as_ref().starts_with("pre")
}
```

---

## 10. Dependency Guidelines

### 10.1 Choosing Dependencies

**Before adding a dependency, ask:**
1. Is it actively maintained? (commits in last 6 months)
2. Does it have reasonable download counts?
3. Is it from a trusted source?
4. Can I implement this easily myself? (<100 lines)
5. Does it bloat compile time significantly?

### 10.2 Feature Flags

```toml
# Use minimal features
serde = { version = "1", features = ["derive"] }  # Only derive
chrono = { version = "0.4", default-features = false, features = ["serde"] }
```

### 10.3 Approved Dependencies for This Project

| Crate | Purpose | Required |
|-------|---------|----------|
| clap | CLI parsing | Yes |
| ratatui | TUI framework | Yes |
| crossterm | Terminal handling | Yes |
| serde + serde_json | JSON handling | Yes |
| toml | Config files | Yes |
| anyhow + thiserror | Error handling | Yes |
| dirs | Platform paths | Yes |
| fuzzy-matcher | Search | Yes |
| chrono | Timestamps | Yes |

**Do not add dependencies without justification.**

---

## 11. TUI-Specific Guidelines

### 11.1 Ratatui Patterns

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

// ✅ Good: Separate UI rendering from logic
impl App {
    pub fn ui(&self, frame: &mut Frame) {
        let chunks = self.create_layout(frame.size());
        
        self.render_header(frame, chunks[0]);
        self.render_filter(frame, chunks[1]);
        self.render_scripts(frame, chunks[2]);
        self.render_description(frame, chunks[3]);
        self.render_footer(frame, chunks[4]);
    }
    
    fn create_layout(&self, area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Header
                Constraint::Length(1),  // Filter
                Constraint::Min(5),     // Scripts (flexible)
                Constraint::Length(4),  // Description
                Constraint::Length(1),  // Footer
            ])
            .split(area)
            .to_vec()
    }
}
```

### 11.2 Input Handling

```rust
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

impl App {
    pub fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Resize(w, h) => {
                self.resize(w, h);
                Ok(false)
            }
            _ => Ok(false),
        }
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Check for quit first
        if matches!(
            (key.code, key.modifiers),
            (KeyCode::Char('q'), KeyModifiers::NONE) |
            (KeyCode::Char('c'), KeyModifiers::CONTROL)
        ) {
            return Ok(true); // Signal quit
        }
        
        // Delegate to mode-specific handler
        match &self.mode {
            AppMode::Normal => self.handle_normal_key(key),
            AppMode::Filter { .. } => self.handle_filter_key(key),
            // ...
        }
        
        Ok(false)
    }
}
```

### 11.3 State Management

```rust
// ✅ Good: Centralized state
pub struct App {
    // Data
    scripts: Vec<Script>,
    config: Config,
    history: History,
    
    // UI State
    mode: AppMode,
    selected: usize,
    scroll_offset: usize,
    
    // Computed (cached)
    visible_scripts: Vec<usize>,
    columns: usize,
}

impl App {
    // Update cached state when needed
    fn update_visible_scripts(&mut self) {
        self.visible_scripts.clear();
        
        for (i, script) in self.scripts.iter().enumerate() {
            if self.matches_filter(script) {
                self.visible_scripts.push(i);
            }
        }
        
        // Adjust selection if needed
        if self.selected >= self.visible_scripts.len() {
            self.selected = self.visible_scripts.len().saturating_sub(1);
        }
    }
}
```

---

## 12. Checklist Before Commits

### 12.1 Pre-Commit Checks

```bash
#!/bin/bash
# Save as .git/hooks/pre-commit

set -e

echo "Running cargo fmt..."
cargo fmt -- --check

echo "Running cargo clippy..."
cargo clippy -- -D warnings

echo "Running cargo test..."
cargo test

echo "All checks passed!"
```

### 12.2 Manual Checklist

- [ ] `cargo fmt` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] No `unwrap()` in production code
- [ ] No `todo!()` or `unimplemented!()`
- [ ] Public API is documented
- [ ] Error messages are helpful
- [ ] No unnecessary allocations in hot paths
- [ ] New dependencies are justified

### 12.3 Before Release

```bash
# Full check
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo test --release
cargo build --release
cargo doc --no-deps

# Check binary size
ls -lh target/release/nrs

# Test binary
./target/release/nrs --version
./target/release/nrs --help
```

---

## Quick Reference Card

```
ALWAYS:
✅ Use Result<T> for fallible operations
✅ Add context to errors with .context()
✅ Use &str over String when possible
✅ Prefer iterators over collecting
✅ Run cargo fmt and clippy
✅ Write tests for public API
✅ Document public items

NEVER:
❌ Use .unwrap() in production
❌ Use panic!() for recoverable errors
❌ Leave todo!() in committed code
❌ Ignore clippy warnings
❌ Add unnecessary dependencies
❌ Allocate in render loops
```
