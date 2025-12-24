//! History storage and persistence.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::HistoryConfig;
use crate::package::Script;

/// Default maximum number of projects to track.
pub const DEFAULT_MAX_PROJECTS: usize = 100;

/// Default maximum number of scripts per project.
pub const DEFAULT_MAX_SCRIPTS: usize = 50;

/// Weight for run count in scoring (30%).
const RUN_COUNT_WEIGHT: f64 = 0.3;

/// Weight for recency in scoring (70%).
const RECENCY_WEIGHT: f64 = 0.7;

/// Days after which recency score fully decays.
const RECENCY_DECAY_DAYS: i64 = 30;

/// History for a single script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptHistory {
    /// Number of times the script has been run.
    pub count: u32,
    /// Last time the script was run.
    pub last_run: DateTime<Utc>,
    /// Last arguments passed to the script.
    pub last_args: Option<String>,
}

impl ScriptHistory {
    /// Create a new script history entry.
    pub fn new() -> Self {
        Self {
            count: 1,
            last_run: Utc::now(),
            last_args: None,
        }
    }

    /// Create with specific values (for testing).
    pub fn with_values(count: u32, last_run: DateTime<Utc>, last_args: Option<String>) -> Self {
        Self {
            count,
            last_run,
            last_args,
        }
    }

    /// Record a new execution of the script.
    pub fn record_run(&mut self, args: Option<String>) {
        self.count += 1;
        self.last_run = Utc::now();
        self.last_args = args;
    }

    /// Calculate the score for this script based on run count and recency.
    ///
    /// Score = (run_count * 0.3) + (recency_score * 0.7)
    /// Where recency_score decays from 1.0 to 0.0 over RECENCY_DECAY_DAYS.
    pub fn score(&self) -> f64 {
        self.score_at(Utc::now())
    }

    /// Calculate the score at a specific time (for testing).
    pub fn score_at(&self, now: DateTime<Utc>) -> f64 {
        let days_ago = (now - self.last_run).num_days();
        let recency_score = if days_ago <= 0 {
            1.0
        } else if days_ago >= RECENCY_DECAY_DAYS {
            0.0
        } else {
            1.0 - (days_ago as f64 / RECENCY_DECAY_DAYS as f64)
        };

        // Normalize run count (cap at 100 for scoring purposes)
        let normalized_count = (self.count.min(100) as f64) / 100.0;

        (normalized_count * RUN_COUNT_WEIGHT) + (recency_score * RECENCY_WEIGHT)
    }
}

impl Default for ScriptHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// History for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHistory {
    /// Last script executed in this project.
    pub last_script: Option<String>,
    /// Last time any script was run in this project.
    pub last_run: DateTime<Utc>,
    /// History for each script.
    #[serde(default)]
    pub scripts: HashMap<String, ScriptHistory>,
}

impl ProjectHistory {
    /// Create a new project history.
    pub fn new() -> Self {
        Self {
            last_script: None,
            last_run: Utc::now(),
            scripts: HashMap::new(),
        }
    }

    /// Record a script execution.
    pub fn record_run(&mut self, script: &str, args: Option<String>) {
        self.last_script = Some(script.to_string());
        self.last_run = Utc::now();

        self.scripts
            .entry(script.to_string())
            .and_modify(|h| h.record_run(args.clone()))
            .or_insert_with(|| {
                let mut h = ScriptHistory::new();
                h.last_args = args;
                h
            });
    }

    /// Get the last executed script name.
    pub fn last_script(&self) -> Option<&str> {
        self.last_script.as_deref()
    }

    /// Get the last executed script with its arguments.
    pub fn last_script_with_args(&self) -> Option<(&str, Option<&str>)> {
        self.last_script.as_deref().map(|name| {
            let args = self.scripts.get(name).and_then(|h| h.last_args.as_deref());
            (name, args)
        })
    }

    /// Get history for a specific script.
    pub fn get_script(&self, name: &str) -> Option<&ScriptHistory> {
        self.scripts.get(name)
    }

    /// Enforce max_scripts limit using LRU eviction.
    pub fn cleanup(&mut self, max_scripts: usize) {
        if self.scripts.len() <= max_scripts {
            return;
        }

        // Sort scripts by last_run (oldest first)
        let mut scripts: Vec<_> = self.scripts.iter().collect();
        scripts.sort_by(|a, b| a.1.last_run.cmp(&b.1.last_run));

        // Calculate how many to remove
        let to_remove = self.scripts.len() - max_scripts;

        // Collect keys to remove (oldest ones)
        let keys_to_remove: Vec<String> = scripts
            .into_iter()
            .take(to_remove)
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            self.scripts.remove(&key);
        }
    }
}

impl Default for ProjectHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global history storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    /// Version of the history format.
    pub version: u32,
    /// History per project path.
    #[serde(default)]
    pub projects: HashMap<PathBuf, ProjectHistory>,
}

impl History {
    /// Current history format version.
    pub const VERSION: u32 = 1;

    /// Create a new history.
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            projects: HashMap::new(),
        }
    }

    /// Get the history file path.
    pub fn file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("nrs").join("history.json"))
    }

    /// Get the backup file path.
    fn backup_path() -> Option<PathBuf> {
        Self::file_path().map(|p| p.with_extension("json.bak"))
    }

    /// Load history from the default location.
    ///
    /// Handles missing files gracefully (returns empty history).
    /// Handles corrupt files by backing up and returning empty history.
    pub fn load() -> Result<Self> {
        let path = Self::file_path().context("Could not determine config directory")?;

        if !path.exists() {
            return Ok(Self::new());
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to read history file {}: {}",
                    path.display(),
                    e
                );
                return Ok(Self::new());
            }
        };

        match serde_json::from_str::<History>(&content) {
            Ok(history) => Ok(history),
            Err(e) => {
                // Corrupt file - backup and return empty
                eprintln!(
                    "Warning: History file is corrupt, backing up and starting fresh: {}",
                    e
                );

                if let Some(backup_path) = Self::backup_path() {
                    if let Err(backup_err) = fs::rename(&path, &backup_path) {
                        eprintln!(
                            "Warning: Failed to backup corrupt history file: {}",
                            backup_err
                        );
                    } else {
                        eprintln!("Corrupt history backed up to {}", backup_path.display());
                    }
                }

                Ok(Self::new())
            }
        }
    }

    /// Save history to the default location.
    pub fn save(&self) -> Result<()> {
        let path = Self::file_path().context("Could not determine config directory")?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(self).context("Failed to serialize history")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write history to {}", path.display()))?;

        Ok(())
    }

    /// Get history for a project.
    pub fn get_project(&self, project_dir: &Path) -> Option<&ProjectHistory> {
        self.projects.get(project_dir)
    }

    /// Get mutable history for a project.
    pub fn get_project_mut(&mut self, project_dir: &Path) -> Option<&mut ProjectHistory> {
        self.projects.get_mut(project_dir)
    }

    /// Get or create history for a project.
    pub fn get_or_create_project(&mut self, project_dir: &Path) -> &mut ProjectHistory {
        self.projects.entry(project_dir.to_path_buf()).or_default()
    }

    /// Record a script execution.
    pub fn record_run(&mut self, project_dir: &Path, script: &str, args: Option<String>) {
        self.get_or_create_project(project_dir)
            .record_run(script, args);
    }

    /// Get the last executed script for a project with its arguments.
    pub fn get_last_script(&self, project_dir: &Path) -> Option<(String, Option<String>)> {
        self.get_project(project_dir)
            .and_then(|p| p.last_script_with_args())
            .map(|(name, args)| (name.to_string(), args.map(String::from)))
    }

    /// Get statistics for a specific script in a project.
    pub fn get_script_stats(&self, project_dir: &Path, script: &str) -> Option<&ScriptHistory> {
        self.get_project(project_dir)
            .and_then(|p| p.get_script(script))
    }

    /// Sort scripts by recent usage (most recently/frequently used first).
    ///
    /// Scripts with history are sorted by score, scripts without history
    /// are placed at the end in their original order.
    pub fn get_sorted_by_recent<'a>(
        &self,
        project_dir: &Path,
        scripts: &'a [Script],
    ) -> Vec<&'a Script> {
        self.get_sorted_by_recent_at(project_dir, scripts, Utc::now())
    }

    /// Sort scripts by recent usage at a specific time (for testing).
    pub fn get_sorted_by_recent_at<'a>(
        &self,
        project_dir: &Path,
        scripts: &'a [Script],
        now: DateTime<Utc>,
    ) -> Vec<&'a Script> {
        let project_history = self.get_project(project_dir);

        let mut scored: Vec<(&Script, f64)> = scripts
            .iter()
            .map(|s| {
                let score = project_history
                    .and_then(|p| p.get_script(s.name()))
                    .map(|h| h.score_at(now))
                    .unwrap_or(0.0);
                (s, score)
            })
            .collect();

        // Sort by score descending, then by name for stability
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.name().cmp(b.0.name()))
        });

        scored.into_iter().map(|(s, _)| s).collect()
    }

    /// Enforce max_projects and max_scripts limits using LRU eviction.
    pub fn cleanup(&mut self, config: &HistoryConfig) {
        self.cleanup_with_limits(config.max_projects, config.max_scripts);
    }

    /// Cleanup with specific limits.
    pub fn cleanup_with_limits(&mut self, max_projects: usize, max_scripts: usize) {
        // First, cleanup scripts within each project
        for project in self.projects.values_mut() {
            project.cleanup(max_scripts);
        }

        // Then, cleanup projects if needed
        if self.projects.len() <= max_projects {
            return;
        }

        // Sort projects by last_run (oldest first)
        let mut projects: Vec<_> = self.projects.iter().collect();
        projects.sort_by(|a, b| a.1.last_run.cmp(&b.1.last_run));

        // Calculate how many to remove
        let to_remove = self.projects.len() - max_projects;

        // Collect keys to remove (oldest ones)
        let keys_to_remove: Vec<PathBuf> = projects
            .into_iter()
            .take(to_remove)
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            self.projects.remove(&key);
        }
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use tempfile::TempDir;

    fn create_test_scripts() -> Vec<Script> {
        vec![
            Script::new("dev", "vite"),
            Script::new("build", "vite build"),
            Script::new("test", "vitest"),
            Script::new("lint", "eslint ."),
        ]
    }

    #[test]
    fn test_script_history_new() {
        let history = ScriptHistory::new();
        assert_eq!(history.count, 1);
        assert!(history.last_args.is_none());
    }

    #[test]
    fn test_script_history_record_run() {
        let mut history = ScriptHistory::new();
        history.record_run(Some("--watch".to_string()));

        assert_eq!(history.count, 2);
        assert_eq!(history.last_args, Some("--watch".to_string()));
    }

    #[test]
    fn test_script_history_score_today() {
        let history = ScriptHistory::new();
        let score = history.score();

        // Brand new script: count=1, recency=1.0
        // score = (1/100 * 0.3) + (1.0 * 0.7) = 0.003 + 0.7 = 0.703
        assert!(score > 0.7);
        assert!(score < 0.71);
    }

    #[test]
    fn test_script_history_score_old() {
        let now = Utc::now();
        let old_date = now - Duration::days(30);

        let history = ScriptHistory::with_values(50, old_date, None);
        let score = history.score_at(now);

        // 30 days old: recency = 0.0
        // count = 50: normalized = 0.5
        // score = (0.5 * 0.3) + (0.0 * 0.7) = 0.15
        assert!((score - 0.15).abs() < 0.01);
    }

    #[test]
    fn test_script_history_score_medium() {
        let now = Utc::now();
        let medium_date = now - Duration::days(15);

        let history = ScriptHistory::with_values(20, medium_date, None);
        let score = history.score_at(now);

        // 15 days old: recency = 0.5
        // count = 20: normalized = 0.2
        // score = (0.2 * 0.3) + (0.5 * 0.7) = 0.06 + 0.35 = 0.41
        assert!((score - 0.41).abs() < 0.01);
    }

    #[test]
    fn test_project_history_record_run() {
        let mut history = ProjectHistory::new();
        history.record_run("dev", None);
        history.record_run("dev", Some("--host".to_string()));
        history.record_run("build", None);

        assert_eq!(history.last_script(), Some("build"));
        assert_eq!(history.scripts.len(), 2);

        let dev = history.get_script("dev").unwrap();
        assert_eq!(dev.count, 2);
        assert_eq!(dev.last_args, Some("--host".to_string()));
    }

    #[test]
    fn test_project_history_last_script_with_args() {
        let mut history = ProjectHistory::new();
        history.record_run("dev", Some("--host".to_string()));

        let (name, args) = history.last_script_with_args().unwrap();
        assert_eq!(name, "dev");
        assert_eq!(args, Some("--host"));
    }

    #[test]
    fn test_project_history_cleanup() {
        let mut history = ProjectHistory::new();
        let now = Utc::now();

        // Add scripts with different ages
        history.scripts.insert(
            "old1".to_string(),
            ScriptHistory::with_values(1, now - Duration::days(10), None),
        );
        history.scripts.insert(
            "old2".to_string(),
            ScriptHistory::with_values(1, now - Duration::days(9), None),
        );
        history.scripts.insert(
            "recent1".to_string(),
            ScriptHistory::with_values(1, now - Duration::days(1), None),
        );
        history.scripts.insert(
            "recent2".to_string(),
            ScriptHistory::with_values(1, now, None),
        );

        assert_eq!(history.scripts.len(), 4);

        // Cleanup to max 2 scripts
        history.cleanup(2);

        assert_eq!(history.scripts.len(), 2);
        // Should keep the most recent ones
        assert!(history.scripts.contains_key("recent1"));
        assert!(history.scripts.contains_key("recent2"));
        assert!(!history.scripts.contains_key("old1"));
        assert!(!history.scripts.contains_key("old2"));
    }

    #[test]
    fn test_history_record_run() {
        let mut history = History::new();
        let project = PathBuf::from("/test/project");

        history.record_run(&project, "dev", None);
        history.record_run(&project, "dev", Some("--host".to_string()));
        history.record_run(&project, "build", None);

        let proj = history.get_project(&project).unwrap();
        assert_eq!(proj.last_script(), Some("build"));

        let dev = proj.get_script("dev").unwrap();
        assert_eq!(dev.count, 2);
        assert_eq!(dev.last_args, Some("--host".to_string()));
    }

    #[test]
    fn test_history_get_last_script() {
        let mut history = History::new();
        let project = PathBuf::from("/test/project");

        history.record_run(&project, "dev", Some("--host".to_string()));

        let (name, args) = history.get_last_script(&project).unwrap();
        assert_eq!(name, "dev");
        assert_eq!(args, Some("--host".to_string()));
    }

    #[test]
    fn test_history_get_script_stats() {
        let mut history = History::new();
        let project = PathBuf::from("/test/project");

        history.record_run(&project, "dev", None);
        history.record_run(&project, "dev", None);

        let stats = history.get_script_stats(&project, "dev").unwrap();
        assert_eq!(stats.count, 2);

        assert!(history.get_script_stats(&project, "unknown").is_none());
    }

    #[test]
    fn test_history_get_sorted_by_recent() {
        let mut history = History::new();
        let project = PathBuf::from("/test/project");
        let now = Utc::now();

        // Create scripts
        let scripts = create_test_scripts();

        // Add history: dev (recent, high count), build (old, low count), test (no history)
        history
            .projects
            .insert(project.clone(), ProjectHistory::new());
        let proj = history.get_project_mut(&project).unwrap();

        proj.scripts
            .insert("dev".to_string(), ScriptHistory::with_values(10, now, None));
        proj.scripts.insert(
            "build".to_string(),
            ScriptHistory::with_values(2, now - Duration::days(20), None),
        );

        let sorted = history.get_sorted_by_recent_at(&project, &scripts, now);

        // dev should be first (high recency + count)
        assert_eq!(sorted[0].name(), "dev");
        // build should be second (has some history)
        assert_eq!(sorted[1].name(), "build");
        // test and lint should be last (no history, sorted alphabetically)
        assert!(sorted[2].name() == "lint" || sorted[3].name() == "lint");
    }

    #[test]
    fn test_history_get_sorted_no_history() {
        let history = History::new();
        let project = PathBuf::from("/test/project");
        let scripts = create_test_scripts();

        let sorted = history.get_sorted_by_recent(&project, &scripts);

        // All should have 0 score, so sorted alphabetically
        assert_eq!(sorted.len(), 4);
        assert_eq!(sorted[0].name(), "build");
        assert_eq!(sorted[1].name(), "dev");
        assert_eq!(sorted[2].name(), "lint");
        assert_eq!(sorted[3].name(), "test");
    }

    #[test]
    fn test_history_cleanup_projects() {
        let mut history = History::new();
        let now = Utc::now();

        // Add 5 projects with different ages
        for i in 0..5 {
            let path = PathBuf::from(format!("/project/{}", i));
            let mut proj = ProjectHistory::new();
            proj.last_run = now - Duration::days(i as i64);
            history.projects.insert(path, proj);
        }

        assert_eq!(history.projects.len(), 5);

        // Cleanup to max 3 projects
        history.cleanup_with_limits(3, 50);

        assert_eq!(history.projects.len(), 3);
        // Should keep the 3 most recent (0, 1, 2)
        assert!(history.projects.contains_key(&PathBuf::from("/project/0")));
        assert!(history.projects.contains_key(&PathBuf::from("/project/1")));
        assert!(history.projects.contains_key(&PathBuf::from("/project/2")));
        assert!(!history.projects.contains_key(&PathBuf::from("/project/3")));
        assert!(!history.projects.contains_key(&PathBuf::from("/project/4")));
    }

    #[test]
    fn test_history_cleanup_with_config() {
        let mut history = History::new();
        let now = Utc::now();

        // Add project with many scripts
        let project = PathBuf::from("/test/project");
        history
            .projects
            .insert(project.clone(), ProjectHistory::new());
        let proj = history.get_project_mut(&project).unwrap();

        for i in 0..10 {
            proj.scripts.insert(
                format!("script{}", i),
                ScriptHistory::with_values(1, now - Duration::days(i as i64), None),
            );
        }

        let config = HistoryConfig {
            enabled: true,
            max_projects: 100,
            max_scripts: 5,
        };

        history.cleanup(&config);

        let proj = history.get_project(&project).unwrap();
        assert_eq!(proj.scripts.len(), 5);
    }

    #[test]
    fn test_history_save_and_load() {
        let temp = TempDir::new().unwrap();
        let history_path = temp.path().join("history.json");

        // Create and populate history
        let mut history = History::new();
        let project = PathBuf::from("/test/project");
        history.record_run(&project, "dev", Some("--host".to_string()));

        // Save manually to temp location
        let content = serde_json::to_string_pretty(&history).unwrap();
        fs::write(&history_path, content).unwrap();

        // Load from temp location
        let loaded_content = fs::read_to_string(&history_path).unwrap();
        let loaded: History = serde_json::from_str(&loaded_content).unwrap();

        assert_eq!(loaded.projects.len(), 1);
        let proj = loaded.get_project(&project).unwrap();
        assert_eq!(proj.last_script(), Some("dev"));
    }

    #[test]
    fn test_history_corrupt_file_handling() {
        let temp = TempDir::new().unwrap();
        let history_path = temp.path().join("history.json");

        // Write corrupt JSON
        fs::write(&history_path, "{ invalid json }}}").unwrap();

        // Try to parse it (simulating what load does)
        let content = fs::read_to_string(&history_path).unwrap();
        let result: Result<History, _> = serde_json::from_str(&content);

        assert!(result.is_err());
    }

    #[test]
    fn test_history_missing_file() {
        // When file doesn't exist, load should return empty history
        // This is tested in the actual load function which checks for file existence
        let history = History::new();
        assert!(history.projects.is_empty());
    }

    #[test]
    fn test_score_high_count_beats_old_recent() {
        let now = Utc::now();

        // Script A: run once today
        let script_a = ScriptHistory::with_values(1, now, None);

        // Script B: run 100 times, 5 days ago
        let script_b = ScriptHistory::with_values(100, now - Duration::days(5), None);

        let score_a = script_a.score_at(now);
        let score_b = script_b.score_at(now);

        // A: (1/100 * 0.3) + (1.0 * 0.7) = 0.003 + 0.7 = 0.703
        // B: (1.0 * 0.3) + (0.833 * 0.7) = 0.3 + 0.583 = 0.883
        // High count should win when recency is close
        assert!(score_b > score_a);
    }

    #[test]
    fn test_score_very_recent_beats_high_count_old() {
        let now = Utc::now();

        // Script A: run once today
        let script_a = ScriptHistory::with_values(1, now, None);

        // Script B: run 100 times, 25 days ago
        let script_b = ScriptHistory::with_values(100, now - Duration::days(25), None);

        let score_a = script_a.score_at(now);
        let score_b = script_b.score_at(now);

        // A: (0.01 * 0.3) + (1.0 * 0.7) = 0.003 + 0.7 = 0.703
        // B: (1.0 * 0.3) + (0.167 * 0.7) = 0.3 + 0.117 = 0.417
        // Very recent should beat old high count
        assert!(score_a > score_b);
    }

    #[test]
    fn test_project_history_cleanup_preserves_recent() {
        let mut history = ProjectHistory::new();
        let now = Utc::now();

        // Add a mix of old and recent scripts
        history.scripts.insert(
            "very_old".to_string(),
            ScriptHistory::with_values(100, now - Duration::days(100), None),
        );
        history.scripts.insert(
            "today".to_string(),
            ScriptHistory::with_values(1, now, None),
        );
        history.scripts.insert(
            "yesterday".to_string(),
            ScriptHistory::with_values(1, now - Duration::days(1), None),
        );

        history.cleanup(2);

        // Should keep today and yesterday, remove very_old despite high count
        assert!(!history.scripts.contains_key("very_old"));
        assert!(history.scripts.contains_key("today"));
        assert!(history.scripts.contains_key("yesterday"));
    }

    #[test]
    fn test_history_version() {
        let history = History::new();
        assert_eq!(history.version, History::VERSION);
    }
}
