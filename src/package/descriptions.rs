//! Script description extraction from various sources in package.json.
//!
//! Descriptions can come from multiple sources, in priority order:
//! 1. `scripts-info` object (highest priority)
//! 2. `ntl.descriptions` object
//! 3. `// comment` prefixes in scripts object
//!
//! If no description is found, the command itself is used as a fallback.

use std::collections::HashMap;

use super::types::{Package, Script};

/// Extract descriptions from all available sources in a Package.
///
/// Sources are checked in priority order:
/// 1. `scripts-info` - Direct descriptions object
/// 2. `ntl.descriptions` - NTL tool format
/// 3. `// comments` - Comment keys in scripts object
///
/// # Examples
///
/// ```ignore
/// let package = parse_package_json(content)?;
/// let descriptions = extract_descriptions(&package);
/// ```
pub fn extract_descriptions(package: &Package) -> HashMap<String, String> {
    let mut descriptions = HashMap::new();

    // Priority 1: scripts-info (highest priority)
    for (name, desc) in &package.scripts_info {
        descriptions.insert(name.clone(), desc.clone());
    }

    // Priority 2: ntl.descriptions
    if let Some(ntl) = &package.ntl {
        for (name, desc) in &ntl.descriptions {
            descriptions
                .entry(name.clone())
                .or_insert_with(|| desc.clone());
        }
    }

    // Priority 3: // comments in scripts
    // Look for keys like "//scriptName", "// scriptName", "//scriptName//"
    for (key, value) in &package.scripts {
        if let Some(script_name) = parse_comment_key(key) {
            descriptions
                .entry(script_name)
                .or_insert_with(|| value.clone());
        }
    }

    descriptions
}

/// Parse a comment key to extract the script name.
///
/// Handles various comment formats:
/// - `//scriptName` -> `scriptName`
/// - `// scriptName` -> `scriptName`
/// - `//scriptName//` -> `scriptName`
/// - `// scriptName //` -> `scriptName`
fn parse_comment_key(key: &str) -> Option<String> {
    if !key.starts_with("//") {
        return None;
    }

    // Remove leading //
    let stripped = key.strip_prefix("//")?;

    // Remove trailing // if present
    let stripped = stripped.strip_suffix("//").unwrap_or(stripped);

    // Trim whitespace
    let script_name = stripped.trim();

    if script_name.is_empty() {
        return None;
    }

    Some(script_name.to_string())
}

/// Get the display description for a script.
///
/// Returns the script's description if set, otherwise falls back to
/// displaying the command with a `$` prefix.
///
/// # Examples
///
/// ```
/// use nrs::package::{Script, get_description};
///
/// let script = Script::with_description("dev", "vite", "Start dev server");
/// assert_eq!(get_description(&script), "Start dev server");
///
/// let script_no_desc = Script::new("build", "vite build");
/// assert_eq!(get_description(&script_no_desc), "$ vite build");
/// ```
pub fn get_description(script: &Script) -> String {
    script
        .description()
        .map(|d| d.to_string())
        .unwrap_or_else(|| format!("$ {}", script.command()))
}

/// Get a short description, truncated if necessary.
///
/// # Arguments
///
/// * `script` - The script to get description for
/// * `max_len` - Maximum length before truncation
pub fn get_short_description(script: &Script, max_len: usize) -> String {
    let desc = get_description(script);
    if desc.len() <= max_len {
        desc
    } else {
        format!("{}...", &desc[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_description_with_desc() {
        let script = Script::with_description("dev", "vite", "Start dev server");
        assert_eq!(get_description(&script), "Start dev server");
    }

    #[test]
    fn test_get_description_fallback() {
        let script = Script::new("dev", "vite");
        assert_eq!(get_description(&script), "$ vite");
    }

    #[test]
    fn test_get_short_description() {
        let script = Script::with_description(
            "dev",
            "vite",
            "This is a very long description that should be truncated",
        );
        let short = get_short_description(&script, 20);
        assert_eq!(short, "This is a very lo...");
        assert!(short.len() <= 20);
    }

    #[test]
    fn test_get_short_description_no_truncation() {
        let script = Script::with_description("dev", "vite", "Short desc");
        let short = get_short_description(&script, 20);
        assert_eq!(short, "Short desc");
    }

    #[test]
    fn test_parse_comment_key_standard() {
        assert_eq!(parse_comment_key("//dev"), Some("dev".to_string()));
        assert_eq!(parse_comment_key("//build"), Some("build".to_string()));
    }

    #[test]
    fn test_parse_comment_key_with_space() {
        assert_eq!(parse_comment_key("// dev"), Some("dev".to_string()));
        assert_eq!(parse_comment_key("//  build"), Some("build".to_string()));
    }

    #[test]
    fn test_parse_comment_key_with_trailing_slashes() {
        assert_eq!(parse_comment_key("//dev//"), Some("dev".to_string()));
        assert_eq!(parse_comment_key("// build //"), Some("build".to_string()));
    }

    #[test]
    fn test_parse_comment_key_non_comment() {
        assert_eq!(parse_comment_key("dev"), None);
        assert_eq!(parse_comment_key("/dev"), None);
    }

    #[test]
    fn test_parse_comment_key_empty() {
        assert_eq!(parse_comment_key("//"), None);
        assert_eq!(parse_comment_key("//  "), None);
        assert_eq!(parse_comment_key("////"), None);
    }

    #[test]
    fn test_extract_descriptions_scripts_info() {
        let package = Package {
            scripts_info: [("dev".to_string(), "Start development".to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let descriptions = extract_descriptions(&package);
        assert_eq!(
            descriptions.get("dev"),
            Some(&"Start development".to_string())
        );
    }

    #[test]
    fn test_extract_descriptions_ntl() {
        use super::super::types::NtlConfig;

        let package = Package {
            ntl: Some(NtlConfig {
                descriptions: [("test".to_string(), "Run tests".to_string())]
                    .into_iter()
                    .collect(),
            }),
            ..Default::default()
        };

        let descriptions = extract_descriptions(&package);
        assert_eq!(descriptions.get("test"), Some(&"Run tests".to_string()));
    }

    #[test]
    fn test_extract_descriptions_comments() {
        let package = Package {
            scripts: [
                ("//lint".to_string(), "Run ESLint".to_string()),
                ("lint".to_string(), "eslint .".to_string()),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let descriptions = extract_descriptions(&package);
        assert_eq!(descriptions.get("lint"), Some(&"Run ESLint".to_string()));
    }

    #[test]
    fn test_extract_descriptions_priority() {
        use super::super::types::NtlConfig;

        // scripts-info should take priority over ntl and comments
        let package = Package {
            scripts: [
                ("//dev".to_string(), "Comment description".to_string()),
                ("dev".to_string(), "vite".to_string()),
            ]
            .into_iter()
            .collect(),
            scripts_info: [("dev".to_string(), "Scripts-info description".to_string())]
                .into_iter()
                .collect(),
            ntl: Some(NtlConfig {
                descriptions: [("dev".to_string(), "NTL description".to_string())]
                    .into_iter()
                    .collect(),
            }),
            ..Default::default()
        };

        let descriptions = extract_descriptions(&package);
        assert_eq!(
            descriptions.get("dev"),
            Some(&"Scripts-info description".to_string())
        );
    }

    #[test]
    fn test_extract_descriptions_ntl_over_comments() {
        use super::super::types::NtlConfig;

        // ntl should take priority over comments
        let package = Package {
            scripts: [
                ("//build".to_string(), "Comment description".to_string()),
                ("build".to_string(), "vite build".to_string()),
            ]
            .into_iter()
            .collect(),
            ntl: Some(NtlConfig {
                descriptions: [("build".to_string(), "NTL description".to_string())]
                    .into_iter()
                    .collect(),
            }),
            ..Default::default()
        };

        let descriptions = extract_descriptions(&package);
        assert_eq!(
            descriptions.get("build"),
            Some(&"NTL description".to_string())
        );
    }
}
