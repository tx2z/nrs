//! CLI integration tests for nrs.
//!
//! These tests verify the command-line interface behavior using assert_cmd.

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;

use crate::integration::fixtures::{
    create_empty_project, create_large_project, create_project, create_project_invalid_json,
    create_project_no_scripts, create_project_with_config, create_project_with_descriptions,
    create_project_with_lifecycle_scripts, create_project_with_lockfile,
    create_project_with_package_manager, scripts_with_special_chars, standard_scripts,
    unicode_scripts, LockfileType,
};

/// Get a Command for the nrs binary.
fn nrs() -> Command {
    cargo_bin_cmd!("nrs")
}

// ==================== Help and Version ====================

#[test]
fn test_help_output() {
    nrs()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Fast interactive TUI for running npm scripts",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Options:"))
        .stdout(predicate::str::contains("--list"))
        .stdout(predicate::str::contains("--script"))
        .stdout(predicate::str::contains("--runner"));
}

#[test]
fn test_help_short() {
    nrs()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_version_output() {
    nrs()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("nrs"))
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

#[test]
fn test_version_short() {
    nrs()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ==================== List Mode ====================

#[test]
fn test_list_basic() {
    let project = create_project(&standard_scripts());

    nrs()
        .arg("--list")
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("dev"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("lint"))
        .stdout(predicate::str::contains("format"))
        .stdout(predicate::str::contains("5 scripts found"));
}

#[test]
fn test_list_short_flag() {
    let project = create_project(&standard_scripts());

    nrs()
        .arg("-l")
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("dev"));
}

#[test]
fn test_list_with_runner_override() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", "--runner", "pnpm"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("pnpm"));
}

#[test]
fn test_list_empty_scripts() {
    let project = create_empty_project();

    nrs()
        .arg("--list")
        .current_dir(project.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No scripts"));
}

#[test]
fn test_list_no_scripts_field() {
    let project = create_project_no_scripts();

    nrs()
        .arg("--list")
        .current_dir(project.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No scripts"));
}

#[test]
fn test_list_special_characters() {
    let project = create_project(&scripts_with_special_chars());

    nrs()
        .arg("--list")
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("build:dev"))
        .stdout(predicate::str::contains("build:prod"))
        .stdout(predicate::str::contains("test:unit"));
}

#[test]
fn test_list_unicode_scripts() {
    let project = create_project(&unicode_scripts());

    nrs()
        .arg("--list")
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("lint"));
}

#[test]
fn test_list_large_project() {
    let project = create_large_project(100);

    nrs()
        .arg("--list")
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("100 scripts found"));
}

#[test]
fn test_list_with_descriptions() {
    let project = create_project_with_descriptions(&[
        ("dev", "vite", "Start development server"),
        ("build", "vite build", "Build for production"),
    ]);

    nrs()
        .arg("--list")
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("dev"))
        .stdout(predicate::str::contains("build"));
}

// ==================== Script Execution ====================

#[test]
fn test_script_valid() {
    let project = create_project(&[("test", "echo success")]);

    nrs()
        .args(["--script", "test"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("success"));
}

#[test]
fn test_script_short_flag() {
    let project = create_project(&[("test", "echo success")]);

    nrs()
        .args(["-n", "test"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("success"));
}

#[test]
fn test_script_invalid() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--script", "nonexistent"])
        .current_dir(project.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_script_with_suggestion() {
    let project = create_project(&standard_scripts());

    // "devv" should suggest "dev"
    nrs()
        .args(["--script", "devv"])
        .current_dir(project.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Did you mean"));
}

#[test]
fn test_script_with_args() {
    let project = create_project(&[("test", "echo")]);

    nrs()
        .args(["--script", "test", "--args", "hello world"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_script_special_characters() {
    let project = create_project(&[("build:prod", "echo production")]);

    nrs()
        .args(["--script", "build:prod"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("production"));
}

// ==================== Dry Run ====================

#[test]
fn test_dry_run() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--script", "dev", "--dry-run"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Would run:"))
        .stdout(predicate::str::contains("npm run dev"));
}

#[test]
fn test_dry_run_short_flag() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["-n", "dev", "-d"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Would run:"));
}

#[test]
fn test_dry_run_with_runner() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--script", "dev", "--dry-run", "--runner", "pnpm"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("pnpm dev"));
}

#[test]
fn test_dry_run_with_args() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--script", "test", "--dry-run", "--args", "--watch"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("-- --watch"));
}

// ==================== Exclude Patterns ====================

#[test]
fn test_exclude_single_pattern() {
    let project = create_project(&standard_scripts());

    let output = nrs()
        .args(["--list", "--exclude", "test"])
        .current_dir(project.path())
        .output()
        .expect("Failed to run nrs");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have 4 scripts (excluding "test")
    assert!(
        stdout.contains("4 scripts found"),
        "Expected 4 scripts, got: {}",
        stdout
    );
    assert!(stdout.contains("dev"), "Should contain 'dev'");
    assert!(stdout.contains("build"), "Should contain 'build'");
    assert!(
        !stdout.contains("vitest"),
        "Should not contain 'vitest' (the test script command)"
    );
}

#[test]
fn test_exclude_multiple_patterns() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", "--exclude", "test", "--exclude", "lint"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("3 scripts found"));
}

#[test]
fn test_exclude_wildcard() {
    let project = create_project(&scripts_with_special_chars());

    nrs()
        .args(["--list", "--exclude", "build:*"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("3 scripts found"));
}

#[test]
fn test_exclude_lifecycle_scripts() {
    let project = create_project_with_lifecycle_scripts();

    // By default, lifecycle scripts should be filtered
    nrs()
        .arg("--list")
        .current_dir(project.path())
        .assert()
        .success();
}

// ==================== Runner Override ====================

#[test]
fn test_runner_npm() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", "--runner", "npm"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("npm"));
}

#[test]
fn test_runner_yarn() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", "--runner", "yarn"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("yarn"));
}

#[test]
fn test_runner_pnpm() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", "--runner", "pnpm"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("pnpm"));
}

#[test]
fn test_runner_bun() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", "--runner", "bun"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("bun"));
}

#[test]
fn test_runner_invalid() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", "--runner", "invalid"])
        .current_dir(project.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

// ==================== Exit Codes ====================

#[test]
fn test_exit_code_success() {
    let project = create_project(&[("test", "true")]);

    nrs()
        .args(["--script", "test"])
        .current_dir(project.path())
        .assert()
        .code(0);
}

#[test]
fn test_exit_code_no_package_json() {
    let temp = tempfile::tempdir().unwrap();

    nrs()
        .arg("--list")
        .current_dir(temp.path())
        .assert()
        .code(2); // NO_PACKAGE_JSON
}

#[test]
fn test_exit_code_no_scripts() {
    let project = create_empty_project();

    nrs()
        .arg("--list")
        .current_dir(project.path())
        .assert()
        .code(3); // NO_SCRIPTS
}

#[test]
fn test_exit_code_script_failed() {
    let project = create_project(&[("fail", "exit 42")]);

    nrs()
        .args(["--script", "fail"])
        .current_dir(project.path())
        .assert()
        .code(42); // Script's exit code
}

// ==================== Debug Mode ====================

#[test]
fn test_debug_mode() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", "--debug"])
        .current_dir(project.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("=== nrs debug mode ==="))
        .stderr(predicate::str::contains("Version:"))
        .stderr(predicate::str::contains("Project directory"))
        .stderr(predicate::str::contains("Package manager"));
}

#[test]
fn test_debug_shows_file_locations() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", "--debug"])
        .current_dir(project.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("History file:"))
        .stderr(predicate::str::contains("package.json:"));
}

// ==================== Path Argument ====================

#[test]
fn test_path_argument() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", project.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("dev"));
}

#[test]
fn test_path_nonexistent() {
    nrs()
        .args(["--list", "/nonexistent/path"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot access directory"));
}

// ==================== Invalid JSON ====================

#[test]
fn test_invalid_json() {
    let project = create_project_invalid_json();

    nrs()
        .arg("--list")
        .current_dir(project.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse"));
}

// ==================== Config ====================

#[test]
fn test_config_file_loaded() {
    let config = r#"
[appearance]
icons = false

[exclude]
patterns = ["lint"]
"#;
    let project = create_project_with_config(&standard_scripts(), config);

    // With config excluding "lint", should have 4 scripts
    nrs()
        .arg("--list")
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("4 scripts found"));
}

#[test]
fn test_no_config_flag() {
    let config = r#"
[exclude]
patterns = ["lint", "test"]
"#;
    let project = create_project_with_config(&standard_scripts(), config);

    // With --no-config, should have all 5 scripts
    nrs()
        .args(["--list", "--no-config"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("5 scripts found"));
}

// ==================== Last Script ====================

#[test]
fn test_last_no_history() {
    let project = create_project(&standard_scripts());

    // Create a temporary history dir to isolate the test
    nrs()
        .arg("--last")
        .current_dir(project.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No previous script"));
}

// ==================== Sort Mode ====================

#[test]
fn test_sort_alpha() {
    let project = create_project(&[
        ("zebra", "echo z"),
        ("alpha", "echo a"),
        ("middle", "echo m"),
    ]);

    nrs()
        .args(["--list", "--sort", "alpha"])
        .current_dir(project.path())
        .assert()
        .success();
}

#[test]
fn test_sort_recent() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", "--sort", "recent"])
        .current_dir(project.path())
        .assert()
        .success();
}

#[test]
fn test_sort_category() {
    let project = create_project(&scripts_with_special_chars());

    nrs()
        .args(["--list", "--sort", "category"])
        .current_dir(project.path())
        .assert()
        .success();
}

#[test]
fn test_sort_invalid() {
    let project = create_project(&standard_scripts());

    nrs()
        .args(["--list", "--sort", "invalid"])
        .current_dir(project.path())
        .assert()
        .failure();
}

// ==================== Runner Detection Integration ====================

#[test]
fn test_detects_npm_from_lockfile() {
    let project = create_project_with_lockfile(&standard_scripts(), LockfileType::Npm);

    nrs()
        .args(["--list", "--debug"])
        .current_dir(project.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("npm"))
        .stderr(predicate::str::contains("package-lock.json"));
}

#[test]
fn test_detects_yarn_from_lockfile() {
    let project = create_project_with_lockfile(&standard_scripts(), LockfileType::Yarn);

    nrs()
        .args(["--list", "--debug"])
        .current_dir(project.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("yarn"))
        .stderr(predicate::str::contains("yarn.lock"));
}

#[test]
fn test_detects_pnpm_from_lockfile() {
    let project = create_project_with_lockfile(&standard_scripts(), LockfileType::Pnpm);

    nrs()
        .args(["--list", "--debug"])
        .current_dir(project.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("pnpm"))
        .stderr(predicate::str::contains("pnpm-lock.yaml"));
}

#[test]
fn test_detects_bun_from_lockfile() {
    let project = create_project_with_lockfile(&standard_scripts(), LockfileType::Bun);

    nrs()
        .args(["--list", "--debug"])
        .current_dir(project.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("bun"))
        .stderr(predicate::str::contains("bun.lockb"));
}

#[test]
fn test_detects_from_package_manager_field() {
    let project = create_project_with_package_manager(&standard_scripts(), "pnpm@8.15.0");

    nrs()
        .args(["--list", "--debug"])
        .current_dir(project.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("pnpm"))
        .stderr(predicate::str::contains("packageManager"));
}
