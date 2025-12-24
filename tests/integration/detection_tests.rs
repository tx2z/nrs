//! Integration tests for package manager detection.
//!
//! These tests verify that nrs correctly detects the package manager
//! from various sources (packageManager field, lock files).

use std::fs;

use npm_run_scripts::package::{detect_runner, detect_runner_reason, Runner};

use crate::integration::fixtures::{
    create_project, create_project_with_lockfile, create_project_with_package_manager,
    create_project_with_pm_and_lockfile, standard_scripts, LockfileType,
};

// ==================== Lock File Detection ====================

#[test]
fn test_detect_npm_from_lockfile() {
    let project = create_project_with_lockfile(&standard_scripts(), LockfileType::Npm);
    let runner = detect_runner(project.path());
    assert_eq!(runner, Runner::Npm);
}

#[test]
fn test_detect_yarn_from_lockfile() {
    let project = create_project_with_lockfile(&standard_scripts(), LockfileType::Yarn);
    let runner = detect_runner(project.path());
    assert_eq!(runner, Runner::Yarn);
}

#[test]
fn test_detect_pnpm_from_lockfile() {
    let project = create_project_with_lockfile(&standard_scripts(), LockfileType::Pnpm);
    let runner = detect_runner(project.path());
    assert_eq!(runner, Runner::Pnpm);
}

#[test]
fn test_detect_bun_from_lockfile() {
    let project = create_project_with_lockfile(&standard_scripts(), LockfileType::Bun);
    let runner = detect_runner(project.path());
    assert_eq!(runner, Runner::Bun);
}

// ==================== packageManager Field Detection ====================

#[test]
fn test_detect_npm_from_package_manager() {
    let project = create_project_with_package_manager(&standard_scripts(), "npm@10.0.0");
    let runner = detect_runner(project.path());
    assert_eq!(runner, Runner::Npm);
}

#[test]
fn test_detect_yarn_from_package_manager() {
    let project = create_project_with_package_manager(&standard_scripts(), "yarn@4.0.0");
    let runner = detect_runner(project.path());
    assert_eq!(runner, Runner::Yarn);
}

#[test]
fn test_detect_pnpm_from_package_manager() {
    let project = create_project_with_package_manager(&standard_scripts(), "pnpm@8.15.0");
    let runner = detect_runner(project.path());
    assert_eq!(runner, Runner::Pnpm);
}

#[test]
fn test_detect_bun_from_package_manager() {
    let project = create_project_with_package_manager(&standard_scripts(), "bun@1.0.0");
    let runner = detect_runner(project.path());
    assert_eq!(runner, Runner::Bun);
}

#[test]
fn test_detect_package_manager_without_version() {
    let project = create_project_with_package_manager(&standard_scripts(), "pnpm");
    let runner = detect_runner(project.path());
    assert_eq!(runner, Runner::Pnpm);
}

#[test]
fn test_detect_package_manager_with_sha() {
    let project =
        create_project_with_package_manager(&standard_scripts(), "yarn@4.0.0+sha256.abc123");
    let runner = detect_runner(project.path());
    assert_eq!(runner, Runner::Yarn);
}

// ==================== Priority Order ====================

#[test]
fn test_package_manager_takes_priority_over_lockfile() {
    // Create project with pnpm in packageManager but yarn.lock
    let project =
        create_project_with_pm_and_lockfile(&standard_scripts(), "pnpm@8.0.0", LockfileType::Yarn);

    let runner = detect_runner(project.path());
    // packageManager should win
    assert_eq!(runner, Runner::Pnpm);
}

#[test]
fn test_bun_lockfile_priority_over_other_lockfiles() {
    // Create project with multiple lock files
    let project = create_project(&standard_scripts());

    // Add multiple lock files
    fs::write(project.path().join("bun.lockb"), "binary").unwrap();
    fs::write(project.path().join("yarn.lock"), "# yarn").unwrap();
    fs::write(project.path().join("package-lock.json"), "{}").unwrap();

    let runner = detect_runner(project.path());
    // Bun should win (checked first after packageManager)
    assert_eq!(runner, Runner::Bun);
}

#[test]
fn test_pnpm_priority_over_yarn_and_npm() {
    let project = create_project(&standard_scripts());

    fs::write(
        project.path().join("pnpm-lock.yaml"),
        "lockfileVersion: 5.4",
    )
    .unwrap();
    fs::write(project.path().join("yarn.lock"), "# yarn").unwrap();
    fs::write(project.path().join("package-lock.json"), "{}").unwrap();

    let runner = detect_runner(project.path());
    // pnpm should win over yarn and npm
    assert_eq!(runner, Runner::Pnpm);
}

#[test]
fn test_yarn_priority_over_npm() {
    let project = create_project(&standard_scripts());

    fs::write(project.path().join("yarn.lock"), "# yarn").unwrap();
    fs::write(project.path().join("package-lock.json"), "{}").unwrap();

    let runner = detect_runner(project.path());
    // yarn should win over npm
    assert_eq!(runner, Runner::Yarn);
}

// ==================== Fallback ====================

#[test]
fn test_fallback_to_npm() {
    let project = create_project(&standard_scripts());
    // No lock file, no packageManager field

    let runner = detect_runner(project.path());
    assert_eq!(runner, Runner::Npm);
}

// ==================== Detection Reason ====================

#[test]
fn test_detect_reason_package_manager() {
    let project = create_project_with_package_manager(&standard_scripts(), "pnpm@8.0.0");

    let (runner, reason) = detect_runner_reason(project.path());
    assert_eq!(runner, Runner::Pnpm);
    assert!(reason.contains("packageManager"));
}

#[test]
fn test_detect_reason_lockfile() {
    let project = create_project_with_lockfile(&standard_scripts(), LockfileType::Yarn);

    let (runner, reason) = detect_runner_reason(project.path());
    assert_eq!(runner, Runner::Yarn);
    assert!(reason.contains("yarn.lock"));
}

#[test]
fn test_detect_reason_default() {
    let project = create_project(&standard_scripts());

    let (runner, reason) = detect_runner_reason(project.path());
    assert_eq!(runner, Runner::Npm);
    assert!(reason.contains("default"));
}

// ==================== Edge Cases ====================

#[test]
fn test_empty_package_manager_field() {
    let temp = tempfile::tempdir().unwrap();

    let package_json = r#"{
  "name": "test",
  "packageManager": "",
  "scripts": { "dev": "echo dev" }
}"#;

    fs::write(temp.path().join("package.json"), package_json).unwrap();

    // Empty packageManager should fall back to npm
    let runner = detect_runner(temp.path());
    assert_eq!(runner, Runner::Npm);
}

#[test]
fn test_invalid_package_manager_field() {
    let temp = tempfile::tempdir().unwrap();

    let package_json = r#"{
  "name": "test",
  "packageManager": "invalid@1.0.0",
  "scripts": { "dev": "echo dev" }
}"#;

    fs::write(temp.path().join("package.json"), package_json).unwrap();

    // Invalid packageManager should fall back to npm
    let runner = detect_runner(temp.path());
    assert_eq!(runner, Runner::Npm);
}

#[test]
fn test_no_package_json_fallback() {
    let temp = tempfile::tempdir().unwrap();
    // No package.json at all

    // Should still return npm as default
    let runner = detect_runner(temp.path());
    assert_eq!(runner, Runner::Npm);
}

// ==================== Runner Methods ====================

#[test]
fn test_runner_run_command_npm() {
    let cmd = Runner::Npm.run_command("dev");
    assert_eq!(cmd, vec!["npm", "run", "dev"]);
}

#[test]
fn test_runner_run_command_yarn() {
    let cmd = Runner::Yarn.run_command("dev");
    assert_eq!(cmd, vec!["yarn", "dev"]);
}

#[test]
fn test_runner_run_command_pnpm() {
    let cmd = Runner::Pnpm.run_command("dev");
    assert_eq!(cmd, vec!["pnpm", "dev"]);
}

#[test]
fn test_runner_run_command_bun() {
    let cmd = Runner::Bun.run_command("dev");
    assert_eq!(cmd, vec!["bun", "run", "dev"]);
}

#[test]
fn test_runner_run_command_with_args_npm() {
    let args = vec!["--watch".to_string()];
    let cmd = Runner::Npm.run_command_with_args("test", &args);
    assert_eq!(cmd, vec!["npm", "run", "test", "--", "--watch"]);
}

#[test]
fn test_runner_run_command_with_args_yarn() {
    let args = vec!["--watch".to_string()];
    let cmd = Runner::Yarn.run_command_with_args("test", &args);
    // Yarn doesn't need --
    assert_eq!(cmd, vec!["yarn", "test", "--watch"]);
}

#[test]
fn test_runner_run_command_with_args_pnpm() {
    let args = vec!["--watch".to_string()];
    let cmd = Runner::Pnpm.run_command_with_args("test", &args);
    // pnpm needs --
    assert_eq!(cmd, vec!["pnpm", "test", "--", "--watch"]);
}

#[test]
fn test_runner_run_command_with_args_bun() {
    let args = vec!["--watch".to_string()];
    let cmd = Runner::Bun.run_command_with_args("test", &args);
    // Bun doesn't need --
    assert_eq!(cmd, vec!["bun", "run", "test", "--watch"]);
}

#[test]
fn test_runner_lock_files() {
    assert_eq!(Runner::Npm.lock_file(), "package-lock.json");
    assert_eq!(Runner::Yarn.lock_file(), "yarn.lock");
    assert_eq!(Runner::Pnpm.lock_file(), "pnpm-lock.yaml");
    assert_eq!(Runner::Bun.lock_file(), "bun.lockb");
}

#[test]
fn test_runner_executables() {
    assert_eq!(Runner::Npm.executable(), "npm");
    assert_eq!(Runner::Yarn.executable(), "yarn");
    assert_eq!(Runner::Pnpm.executable(), "pnpm");
    assert_eq!(Runner::Bun.executable(), "bun");
}

#[test]
fn test_runner_from_str() {
    assert_eq!("npm".parse::<Runner>().unwrap(), Runner::Npm);
    assert_eq!("yarn".parse::<Runner>().unwrap(), Runner::Yarn);
    assert_eq!("pnpm".parse::<Runner>().unwrap(), Runner::Pnpm);
    assert_eq!("bun".parse::<Runner>().unwrap(), Runner::Bun);
}

#[test]
fn test_runner_from_str_case_insensitive() {
    assert_eq!("NPM".parse::<Runner>().unwrap(), Runner::Npm);
    assert_eq!("YARN".parse::<Runner>().unwrap(), Runner::Yarn);
    assert_eq!("PnPm".parse::<Runner>().unwrap(), Runner::Pnpm);
    assert_eq!("BUN".parse::<Runner>().unwrap(), Runner::Bun);
}

#[test]
fn test_runner_from_str_invalid() {
    let result = "invalid".parse::<Runner>();
    assert!(result.is_err());
}
