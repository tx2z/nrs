//! nrs - npm Run Scripts
//!
//! Entry point for the nrs CLI application.

use std::io::{self, IsTerminal};
use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result};

use npm_run_scripts::cli::Cli;
use npm_run_scripts::config::Config;
use npm_run_scripts::error::{exit_code, NrsError};
use npm_run_scripts::history::History;
use npm_run_scripts::package::{detect_runner_reason, parse_scripts, Runner, Scripts};
use npm_run_scripts::runner::execute_script;
use npm_run_scripts::tui::{run_tui, App};
use npm_run_scripts::utils::{
    find_project_root, global_config_file, history_file, local_config_file,
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => {
            // Check if it's one of our custom errors with good formatting
            if let Some(nrs_err) = err.downcast_ref::<NrsError>() {
                eprintln!("Error: {nrs_err}");
                return ExitCode::from(nrs_err.exit_code() as u8);
            }
            eprintln!("Error: {err:#}");
            ExitCode::from(exit_code::GENERAL_ERROR as u8)
        }
    }
}

fn run() -> Result<i32> {
    let cli = Cli::parse_args();

    // Handle shell completions early
    if let Some(shell) = cli.completions {
        Cli::generate_completions(shell);
        return Ok(exit_code::SUCCESS);
    }

    if cli.debug {
        print_debug_header();
        eprintln!("Debug: CLI arguments = {cli:#?}");
    }

    // Find project root
    let project_dir =
        find_project_root(&cli.project_dir()).context("Failed to find project directory")?;

    if cli.debug {
        eprintln!("Debug: Project directory = {}", project_dir.display());
        print_debug_paths(&project_dir);
    }

    // Detect package manager
    let (runner, runner_reason) = if let Some(r) = cli.runner_override() {
        (r, "CLI --runner flag".to_string())
    } else {
        detect_runner_reason(&project_dir)
    };

    if cli.debug {
        eprintln!("Debug: Package manager = {} ({})", runner, runner_reason);
    }

    // Parse scripts
    let scripts = parse_scripts(&project_dir).context("Failed to parse scripts")?;

    if scripts.is_empty() {
        let package_json_path = project_dir.join("package.json");
        return Err(NrsError::NoScriptsAt {
            path: package_json_path,
        }
        .into());
    }

    if cli.debug {
        eprintln!("Debug: Found {} scripts", scripts.len());
        print_debug_scripts(&scripts);
    }

    // Load config for exclude patterns (used in both list and TUI modes)
    let config = if cli.no_config {
        Config::default()
    } else {
        npm_run_scripts::config::load_config(cli.config.as_deref(), &project_dir)
            .unwrap_or_default()
    };

    // Combine config and CLI exclude patterns
    let mut exclude_patterns = config.exclude.patterns.clone();
    exclude_patterns.extend(cli.exclude.clone());

    // Apply exclude patterns
    let scripts = if !exclude_patterns.is_empty() {
        scripts.without_matching(&exclude_patterns)
    } else {
        scripts
    };

    // Handle different modes
    if cli.list {
        // List mode: print scripts and exit
        return list_scripts(&scripts, runner);
    }

    if cli.last {
        // Rerun last script
        let history = History::load().unwrap_or_default();

        let (script_name, stored_args) =
            history.get_last_script(&project_dir).ok_or_else(|| {
                anyhow::anyhow!(
                    "No previous script found for this project. Run nrs first to execute a script."
                )
            })?;

        // Check if the script still exists
        if scripts.get(&script_name).is_none() {
            anyhow::bail!("Script '{}' no longer exists in package.json", script_name);
        }

        // Use CLI args if provided, otherwise use stored args from history
        let args_str = cli.args.as_deref().or(stored_args.as_deref());

        // Print what we're running
        eprintln!(
            "Rerunning: {}{}",
            script_name,
            args_str.map(|a| format!(" {}", a)).unwrap_or_default()
        );

        let args_vec: Vec<String> = args_str
            .map(|a| a.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        let result = execute_script(runner, &script_name, &args_vec, &project_dir, cli.dry_run)?;

        return Ok(result.code().unwrap_or(0));
    }

    if let Some(script_name) = &cli.script {
        // Direct script execution
        return run_script_by_name(
            &scripts,
            runner,
            script_name,
            cli.args.as_deref(),
            &project_dir,
            cli.dry_run,
        );
    }

    // TUI mode
    let history = History::load().unwrap_or_default();

    // Get project name
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    // Filter out lifecycle scripts (exclude patterns already applied above)
    let scripts = scripts.without_lifecycle();

    // Create and run the app
    let app = App::new(
        scripts,
        config,
        history,
        project_name,
        project_dir.clone(),
        runner,
    );

    let scripts_to_run = run_tui(app).context("TUI error")?;

    // Execute selected scripts
    if scripts_to_run.is_empty() {
        return Ok(exit_code::SUCCESS);
    }

    // Execute scripts
    for (i, script_run) in scripts_to_run.iter().enumerate() {
        if scripts_to_run.len() > 1 {
            println!(
                "\n\x1b[1;36mRunning {}/{}: {}...\x1b[0m",
                i + 1,
                scripts_to_run.len(),
                script_run.script.name()
            );
        }

        // Record in history
        let mut history = History::load().unwrap_or_default();
        history.record_run(
            &project_dir,
            script_run.script.name(),
            script_run.args.clone(),
        );
        let _ = history.save();

        let args: Vec<String> = script_run
            .args
            .as_ref()
            .map(|a| a.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        let result = execute_script(
            runner,
            script_run.script.name(),
            &args,
            &project_dir,
            cli.dry_run,
        )?;

        let code = result.code().unwrap_or(0);
        if code != 0 {
            return Ok(code);
        }
    }

    Ok(exit_code::SUCCESS)
}

/// Run a script by name directly (non-TUI mode).
fn run_script_by_name(
    scripts: &Scripts,
    runner: Runner,
    script_name: &str,
    args: Option<&str>,
    project_dir: &std::path::Path,
    dry_run: bool,
) -> Result<i32> {
    if scripts.get(script_name).is_none() {
        // Get available script names for suggestions
        let script_names: Vec<&str> = scripts.iter().map(|s| s.name()).collect();
        let err = NrsError::script_not_found_with_suggestions(script_name, &script_names);
        eprintln!("Error: {err}");
        eprintln!();
        eprintln!("Available scripts:");
        for script in scripts.iter() {
            eprintln!("  {}", script.name());
        }
        return Ok(exit_code::GENERAL_ERROR);
    }

    let args_vec: Vec<String> = args
        .map(|a| a.split_whitespace().map(String::from).collect())
        .unwrap_or_default();

    // Record in history
    let mut history = History::load().unwrap_or_default();
    history.record_run(project_dir, script_name, args.map(String::from));
    let _ = history.save();

    let result = execute_script(runner, script_name, &args_vec, project_dir, dry_run)?;

    Ok(result.code().unwrap_or(0))
}

/// List scripts in a nice format (non-TUI mode).
fn list_scripts(scripts: &Scripts, runner: Runner) -> Result<i32> {
    let use_colors = io::stdout().is_terminal();

    // Print header
    if use_colors {
        println!("\x1b[1;36mAvailable scripts ({}):\x1b[0m", runner);
    } else {
        println!("Available scripts ({}):", runner);
    }
    println!();

    // Find the longest script name for alignment
    let max_name_len = scripts
        .iter()
        .map(|s| s.name().len())
        .max()
        .unwrap_or(0)
        .min(30);

    // Print each script
    for script in scripts.iter() {
        let name = script.name();
        let command = script.command();
        let description = script.description();

        if use_colors {
            // Script name in green
            print!("  \x1b[1;32m{:width$}\x1b[0m", name, width = max_name_len);

            // Command in dim
            print!("  \x1b[2m{}\x1b[0m", truncate_string(command, 50));

            // Description if available
            if let Some(desc) = description {
                print!("  \x1b[33m{}\x1b[0m", truncate_string(desc, 40));
            }
        } else {
            print!("  {:width$}", name, width = max_name_len);
            print!("  {}", truncate_string(command, 50));

            if let Some(desc) = description {
                print!("  {}", truncate_string(desc, 40));
            }
        }

        println!();
    }

    // Print count
    println!();
    if use_colors {
        println!("\x1b[2m{} scripts found\x1b[0m", scripts.len());
    } else {
        println!("{} scripts found", scripts.len());
    }

    Ok(exit_code::SUCCESS)
}

/// Truncate a string to a maximum length, adding ellipsis if needed.
/// Handles Unicode characters properly.
fn truncate_string(s: &str, max_len: usize) -> String {
    if max_len < 4 {
        return s.chars().take(max_len).collect();
    }

    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

// ==================== Debug Functions ====================

/// Print debug header with version info.
fn print_debug_header() {
    eprintln!("=== nrs debug mode ===");
    eprintln!("Version: {}", env!("CARGO_PKG_VERSION"));
    eprintln!();
}

/// Print debug information about file paths.
fn print_debug_paths(project_dir: &Path) {
    eprintln!("Debug: File locations:");

    // History file
    if let Some(hist) = history_file() {
        let exists = hist.exists();
        eprintln!("  History file: {} (exists: {})", hist.display(), exists);
    } else {
        eprintln!("  History file: <not available>");
    }

    // Global config
    if let Some(cfg) = global_config_file() {
        let exists = cfg.exists();
        eprintln!("  Global config: {} (exists: {})", cfg.display(), exists);
    } else {
        eprintln!("  Global config: <not available>");
    }

    // Local config
    if let Some(cfg) = local_config_file(project_dir) {
        eprintln!("  Local config: {} (exists: true)", cfg.display());
    } else {
        eprintln!(
            "  Local config: {}/.nrsrc.toml (exists: false)",
            project_dir.display()
        );
    }

    // Package.json
    let package_json = project_dir.join("package.json");
    eprintln!(
        "  package.json: {} (exists: {})",
        package_json.display(),
        package_json.exists()
    );

    eprintln!();
}

/// Print debug information about detected scripts.
fn print_debug_scripts(scripts: &Scripts) {
    eprintln!("Debug: Scripts found:");
    for script in scripts.iter().take(10) {
        let desc = script
            .description()
            .map(|d| format!(" - {}", truncate_string(d, 40)))
            .unwrap_or_default();
        eprintln!(
            "  {} = {}{}",
            script.name(),
            truncate_string(script.command(), 50),
            desc
        );
    }
    if scripts.len() > 10 {
        eprintln!("  ... and {} more", scripts.len() - 10);
    }
    eprintln!();
}
