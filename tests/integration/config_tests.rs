//! Integration tests for configuration loading and merging.

use npm_run_scripts::config::{AppearanceConfig, Config, ExcludeConfig};

use crate::integration::fixtures::{create_project_with_config, standard_scripts};

// ==================== Config Defaults ====================

#[test]
fn test_default_config() {
    let config = Config::default();

    assert!(config.appearance.icons);
    assert!(!config.appearance.compact);
    assert!(config.exclude.patterns.is_empty());
}

#[test]
fn test_config_new() {
    let config = Config::new();

    // Should be same as default
    assert!(config.appearance.icons);
    assert!(config.exclude.patterns.is_empty());
}

// ==================== Config Merging ====================

#[test]
fn test_config_merge() {
    let mut base = Config::default();
    base.appearance.icons = true;
    base.appearance.compact = true;
    base.exclude.patterns = vec!["base".to_string()];

    let overlay = Config {
        appearance: AppearanceConfig {
            icons: false,
            ..Default::default()
        },
        exclude: ExcludeConfig {
            patterns: vec!["overlay".to_string()],
        },
        ..Default::default()
    };

    base.merge(overlay);

    // Overlay value should win
    assert!(!base.appearance.icons);
    // Patterns should be merged
    assert!(base.exclude.patterns.contains(&"base".to_string()));
    assert!(base.exclude.patterns.contains(&"overlay".to_string()));
}

#[test]
fn test_exclude_patterns_merge() {
    let mut base = Config::default();
    base.exclude.patterns = vec!["a".to_string(), "b".to_string()];

    let overlay = Config {
        exclude: ExcludeConfig {
            patterns: vec!["b".to_string(), "c".to_string()],
        },
        ..Default::default()
    };

    base.merge(overlay);

    // Should have patterns from both
    assert!(base.exclude.patterns.contains(&"a".to_string()));
    assert!(base.exclude.patterns.contains(&"b".to_string()));
    assert!(base.exclude.patterns.contains(&"c".to_string()));
}

// ==================== Appearance Config ====================

#[test]
fn test_appearance_config_defaults() {
    let config = AppearanceConfig::default();

    assert!(config.icons);
    assert!(!config.compact);
    assert!(config.show_footer);
}

// ==================== Exclude Config ====================

#[test]
fn test_exclude_config_defaults() {
    let config = ExcludeConfig::default();

    assert!(config.patterns.is_empty());
}

// ==================== Config Serialization ====================

#[test]
fn test_config_toml_roundtrip() {
    let mut config = Config::default();
    config.appearance.icons = false;
    config.exclude.patterns = vec!["test".to_string()];

    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
    let parsed: Config = toml::from_str(&toml_str).expect("Failed to parse");

    assert!(!parsed.appearance.icons);
    assert_eq!(parsed.exclude.patterns, vec!["test"]);
}

#[test]
fn test_partial_config_parsing() {
    let toml_str = r#"
[appearance]
icons = false
"#;

    let config: Config = toml::from_str(toml_str).expect("Failed to parse");

    // Modified value
    assert!(!config.appearance.icons);

    // Default values for unspecified fields
    assert!(config.exclude.patterns.is_empty());
}

#[test]
fn test_empty_config_parsing() {
    let toml_str = "";

    let config: Config = toml::from_str(toml_str).expect("Failed to parse");

    // Should use all defaults
    assert!(config.appearance.icons);
}

#[test]
fn test_config_with_exclude_patterns() {
    let toml_str = r#"
[exclude]
patterns = ["test", "lint", "pre*"]
"#;

    let config: Config = toml::from_str(toml_str).expect("Failed to parse");

    assert_eq!(config.exclude.patterns.len(), 3);
    assert!(config.exclude.patterns.contains(&"test".to_string()));
    assert!(config.exclude.patterns.contains(&"lint".to_string()));
    assert!(config.exclude.patterns.contains(&"pre*".to_string()));
}

// ==================== History Config ====================

#[test]
fn test_history_config_defaults() {
    let config = Config::default();

    assert!(config.history.enabled);
    assert!(config.history.max_projects > 0);
    assert!(config.history.max_scripts > 0);
}

#[test]
fn test_history_config_parsing() {
    let toml_str = r#"
[history]
enabled = false
max_projects = 50
max_scripts = 25
"#;

    let config: Config = toml::from_str(toml_str).expect("Failed to parse");

    assert!(!config.history.enabled);
    assert_eq!(config.history.max_projects, 50);
    assert_eq!(config.history.max_scripts, 25);
}

// ==================== General Config ====================

#[test]
fn test_general_config_runner() {
    let toml_str = r#"
[general]
runner = "pnpm"
"#;

    let config: Config = toml::from_str(toml_str).expect("Failed to parse");

    assert!(config.general.runner.is_some());
}

// ==================== Scripts Config ====================

#[test]
fn test_scripts_config() {
    let toml_str = r#"
[scripts]
descriptions = { dev = "Start dev server", build = "Build production" }
aliases = { d = "dev", b = "build" }
"#;

    let config: Config = toml::from_str(toml_str).expect("Failed to parse");

    assert_eq!(
        config.scripts.descriptions.get("dev"),
        Some(&"Start dev server".to_string())
    );
    assert_eq!(config.scripts.aliases.get("d"), Some(&"dev".to_string()));
}

// ==================== Full Config ====================

#[test]
fn test_full_config_parsing() {
    let toml_str = r#"
[general]
runner = "npm"
show_command_preview = true

[appearance]
icons = true
compact = false
show_footer = true

[exclude]
patterns = ["pre*", "post*"]

[history]
enabled = true
max_projects = 100
max_scripts = 50

[scripts]
descriptions = { dev = "Development server" }
"#;

    let config: Config = toml::from_str(toml_str).expect("Failed to parse");

    // General
    assert!(config.general.runner.is_some());
    assert!(config.general.show_command_preview);

    // Appearance
    assert!(config.appearance.icons);
    assert!(!config.appearance.compact);
    assert!(config.appearance.show_footer);

    // Exclude
    assert_eq!(config.exclude.patterns.len(), 2);

    // History
    assert!(config.history.enabled);
    assert_eq!(config.history.max_projects, 100);

    // Scripts
    assert!(config.scripts.descriptions.contains_key("dev"));
}

// ==================== Config Path Helpers ====================

#[test]
fn test_user_config_path() {
    let path = Config::user_config_path();
    assert!(path.is_some());
    let path = path.unwrap();
    assert!(path.ends_with("config.toml"));
}

// ==================== Integration with CLI ====================

#[test]
fn test_config_file_in_project() {
    let config_toml = r#"
[appearance]
icons = false

[exclude]
patterns = ["lint"]
"#;

    let project = create_project_with_config(&standard_scripts(), config_toml);

    // Verify the config file exists
    let config_path = project.path().join(".nrsrc.toml");
    assert!(config_path.exists());

    // Parse it
    let content = std::fs::read_to_string(&config_path).expect("Failed to read");
    let config: Config = toml::from_str(&content).expect("Failed to parse");

    assert!(!config.appearance.icons);
    assert_eq!(config.exclude.patterns, vec!["lint"]);
}

// ==================== Edge Cases ====================

#[test]
fn test_config_with_comments() {
    let toml_str = r#"
# This is a comment
[appearance]
icons = false  # Disable icons

# Another comment
[exclude]
# Comment before patterns
patterns = ["test"]
"#;

    let config: Config = toml::from_str(toml_str).expect("Failed to parse");

    assert!(!config.appearance.icons);
    assert_eq!(config.exclude.patterns, vec!["test"]);
}

#[test]
fn test_invalid_toml() {
    let toml_str = "this is not valid TOML [[[";

    let result: Result<Config, _> = toml::from_str(toml_str);
    assert!(result.is_err());
}
