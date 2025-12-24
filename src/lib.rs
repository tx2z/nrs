//! nrs - Node Run Scripts
//!
//! A fast, interactive terminal user interface (TUI) for discovering
//! and executing npm/yarn/pnpm/bun scripts defined in `package.json` files.
//!
//! # Features
//!
//! - **Fast**: Sub-50ms startup time (Rust native binary)
//! - **Intuitive**: Number keys for quick execution, fuzzy search, visual grid
//! - **Smart**: Auto-detect package manager, remember history, show descriptions
//! - **Cross-platform**: Linux and macOS support
//! - **Zero-config**: Works out of the box, optional configuration for power users
//!
//! # Modules
//!
//! - [`cli`] - Command-line interface argument parsing
//! - [`config`] - Configuration file loading and types
//! - [`error`] - Error types and result helpers
//! - [`filter`] - Fuzzy filtering for scripts
//! - [`history`] - Script execution history tracking
//! - [`package`] - Package.json parsing and package manager detection
//! - [`runner`] - Script execution
//! - [`tui`] - Terminal user interface
//! - [`utils`] - Path and terminal utilities
//!
//! # Example
//!
//! ```no_run
//! use nrs::package::{parse_scripts, detect_runner, Runner};
//! use std::path::Path;
//!
//! // Parse scripts from a project
//! let project_dir = Path::new("./my-project");
//! let scripts = parse_scripts(project_dir).expect("Failed to parse scripts");
//!
//! // Detect the package manager
//! let runner = detect_runner(project_dir);
//!
//! // Get the command for a script
//! let cmd = runner.run_command("dev");
//! println!("Command: {:?}", cmd);
//! ```

/// CLI argument definitions.
pub mod cli;

/// Configuration system for loading and merging settings.
pub mod config;

/// Error types and result helpers.
pub mod error;

/// Fuzzy filtering for scripts.
pub mod filter;

/// Script execution history tracking.
pub mod history;

/// Package.json parsing and package manager detection.
pub mod package;

/// Script execution.
pub mod runner;

/// Terminal user interface.
pub mod tui;

/// Path and terminal utilities.
pub mod utils;

// Re-export commonly used types
pub use cli::Cli;
pub use config::Config;
pub use error::{NrsError, Result};
pub use package::{Runner, Script, Scripts};
