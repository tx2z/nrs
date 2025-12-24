//! Runner module for nrs.
//!
//! Handles script execution with the appropriate package manager.

mod executor;

pub use executor::{
    execute_script, execute_workspace_script, format_dry_run_command,
    format_workspace_dry_run_command, run_script, run_script_in_dir, run_scripts,
    run_scripts_in_dir, run_workspace_script, ExecutionResult, EXIT_CODE_INTERRUPTED,
};
