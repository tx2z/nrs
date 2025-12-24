//! Custom error types for nrs.
//!
//! Uses thiserror for ergonomic error definitions.

use std::path::PathBuf;

use thiserror::Error;

/// Exit codes for nrs.
pub mod exit_code {
    /// Success.
    pub const SUCCESS: i32 = 0;
    /// General error.
    pub const GENERAL_ERROR: i32 = 1;
    /// No package.json found.
    pub const NO_PACKAGE_JSON: i32 = 2;
    /// No scripts defined.
    pub const NO_SCRIPTS: i32 = 3;
    /// Script execution failed.
    pub const SCRIPT_FAILED: i32 = 4;
    /// Invalid configuration.
    pub const INVALID_CONFIG: i32 = 5;
    /// Interrupted (Ctrl+C).
    pub const INTERRUPTED: i32 = 130;
}

/// Main error type for nrs.
#[derive(Error, Debug)]
pub enum NrsError {
    /// No package.json found.
    #[error(
        "No package.json found in {path} or any parent directory (searched up to {depth} levels)"
    )]
    NoPackageJson { path: PathBuf, depth: usize },

    /// Failed to parse package.json with location details.
    #[error("Failed to parse package.json at {path}:\n  {message}")]
    ParseErrorWithContext {
        path: PathBuf,
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },

    /// Failed to parse package.json (legacy, kept for From impl).
    #[error("Failed to parse package.json: {0}")]
    ParseError(#[from] serde_json::Error),

    /// No scripts defined in package.json.
    #[error("No scripts defined in package.json at {path}\n\nTip: Add scripts to your package.json:\n  {{\n    \"scripts\": {{\n      \"dev\": \"your-command\",\n      \"build\": \"your-build-command\"\n    }}\n  }}")]
    NoScriptsAt { path: PathBuf },

    /// No scripts defined in package.json (legacy).
    #[error("No scripts defined in package.json")]
    NoScripts,

    /// Empty scripts object.
    #[error("The scripts object in {path} is empty\n\nTip: Add scripts to your package.json:\n  {{\n    \"scripts\": {{\n      \"dev\": \"your-command\"\n    }}\n  }}")]
    EmptyScripts { path: PathBuf },

    /// Scripts field is not an object.
    #[error("Invalid scripts field in {path}: expected an object, got {actual_type}\n\nTip: The scripts field must be an object:\n  \"scripts\": {{ \"name\": \"command\" }}")]
    InvalidScriptsType { path: PathBuf, actual_type: String },

    /// Script not found.
    #[error("Script '{name}' not found in package.json")]
    ScriptNotFound { name: String },

    /// Script not found with suggestions.
    #[error("Script '{name}' not found\n\nDid you mean: {suggestions}?\n\nRun 'nrs --list' to see all available scripts.")]
    ScriptNotFoundWithSuggestions { name: String, suggestions: String },

    /// Script execution failed.
    #[error("Script '{name}' failed with exit code {code}")]
    ScriptFailed { name: String, code: i32 },

    /// Configuration error.
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Invalid configuration file.
    #[error("Invalid config at {path}:\n  {message}\n\nTip: Check the config file syntax and ensure all values are valid.")]
    InvalidConfig { path: PathBuf, message: String },

    /// Terminal too small.
    #[error("Terminal too small (minimum: {min_width}x{min_height}, current: {width}x{height})\n\nTip: Resize your terminal window or use --list for non-interactive mode.")]
    TerminalTooSmall {
        width: u16,
        height: u16,
        min_width: u16,
        min_height: u16,
    },

    /// No history found for rerun.
    #[error("No previous script found for this project\n\nTip: Run 'nrs' first to execute a script, then use 'nrs --last' to rerun it.")]
    NoHistory,

    /// All scripts excluded by patterns.
    #[error("All {total} scripts are excluded by your exclude patterns\n\nActive exclude patterns: {patterns}\n\nTip: Review your exclude patterns in config or use --exclude flag.")]
    AllScriptsExcluded { total: usize, patterns: String },

    /// No scripts match filter.
    #[error("No scripts match the filter '{filter}'\n\nTip: Press Escape to clear the filter, or try a different search term.")]
    NoFilterMatch { filter: String },

    /// IO error with path context.
    #[error("Failed to {operation} '{path}': {source}")]
    IoWithContext {
        operation: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl NrsError {
    /// Get the exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            NrsError::NoPackageJson { .. } => exit_code::NO_PACKAGE_JSON,
            NrsError::ParseError(_) => exit_code::NO_PACKAGE_JSON,
            NrsError::ParseErrorWithContext { .. } => exit_code::NO_PACKAGE_JSON,
            NrsError::NoScripts => exit_code::NO_SCRIPTS,
            NrsError::NoScriptsAt { .. } => exit_code::NO_SCRIPTS,
            NrsError::EmptyScripts { .. } => exit_code::NO_SCRIPTS,
            NrsError::InvalidScriptsType { .. } => exit_code::NO_PACKAGE_JSON,
            NrsError::ScriptNotFound { .. } => exit_code::GENERAL_ERROR,
            NrsError::ScriptNotFoundWithSuggestions { .. } => exit_code::GENERAL_ERROR,
            NrsError::ScriptFailed { .. } => exit_code::SCRIPT_FAILED,
            NrsError::ConfigError { .. } => exit_code::INVALID_CONFIG,
            NrsError::InvalidConfig { .. } => exit_code::INVALID_CONFIG,
            NrsError::TerminalTooSmall { .. } => exit_code::GENERAL_ERROR,
            NrsError::NoHistory => exit_code::GENERAL_ERROR,
            NrsError::AllScriptsExcluded { .. } => exit_code::NO_SCRIPTS,
            NrsError::NoFilterMatch { .. } => exit_code::GENERAL_ERROR,
            NrsError::IoWithContext { .. } => exit_code::GENERAL_ERROR,
            NrsError::Io(_) => exit_code::GENERAL_ERROR,
        }
    }

    /// Create a script not found error with suggestions based on available scripts.
    pub fn script_not_found_with_suggestions(name: &str, scripts: &[&str]) -> Self {
        let suggestions = find_similar_scripts(name, scripts);
        if suggestions.is_empty() {
            NrsError::ScriptNotFound {
                name: name.to_string(),
            }
        } else {
            NrsError::ScriptNotFoundWithSuggestions {
                name: name.to_string(),
                suggestions: suggestions.join(", "),
            }
        }
    }
}

/// Find similar script names using simple string distance.
fn find_similar_scripts(name: &str, scripts: &[&str]) -> Vec<String> {
    let name_lower = name.to_lowercase();
    let mut matches: Vec<(String, usize)> = scripts
        .iter()
        .filter_map(|&s| {
            let s_lower = s.to_lowercase();
            let dist = simple_distance(&name_lower, &s_lower);
            // Include if distance is small enough or contains the search term
            if dist <= 3 || s_lower.contains(&name_lower) || name_lower.contains(&s_lower) {
                Some((s.to_string(), dist))
            } else {
                None
            }
        })
        .collect();

    // Sort by distance
    matches.sort_by_key(|(_, d)| *d);

    // Return top 3 suggestions
    matches
        .into_iter()
        .take(3)
        .map(|(s, _)| format!("'{}'", s))
        .collect()
}

/// Simple Levenshtein-like distance calculation.
fn simple_distance(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let len_a = a_chars.len();
    let len_b = b_chars.len();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    // Simple Levenshtein for short strings
    if len_a > 20 || len_b > 20 {
        // For long strings, just use length difference + common prefix check
        let common_prefix = a_chars
            .iter()
            .zip(b_chars.iter())
            .take_while(|(a, b)| a == b)
            .count();
        return len_a.abs_diff(len_b) + (len_a.min(len_b) - common_prefix);
    }

    let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len_a + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len_b + 1) {
        *cell = j;
    }

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len_a][len_b]
}

/// Result type alias for nrs operations.
pub type Result<T> = std::result::Result<T, NrsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_exit_codes() {
        let err = NrsError::NoPackageJson {
            path: PathBuf::from("."),
            depth: 10,
        };
        assert_eq!(err.exit_code(), exit_code::NO_PACKAGE_JSON);

        let err = NrsError::NoScripts;
        assert_eq!(err.exit_code(), exit_code::NO_SCRIPTS);

        let err = NrsError::ScriptFailed {
            name: "test".to_string(),
            code: 1,
        };
        assert_eq!(err.exit_code(), exit_code::SCRIPT_FAILED);

        let err = NrsError::NoScriptsAt {
            path: PathBuf::from("/test"),
        };
        assert_eq!(err.exit_code(), exit_code::NO_SCRIPTS);

        let err = NrsError::AllScriptsExcluded {
            total: 5,
            patterns: "pre*, post*".to_string(),
        };
        assert_eq!(err.exit_code(), exit_code::NO_SCRIPTS);
    }

    #[test]
    fn test_error_messages() {
        let err = NrsError::ScriptNotFound {
            name: "dev".to_string(),
        };
        assert!(err.to_string().contains("Script 'dev' not found"));

        let err = NrsError::NoScripts;
        assert_eq!(err.to_string(), "No scripts defined in package.json");

        let err = NrsError::NoHistory;
        assert!(err.to_string().contains("No previous script found"));
        assert!(err.to_string().contains("Tip:")); // Should have helpful tip
    }

    #[test]
    fn test_script_not_found_with_suggestions() {
        let scripts = vec!["dev", "build", "test", "lint", "format"];

        // Test with close match
        let err = NrsError::script_not_found_with_suggestions("devv", &scripts);
        let msg = err.to_string();
        assert!(msg.contains("'dev'"), "Should suggest 'dev' for 'devv'");

        // Test with no close match
        let err = NrsError::script_not_found_with_suggestions("xyz123", &scripts);
        let msg = err.to_string();
        // Should be simple not found without suggestions
        assert!(msg.contains("xyz123"));
    }

    #[test]
    fn test_simple_distance() {
        assert_eq!(simple_distance("", ""), 0);
        assert_eq!(simple_distance("abc", "abc"), 0);
        assert_eq!(simple_distance("abc", ""), 3);
        assert_eq!(simple_distance("", "abc"), 3);
        assert_eq!(simple_distance("abc", "abd"), 1);
        assert_eq!(simple_distance("dev", "devv"), 1);
        assert_eq!(simple_distance("build", "biuld"), 2);
    }

    #[test]
    fn test_find_similar_scripts() {
        let scripts = vec!["dev", "build", "test", "lint", "format"];

        let similar = find_similar_scripts("dev", &scripts);
        assert!(similar.iter().any(|s| s.contains("dev")));

        let similar = find_similar_scripts("buid", &scripts);
        assert!(similar.iter().any(|s| s.contains("build")));

        // Substring match
        let similar = find_similar_scripts("tes", &scripts);
        assert!(similar.iter().any(|s| s.contains("test")));
    }

    #[test]
    fn test_error_with_path_context() {
        let err = NrsError::NoPackageJson {
            path: PathBuf::from("/home/user/project"),
            depth: 10,
        };
        let msg = err.to_string();
        assert!(msg.contains("/home/user/project"));
        assert!(msg.contains("10"));
    }
}
