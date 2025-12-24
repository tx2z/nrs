//! Script parsing from package.json.

use std::path::Path;

use anyhow::{bail, Context, Result};

use super::descriptions::extract_descriptions;
use super::types::{Package, Script, Scripts};

/// Parse a package.json file from a directory.
///
/// # Arguments
///
/// * `project_dir` - The directory containing package.json
///
/// # Errors
///
/// Returns an error if:
/// - The package.json file cannot be read
/// - The JSON is malformed
pub fn parse_scripts(project_dir: &Path) -> Result<Scripts> {
    let package_json = project_dir.join("package.json");
    let content = std::fs::read_to_string(&package_json)
        .with_context(|| format!("Failed to read {}", package_json.display()))?;

    parse_scripts_from_json(&content)
}

/// Parse the full package.json structure.
///
/// # Arguments
///
/// * `content` - The raw JSON content
///
/// # Errors
///
/// Returns an error if the JSON is malformed.
pub fn parse_package_json(content: &str) -> Result<Package> {
    // First, try to parse as valid JSON
    let json: serde_json::Value = serde_json::from_str(content).map_err(|e| {
        let msg = format_json_error(content, &e);
        anyhow::anyhow!("Failed to parse package.json: {msg}")
    })?;

    // Then deserialize into our Package struct
    let package: Package =
        serde_json::from_value(json).context("Failed to parse package.json structure")?;

    Ok(package)
}

/// Parse scripts from package.json content.
///
/// # Arguments
///
/// * `content` - The raw JSON content
///
/// # Errors
///
/// Returns an error if the JSON is malformed.
///
/// # Examples
///
/// ```
/// use nrs::package::scripts::parse_scripts_from_json;
///
/// let json = r#"{"scripts": {"dev": "vite", "build": "vite build"}}"#;
/// let scripts = parse_scripts_from_json(json).unwrap();
/// assert_eq!(scripts.len(), 2);
/// ```
pub fn parse_scripts_from_json(content: &str) -> Result<Scripts> {
    let package = parse_package_json(content)?;
    extract_scripts_from_package(&package)
}

/// Parse scripts from package.json content, returning error if no scripts exist.
///
/// # Errors
///
/// Returns an error if:
/// - The JSON is malformed
/// - No scripts are defined in package.json
pub fn parse_scripts_required(content: &str) -> Result<Scripts> {
    let scripts = parse_scripts_from_json(content)?;

    if scripts.is_empty() {
        bail!("No scripts defined in package.json");
    }

    Ok(scripts)
}

/// Extract scripts from a parsed Package.
pub fn extract_scripts_from_package(package: &Package) -> Result<Scripts> {
    // Extract descriptions from various sources
    let descriptions = extract_descriptions(package);

    let mut scripts = Scripts::new();

    for (name, command) in &package.scripts {
        // Skip comment entries (keys starting with //)
        if name.starts_with("//") {
            continue;
        }

        let mut script = Script::new(name, command);

        // Add description if available
        if let Some(desc) = descriptions.get(name) {
            script.set_description(desc);
        }

        scripts.add(script);
    }

    // Sort alphabetically for consistent ordering
    scripts.sort_alphabetically();

    Ok(scripts)
}

/// Format a JSON parsing error with context.
fn format_json_error(content: &str, error: &serde_json::Error) -> String {
    let line = error.line();
    let column = error.column();

    // Try to show the problematic line
    if let Some(error_line) = content.lines().nth(line.saturating_sub(1)) {
        let pointer = " ".repeat(column.saturating_sub(1)) + "^";
        format!(
            "{}\n  at line {}, column {}:\n    {}\n    {}",
            error, line, column, error_line, pointer
        )
    } else {
        format!("{} at line {}, column {}", error, line, column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_scripts() {
        let json = r#"{
            "name": "test-project",
            "scripts": {
                "dev": "vite",
                "build": "vite build"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(scripts.len(), 2);
        assert!(scripts.get("dev").is_some());
        assert_eq!(scripts.get("dev").unwrap().command(), "vite");
    }

    #[test]
    fn test_parse_empty_scripts() {
        let json = r#"{
            "name": "test-project",
            "scripts": {}
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert!(scripts.is_empty());
    }

    #[test]
    fn test_parse_no_scripts_field() {
        let json = r#"{
            "name": "test-project"
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert!(scripts.is_empty());
    }

    #[test]
    fn test_parse_scripts_required_fails_when_empty() {
        let json = r#"{
            "name": "test-project",
            "scripts": {}
        }"#;

        let result = parse_scripts_required(json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No scripts defined"));
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"{ invalid json }"#;

        let result = parse_scripts_from_json(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to parse"));
    }

    #[test]
    fn test_parse_skips_comment_entries() {
        let json = r#"{
            "scripts": {
                "//dev": "This is a comment",
                "dev": "vite",
                "// build": "Another comment",
                "build": "vite build"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(scripts.len(), 2);
        assert!(scripts.get("dev").is_some());
        assert!(scripts.get("build").is_some());
        assert!(scripts.get("//dev").is_none());
    }

    #[test]
    fn test_parse_with_scripts_info_descriptions() {
        let json = r#"{
            "scripts": {
                "dev": "vite",
                "build": "vite build"
            },
            "scripts-info": {
                "dev": "Start development server",
                "build": "Build for production"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(
            scripts.get("dev").unwrap().description(),
            Some("Start development server")
        );
        assert_eq!(
            scripts.get("build").unwrap().description(),
            Some("Build for production")
        );
    }

    #[test]
    fn test_parse_with_ntl_descriptions() {
        let json = r#"{
            "scripts": {
                "test": "vitest"
            },
            "ntl": {
                "descriptions": {
                    "test": "Run tests with vitest"
                }
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(
            scripts.get("test").unwrap().description(),
            Some("Run tests with vitest")
        );
    }

    #[test]
    fn test_parse_with_comment_descriptions() {
        let json = r#"{
            "scripts": {
                "//lint": "Run ESLint",
                "lint": "eslint ."
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(
            scripts.get("lint").unwrap().description(),
            Some("Run ESLint")
        );
    }

    #[test]
    fn test_parse_scripts_sorted_alphabetically() {
        let json = r#"{
            "scripts": {
                "zebra": "echo z",
                "alpha": "echo a",
                "middle": "echo m"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        let names: Vec<_> = scripts.iter().map(|s| s.name()).collect();
        assert_eq!(names, vec!["alpha", "middle", "zebra"]);
    }

    #[test]
    fn test_parse_package_json_full() {
        let json = r#"{
            "name": "my-app",
            "version": "1.0.0",
            "description": "A test application",
            "packageManager": "pnpm@8.0.0",
            "scripts": {
                "dev": "vite"
            }
        }"#;

        let package = parse_package_json(json).unwrap();
        assert_eq!(package.name, "my-app");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.description, Some("A test application".to_string()));
        assert_eq!(package.package_manager, Some("pnpm@8.0.0".to_string()));
        assert!(package.has_scripts());
    }

    #[test]
    fn test_parse_special_characters_in_script_names() {
        let json = r#"{
            "scripts": {
                "build:prod": "vite build --mode production",
                "test:unit": "vitest",
                "lint:fix": "eslint --fix ."
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(scripts.len(), 3);
        assert!(scripts.get("build:prod").is_some());
        assert!(scripts.get("test:unit").is_some());
        assert!(scripts.get("lint:fix").is_some());
    }

    #[test]
    fn test_lifecycle_scripts_filtered() {
        let json = r#"{
            "scripts": {
                "dev": "vite",
                "preinstall": "echo preinstall",
                "postinstall": "husky install",
                "build": "vite build"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(scripts.len(), 4);

        let filtered = scripts.without_lifecycle();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.get("dev").is_some());
        assert!(filtered.get("build").is_some());
        assert!(filtered.get("preinstall").is_none());
        assert!(filtered.get("postinstall").is_none());
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_parse_unicode_script_names() {
        let json = r#"{
            "scripts": {
                "开发": "vite",
                "ビルド": "vite build",
                "тест": "vitest",
                "développement": "vite dev"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(scripts.len(), 4);
        assert!(scripts.get("开发").is_some());
        assert!(scripts.get("ビルド").is_some());
        assert!(scripts.get("тест").is_some());
        assert!(scripts.get("développement").is_some());
    }

    #[test]
    fn test_parse_emoji_script_names() {
        let json = r#"{
            "scripts": {
                "🚀": "npm start",
                "🔧:fix": "eslint --fix .",
                "test:🎉": "vitest"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(scripts.len(), 3);
        assert!(scripts.get("🚀").is_some());
        assert!(scripts.get("🔧:fix").is_some());
        assert!(scripts.get("test:🎉").is_some());
    }

    #[test]
    fn test_parse_empty_command() {
        let json = r#"{
            "scripts": {
                "empty": "",
                "normal": "echo hello"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(scripts.len(), 2);
        assert_eq!(scripts.get("empty").unwrap().command(), "");
        assert_eq!(scripts.get("normal").unwrap().command(), "echo hello");
    }

    #[test]
    fn test_parse_very_long_script_name() {
        let long_name = "a".repeat(200);
        let json = format!(
            r#"{{
            "scripts": {{
                "{}": "echo test"
            }}
        }}"#,
            long_name
        );

        let scripts = parse_scripts_from_json(&json).unwrap();
        assert_eq!(scripts.len(), 1);
        assert!(scripts.get(&long_name).is_some());
    }

    #[test]
    fn test_parse_very_long_command() {
        let long_command = "echo ".to_string() + &"x".repeat(10000);
        let json = format!(
            r#"{{
            "scripts": {{
                "test": "{}"
            }}
        }}"#,
            long_command
        );

        let scripts = parse_scripts_from_json(&json).unwrap();
        assert_eq!(scripts.get("test").unwrap().command(), long_command);
    }

    #[test]
    fn test_parse_many_scripts() {
        // Generate a package.json with 1000 scripts
        let mut scripts_obj = String::from("{");
        for i in 0..1000 {
            if i > 0 {
                scripts_obj.push(',');
            }
            scripts_obj.push_str(&format!(r#""script_{}": "echo {}""#, i, i));
        }
        scripts_obj.push('}');

        let json = format!(r#"{{"scripts": {}}}"#, scripts_obj);

        let scripts = parse_scripts_from_json(&json).unwrap();
        assert_eq!(scripts.len(), 1000);
        assert!(scripts.get("script_0").is_some());
        assert!(scripts.get("script_999").is_some());
    }

    #[test]
    fn test_parse_special_characters_in_command() {
        let json = r#"{
            "scripts": {
                "test": "echo \"hello world\" && echo 'single quotes'",
                "env": "FOO=bar BAZ=\"quoted value\" npm start",
                "redirect": "npm build > output.log 2>&1",
                "pipe": "cat file.txt | grep pattern | wc -l",
                "subshell": "$(npm bin)/eslint .",
                "semicolon": "echo first; echo second",
                "escape": "echo \\\"escaped\\\"",
                "dollar": "echo $HOME $USER ${PWD}"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(scripts.len(), 8);
        assert!(scripts.get("test").is_some());
        assert!(scripts.get("env").is_some());
        assert!(scripts.get("redirect").is_some());
        assert!(scripts.get("pipe").is_some());
        assert!(scripts.get("subshell").is_some());
        assert!(scripts.get("semicolon").is_some());
        assert!(scripts.get("escape").is_some());
        assert!(scripts.get("dollar").is_some());
    }

    #[test]
    fn test_parse_multiline_command() {
        // package.json doesn't support actual multiline strings, but escaped newlines
        let json = r#"{
            "scripts": {
                "complex": "echo start && npm test && npm build && echo done"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert!(scripts.get("complex").is_some());
    }

    #[test]
    fn test_parse_json_with_trailing_comma() {
        // Standard JSON doesn't allow trailing commas, verify we reject it
        let json = r#"{
            "scripts": {
                "dev": "vite",
            }
        }"#;

        let result = parse_scripts_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_with_comments() {
        // Standard JSON doesn't allow comments, verify we reject them
        let json = r#"{
            // This is a comment
            "scripts": {
                "dev": "vite"
            }
        }"#;

        let result = parse_scripts_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_minimal_valid_json() {
        let json = r#"{}"#;
        let scripts = parse_scripts_from_json(json).unwrap();
        assert!(scripts.is_empty());
    }

    #[test]
    fn test_parse_scripts_field_as_array() {
        // scripts should be an object, not an array
        let json = r#"{
            "scripts": ["dev", "build"]
        }"#;

        let result = parse_scripts_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_scripts_field_as_string() {
        // scripts should be an object, not a string
        let json = r#"{
            "scripts": "dev"
        }"#;

        let result = parse_scripts_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_scripts_field_as_null() {
        let json = r#"{
            "scripts": null
        }"#;

        let result = parse_scripts_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_script_value_as_number() {
        // Script values should be strings, not numbers
        let json = r#"{
            "scripts": {
                "test": 123
            }
        }"#;

        let result = parse_scripts_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_script_value_as_object() {
        // Script values should be strings, not objects
        let json = r#"{
            "scripts": {
                "test": {"command": "vitest"}
            }
        }"#;

        let result = parse_scripts_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_json_error_shows_context() {
        let json = r#"{
    "scripts": {
        "dev": vite
    }
}"#;

        let result = parse_scripts_from_json(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        // Should show line and column information
        assert!(err.contains("line"));
        assert!(err.contains("column"));
    }

    #[test]
    fn test_parse_whitespace_in_script_names() {
        // While unusual, whitespace in keys is valid JSON
        let json = r#"{
            "scripts": {
                " dev ": "vite",
                "build test": "vite build"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(scripts.len(), 2);
        assert!(scripts.get(" dev ").is_some());
        assert!(scripts.get("build test").is_some());
    }

    #[test]
    fn test_parse_hyphen_and_underscore_names() {
        let json = r#"{
            "scripts": {
                "my-script": "echo hyphen",
                "my_script": "echo underscore",
                "my-long-script-name": "echo long",
                "__internal__": "echo internal"
            }
        }"#;

        let scripts = parse_scripts_from_json(json).unwrap();
        assert_eq!(scripts.len(), 4);
        assert!(scripts.get("my-script").is_some());
        assert!(scripts.get("my_script").is_some());
        assert!(scripts.get("my-long-script-name").is_some());
        assert!(scripts.get("__internal__").is_some());
    }
}
