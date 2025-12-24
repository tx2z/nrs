//! Build script for nrs.
//!
//! Generates man pages using clap_mangen.

use std::env;
use std::fs;
use std::path::PathBuf;

use clap::{CommandFactory, Parser, ValueEnum};

/// Minimal CLI struct for man page generation.
///
/// This duplicates the CLI definition to avoid build dependency issues.
#[derive(Parser)]
#[command(name = "nrs")]
#[command(
    author,
    version,
    about = "Fast interactive TUI for running npm scripts"
)]
#[command(
    long_about = "nrs is a fast, interactive terminal user interface (TUI) for discovering \
    and executing npm/yarn/pnpm/bun scripts defined in package.json files.\n\n\
    Run without arguments to launch the interactive TUI. Use arrow keys to navigate, \
    Enter to run a script, or press 1-9 for quick selection."
)]
struct Cli {
    /// Path to project directory (default: current directory)
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// Rerun last executed script (no TUI)
    #[arg(short = 'L', long = "last")]
    last: bool,

    /// List scripts non-interactively (no TUI)
    #[arg(short, long)]
    list: bool,

    /// Exclude scripts matching pattern (can be repeated)
    #[arg(short, long, value_name = "PATTERN")]
    exclude: Vec<String>,

    /// Initial sort mode
    #[arg(short, long, value_name = "MODE", value_enum)]
    sort: Option<SortMode>,

    /// Override package manager
    #[arg(short, long, value_name = "RUNNER", value_enum)]
    runner: Option<Runner>,

    /// Arguments to pass to the selected script
    #[arg(short, long, value_name = "ARGS", allow_hyphen_values = true)]
    args: Option<String>,

    /// Run script directly without TUI
    #[arg(short = 'n', long = "script", value_name = "NAME")]
    script: Option<String>,

    /// Show command without executing
    #[arg(short, long)]
    dry_run: bool,

    /// Path to config file
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Ignore config files
    #[arg(long)]
    no_config: bool,

    /// Enable debug output
    #[arg(long)]
    debug: bool,

    /// Generate shell completions
    #[arg(long, value_name = "SHELL", value_enum)]
    completions: Option<Shell>,
}

#[derive(Clone, Copy, ValueEnum)]
enum SortMode {
    Recent,
    Alpha,
    Category,
}

#[derive(Clone, Copy, ValueEnum)]
enum Runner {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

#[derive(Clone, Copy, ValueEnum)]
enum Shell {
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
}

fn main() {
    // Only generate man pages for release builds or when explicitly requested
    let profile = env::var("PROFILE").unwrap_or_default();
    if profile != "release" && env::var("NRS_GEN_MANPAGE").is_err() {
        return;
    }

    let out_dir = match env::var_os("OUT_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => return,
    };

    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);

    let mut buffer = Vec::new();
    man.render(&mut buffer)
        .expect("Failed to generate man page");

    // Write to the build output directory
    let man_path = out_dir.join("nrs.1");
    fs::write(&man_path, buffer).expect("Failed to write man page");

    // Also copy to docs directory for distribution
    let docs_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("docs");
    if docs_dir.exists() {
        let _ = fs::copy(&man_path, docs_dir.join("nrs.1"));
    }

    println!("cargo:rerun-if-changed=build.rs");
}
