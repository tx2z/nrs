//! CLI argument definitions for nrs.
//!
//! Uses clap with derive macros for argument parsing.
//!
//! # Example
//!
//! ```no_run
//! use nrs::cli::Cli;
//!
//! let cli = Cli::parse_args();
//! println!("Project dir: {:?}", cli.project_dir());
//! ```

use std::path::PathBuf;

use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, Shell};

use crate::config::SortMode;
use crate::package::Runner;

/// Fast interactive TUI for running npm scripts.
#[derive(Parser, Debug)]
#[command(name = "nrs")]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = false)]
pub struct Cli {
    /// Path to project directory (default: current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Rerun last executed script (no TUI)
    #[arg(short = 'L', long = "last")]
    pub last: bool,

    /// List scripts non-interactively (no TUI)
    #[arg(short, long)]
    pub list: bool,

    /// Exclude scripts matching pattern (can be repeated)
    #[arg(short, long, value_name = "PATTERN")]
    pub exclude: Vec<String>,

    /// Initial sort mode
    #[arg(short, long, value_name = "MODE", value_enum)]
    pub sort: Option<CliSortMode>,

    /// Override package manager
    #[arg(short, long, value_name = "RUNNER", value_enum)]
    pub runner: Option<CliRunner>,

    /// Arguments to pass to the selected script
    #[arg(short, long, value_name = "ARGS", allow_hyphen_values = true)]
    pub args: Option<String>,

    /// Run script directly without TUI
    #[arg(short = 'n', long = "script", value_name = "NAME")]
    pub script: Option<String>,

    /// Show command without executing
    #[arg(short, long)]
    pub dry_run: bool,

    /// Path to config file
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Ignore config files
    #[arg(long)]
    pub no_config: bool,

    /// Enable debug output
    #[arg(long)]
    pub debug: bool,

    /// Generate shell completions
    #[arg(long, value_name = "SHELL", value_enum)]
    pub completions: Option<CliShell>,
}

/// Shell type for completion generation.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliShell {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    Powershell,
    /// Elvish shell
    Elvish,
}

/// Sort mode for CLI parsing.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliSortMode {
    /// Sort by most recently used.
    Recent,
    /// Sort alphabetically.
    Alpha,
    /// Group by category/prefix.
    Category,
}

impl From<CliSortMode> for SortMode {
    fn from(mode: CliSortMode) -> Self {
        match mode {
            CliSortMode::Recent => SortMode::Recent,
            CliSortMode::Alpha => SortMode::Alpha,
            CliSortMode::Category => SortMode::Category,
        }
    }
}

/// Package manager for CLI parsing.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliRunner {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl From<CliRunner> for Runner {
    fn from(runner: CliRunner) -> Self {
        match runner {
            CliRunner::Npm => Runner::Npm,
            CliRunner::Yarn => Runner::Yarn,
            CliRunner::Pnpm => Runner::Pnpm,
            CliRunner::Bun => Runner::Bun,
        }
    }
}

impl Cli {
    /// Parse command line arguments.
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    /// Get the project directory.
    ///
    /// Returns the provided path or the current directory.
    pub fn project_dir(&self) -> PathBuf {
        self.path
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    /// Check if TUI should be shown.
    pub fn should_show_tui(&self) -> bool {
        !self.list && !self.last && self.script.is_none()
    }

    /// Get the sort mode.
    pub fn sort_mode(&self) -> Option<SortMode> {
        self.sort.map(Into::into)
    }

    /// Get the runner override.
    pub fn runner_override(&self) -> Option<Runner> {
        self.runner.map(Into::into)
    }

    /// Generate shell completions and write to stdout.
    pub fn generate_completions(shell: CliShell) {
        let mut cmd = Cli::command();
        let shell = match shell {
            CliShell::Bash => Shell::Bash,
            CliShell::Zsh => Shell::Zsh,
            CliShell::Fish => Shell::Fish,
            CliShell::Powershell => Shell::PowerShell,
            CliShell::Elvish => Shell::Elvish,
        };
        generate(shell, &mut cmd, "nrs", &mut std::io::stdout());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_project_dir() {
        let cli = Cli {
            path: None,
            last: false,
            list: false,
            exclude: vec![],
            sort: None,
            runner: None,
            args: None,
            script: None,
            dry_run: false,
            config: None,
            no_config: false,
            debug: false,
            completions: None,
        };

        // Should return current directory
        assert!(cli.project_dir().is_absolute() || cli.project_dir() == PathBuf::from("."));
    }

    #[test]
    fn test_should_show_tui() {
        let mut cli = Cli {
            path: None,
            last: false,
            list: false,
            exclude: vec![],
            sort: None,
            runner: None,
            args: None,
            script: None,
            dry_run: false,
            config: None,
            no_config: false,
            debug: false,
            completions: None,
        };

        assert!(cli.should_show_tui());

        cli.list = true;
        assert!(!cli.should_show_tui());

        cli.list = false;
        cli.last = true;
        assert!(!cli.should_show_tui());

        cli.last = false;
        cli.script = Some("dev".to_string());
        assert!(!cli.should_show_tui());
    }
}
