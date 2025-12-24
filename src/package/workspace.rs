//! Monorepo and workspace support.
//!
//! Detects and manages workspaces in monorepo setups using:
//! - npm/yarn workspaces (package.json `workspaces` field)
//! - pnpm workspaces (pnpm-workspace.yaml)
//! - Lerna (lerna.json)

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use super::scripts::parse_scripts;
use super::types::Script;

/// Represents a workspace in a monorepo.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Name of the workspace package.
    name: String,
    /// Path to the workspace directory.
    path: PathBuf,
    /// Scripts available in this workspace.
    scripts: Vec<Script>,
}

impl Workspace {
    /// Create a new workspace.
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            scripts: Vec::new(),
        }
    }

    /// Create a new workspace with scripts.
    pub fn with_scripts(
        name: impl Into<String>,
        path: impl Into<PathBuf>,
        scripts: Vec<Script>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            scripts,
        }
    }

    /// Get the workspace name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the workspace path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the workspace scripts.
    pub fn scripts(&self) -> &[Script] {
        &self.scripts
    }

    /// Set the workspace scripts.
    pub fn set_scripts(&mut self, scripts: Vec<Script>) {
        self.scripts = scripts;
    }

    /// Check if the workspace has scripts.
    pub fn has_scripts(&self) -> bool {
        !self.scripts.is_empty()
    }

    /// Load scripts from the workspace's package.json.
    pub fn load_scripts(&mut self) -> Result<()> {
        let package_json = self.path.join("package.json");
        if package_json.exists() {
            self.scripts = parse_scripts(&self.path)
                .map(|scripts| scripts.into_iter().collect())
                .unwrap_or_default();
        }
        Ok(())
    }
}

/// Result of workspace detection.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceInfo {
    /// Whether this is a monorepo root.
    pub is_monorepo: bool,
    /// The type of workspace configuration detected.
    pub workspace_type: Option<WorkspaceType>,
    /// List of workspaces found.
    pub workspaces: Vec<Workspace>,
}

/// Type of workspace configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceType {
    /// npm/yarn workspaces (package.json)
    Npm,
    /// pnpm workspaces (pnpm-workspace.yaml)
    Pnpm,
    /// Lerna (lerna.json)
    Lerna,
}

impl std::fmt::Display for WorkspaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceType::Npm => write!(f, "npm workspaces"),
            WorkspaceType::Pnpm => write!(f, "pnpm workspaces"),
            WorkspaceType::Lerna => write!(f, "lerna"),
        }
    }
}

/// pnpm-workspace.yaml structure.
#[derive(Debug, Deserialize)]
struct PnpmWorkspace {
    packages: Option<Vec<String>>,
}

/// lerna.json structure.
#[derive(Debug, Deserialize)]
struct LernaConfig {
    packages: Option<Vec<String>>,
}

/// Detect workspaces in a monorepo.
///
/// Checks for:
/// - `workspaces` field in package.json
/// - `pnpm-workspace.yaml`
/// - `lerna.json`
///
/// Returns a list of workspaces with their scripts loaded.
pub fn detect_workspaces(project_dir: &Path) -> Result<Vec<Workspace>> {
    let info = detect_workspace_info(project_dir)?;
    Ok(info.workspaces)
}

/// Detect workspace configuration and return detailed info.
pub fn detect_workspace_info(project_dir: &Path) -> Result<WorkspaceInfo> {
    // Check for pnpm-workspace.yaml first (most specific)
    let pnpm_workspace = project_dir.join("pnpm-workspace.yaml");
    if pnpm_workspace.exists() {
        return detect_pnpm_workspaces(project_dir, &pnpm_workspace);
    }

    // Check for lerna.json
    let lerna_json = project_dir.join("lerna.json");
    if lerna_json.exists() {
        return detect_lerna_workspaces(project_dir, &lerna_json);
    }

    // Check for workspaces in package.json
    let package_json = project_dir.join("package.json");
    if package_json.exists() {
        let info = detect_npm_workspaces(project_dir, &package_json)?;
        if info.is_monorepo {
            return Ok(info);
        }
    }

    Ok(WorkspaceInfo::default())
}

/// Check if a directory is a monorepo root.
pub fn is_monorepo(project_dir: &Path) -> bool {
    let pnpm_workspace = project_dir.join("pnpm-workspace.yaml");
    if pnpm_workspace.exists() {
        return true;
    }

    let lerna_json = project_dir.join("lerna.json");
    if lerna_json.exists() {
        return true;
    }

    let package_json = project_dir.join("package.json");
    if package_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&package_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                return json.get("workspaces").is_some();
            }
        }
    }

    false
}

/// Detect workspaces from package.json workspaces field.
fn detect_npm_workspaces(project_dir: &Path, package_json: &Path) -> Result<WorkspaceInfo> {
    let content = std::fs::read_to_string(package_json)
        .with_context(|| format!("Failed to read {}", package_json.display()))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", package_json.display()))?;

    let workspace_patterns = match json.get("workspaces") {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(String::from)
            .collect::<Vec<_>>(),
        Some(serde_json::Value::Object(obj)) => obj
            .get("packages")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
        _ => return Ok(WorkspaceInfo::default()),
    };

    if workspace_patterns.is_empty() {
        return Ok(WorkspaceInfo::default());
    }

    let workspaces = resolve_workspace_patterns(project_dir, &workspace_patterns)?;

    Ok(WorkspaceInfo {
        is_monorepo: true,
        workspace_type: Some(WorkspaceType::Npm),
        workspaces,
    })
}

/// Detect workspaces from pnpm-workspace.yaml.
fn detect_pnpm_workspaces(project_dir: &Path, workspace_file: &Path) -> Result<WorkspaceInfo> {
    let content = std::fs::read_to_string(workspace_file)
        .with_context(|| format!("Failed to read {}", workspace_file.display()))?;

    let config: PnpmWorkspace = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse {}", workspace_file.display()))?;

    let patterns = config.packages.unwrap_or_default();

    if patterns.is_empty() {
        return Ok(WorkspaceInfo {
            is_monorepo: true,
            workspace_type: Some(WorkspaceType::Pnpm),
            workspaces: Vec::new(),
        });
    }

    let workspaces = resolve_workspace_patterns(project_dir, &patterns)?;

    Ok(WorkspaceInfo {
        is_monorepo: true,
        workspace_type: Some(WorkspaceType::Pnpm),
        workspaces,
    })
}

/// Detect workspaces from lerna.json.
fn detect_lerna_workspaces(project_dir: &Path, lerna_file: &Path) -> Result<WorkspaceInfo> {
    let content = std::fs::read_to_string(lerna_file)
        .with_context(|| format!("Failed to read {}", lerna_file.display()))?;

    let config: LernaConfig = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", lerna_file.display()))?;

    // Lerna defaults to "packages/*" if not specified
    let patterns = config
        .packages
        .unwrap_or_else(|| vec!["packages/*".to_string()]);

    let workspaces = resolve_workspace_patterns(project_dir, &patterns)?;

    Ok(WorkspaceInfo {
        is_monorepo: true,
        workspace_type: Some(WorkspaceType::Lerna),
        workspaces,
    })
}

/// Resolve workspace glob patterns to actual directories.
fn resolve_workspace_patterns(project_dir: &Path, patterns: &[String]) -> Result<Vec<Workspace>> {
    let mut workspaces = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for pattern in patterns {
        // Skip negation patterns (we'll filter later if needed)
        if pattern.starts_with('!') {
            continue;
        }

        // Normalize the pattern
        let normalized_pattern = normalize_glob_pattern(pattern);
        let full_pattern = project_dir.join(&normalized_pattern);

        // Use glob to expand the pattern
        let glob_pattern = full_pattern.to_string_lossy();

        match glob::glob(&glob_pattern) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    // Skip if not a directory
                    if !entry.is_dir() {
                        continue;
                    }

                    // Skip if we've already seen this path
                    if !seen_paths.insert(entry.clone()) {
                        continue;
                    }

                    // Check for package.json
                    let package_json = entry.join("package.json");
                    if !package_json.exists() {
                        continue;
                    }

                    // Read workspace info
                    if let Some(workspace) = create_workspace_from_path(&entry) {
                        workspaces.push(workspace);
                    }
                }
            }
            Err(_) => {
                // If glob fails, try as a direct path
                let direct_path = project_dir.join(pattern.trim_end_matches("/*"));
                if direct_path.is_dir()
                    && direct_path.join("package.json").exists()
                    && seen_paths.insert(direct_path.clone())
                {
                    if let Some(workspace) = create_workspace_from_path(&direct_path) {
                        workspaces.push(workspace);
                    }
                }
            }
        }
    }

    // Sort workspaces by name
    workspaces.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(workspaces)
}

/// Normalize a glob pattern for the glob crate.
fn normalize_glob_pattern(pattern: &str) -> String {
    let mut normalized = pattern.to_string();

    // Replace ** with a temp marker to handle it specially
    normalized = normalized.replace("**", "\0DOUBLESTAR\0");

    // If pattern ends with *, it should match directories
    // glob crate handles this, but we want to ensure we're matching directories
    if normalized.ends_with('\0') {
        normalized.push('*');
    }

    // Restore double star
    normalized = normalized.replace("\0DOUBLESTAR\0", "**");

    normalized
}

/// Create a Workspace from a directory path.
fn create_workspace_from_path(path: &Path) -> Option<Workspace> {
    let package_json = path.join("package.json");
    let content = std::fs::read_to_string(&package_json).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    // Get the package name, falling back to directory name
    let name = json
        .get("name")
        .and_then(|n| n.as_str())
        .map(String::from)
        .or_else(|| path.file_name().and_then(|n| n.to_str()).map(String::from))?;

    // Parse scripts - parse_scripts takes a directory path
    let scripts: Vec<Script> = parse_scripts(path)
        .map(|s| s.into_iter().collect())
        .unwrap_or_default();

    Some(Workspace::with_scripts(name, path.to_path_buf(), scripts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_package_json(dir: &Path, name: &str, scripts: &[(&str, &str)]) {
        let scripts_obj: serde_json::Map<_, _> = scripts
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
            .collect();

        let package = serde_json::json!({
            "name": name,
            "version": "1.0.0",
            "scripts": scripts_obj
        });

        fs::write(
            dir.join("package.json"),
            serde_json::to_string_pretty(&package).unwrap(),
        )
        .unwrap();
    }

    fn create_monorepo(temp: &TempDir, workspace_type: &str) -> PathBuf {
        let root = temp.path().to_path_buf();

        // Create root package.json
        let root_scripts = [("dev", "turbo dev"), ("build", "turbo build")];

        match workspace_type {
            "npm" => {
                let package = serde_json::json!({
                    "name": "monorepo",
                    "private": true,
                    "workspaces": ["packages/*"],
                    "scripts": {
                        "dev": "turbo dev",
                        "build": "turbo build"
                    }
                });
                fs::write(root.join("package.json"), package.to_string()).unwrap();
            }
            "pnpm" => {
                create_package_json(&root, "monorepo", &root_scripts);
                fs::write(
                    root.join("pnpm-workspace.yaml"),
                    "packages:\n  - packages/*\n",
                )
                .unwrap();
            }
            "lerna" => {
                create_package_json(&root, "monorepo", &root_scripts);
                fs::write(
                    root.join("lerna.json"),
                    r#"{"packages": ["packages/*"], "version": "1.0.0"}"#,
                )
                .unwrap();
            }
            _ => panic!("Unknown workspace type"),
        }

        // Create packages directory
        let packages_dir = root.join("packages");
        fs::create_dir_all(&packages_dir).unwrap();

        // Create package A
        let pkg_a = packages_dir.join("pkg-a");
        fs::create_dir_all(&pkg_a).unwrap();
        create_package_json(
            &pkg_a,
            "@monorepo/pkg-a",
            &[("build", "tsc"), ("test", "jest")],
        );

        // Create package B
        let pkg_b = packages_dir.join("pkg-b");
        fs::create_dir_all(&pkg_b).unwrap();
        create_package_json(
            &pkg_b,
            "@monorepo/pkg-b",
            &[("dev", "vite"), ("build", "vite build")],
        );

        root
    }

    // ==================== Basic Tests ====================

    #[test]
    fn test_workspace_new() {
        let ws = Workspace::new("test", "/path/to/workspace");
        assert_eq!(ws.name(), "test");
        assert_eq!(ws.path(), Path::new("/path/to/workspace"));
        assert!(ws.scripts().is_empty());
    }

    #[test]
    fn test_workspace_with_scripts() {
        let scripts = vec![Script::new("build", "tsc"), Script::new("test", "jest")];
        let ws = Workspace::with_scripts("test", "/path", scripts.clone());
        assert_eq!(ws.scripts().len(), 2);
        assert!(ws.has_scripts());
    }

    // ==================== npm Workspace Tests ====================

    #[test]
    fn test_detect_npm_workspaces() {
        let temp = TempDir::new().unwrap();
        let root = create_monorepo(&temp, "npm");

        let info = detect_workspace_info(&root).unwrap();
        assert!(info.is_monorepo);
        assert_eq!(info.workspace_type, Some(WorkspaceType::Npm));
        assert_eq!(info.workspaces.len(), 2);

        let names: Vec<&str> = info.workspaces.iter().map(|w| w.name()).collect();
        assert!(names.contains(&"@monorepo/pkg-a"));
        assert!(names.contains(&"@monorepo/pkg-b"));
    }

    #[test]
    fn test_detect_npm_workspaces_object_format() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // Use object format for workspaces
        let package = serde_json::json!({
            "name": "monorepo",
            "workspaces": {
                "packages": ["packages/*"]
            }
        });
        fs::write(root.join("package.json"), package.to_string()).unwrap();

        // Create packages
        let packages_dir = root.join("packages");
        fs::create_dir_all(&packages_dir).unwrap();
        let pkg = packages_dir.join("pkg");
        fs::create_dir_all(&pkg).unwrap();
        create_package_json(&pkg, "pkg", &[("build", "tsc")]);

        let info = detect_workspace_info(root).unwrap();
        assert!(info.is_monorepo);
        assert_eq!(info.workspaces.len(), 1);
    }

    // ==================== pnpm Workspace Tests ====================

    #[test]
    fn test_detect_pnpm_workspaces() {
        let temp = TempDir::new().unwrap();
        let root = create_monorepo(&temp, "pnpm");

        let info = detect_workspace_info(&root).unwrap();
        assert!(info.is_monorepo);
        assert_eq!(info.workspace_type, Some(WorkspaceType::Pnpm));
        assert_eq!(info.workspaces.len(), 2);
    }

    #[test]
    fn test_detect_pnpm_empty_packages() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_package_json(root, "monorepo", &[]);
        fs::write(root.join("pnpm-workspace.yaml"), "packages: []\n").unwrap();

        let info = detect_workspace_info(root).unwrap();
        assert!(info.is_monorepo);
        assert_eq!(info.workspace_type, Some(WorkspaceType::Pnpm));
        assert!(info.workspaces.is_empty());
    }

    // ==================== Lerna Tests ====================

    #[test]
    fn test_detect_lerna_workspaces() {
        let temp = TempDir::new().unwrap();
        let root = create_monorepo(&temp, "lerna");

        let info = detect_workspace_info(&root).unwrap();
        assert!(info.is_monorepo);
        assert_eq!(info.workspace_type, Some(WorkspaceType::Lerna));
        assert_eq!(info.workspaces.len(), 2);
    }

    #[test]
    fn test_detect_lerna_default_packages() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_package_json(root, "monorepo", &[]);
        fs::write(root.join("lerna.json"), r#"{"version": "1.0.0"}"#).unwrap();

        // Create default packages directory
        let packages_dir = root.join("packages");
        fs::create_dir_all(&packages_dir).unwrap();
        let pkg = packages_dir.join("pkg");
        fs::create_dir_all(&pkg).unwrap();
        create_package_json(&pkg, "pkg", &[("build", "tsc")]);

        let info = detect_workspace_info(root).unwrap();
        assert!(info.is_monorepo);
        assert_eq!(info.workspace_type, Some(WorkspaceType::Lerna));
        assert_eq!(info.workspaces.len(), 1);
    }

    // ==================== is_monorepo Tests ====================

    #[test]
    fn test_is_monorepo_npm() {
        let temp = TempDir::new().unwrap();
        let root = create_monorepo(&temp, "npm");
        assert!(is_monorepo(&root));
    }

    #[test]
    fn test_is_monorepo_pnpm() {
        let temp = TempDir::new().unwrap();
        let root = create_monorepo(&temp, "pnpm");
        assert!(is_monorepo(&root));
    }

    #[test]
    fn test_is_monorepo_lerna() {
        let temp = TempDir::new().unwrap();
        let root = create_monorepo(&temp, "lerna");
        assert!(is_monorepo(&root));
    }

    #[test]
    fn test_is_not_monorepo() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        create_package_json(root, "simple-project", &[("build", "tsc")]);
        assert!(!is_monorepo(root));
    }

    // ==================== Workspace Scripts Tests ====================

    #[test]
    fn test_workspace_scripts_loaded() {
        let temp = TempDir::new().unwrap();
        let root = create_monorepo(&temp, "npm");

        let workspaces = detect_workspaces(&root).unwrap();

        // Find pkg-a and check its scripts
        let pkg_a = workspaces.iter().find(|w| w.name() == "@monorepo/pkg-a");
        assert!(pkg_a.is_some());

        let pkg_a = pkg_a.unwrap();
        assert!(pkg_a.has_scripts());

        let script_names: Vec<&str> = pkg_a.scripts().iter().map(|s| s.name()).collect();
        assert!(script_names.contains(&"build"));
        assert!(script_names.contains(&"test"));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_no_workspaces() {
        let temp = TempDir::new().unwrap();
        create_package_json(temp.path(), "simple", &[("build", "tsc")]);

        let info = detect_workspace_info(temp.path()).unwrap();
        assert!(!info.is_monorepo);
        assert!(info.workspaces.is_empty());
    }

    #[test]
    fn test_workspace_without_package_json() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // Create workspaces config pointing to a dir without package.json
        let package = serde_json::json!({
            "name": "monorepo",
            "workspaces": ["packages/*"]
        });
        fs::write(root.join("package.json"), package.to_string()).unwrap();

        let packages_dir = root.join("packages");
        fs::create_dir_all(&packages_dir).unwrap();

        // Create directory without package.json
        fs::create_dir_all(packages_dir.join("no-pkg")).unwrap();

        let info = detect_workspace_info(root).unwrap();
        assert!(info.is_monorepo);
        // Should not include the directory without package.json
        assert!(info.workspaces.is_empty());
    }

    #[test]
    fn test_workspace_type_display() {
        assert_eq!(format!("{}", WorkspaceType::Npm), "npm workspaces");
        assert_eq!(format!("{}", WorkspaceType::Pnpm), "pnpm workspaces");
        assert_eq!(format!("{}", WorkspaceType::Lerna), "lerna");
    }

    // ==================== Multiple Patterns Tests ====================

    #[test]
    fn test_multiple_workspace_patterns() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        let package = serde_json::json!({
            "name": "monorepo",
            "workspaces": ["packages/*", "apps/*"]
        });
        fs::write(root.join("package.json"), package.to_string()).unwrap();

        // Create packages
        let packages_dir = root.join("packages");
        fs::create_dir_all(&packages_dir).unwrap();
        let pkg = packages_dir.join("lib");
        fs::create_dir_all(&pkg).unwrap();
        create_package_json(&pkg, "@monorepo/lib", &[("build", "tsc")]);

        // Create apps
        let apps_dir = root.join("apps");
        fs::create_dir_all(&apps_dir).unwrap();
        let app = apps_dir.join("web");
        fs::create_dir_all(&app).unwrap();
        create_package_json(&app, "@monorepo/web", &[("dev", "vite")]);

        let info = detect_workspace_info(root).unwrap();
        assert!(info.is_monorepo);
        assert_eq!(info.workspaces.len(), 2);

        let names: Vec<&str> = info.workspaces.iter().map(|w| w.name()).collect();
        assert!(names.contains(&"@monorepo/lib"));
        assert!(names.contains(&"@monorepo/web"));
    }
}
