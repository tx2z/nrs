//! Package module for nrs.
//!
//! Handles package.json parsing, script extraction, and package manager detection.

mod descriptions;
mod manager;
pub mod scripts;
mod types;
mod workspace;

pub use descriptions::{extract_descriptions, get_description, get_short_description};
pub use manager::{detect_runner, detect_runner_reason, has_lock_file, Runner};
pub use scripts::{
    parse_package_json, parse_scripts, parse_scripts_from_json, parse_scripts_required,
};
pub use types::{
    is_lifecycle_script, NtlConfig, Package, Script, Scripts, WorkspacesConfig, LIFECYCLE_SCRIPTS,
};
pub use workspace::{
    detect_workspace_info, detect_workspaces, is_monorepo, Workspace, WorkspaceInfo, WorkspaceType,
};
