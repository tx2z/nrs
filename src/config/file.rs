//! Configuration file loading and parsing.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::types::Config;

/// Load configuration from the specified path.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
fn load_config_from_path(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    Ok(config)
}

/// Load configuration with proper priority and merging.
///
/// Searches for config files in order of priority (lowest to highest):
/// 1. `~/.config/nrs/config.toml` (user-level, lowest priority)
/// 2. `.nrsrc.toml` in project root (project-level)
/// 3. CLI argument `--config <path>` (highest priority)
///
/// Configs are merged with higher priority configs overriding lower priority ones.
/// Missing config files are handled gracefully (defaults are used).
///
/// # Arguments
///
/// * `cli_config_path` - Optional path to config file specified via CLI argument
/// * `project_dir` - The project directory (where package.json is located)
///
/// # Errors
///
/// Returns an error if a specified config file (via CLI) cannot be read or parsed.
/// Missing default config files are not treated as errors.
pub fn load_config(cli_config_path: Option<&Path>, project_dir: &Path) -> Result<Config> {
    let mut config = Config::default();

    // Load user-level config (lowest priority)
    if let Some(user_config_path) = Config::user_config_path() {
        if user_config_path.exists() {
            match load_config_from_path(&user_config_path) {
                Ok(user_config) => config.merge(user_config),
                Err(e) => {
                    // Log warning but don't fail - use defaults
                    eprintln!(
                        "Warning: Failed to load user config at {}: {}",
                        user_config_path.display(),
                        e
                    );
                }
            }
        }
    }

    // Load project-level config (medium priority)
    let project_config_path = project_dir.join(".nrsrc.toml");
    if project_config_path.exists() {
        match load_config_from_path(&project_config_path) {
            Ok(project_config) => config.merge(project_config),
            Err(e) => {
                // Log warning but don't fail - use what we have so far
                eprintln!(
                    "Warning: Failed to load project config at {}: {}",
                    project_config_path.display(),
                    e
                );
            }
        }
    }

    // Load CLI-specified config (highest priority)
    if let Some(cli_path) = cli_config_path {
        let cli_config = load_config_from_path(cli_path).with_context(|| {
            format!(
                "Failed to load config from CLI-specified path: {}",
                cli_path.display()
            )
        })?;
        config.merge(cli_config);
    }

    Ok(config)
}

/// Generate an example configuration file with all options documented.
pub fn generate_example_config() -> String {
    r#"# nrs Configuration File
# Place this file at ~/.config/nrs/config.toml for global settings
# or .nrsrc.toml in your project directory for project-specific settings

# General settings
[general]
# Default package manager (overrides auto-detection)
# Options: "npm", "yarn", "pnpm", "bun"
# runner = "pnpm"

# Default sort mode: "recent", "alpha", "category"
default_sort = "recent"

# Column direction: "horizontal", "vertical"
# horizontal: 1 2 3 4 / 5 6 7 8
# vertical: 1 4 7 / 2 5 8 / 3 6 9
column_direction = "horizontal"

# Show command preview in description panel
show_command_preview = true

# Maximum items to show (0 = unlimited)
max_items = 0

# Filter settings
[filter]
# Search in descriptions too
search_descriptions = true

# Fuzzy matching
fuzzy = true

# Case sensitive search
case_sensitive = false

# History settings
[history]
# Enable history tracking
enabled = true

# Max projects to remember
max_projects = 100

# Max scripts per project
max_scripts = 50

# Exclude patterns
[exclude]
# Global patterns to exclude (glob syntax)
patterns = [
    # "pre*",
    # "post*",
]

# Appearance settings
[appearance]
# Color theme: "default", "minimal", "none"
theme = "default"

# Show icons
icons = true

# Show help footer
show_footer = true

# Compact mode (less padding)
compact = false

# Keybindings (advanced)
[keybindings]
# Custom keybindings
# quit = ["q", "Ctrl+c"]
# run = ["Enter", "o"]
# filter = ["/", "Ctrl+f"]

# Script customizations
[scripts]

# Custom descriptions for scripts (override package.json)
[scripts.descriptions]
# dev = "Start dev server on port 3000"
# build = "Production build with minification"

# Script aliases
[scripts.aliases]
# d = "dev"
# b = "build"
# t = "test"
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    #[test]
    fn test_load_default_config_returns_defaults() {
        let temp = create_temp_dir();
        let config = load_config(None, temp.path()).unwrap();

        assert!(config.history.enabled);
        assert_eq!(config.history.max_projects, 100);
        assert!(config.filter.fuzzy);
        assert!(config.appearance.icons);
    }

    #[test]
    fn test_load_project_config() {
        let temp = create_temp_dir();

        let config_content = r#"
[general]
runner = "yarn"
default_sort = "alpha"

[filter]
fuzzy = false
"#;

        fs::write(temp.path().join(".nrsrc.toml"), config_content).unwrap();

        let config = load_config(None, temp.path()).unwrap();

        assert_eq!(config.general.runner, Some(crate::package::Runner::Yarn));
        assert_eq!(config.general.default_sort, super::super::SortMode::Alpha);
        assert!(!config.filter.fuzzy);
    }

    #[test]
    fn test_load_cli_config_overrides() {
        let temp = create_temp_dir();

        // Create project config
        let project_config = r#"
[general]
runner = "yarn"

[appearance]
icons = false
"#;
        fs::write(temp.path().join(".nrsrc.toml"), project_config).unwrap();

        // Create CLI config that also specifies appearance to preserve the setting
        let cli_config_path = temp.path().join("cli-config.toml");
        let cli_config = r#"
[general]
runner = "pnpm"

[appearance]
icons = false
"#;
        fs::write(&cli_config_path, cli_config).unwrap();

        let config = load_config(Some(&cli_config_path), temp.path()).unwrap();

        // CLI config should override project config for runner
        assert_eq!(config.general.runner, Some(crate::package::Runner::Pnpm));
        // CLI config preserves the icons setting
        assert!(!config.appearance.icons);
    }

    #[test]
    fn test_cli_config_uses_defaults_when_section_not_specified() {
        let temp = create_temp_dir();

        // Create project config with custom appearance
        let project_config = r#"
[appearance]
icons = false
compact = true
"#;
        fs::write(temp.path().join(".nrsrc.toml"), project_config).unwrap();

        // CLI config doesn't specify appearance, so it will use defaults
        let cli_config_path = temp.path().join("cli-config.toml");
        let cli_config = r#"
[general]
runner = "pnpm"
"#;
        fs::write(&cli_config_path, cli_config).unwrap();

        let config = load_config(Some(&cli_config_path), temp.path()).unwrap();

        // Note: When CLI config is loaded, sections not specified use defaults.
        // These defaults then merge over the project config.
        // This means project-specific settings may be overwritten by CLI defaults.
        assert_eq!(config.general.runner, Some(crate::package::Runner::Pnpm));
        // Appearance uses CLI defaults (icons = true) because CLI config didn't specify it
        assert!(config.appearance.icons);
    }

    #[test]
    fn test_load_cli_config_file_not_found() {
        let temp = create_temp_dir();
        let non_existent = temp.path().join("does-not-exist.toml");

        let result = load_config(Some(&non_existent), temp.path());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to load config"));
    }

    #[test]
    fn test_invalid_toml_handling() {
        let temp = create_temp_dir();

        let invalid_toml = "this is not valid { toml }}}";
        let cli_config_path = temp.path().join("invalid.toml");
        fs::write(&cli_config_path, invalid_toml).unwrap();

        let result = load_config(Some(&cli_config_path), temp.path());

        assert!(result.is_err());
    }

    #[test]
    fn test_partial_config() {
        let temp = create_temp_dir();

        // Config that only specifies some values
        let config_content = r#"
[history]
max_projects = 50
"#;

        fs::write(temp.path().join(".nrsrc.toml"), config_content).unwrap();

        let config = load_config(None, temp.path()).unwrap();

        // Specified value should be set
        assert_eq!(config.history.max_projects, 50);
        // Other values should use defaults
        assert!(config.history.enabled);
        assert_eq!(config.history.max_scripts, 50);
        assert!(config.filter.fuzzy);
    }

    #[test]
    fn test_exclude_patterns_merge() {
        let temp = create_temp_dir();

        // Create project config with exclude patterns
        let project_config = r#"
[exclude]
patterns = ["test*", "lint*"]
"#;
        fs::write(temp.path().join(".nrsrc.toml"), project_config).unwrap();

        // Create CLI config with additional patterns
        let cli_config_path = temp.path().join("cli.toml");
        let cli_config = r#"
[exclude]
patterns = ["debug*"]
"#;
        fs::write(&cli_config_path, cli_config).unwrap();

        let config = load_config(Some(&cli_config_path), temp.path()).unwrap();

        // Patterns should be merged, not replaced
        assert_eq!(config.exclude.patterns.len(), 3);
        assert!(config.exclude.patterns.contains(&"test*".to_string()));
        assert!(config.exclude.patterns.contains(&"lint*".to_string()));
        assert!(config.exclude.patterns.contains(&"debug*".to_string()));
    }

    #[test]
    fn test_scripts_config() {
        let temp = create_temp_dir();

        let config_content = r#"
[scripts.descriptions]
dev = "Start development server"
build = "Build for production"

[scripts.aliases]
d = "dev"
b = "build"
"#;

        fs::write(temp.path().join(".nrsrc.toml"), config_content).unwrap();

        let config = load_config(None, temp.path()).unwrap();

        assert_eq!(
            config.scripts.descriptions.get("dev"),
            Some(&"Start development server".to_string())
        );
        assert_eq!(config.scripts.aliases.get("d"), Some(&"dev".to_string()));
    }

    #[test]
    fn test_generate_example_config() {
        let example = generate_example_config();

        // Verify it contains key sections
        assert!(example.contains("[general]"));
        assert!(example.contains("[filter]"));
        assert!(example.contains("[history]"));
        assert!(example.contains("[exclude]"));
        assert!(example.contains("[appearance]"));
        assert!(example.contains("[keybindings]"));
        assert!(example.contains("[scripts]"));
        assert!(example.contains("[scripts.descriptions]"));
        assert!(example.contains("[scripts.aliases]"));

        // Verify it's valid TOML (should parse without error)
        let result: Result<Config, _> = toml::from_str(&example);
        assert!(result.is_ok(), "Example config should be valid TOML");
    }

    #[test]
    fn test_all_config_options_have_defaults() {
        let config = Config::default();

        // Verify all fields have sensible defaults
        assert!(config.general.runner.is_none());
        assert_eq!(config.general.default_sort, super::super::SortMode::Recent);
        assert!(config.general.show_command_preview);
        assert!(config.filter.search_descriptions);
        assert!(config.history.enabled);
        assert!(config.exclude.patterns.is_empty());
        assert!(config.appearance.icons);
        assert!(config.keybindings.quit.is_empty());
        assert!(config.scripts.descriptions.is_empty());
    }
}
