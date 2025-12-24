//! Integration tests for package.json parsing using fixtures.

use npm_run_scripts::package::{
    get_description, parse_package_json, parse_scripts_from_json, parse_scripts_required,
};

/// Load a fixture file.
fn load_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{name}");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to load fixture {path}: {e}"))
}

#[test]
fn test_basic_package() {
    let content = load_fixture("basic.json");
    let scripts = parse_scripts_from_json(&content).unwrap();

    assert_eq!(scripts.len(), 4);
    assert!(scripts.get("dev").is_some());
    assert!(scripts.get("build").is_some());
    assert!(scripts.get("test").is_some());
    assert!(scripts.get("lint").is_some());

    assert_eq!(scripts.get("dev").unwrap().command(), "vite");
}

#[test]
fn test_scripts_info_descriptions() {
    let content = load_fixture("with-scripts-info.json");
    let scripts = parse_scripts_from_json(&content).unwrap();

    let dev = scripts.get("dev").unwrap();
    assert_eq!(
        dev.description(),
        Some("Start development server with hot reload")
    );

    let build = scripts.get("build").unwrap();
    assert_eq!(
        build.description(),
        Some("Build optimized production bundle")
    );

    // All scripts should have descriptions
    for script in scripts.iter() {
        assert!(
            script.description().is_some(),
            "Script {} should have description",
            script.name()
        );
    }
}

#[test]
fn test_ntl_descriptions() {
    let content = load_fixture("with-ntl.json");
    let scripts = parse_scripts_from_json(&content).unwrap();

    assert_eq!(
        scripts.get("start").unwrap().description(),
        Some("Start the production server")
    );
    assert_eq!(
        scripts.get("dev").unwrap().description(),
        Some("Start development server with auto-reload")
    );
}

#[test]
fn test_comment_descriptions() {
    let content = load_fixture("with-comments.json");
    let scripts = parse_scripts_from_json(&content).unwrap();

    // Should have 4 actual scripts (not comment entries)
    assert_eq!(scripts.len(), 4);

    // Check descriptions extracted from comments
    assert_eq!(
        scripts.get("dev").unwrap().description(),
        Some("Start the development server")
    );
    assert_eq!(
        scripts.get("build").unwrap().description(),
        Some("Build for production")
    );
    assert_eq!(
        scripts.get("start").unwrap().description(),
        Some("Start production server")
    );

    // lint doesn't have a comment, so no description
    assert_eq!(scripts.get("lint").unwrap().description(), None);
}

#[test]
fn test_lifecycle_scripts_filtering() {
    let content = load_fixture("with-lifecycle.json");
    let scripts = parse_scripts_from_json(&content).unwrap();

    // Should have all 10 scripts
    assert_eq!(scripts.len(), 10);

    // After filtering, should have only 3 user scripts
    let filtered = scripts.without_lifecycle();
    assert_eq!(filtered.len(), 3);

    assert!(filtered.get("dev").is_some());
    assert!(filtered.get("build").is_some());
    assert!(filtered.get("test").is_some());

    // Lifecycle scripts should be gone
    assert!(filtered.get("preinstall").is_none());
    assert!(filtered.get("postinstall").is_none());
    assert!(filtered.get("prepare").is_none());
}

#[test]
fn test_empty_scripts() {
    let content = load_fixture("empty-scripts.json");
    let scripts = parse_scripts_from_json(&content).unwrap();

    assert!(scripts.is_empty());
    assert_eq!(scripts.len(), 0);
}

#[test]
fn test_empty_scripts_required_fails() {
    let content = load_fixture("empty-scripts.json");
    let result = parse_scripts_required(&content);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No scripts defined"));
}

#[test]
fn test_no_scripts_field() {
    let content = load_fixture("no-scripts.json");
    let scripts = parse_scripts_from_json(&content).unwrap();

    assert!(scripts.is_empty());
}

#[test]
fn test_special_characters_in_script_names() {
    let content = load_fixture("special-characters.json");
    let scripts = parse_scripts_from_json(&content).unwrap();

    assert_eq!(scripts.len(), 10);

    // Verify colons in script names
    assert!(scripts.get("build:dev").is_some());
    assert!(scripts.get("build:prod").is_some());
    assert!(scripts.get("test:unit").is_some());
    assert!(scripts.get("test:integration").is_some());
    assert!(scripts.get("test:e2e").is_some());
    assert!(scripts.get("db:migrate").is_some());
    assert!(scripts.get("docker:build").is_some());
}

#[test]
fn test_monorepo_package() {
    let content = load_fixture("monorepo.json");
    let package = parse_package_json(&content).unwrap();

    assert_eq!(package.name, "monorepo-project");
    assert!(package.is_monorepo());
    assert_eq!(package.package_manager_name(), Some("pnpm"));

    let workspaces = package.workspaces.as_ref().unwrap();
    let patterns = workspaces.patterns();
    assert_eq!(patterns.len(), 2);
    assert!(patterns.contains(&"packages/*".to_string()));
    assert!(patterns.contains(&"apps/*".to_string()));
}

#[test]
fn test_complex_package_priority() {
    let content = load_fixture("complex.json");
    let scripts = parse_scripts_from_json(&content).unwrap();

    // scripts-info takes priority
    assert_eq!(
        scripts.get("dev").unwrap().description(),
        Some("Start all development servers concurrently")
    );

    // ntl.descriptions for lint
    assert_eq!(
        scripts.get("lint").unwrap().description(),
        Some("Check code quality with ESLint")
    );

    // Comment descriptions for test (since not in scripts-info or ntl)
    assert_eq!(
        scripts.get("test").unwrap().description(),
        Some("Run the full test suite")
    );
}

#[test]
fn test_get_description_fallback() {
    let content = load_fixture("basic.json");
    let scripts = parse_scripts_from_json(&content).unwrap();

    // No descriptions in basic.json, should fall back to command
    let dev = scripts.get("dev").unwrap();
    let desc = get_description(dev);
    assert_eq!(desc, "$ vite");

    let build = scripts.get("build").unwrap();
    let desc = get_description(build);
    assert_eq!(desc, "$ vite build");
}

#[test]
fn test_package_display_name() {
    let content = load_fixture("complex.json");
    let package = parse_package_json(&content).unwrap();

    assert_eq!(package.display_name(), "@company/complex-app");
    assert_eq!(package.version, "3.2.1");
    assert_eq!(
        package.description,
        Some("A complex application with many scripts".to_string())
    );
}

#[test]
fn test_invalid_json() {
    let content = "{ invalid json }";
    let result = parse_scripts_from_json(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Failed to parse"));
}

#[test]
fn test_parse_empty_json_object() {
    let content = "{}";
    let scripts = parse_scripts_from_json(content).unwrap();
    assert!(scripts.is_empty());

    let package = parse_package_json(content).unwrap();
    assert_eq!(package.display_name(), "unnamed");
}

#[test]
fn test_script_sorting() {
    let content = load_fixture("basic.json");
    let scripts = parse_scripts_from_json(&content).unwrap();

    let names: Vec<_> = scripts.iter().map(|s| s.name()).collect();
    // Should be sorted alphabetically
    assert_eq!(names, vec!["build", "dev", "lint", "test"]);
}
