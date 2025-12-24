//! Integration tests for history recording and loading.

use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use npm_run_scripts::config::HistoryConfig;
use npm_run_scripts::history::History;
use tempfile::TempDir;

/// Create a temporary history file location.
fn temp_history_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Create a history file path in the temp directory.
fn temp_history_path(temp: &TempDir) -> PathBuf {
    temp.path().join("history.json")
}

// ==================== Basic Operations ====================

#[test]
fn test_history_new() {
    let history = History::default();
    assert!(history.projects.is_empty());
}

#[test]
fn test_history_record_run() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");

    history.record_run(&project_path, "dev", None);

    assert!(!history.projects.is_empty());
    let project_history = history.get_project(&project_path);
    assert!(project_history.is_some());

    let last = project_history.unwrap().last_script();
    assert!(last.is_some());
    assert_eq!(last.unwrap(), "dev");
}

#[test]
fn test_history_record_with_args() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");

    history.record_run(&project_path, "test", Some("--watch".to_string()));

    let project_history = history.get_project(&project_path);
    let (name, args) = project_history.unwrap().last_script_with_args().unwrap();
    assert_eq!(name, "test");
    assert_eq!(args, Some("--watch"));
}

#[test]
fn test_history_multiple_runs() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");

    history.record_run(&project_path, "dev", None);
    history.record_run(&project_path, "build", None);
    history.record_run(&project_path, "test", None);

    let project_history = history.get_project(&project_path).unwrap();
    assert_eq!(project_history.scripts.len(), 3);

    // Last script should be "test"
    let last = project_history.last_script();
    assert_eq!(last.unwrap(), "test");
}

#[test]
fn test_history_run_count() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");

    history.record_run(&project_path, "dev", None);
    history.record_run(&project_path, "dev", None);
    history.record_run(&project_path, "dev", None);

    let project_history = history.get_project(&project_path).unwrap();
    let dev_script = project_history.scripts.get("dev");
    assert!(dev_script.is_some());
    assert_eq!(dev_script.unwrap().count, 3);
}

// ==================== Persistence ====================

#[test]
fn test_history_save_and_load() {
    let temp = temp_history_dir();
    let history_path = temp_history_path(&temp);

    // Create and save history
    let mut history = History::default();
    history.record_run(&PathBuf::from("/test/project"), "dev", None);
    history.record_run(&PathBuf::from("/test/project"), "build", None);

    let content = serde_json::to_string_pretty(&history).expect("Failed to serialize");
    fs::write(&history_path, content).expect("Failed to write");

    // Load history
    let loaded_content = fs::read_to_string(&history_path).expect("Failed to read");
    let loaded: History = serde_json::from_str(&loaded_content).expect("Failed to parse");

    let project_history = loaded.get_project(&PathBuf::from("/test/project"));
    assert!(project_history.is_some());
    assert_eq!(project_history.unwrap().scripts.len(), 2);
}

#[test]
fn test_history_missing_file() {
    let temp = temp_history_dir();
    let history_path = temp_history_path(&temp);

    // File doesn't exist
    let result = fs::read_to_string(&history_path);
    assert!(result.is_err());
}

#[test]
fn test_history_corrupt_file_recovery() {
    let temp = temp_history_dir();
    let history_path = temp_history_path(&temp);

    // Write corrupt content
    fs::write(&history_path, "{ invalid json }").expect("Failed to write");

    // Should fail to parse
    let loaded_content = fs::read_to_string(&history_path).expect("Failed to read");
    let result: Result<History, _> = serde_json::from_str(&loaded_content);
    assert!(result.is_err());

    // Application would typically fall back to default
    let fallback = History::default();
    assert!(fallback.projects.is_empty());
}

#[test]
fn test_history_empty_file() {
    let temp = temp_history_dir();
    let history_path = temp_history_path(&temp);

    // Write empty content
    fs::write(&history_path, "").expect("Failed to write");

    let loaded_content = fs::read_to_string(&history_path).expect("Failed to read");
    let result: Result<History, _> = serde_json::from_str(&loaded_content);
    assert!(result.is_err());
}

// ==================== LRU Eviction ====================

#[test]
fn test_history_cleanup_projects() {
    let mut history = History::default();
    let config = HistoryConfig {
        enabled: true,
        max_projects: 3,
        max_scripts: 100,
    };

    // Add 5 projects
    for i in 0..5 {
        let path = PathBuf::from(format!("/project/{}", i));
        history.record_run(&path, "dev", None);
    }

    assert_eq!(history.projects.len(), 5);

    // Cleanup should remove oldest projects
    history.cleanup(&config);

    assert_eq!(history.projects.len(), 3);
}

#[test]
fn test_history_cleanup_scripts() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");
    let config = HistoryConfig {
        enabled: true,
        max_projects: 10,
        max_scripts: 3,
    };

    // Add 5 scripts
    for i in 0..5 {
        history.record_run(&project_path, &format!("script{}", i), None);
    }

    let project_history = history.get_project(&project_path).unwrap();
    assert_eq!(project_history.scripts.len(), 5);

    // Cleanup should remove least used scripts
    history.cleanup(&config);

    let project_history = history.get_project(&project_path).unwrap();
    assert_eq!(project_history.scripts.len(), 3);
}

// ==================== Get Last Script ====================

#[test]
fn test_get_last_script() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");

    history.record_run(&project_path, "dev", None);
    history.record_run(&project_path, "build", None);
    history.record_run(&project_path, "test", Some("--watch".to_string()));

    let project_history = history.get_project(&project_path).unwrap();
    let (name, args) = project_history.last_script_with_args().unwrap();

    assert_eq!(name, "test");
    assert_eq!(args, Some("--watch"));
}

#[test]
fn test_get_last_script_no_history() {
    let history = History::default();
    let project_path = PathBuf::from("/test/project");

    let project_history = history.get_project(&project_path);
    assert!(project_history.is_none());
}

// ==================== Script Stats ====================

#[test]
fn test_script_stats_run_count() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");

    for _ in 0..5 {
        history.record_run(&project_path, "dev", None);
    }

    let project_history = history.get_project(&project_path).unwrap();
    let stats = project_history.scripts.get("dev").unwrap();

    assert_eq!(stats.count, 5);
}

#[test]
fn test_script_stats_last_run_time() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");

    let before = Utc::now();
    history.record_run(&project_path, "dev", None);
    let after = Utc::now();

    let project_history = history.get_project(&project_path).unwrap();
    let stats = project_history.scripts.get("dev").unwrap();

    assert!(stats.last_run >= before);
    assert!(stats.last_run <= after);
}

// ==================== Multiple Projects ====================

#[test]
fn test_multiple_projects() {
    let mut history = History::default();

    history.record_run(&PathBuf::from("/project/a"), "dev", None);
    history.record_run(&PathBuf::from("/project/b"), "build", None);
    history.record_run(&PathBuf::from("/project/c"), "test", None);

    assert_eq!(history.projects.len(), 3);

    let project_a = history.get_project(&PathBuf::from("/project/a"));
    assert!(project_a.is_some());
    assert!(project_a.unwrap().scripts.contains_key("dev"));

    let project_b = history.get_project(&PathBuf::from("/project/b"));
    assert!(project_b.is_some());
    assert!(project_b.unwrap().scripts.contains_key("build"));
}

#[test]
fn test_project_isolation() {
    let mut history = History::default();

    // Run "dev" in project A
    history.record_run(&PathBuf::from("/project/a"), "dev", None);

    // Run "build" in project B
    history.record_run(&PathBuf::from("/project/b"), "build", None);

    // Project A should not have "build"
    let project_a = history.get_project(&PathBuf::from("/project/a")).unwrap();
    assert!(!project_a.scripts.contains_key("build"));

    // Project B should not have "dev"
    let project_b = history.get_project(&PathBuf::from("/project/b")).unwrap();
    assert!(!project_b.scripts.contains_key("dev"));
}

// ==================== Edge Cases ====================

#[test]
fn test_empty_script_name() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");

    // Empty script name is allowed
    history.record_run(&project_path, "", None);

    let project_history = history.get_project(&project_path).unwrap();
    assert!(project_history.scripts.contains_key(""));
}

#[test]
fn test_unicode_script_name() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");

    history.record_run(&project_path, "开发", None);
    history.record_run(&project_path, "テスト", None);

    let project_history = history.get_project(&project_path).unwrap();
    assert!(project_history.scripts.contains_key("开发"));
    assert!(project_history.scripts.contains_key("テスト"));
}

#[test]
fn test_special_chars_in_path() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project with spaces/and-dashes");

    history.record_run(&project_path, "dev", None);

    let project_history = history.get_project(&project_path);
    assert!(project_history.is_some());
}

#[test]
fn test_very_long_args() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");
    let long_args = "a".repeat(10000);

    history.record_run(&project_path, "test", Some(long_args.clone()));

    let project_history = history.get_project(&project_path).unwrap();
    let (_, args) = project_history.last_script_with_args().unwrap();
    assert_eq!(args, Some(long_args.as_str()));
}

// ==================== Version Compatibility ====================

#[test]
fn test_history_version() {
    let history = History::default();
    assert_eq!(history.version, 1);
}

#[test]
fn test_history_serialization_format() {
    let mut history = History::default();
    history.record_run(&PathBuf::from("/test"), "dev", None);

    let json = serde_json::to_string_pretty(&history).expect("Failed to serialize");

    // Should contain expected fields
    assert!(json.contains("\"version\""));
    assert!(json.contains("\"projects\""));
    assert!(json.contains("\"scripts\""));
    assert!(json.contains("\"count\""));
    assert!(json.contains("\"last_run\""));
}

// ==================== Score Calculation ====================

#[test]
fn test_script_score() {
    let mut history = History::default();
    let project_path = PathBuf::from("/test/project");

    // Run scripts different numbers of times
    for _ in 0..10 {
        history.record_run(&project_path, "frequent", None);
    }
    history.record_run(&project_path, "rare", None);

    let project_history = history.get_project(&project_path).unwrap();

    let frequent = project_history.scripts.get("frequent").unwrap();
    let rare = project_history.scripts.get("rare").unwrap();

    // Frequent should have higher score
    assert!(frequent.score() > rare.score());
}
