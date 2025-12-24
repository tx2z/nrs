//! Configuration module for nrs.
//!
//! Handles loading and merging configuration from multiple sources:
//! - CLI arguments (highest priority)
//! - Project-level `.nrsrc.toml`
//! - User-level `~/.config/nrs/config.toml`

pub mod file;
mod types;

pub use file::{generate_example_config, load_config};
pub use types::{
    AppearanceConfig, ColumnDirection, Config, ExcludeConfig, FilterConfig, GeneralConfig,
    HistoryConfig, KeybindingsConfig, ScriptsConfig, SortMode, Theme,
};
