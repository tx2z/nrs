//! Configuration type definitions.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::package::Runner;

/// Sort mode for script display.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortMode {
    /// Sort by most recently used.
    #[default]
    Recent,
    /// Sort alphabetically by name.
    Alpha,
    /// Group by category/prefix.
    Category,
}

/// Column direction for grid layout.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnDirection {
    /// Fill rows first (1 2 3 4 / 5 6 7 8).
    #[default]
    Horizontal,
    /// Fill columns first (1 4 7 / 2 5 8 / 3 6 9).
    Vertical,
}

/// Color theme for the TUI.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Full color theme.
    #[default]
    Default,
    /// Minimal colors.
    Minimal,
    /// No colors (monochrome).
    None,
}

/// General configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Override package manager detection.
    #[serde(default)]
    pub runner: Option<Runner>,
    /// Default sort mode.
    #[serde(default)]
    pub default_sort: SortMode,
    /// Column direction in grid.
    #[serde(default)]
    pub column_direction: ColumnDirection,
    /// Show command preview in description panel.
    #[serde(default = "default_true")]
    pub show_command_preview: bool,
    /// Maximum items to show (0 = unlimited).
    #[serde(default)]
    pub max_items: usize,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            runner: None,
            default_sort: SortMode::default(),
            column_direction: ColumnDirection::default(),
            show_command_preview: true,
            max_items: 0,
        }
    }
}

/// Filter configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Search in descriptions too.
    #[serde(default = "default_true")]
    pub search_descriptions: bool,
    /// Enable fuzzy matching.
    #[serde(default = "default_true")]
    pub fuzzy: bool,
    /// Case sensitive search.
    #[serde(default)]
    pub case_sensitive: bool,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            search_descriptions: true,
            fuzzy: true,
            case_sensitive: false,
        }
    }
}

/// History configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Enable history tracking.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Max projects to remember.
    #[serde(default = "default_max_projects")]
    pub max_projects: usize,
    /// Max scripts per project.
    #[serde(default = "default_max_scripts")]
    pub max_scripts: usize,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_projects: 100,
            max_scripts: 50,
        }
    }
}

/// Exclude patterns configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExcludeConfig {
    /// Glob patterns to exclude.
    #[serde(default)]
    pub patterns: Vec<String>,
}

/// Appearance configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// Color theme.
    #[serde(default)]
    pub theme: Theme,
    /// Show icons.
    #[serde(default = "default_true")]
    pub icons: bool,
    /// Show help footer.
    #[serde(default = "default_true")]
    pub show_footer: bool,
    /// Compact mode (less padding).
    #[serde(default)]
    pub compact: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            icons: true,
            show_footer: true,
            compact: false,
        }
    }
}

/// Keybindings configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    /// Quit keys (e.g., ["q", "Ctrl+c"]).
    #[serde(default)]
    pub quit: Vec<String>,
    /// Run script keys.
    #[serde(default)]
    pub run: Vec<String>,
    /// Enter filter mode keys.
    #[serde(default)]
    pub filter: Vec<String>,
}

/// Scripts configuration for custom descriptions and aliases.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScriptsConfig {
    /// Custom descriptions for scripts (overrides package.json).
    #[serde(default)]
    pub descriptions: HashMap<String, String>,
    /// Script aliases (alias -> script name).
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

/// Main configuration structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// General settings.
    #[serde(default)]
    pub general: GeneralConfig,
    /// Filter settings.
    #[serde(default)]
    pub filter: FilterConfig,
    /// History settings.
    #[serde(default)]
    pub history: HistoryConfig,
    /// Exclude patterns.
    #[serde(default)]
    pub exclude: ExcludeConfig,
    /// Appearance settings.
    #[serde(default)]
    pub appearance: AppearanceConfig,
    /// Keybindings settings.
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
    /// Scripts configuration.
    #[serde(default)]
    pub scripts: ScriptsConfig,
}

impl Config {
    /// Create a new configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the config file path for the user's home directory.
    pub fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("nrs").join("config.toml"))
    }

    /// Merge another config into this one (other takes precedence for set values).
    pub fn merge(&mut self, other: Config) {
        // General settings
        if other.general.runner.is_some() {
            self.general.runner = other.general.runner;
        }
        self.general.default_sort = other.general.default_sort;
        self.general.column_direction = other.general.column_direction;
        self.general.show_command_preview = other.general.show_command_preview;
        if other.general.max_items > 0 {
            self.general.max_items = other.general.max_items;
        }

        // Filter settings
        self.filter = other.filter;

        // History settings
        self.history = other.history;

        // Exclude patterns - append rather than replace
        self.exclude.patterns.extend(other.exclude.patterns);

        // Appearance settings
        self.appearance = other.appearance;

        // Keybindings - only override if not empty
        if !other.keybindings.quit.is_empty() {
            self.keybindings.quit = other.keybindings.quit;
        }
        if !other.keybindings.run.is_empty() {
            self.keybindings.run = other.keybindings.run;
        }
        if !other.keybindings.filter.is_empty() {
            self.keybindings.filter = other.keybindings.filter;
        }

        // Scripts - merge hashmaps
        self.scripts.descriptions.extend(other.scripts.descriptions);
        self.scripts.aliases.extend(other.scripts.aliases);
    }
}

fn default_true() -> bool {
    true
}

fn default_max_projects() -> usize {
    100
}

fn default_max_scripts() -> usize {
    50
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.filter.fuzzy);
        assert!(config.filter.search_descriptions);
        assert!(!config.filter.case_sensitive);
        assert!(config.history.enabled);
        assert_eq!(config.history.max_projects, 100);
        assert_eq!(config.history.max_scripts, 50);
        assert!(config.appearance.icons);
        assert!(config.appearance.show_footer);
        assert!(!config.appearance.compact);
        assert_eq!(config.appearance.theme, Theme::Default);
    }

    #[test]
    fn test_sort_mode_serialization() {
        let json = serde_json::to_string(&SortMode::Alpha).unwrap();
        assert_eq!(json, "\"alpha\"");

        let mode: SortMode = serde_json::from_str("\"category\"").unwrap();
        assert_eq!(mode, SortMode::Category);
    }

    #[test]
    fn test_column_direction_serialization() {
        let json = serde_json::to_string(&ColumnDirection::Vertical).unwrap();
        assert_eq!(json, "\"vertical\"");

        let dir: ColumnDirection = serde_json::from_str("\"horizontal\"").unwrap();
        assert_eq!(dir, ColumnDirection::Horizontal);
    }

    #[test]
    fn test_theme_serialization() {
        let json = serde_json::to_string(&Theme::Minimal).unwrap();
        assert_eq!(json, "\"minimal\"");

        let theme: Theme = serde_json::from_str("\"none\"").unwrap();
        assert_eq!(theme, Theme::None);
    }

    #[test]
    fn test_config_merge() {
        let mut base = Config::default();
        base.exclude.patterns.push("test*".to_string());

        let mut override_config = Config::default();
        override_config.general.runner = Some(Runner::Pnpm);
        override_config.exclude.patterns.push("lint*".to_string());
        override_config
            .scripts
            .descriptions
            .insert("dev".to_string(), "Start dev server".to_string());

        base.merge(override_config);

        assert_eq!(base.general.runner, Some(Runner::Pnpm));
        assert_eq!(base.exclude.patterns.len(), 2);
        assert!(base.exclude.patterns.contains(&"test*".to_string()));
        assert!(base.exclude.patterns.contains(&"lint*".to_string()));
        assert_eq!(
            base.scripts.descriptions.get("dev"),
            Some(&"Start dev server".to_string())
        );
    }
}
