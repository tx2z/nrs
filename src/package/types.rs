//! Type definitions for package.json parsing.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// A script defined in package.json.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Script {
    name: String,
    command: String,
    description: Option<String>,
}

impl Script {
    /// Create a new script.
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            description: None,
        }
    }

    /// Create a new script with a description.
    pub fn with_description(
        name: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            description: Some(description.into()),
        }
    }

    /// Get the script name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the script command.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Get the script description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Set the description.
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    /// Check if this is a lifecycle script.
    pub fn is_lifecycle(&self) -> bool {
        is_lifecycle_script(&self.name)
    }

    /// Check if this is a pre/post script for another script.
    pub fn is_hook_for(&self, script_name: &str) -> bool {
        self.name == format!("pre{script_name}") || self.name == format!("post{script_name}")
    }
}

impl fmt::Debug for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Script")
            .field("name", &self.name)
            .field("command", &self.command)
            .field("description", &self.description)
            .finish()
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(desc) = &self.description {
            write!(f, "{}: {} ({})", self.name, self.command, desc)
        } else {
            write!(f, "{}: {}", self.name, self.command)
        }
    }
}

/// Collection of scripts from a project.
#[derive(Debug, Clone, Default)]
pub struct Scripts {
    scripts: Vec<Script>,
}

impl Scripts {
    /// Create a new empty scripts collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a vector of scripts.
    pub fn from_vec(scripts: Vec<Script>) -> Self {
        Self { scripts }
    }

    /// Add a script to the collection.
    pub fn add(&mut self, script: Script) {
        self.scripts.push(script);
    }

    /// Get the number of scripts.
    pub fn len(&self) -> usize {
        self.scripts.len()
    }

    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.scripts.is_empty()
    }

    /// Get an iterator over the scripts.
    pub fn iter(&self) -> impl Iterator<Item = &Script> {
        self.scripts.iter()
    }

    /// Get a mutable iterator over the scripts.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Script> {
        self.scripts.iter_mut()
    }

    /// Get the scripts as a slice.
    pub fn as_slice(&self) -> &[Script] {
        &self.scripts
    }

    /// Get a script by name.
    pub fn get(&self, name: &str) -> Option<&Script> {
        self.scripts.iter().find(|s| s.name == name)
    }

    /// Get a mutable script by name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Script> {
        self.scripts.iter_mut().find(|s| s.name == name)
    }

    /// Filter out lifecycle scripts.
    pub fn without_lifecycle(&self) -> Self {
        Self {
            scripts: self
                .scripts
                .iter()
                .filter(|s| !s.is_lifecycle())
                .cloned()
                .collect(),
        }
    }

    /// Filter out scripts matching the given patterns.
    /// Supports glob patterns with '*' wildcard.
    pub fn without_matching(&self, patterns: &[String]) -> Self {
        if patterns.is_empty() {
            return self.clone();
        }

        Self {
            scripts: self
                .scripts
                .iter()
                .filter(|s| !matches_any_pattern(s.name(), patterns))
                .cloned()
                .collect(),
        }
    }

    /// Get script names as a vector.
    pub fn names(&self) -> Vec<&str> {
        self.scripts.iter().map(|s| s.name()).collect()
    }

    /// Sort scripts alphabetically by name.
    pub fn sort_alphabetically(&mut self) {
        self.scripts.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

impl IntoIterator for Scripts {
    type Item = Script;
    type IntoIter = std::vec::IntoIter<Script>;

    fn into_iter(self) -> Self::IntoIter {
        self.scripts.into_iter()
    }
}

impl<'a> IntoIterator for &'a Scripts {
    type Item = &'a Script;
    type IntoIter = std::slice::Iter<'a, Script>;

    fn into_iter(self) -> Self::IntoIter {
        self.scripts.iter()
    }
}

/// Parsed package.json structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Package {
    /// Package name.
    #[serde(default)]
    pub name: String,

    /// Package version.
    #[serde(default)]
    pub version: String,

    /// Package description.
    #[serde(default)]
    pub description: Option<String>,

    /// Package manager specification (e.g., "pnpm@8.0.0").
    #[serde(default, rename = "packageManager")]
    pub package_manager: Option<String>,

    /// Raw scripts object.
    #[serde(default)]
    pub scripts: HashMap<String, String>,

    /// Scripts info for descriptions.
    #[serde(default, rename = "scripts-info")]
    pub scripts_info: HashMap<String, String>,

    /// NTL configuration (for descriptions).
    #[serde(default)]
    pub ntl: Option<NtlConfig>,

    /// Workspaces configuration.
    #[serde(default)]
    pub workspaces: Option<WorkspacesConfig>,
}

impl Package {
    /// Get the package name, or "unnamed" if not set.
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            "unnamed"
        } else {
            &self.name
        }
    }

    /// Check if this package has any scripts.
    pub fn has_scripts(&self) -> bool {
        !self.scripts.is_empty()
    }

    /// Get the number of scripts.
    pub fn script_count(&self) -> usize {
        self.scripts.keys().filter(|k| !k.starts_with("//")).count()
    }

    /// Check if this is a monorepo (has workspaces).
    pub fn is_monorepo(&self) -> bool {
        self.workspaces.is_some()
    }

    /// Extract the package manager name from the packageManager field.
    pub fn package_manager_name(&self) -> Option<&str> {
        self.package_manager
            .as_ref()
            .map(|pm| pm.split('@').next().unwrap_or(pm))
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.display_name(), self.version)
    }
}

/// NTL configuration structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NtlConfig {
    /// Script descriptions.
    #[serde(default)]
    pub descriptions: HashMap<String, String>,
}

/// Workspaces configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkspacesConfig {
    /// Simple array of glob patterns.
    Array(Vec<String>),
    /// Object with packages field.
    Object { packages: Vec<String> },
}

impl WorkspacesConfig {
    /// Get the workspace patterns.
    pub fn patterns(&self) -> &[String] {
        match self {
            WorkspacesConfig::Array(patterns) => patterns,
            WorkspacesConfig::Object { packages } => packages,
        }
    }
}

/// Lifecycle scripts that are hidden by default.
pub const LIFECYCLE_SCRIPTS: &[&str] = &[
    "preinstall",
    "install",
    "postinstall",
    "preuninstall",
    "uninstall",
    "postuninstall",
    "prepublish",
    "prepublishOnly",
    "publish",
    "postpublish",
    "preversion",
    "version",
    "postversion",
    "prepack",
    "pack",
    "postpack",
    "prepare",
    "preshrinkwrap",
    "shrinkwrap",
    "postshrinkwrap",
];

/// Check if a script name is a lifecycle script.
pub fn is_lifecycle_script(name: &str) -> bool {
    LIFECYCLE_SCRIPTS.contains(&name)
}

/// Check if a name matches any of the given patterns.
/// Supports simple glob patterns:
/// - `*` matches any sequence of characters
/// - Exact match if no wildcards
fn matches_any_pattern(name: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .any(|pattern| matches_pattern(name, pattern))
}

/// Check if a name matches a simple glob pattern.
fn matches_pattern(name: &str, pattern: &str) -> bool {
    if !pattern.contains('*') {
        // Exact match
        return name == pattern;
    }

    // Simple glob matching with * wildcard
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 2 {
        // Single wildcard
        let (prefix, suffix) = (parts[0], parts[1]);
        name.starts_with(prefix) && name.ends_with(suffix)
    } else if parts.len() == 1 {
        // Just a wildcard (matches everything)
        true
    } else {
        // Multiple wildcards - do a more complex match
        let mut remaining = name;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            if i == 0 {
                // First part must be a prefix
                if !remaining.starts_with(part) {
                    return false;
                }
                remaining = &remaining[part.len()..];
            } else if i == parts.len() - 1 {
                // Last part must be a suffix
                if !remaining.ends_with(part) {
                    return false;
                }
            } else {
                // Middle parts must exist somewhere
                if let Some(pos) = remaining.find(part) {
                    remaining = &remaining[pos + part.len()..];
                } else {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_display() {
        let script = Script::new("dev", "vite");
        assert_eq!(format!("{script}"), "dev: vite");

        let script_with_desc = Script::with_description("build", "vite build", "Build for prod");
        assert_eq!(
            format!("{script_with_desc}"),
            "build: vite build (Build for prod)"
        );
    }

    #[test]
    fn test_script_is_lifecycle() {
        let dev = Script::new("dev", "vite");
        assert!(!dev.is_lifecycle());

        let install = Script::new("postinstall", "husky install");
        assert!(install.is_lifecycle());
    }

    #[test]
    fn test_script_is_hook_for() {
        let prebuild = Script::new("prebuild", "echo 'before build'");
        assert!(prebuild.is_hook_for("build"));
        assert!(!prebuild.is_hook_for("test"));

        let posttest = Script::new("posttest", "echo 'after test'");
        assert!(posttest.is_hook_for("test"));
    }

    #[test]
    fn test_scripts_collection() {
        let mut scripts = Scripts::new();
        scripts.add(Script::new("dev", "vite"));
        scripts.add(Script::new("build", "vite build"));

        assert_eq!(scripts.len(), 2);
        assert!(!scripts.is_empty());
        assert!(scripts.get("dev").is_some());
        assert!(scripts.get("unknown").is_none());
    }

    #[test]
    fn test_scripts_names() {
        let mut scripts = Scripts::new();
        scripts.add(Script::new("build", "vite build"));
        scripts.add(Script::new("dev", "vite"));

        let names = scripts.names();
        assert!(names.contains(&"dev"));
        assert!(names.contains(&"build"));
    }

    #[test]
    fn test_package_display_name() {
        let pkg = Package {
            name: "my-app".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };
        assert_eq!(pkg.display_name(), "my-app");

        let unnamed = Package::default();
        assert_eq!(unnamed.display_name(), "unnamed");
    }

    #[test]
    fn test_package_manager_name() {
        let pkg = Package {
            package_manager: Some("pnpm@8.0.0".to_string()),
            ..Default::default()
        };
        assert_eq!(pkg.package_manager_name(), Some("pnpm"));

        let pkg_no_version = Package {
            package_manager: Some("yarn".to_string()),
            ..Default::default()
        };
        assert_eq!(pkg_no_version.package_manager_name(), Some("yarn"));
    }

    #[test]
    fn test_lifecycle_scripts() {
        assert!(is_lifecycle_script("preinstall"));
        assert!(is_lifecycle_script("postpublish"));
        assert!(is_lifecycle_script("prepare"));
        assert!(!is_lifecycle_script("dev"));
        assert!(!is_lifecycle_script("build"));
        assert!(!is_lifecycle_script("test"));
    }
}
