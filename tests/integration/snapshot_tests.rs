//! Snapshot tests using insta.
//!
//! These tests capture and verify the output format of various commands.

use assert_cmd::Command;

use crate::integration::fixtures::{
    create_project, create_project_with_descriptions, standard_scripts,
};

/// Get a Command for the nrs binary.
fn nrs() -> Command {
    Command::cargo_bin("nrs").expect("Failed to find nrs binary")
}

// ==================== Help Output Snapshots ====================

#[test]
fn test_snapshot_help_output() {
    let output = nrs().arg("--help").output().expect("Failed to run nrs");

    let stdout = String::from_utf8_lossy(&output.stdout);

    insta::assert_snapshot!("help_output", stdout);
}

#[test]
fn test_snapshot_help_short_output() {
    let output = nrs().arg("-h").output().expect("Failed to run nrs");

    let stdout = String::from_utf8_lossy(&output.stdout);

    insta::assert_snapshot!("help_short_output", stdout);
}

// ==================== Version Output Snapshots ====================

#[test]
fn test_snapshot_version_output() {
    let output = nrs().arg("--version").output().expect("Failed to run nrs");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Version output is simple, but still worth snapshotting
    insta::assert_snapshot!("version_output", stdout);
}

// ==================== List Output Snapshots ====================

#[test]
fn test_snapshot_list_basic() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .arg("--list")
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Normalize the output for consistent snapshots
    let normalized = normalize_list_output(&stdout);

    insta::assert_snapshot!("list_basic", normalized);
}

#[test]
fn test_snapshot_list_with_descriptions() {
    let project = create_project_with_descriptions(&[
        ("dev", "vite", "Start development server"),
        ("build", "vite build", "Build for production"),
        ("test", "vitest", "Run test suite"),
        ("lint", "eslint .", "Lint code"),
    ]);

    let output = nrs()
        .arg("--list")
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let normalized = normalize_list_output(&stdout);

    insta::assert_snapshot!("list_with_descriptions", normalized);
}

#[test]
fn test_snapshot_list_empty_project() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("package.json"),
        r#"{"name": "empty", "scripts": {}}"#,
    )
    .unwrap();

    let output = nrs()
        .arg("--list")
        .current_dir(temp.path())
        .output()
        .expect("Failed to run nrs");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let normalized = normalize_error_path(&stderr);

    insta::assert_snapshot!("list_empty_project", normalized);
}

// ==================== Dry Run Output Snapshots ====================

#[test]
fn test_snapshot_dry_run_npm() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .args(["--script", "dev", "--dry-run"])
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stdout = String::from_utf8_lossy(&output.stdout);

    insta::assert_snapshot!("dry_run_npm", stdout);
}

#[test]
fn test_snapshot_dry_run_yarn() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .args(["--script", "dev", "--dry-run", "--runner", "yarn"])
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stdout = String::from_utf8_lossy(&output.stdout);

    insta::assert_snapshot!("dry_run_yarn", stdout);
}

#[test]
fn test_snapshot_dry_run_pnpm() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .args(["--script", "dev", "--dry-run", "--runner", "pnpm"])
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stdout = String::from_utf8_lossy(&output.stdout);

    insta::assert_snapshot!("dry_run_pnpm", stdout);
}

#[test]
fn test_snapshot_dry_run_bun() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .args(["--script", "dev", "--dry-run", "--runner", "bun"])
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stdout = String::from_utf8_lossy(&output.stdout);

    insta::assert_snapshot!("dry_run_bun", stdout);
}

#[test]
fn test_snapshot_dry_run_with_args() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .args([
            "--script",
            "test",
            "--dry-run",
            "--args",
            "--watch --coverage",
        ])
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stdout = String::from_utf8_lossy(&output.stdout);

    insta::assert_snapshot!("dry_run_with_args", stdout);
}

// ==================== Error Message Snapshots ====================

#[test]
fn test_snapshot_error_no_package_json() {
    let temp = tempfile::tempdir().unwrap();

    let output = nrs()
        .arg("--list")
        .current_dir(temp.path())
        .output()
        .expect("Failed to run nrs");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Normalize the path in the error message
    let normalized = normalize_error_path(&stderr);

    insta::assert_snapshot!("error_no_package_json", normalized);
}

#[test]
fn test_snapshot_error_invalid_json() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("package.json"), "{ invalid json }").unwrap();

    let output = nrs()
        .arg("--list")
        .current_dir(temp.path())
        .output()
        .expect("Failed to run nrs");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Normalize the path in the error message
    let normalized = normalize_error_path(&stderr);

    insta::assert_snapshot!("error_invalid_json", normalized);
}

#[test]
fn test_snapshot_error_script_not_found() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .args(["--script", "nonexistent"])
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stderr = String::from_utf8_lossy(&output.stderr);

    insta::assert_snapshot!("error_script_not_found", stderr);
}

#[test]
fn test_snapshot_error_script_not_found_with_suggestion() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .args(["--script", "devv"]) // typo for "dev"
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stderr = String::from_utf8_lossy(&output.stderr);

    insta::assert_snapshot!("error_script_not_found_suggestion", stderr);
}

#[test]
fn test_snapshot_error_no_history() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .arg("--last")
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stderr = String::from_utf8_lossy(&output.stderr);

    insta::assert_snapshot!("error_no_history", stderr);
}

#[test]
fn test_snapshot_error_invalid_runner() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .args(["--list", "--runner", "invalid"])
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stderr = String::from_utf8_lossy(&output.stderr);

    insta::assert_snapshot!("error_invalid_runner", stderr);
}

// ==================== Helper Functions ====================

/// Normalize list output for consistent snapshots.
///
/// Replaces dynamic content that may vary between runs.
fn normalize_list_output(output: &str) -> String {
    // The output should be fairly stable, just ensure consistent line endings
    output.replace("\r\n", "\n")
}

/// Normalize error paths for consistent snapshots.
///
/// Replaces absolute paths with placeholders.
fn normalize_error_path(error: &str) -> String {
    // Replace temporary directory paths with a placeholder
    // Matches various temp path formats:
    // - /var/folders/.../T/.tmpXXX/...
    // - /private/var/folders/.../T/.tmpXXX/...
    // - /tmp/...
    let re = regex::Regex::new(r"(/private)?/var/folders/[^\s]+|/tmp/[^\s]+").unwrap();
    re.replace_all(error, "<TEMP_PATH>").to_string()
}

// ==================== Debug Output Snapshots ====================

#[test]
fn test_snapshot_debug_output() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .args(["--list", "--debug"])
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Normalize paths and timestamps
    let normalized = normalize_debug_output(&stderr);

    insta::assert_snapshot!("debug_output", normalized);
}

/// Normalize debug output for consistent snapshots.
fn normalize_debug_output(output: &str) -> String {
    let mut result = output.to_string();

    // Replace absolute paths
    let path_re =
        regex::Regex::new(r"(/[^\s:]+)+").unwrap_or_else(|_| regex::Regex::new(r"/.+").unwrap());
    result = path_re.replace_all(&result, "<PATH>").to_string();

    // Keep the structure but normalize variable parts
    result.replace("\r\n", "\n")
}
