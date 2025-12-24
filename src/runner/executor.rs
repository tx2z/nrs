//! Script execution.
//!
//! Handles running scripts with the appropriate package manager,
//! including single script execution, batch execution, and dry-run mode.

use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, ExitStatus};

use anyhow::{Context, Result};

use crate::package::{Runner, Script};

/// Exit code when interrupted by Ctrl+C (SIGINT).
/// On Unix, this is 128 + signal number (SIGINT = 2).
pub const EXIT_CODE_INTERRUPTED: i32 = 130;

/// Result of script execution.
#[derive(Debug)]
pub struct ExecutionResult {
    /// Exit status of the script.
    pub status: ExitStatus,
    /// The command that was executed.
    pub command: String,
}

impl ExecutionResult {
    /// Check if the execution was successful.
    pub fn success(&self) -> bool {
        self.status.success()
    }

    /// Get the exit code.
    pub fn code(&self) -> Option<i32> {
        self.status.code()
    }
}

/// Run a single script with the given runner.
///
/// # Arguments
///
/// * `runner` - The package manager to use
/// * `script` - The script to run
/// * `args` - Optional additional arguments to pass to the script
/// * `dry_run` - If true, print the command without executing
///
/// # Returns
///
/// Returns the exit code of the script (0 for success, non-zero for failure).
/// Returns 130 if interrupted by Ctrl+C.
///
/// # Errors
///
/// Returns an error if the script fails to spawn.
pub fn run_script(
    runner: Runner,
    script: &Script,
    args: Option<&str>,
    dry_run: bool,
) -> Result<i32> {
    let args_vec: Vec<String> = args
        .map(|a| shell_words::split(a).unwrap_or_else(|_| vec![a.to_string()]))
        .unwrap_or_default();

    let project_dir = std::env::current_dir().context("Failed to get current directory")?;

    execute_script(runner, script.name(), &args_vec, &project_dir, dry_run)
        .map(|result| result.code().unwrap_or(EXIT_CODE_INTERRUPTED))
}

/// Run a single script in a specific directory.
///
/// # Arguments
///
/// * `runner` - The package manager to use
/// * `script` - The script to run
/// * `args` - Optional additional arguments to pass to the script
/// * `project_dir` - The project directory to run in
/// * `dry_run` - If true, print the command without executing
///
/// # Returns
///
/// Returns the exit code of the script (0 for success, non-zero for failure).
/// Returns 130 if interrupted by Ctrl+C.
///
/// # Errors
///
/// Returns an error if the script fails to spawn.
pub fn run_script_in_dir(
    runner: Runner,
    script: &Script,
    args: Option<&str>,
    project_dir: &Path,
    dry_run: bool,
) -> Result<i32> {
    let args_vec: Vec<String> = args
        .map(|a| shell_words::split(a).unwrap_or_else(|_| vec![a.to_string()]))
        .unwrap_or_default();

    execute_script(runner, script.name(), &args_vec, project_dir, dry_run)
        .map(|result| result.code().unwrap_or(EXIT_CODE_INTERRUPTED))
}

/// Run multiple scripts sequentially.
///
/// Scripts are run in order. Execution stops on the first failure.
/// Progress is printed to stdout: "Running 1/3: dev..."
///
/// # Arguments
///
/// * `runner` - The package manager to use
/// * `scripts` - Scripts to run with their optional arguments
/// * `dry_run` - If true, print the commands without executing
///
/// # Returns
///
/// Returns a vector of exit codes for each script that was executed.
/// If a script fails, the vector will contain exit codes up to and including
/// the failed script.
///
/// # Errors
///
/// Returns an error if any script fails to spawn.
pub fn run_scripts(
    runner: Runner,
    scripts: &[(&Script, Option<String>)],
    dry_run: bool,
) -> Result<Vec<i32>> {
    let total = scripts.len();
    let mut results = Vec::with_capacity(total);
    let project_dir = std::env::current_dir().context("Failed to get current directory")?;

    for (i, (script, args)) in scripts.iter().enumerate() {
        // Print progress
        println!(
            "\n\x1b[1;36mRunning {}/{}: {}...\x1b[0m",
            i + 1,
            total,
            script.name()
        );
        io::stdout().flush().ok();

        let args_vec: Vec<String> = args
            .as_ref()
            .map(|a| shell_words::split(a).unwrap_or_else(|_| vec![a.to_string()]))
            .unwrap_or_default();

        let result = execute_script(runner, script.name(), &args_vec, &project_dir, dry_run)?;
        let exit_code = result.code().unwrap_or(EXIT_CODE_INTERRUPTED);
        results.push(exit_code);

        // Stop on first failure (non-zero exit code)
        if exit_code != 0 {
            println!(
                "\n\x1b[1;31mScript '{}' failed with exit code {}\x1b[0m",
                script.name(),
                exit_code
            );
            break;
        }
    }

    Ok(results)
}

/// Run multiple scripts sequentially in a specific directory.
///
/// Scripts are run in order. Execution stops on the first failure.
/// Progress is printed to stdout: "Running 1/3: dev..."
///
/// # Arguments
///
/// * `runner` - The package manager to use
/// * `scripts` - Scripts to run with their optional arguments
/// * `project_dir` - The project directory to run in
/// * `dry_run` - If true, print the commands without executing
///
/// # Returns
///
/// Returns a vector of exit codes for each script that was executed.
/// If a script fails, the vector will contain exit codes up to and including
/// the failed script.
///
/// # Errors
///
/// Returns an error if any script fails to spawn.
pub fn run_scripts_in_dir(
    runner: Runner,
    scripts: &[(&Script, Option<String>)],
    project_dir: &Path,
    dry_run: bool,
) -> Result<Vec<i32>> {
    let total = scripts.len();
    let mut results = Vec::with_capacity(total);

    for (i, (script, args)) in scripts.iter().enumerate() {
        // Print progress
        println!(
            "\n\x1b[1;36mRunning {}/{}: {}...\x1b[0m",
            i + 1,
            total,
            script.name()
        );
        io::stdout().flush().ok();

        let args_vec: Vec<String> = args
            .as_ref()
            .map(|a| shell_words::split(a).unwrap_or_else(|_| vec![a.to_string()]))
            .unwrap_or_default();

        let result = execute_script(runner, script.name(), &args_vec, project_dir, dry_run)?;
        let exit_code = result.code().unwrap_or(EXIT_CODE_INTERRUPTED);
        results.push(exit_code);

        // Stop on first failure (non-zero exit code)
        if exit_code != 0 {
            println!(
                "\n\x1b[1;31mScript '{}' failed with exit code {}\x1b[0m",
                script.name(),
                exit_code
            );
            break;
        }
    }

    Ok(results)
}

/// Execute a script with the given runner.
///
/// This is the low-level execution function that spawns the process.
///
/// # Arguments
///
/// * `runner` - The package manager to use
/// * `script` - The script name to run
/// * `args` - Additional arguments to pass to the script
/// * `project_dir` - The project directory to run in
/// * `dry_run` - If true, print the command without executing
///
/// # Errors
///
/// Returns an error if the script fails to execute.
pub fn execute_script(
    runner: Runner,
    script: &str,
    args: &[String],
    project_dir: &Path,
    dry_run: bool,
) -> Result<ExecutionResult> {
    let cmd_parts = runner.run_command_with_args(script, args);
    let command_str = cmd_parts.join(" ");

    if dry_run {
        println!("Would run: {command_str}");
        return Ok(ExecutionResult {
            status: std::process::ExitStatus::default(),
            command: command_str,
        });
    }

    let mut command = Command::new(&cmd_parts[0]);
    command.args(&cmd_parts[1..]);
    command.current_dir(project_dir);

    // Inherit stdio for interactive scripts
    command.stdin(std::process::Stdio::inherit());
    command.stdout(std::process::Stdio::inherit());
    command.stderr(std::process::Stdio::inherit());

    let status = command
        .status()
        .with_context(|| format!("Failed to execute: {command_str}"))?;

    Ok(ExecutionResult {
        status,
        command: command_str,
    })
}

/// Format a command for display in dry-run mode.
pub fn format_dry_run_command(runner: Runner, script: &str, args: Option<&str>) -> String {
    let args_vec: Vec<String> = args
        .map(|a| shell_words::split(a).unwrap_or_else(|_| vec![a.to_string()]))
        .unwrap_or_default();

    let cmd = runner.run_command_with_args(script, &args_vec);
    format!("Would run: {}", cmd.join(" "))
}

/// Execute a workspace script with the given runner.
///
/// This runs a script in a specific workspace package from the monorepo root.
///
/// # Arguments
///
/// * `runner` - The package manager to use
/// * `workspace` - The workspace name (package name)
/// * `script` - The script name to run
/// * `args` - Additional arguments to pass to the script
/// * `project_dir` - The project directory (monorepo root) to run in
/// * `dry_run` - If true, print the command without executing
///
/// # Errors
///
/// Returns an error if the script fails to execute.
pub fn execute_workspace_script(
    runner: Runner,
    workspace: &str,
    script: &str,
    args: &[String],
    project_dir: &Path,
    dry_run: bool,
) -> Result<ExecutionResult> {
    let cmd_parts = runner.workspace_command_with_args(workspace, script, args);
    let command_str = cmd_parts.join(" ");

    if dry_run {
        println!("Would run: {command_str}");
        return Ok(ExecutionResult {
            status: std::process::ExitStatus::default(),
            command: command_str,
        });
    }

    let mut command = Command::new(&cmd_parts[0]);
    command.args(&cmd_parts[1..]);
    command.current_dir(project_dir);

    // Inherit stdio for interactive scripts
    command.stdin(std::process::Stdio::inherit());
    command.stdout(std::process::Stdio::inherit());
    command.stderr(std::process::Stdio::inherit());

    let status = command
        .status()
        .with_context(|| format!("Failed to execute: {command_str}"))?;

    Ok(ExecutionResult {
        status,
        command: command_str,
    })
}

/// Run a workspace script.
///
/// # Arguments
///
/// * `runner` - The package manager to use
/// * `workspace` - The workspace name (package name)
/// * `script` - The script to run
/// * `args` - Optional additional arguments to pass to the script
/// * `project_dir` - The project directory (monorepo root) to run in
/// * `dry_run` - If true, print the command without executing
///
/// # Returns
///
/// Returns the exit code of the script (0 for success, non-zero for failure).
pub fn run_workspace_script(
    runner: Runner,
    workspace: &str,
    script: &Script,
    args: Option<&str>,
    project_dir: &Path,
    dry_run: bool,
) -> Result<i32> {
    let args_vec: Vec<String> = args
        .map(|a| shell_words::split(a).unwrap_or_else(|_| vec![a.to_string()]))
        .unwrap_or_default();

    execute_workspace_script(
        runner,
        workspace,
        script.name(),
        &args_vec,
        project_dir,
        dry_run,
    )
    .map(|result| result.code().unwrap_or(EXIT_CODE_INTERRUPTED))
}

/// Format a workspace command for display in dry-run mode.
pub fn format_workspace_dry_run_command(
    runner: Runner,
    workspace: &str,
    script: &str,
    args: Option<&str>,
) -> String {
    let args_vec: Vec<String> = args
        .map(|a| shell_words::split(a).unwrap_or_else(|_| vec![a.to_string()]))
        .unwrap_or_default();

    let cmd = runner.workspace_command_with_args(workspace, script, &args_vec);
    format!("Would run: {}", cmd.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dry_run() {
        let result = execute_script(Runner::Npm, "test", &[], Path::new("."), true).unwrap();

        assert_eq!(result.command, "npm run test");
    }

    #[test]
    fn test_dry_run_with_args() {
        let args = vec!["--watch".to_string(), "--coverage".to_string()];
        let result = execute_script(Runner::Npm, "test", &args, Path::new("."), true).unwrap();

        assert_eq!(result.command, "npm run test -- --watch --coverage");
    }

    #[test]
    fn test_format_dry_run_command() {
        assert_eq!(
            format_dry_run_command(Runner::Npm, "dev", None),
            "Would run: npm run dev"
        );

        assert_eq!(
            format_dry_run_command(Runner::Npm, "dev", Some("--host")),
            "Would run: npm run dev -- --host"
        );

        assert_eq!(
            format_dry_run_command(Runner::Yarn, "dev", Some("--host")),
            "Would run: yarn dev --host"
        );

        assert_eq!(
            format_dry_run_command(Runner::Pnpm, "dev", Some("--host")),
            "Would run: pnpm dev -- --host"
        );

        assert_eq!(
            format_dry_run_command(Runner::Bun, "dev", Some("--host")),
            "Would run: bun run dev --host"
        );
    }

    #[test]
    fn test_run_script_dry_run() {
        let script = Script::new("dev", "vite");
        let result = run_script(Runner::Npm, &script, None, true).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_run_script_with_args_dry_run() {
        let script = Script::new("dev", "vite");
        let result = run_script(Runner::Npm, &script, Some("--host"), true).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_run_scripts_dry_run() {
        let script1 = Script::new("build", "vite build");
        let script2 = Script::new("test", "vitest");
        let scripts: Vec<(&Script, Option<String>)> =
            vec![(&script1, None), (&script2, Some("--coverage".to_string()))];

        let results = run_scripts(Runner::Npm, &scripts, true).unwrap();
        assert_eq!(results, vec![0, 0]);
    }
}
