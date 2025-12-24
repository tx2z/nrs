//! Path utilities.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::error::NrsError;

/// Maximum number of parent directories to search.
pub const MAX_SEARCH_DEPTH: usize = 10;

/// Find the package.json file starting from the given directory.
///
/// Searches the given directory and up to 10 parent directories.
///
/// # Errors
///
/// Returns an error if no package.json is found.
pub fn find_package_json(start_dir: &Path) -> Result<PathBuf> {
    let start = start_dir.canonicalize().with_context(|| {
        format!(
            "Cannot access directory '{}': path does not exist or is not accessible",
            start_dir.display()
        )
    })?;

    let mut current = start.as_path();
    let mut depth = 0;

    while depth < MAX_SEARCH_DEPTH {
        let package_json = current.join("package.json");
        if package_json.exists() {
            return Ok(package_json);
        }

        match current.parent() {
            Some(parent) if parent != current => {
                current = parent;
                depth += 1;
            }
            _ => break,
        }
    }

    Err(NrsError::NoPackageJson {
        path: start,
        depth: MAX_SEARCH_DEPTH,
    }
    .into())
}

/// Find the project root (directory containing package.json).
///
/// # Errors
///
/// Returns an error if no package.json is found.
pub fn find_project_root(start_dir: &Path) -> Result<PathBuf> {
    let package_json = find_package_json(start_dir)?;
    Ok(package_json
        .parent()
        .expect("package.json should have parent")
        .to_path_buf())
}

/// Get the config directory for nrs.
///
/// Returns `~/.config/nrs` on Unix-like systems.
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("nrs"))
}

/// Get the history file path.
///
/// Returns `~/.config/nrs/history.json`.
pub fn history_file() -> Option<PathBuf> {
    config_dir().map(|p| p.join("history.json"))
}

/// Get the global config file path.
///
/// Returns `~/.config/nrs/config.toml`.
pub fn global_config_file() -> Option<PathBuf> {
    config_dir().map(|p| p.join("config.toml"))
}

/// Find local config file in project directory.
///
/// Looks for `.nrsrc.toml` in the given directory.
pub fn local_config_file(project_dir: &Path) -> Option<PathBuf> {
    let config_file = project_dir.join(".nrsrc.toml");
    if config_file.exists() {
        Some(config_file)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_find_package_json_in_current_dir() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("package.json"), "{}").unwrap();

        let result = find_package_json(temp.path());
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("package.json"));
    }

    #[test]
    fn test_find_package_json_in_parent() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("package.json"), "{}").unwrap();

        let subdir = temp.path().join("src");
        std::fs::create_dir(&subdir).unwrap();

        let result = find_package_json(&subdir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_package_json_not_found() {
        let temp = TempDir::new().unwrap();
        let result = find_package_json(temp.path());
        assert!(result.is_err());
    }
}
