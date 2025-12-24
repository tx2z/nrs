//! Package manager detection and command building.
//!
//! Detects the appropriate package manager for a project based on:
//! 1. `packageManager` field in package.json (highest priority)
//! 2. Lock file detection
//! 3. Fallback to npm

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Supported package managers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Runner {
    /// Node Package Manager (npm)
    #[default]
    Npm,
    /// Yarn package manager
    Yarn,
    /// pnpm - Fast, disk space efficient package manager
    Pnpm,
    /// Bun - Fast all-in-one JavaScript runtime
    Bun,
}

impl Runner {
    /// Get the executable name for this runner.
    pub fn executable(&self) -> &'static str {
        match self {
            Runner::Npm => "npm",
            Runner::Yarn => "yarn",
            Runner::Pnpm => "pnpm",
            Runner::Bun => "bun",
        }
    }

    /// Get the base run command (without script name).
    ///
    /// Returns the command prefix used to run scripts:
    /// - npm: "npm run"
    /// - yarn: "yarn"
    /// - pnpm: "pnpm"
    /// - bun: "bun run"
    pub fn run_prefix(&self) -> &'static str {
        match self {
            Runner::Npm => "npm run",
            Runner::Yarn => "yarn",
            Runner::Pnpm => "pnpm",
            Runner::Bun => "bun run",
        }
    }

    /// Get the command to run a script as a vector of arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use nrs::package::Runner;
    ///
    /// let cmd = Runner::Npm.run_command("dev");
    /// assert_eq!(cmd, vec!["npm", "run", "dev"]);
    ///
    /// let cmd = Runner::Yarn.run_command("build");
    /// assert_eq!(cmd, vec!["yarn", "build"]);
    /// ```
    pub fn run_command(&self, script: &str) -> Vec<String> {
        match self {
            Runner::Npm => vec!["npm".into(), "run".into(), script.into()],
            Runner::Yarn => vec!["yarn".into(), script.into()],
            Runner::Pnpm => vec!["pnpm".into(), script.into()],
            Runner::Bun => vec!["bun".into(), "run".into(), script.into()],
        }
    }

    /// Get the command to run a script with additional arguments.
    ///
    /// # Arguments
    ///
    /// * `script` - The script name to run
    /// * `args` - Additional arguments to pass to the script
    ///
    /// # Examples
    ///
    /// ```
    /// use nrs::package::Runner;
    ///
    /// let args = vec!["--watch".to_string(), "--coverage".to_string()];
    /// let cmd = Runner::Npm.run_command_with_args("test", &args);
    /// assert_eq!(cmd, vec!["npm", "run", "test", "--", "--watch", "--coverage"]);
    /// ```
    pub fn run_command_with_args(&self, script: &str, args: &[String]) -> Vec<String> {
        let mut cmd = self.run_command(script);

        if !args.is_empty() {
            // npm and pnpm require -- before args to pass them to the script
            if matches!(self, Runner::Npm | Runner::Pnpm) {
                cmd.push("--".into());
            }
            cmd.extend(args.iter().cloned());
        }

        cmd
    }

    /// Format the run command as a string for display.
    ///
    /// # Examples
    ///
    /// ```
    /// use nrs::package::Runner;
    ///
    /// assert_eq!(Runner::Npm.format_command("dev"), "npm run dev");
    /// assert_eq!(Runner::Yarn.format_command("build"), "yarn build");
    /// ```
    pub fn format_command(&self, script: &str) -> String {
        self.run_command(script).join(" ")
    }

    /// Format the run command with arguments as a string for display.
    ///
    /// # Examples
    ///
    /// ```
    /// use nrs::package::Runner;
    ///
    /// let args = vec!["--watch".to_string()];
    /// assert_eq!(
    ///     Runner::Npm.format_command_with_args("test", &args),
    ///     "npm run test -- --watch"
    /// );
    /// ```
    pub fn format_command_with_args(&self, script: &str, args: &[String]) -> String {
        self.run_command_with_args(script, args).join(" ")
    }

    /// Get the display name for the runner.
    pub fn display_name(&self) -> &'static str {
        match self {
            Runner::Npm => "npm",
            Runner::Yarn => "yarn",
            Runner::Pnpm => "pnpm",
            Runner::Bun => "bun",
        }
    }

    /// Get the icon/emoji for the runner.
    pub fn icon(&self) -> &'static str {
        match self {
            Runner::Npm => "\u{1F4E6}",  // 📦 package
            Runner::Yarn => "\u{1F9F6}", // 🧶 yarn
            Runner::Pnpm => "\u{1F4C0}", // 📀 disc
            Runner::Bun => "\u{1F95F}",  // 🥟 dumpling
        }
    }

    /// Get the lock file name for this runner.
    pub fn lock_file(&self) -> &'static str {
        match self {
            Runner::Npm => "package-lock.json",
            Runner::Yarn => "yarn.lock",
            Runner::Pnpm => "pnpm-lock.yaml",
            Runner::Bun => "bun.lockb",
        }
    }

    /// Get all supported runners.
    pub fn all() -> &'static [Runner] {
        &[Runner::Npm, Runner::Yarn, Runner::Pnpm, Runner::Bun]
    }

    /// Get the command to run a workspace script.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace/package name
    /// * `script` - The script name to run
    pub fn workspace_command(&self, workspace: &str, script: &str) -> Vec<String> {
        match self {
            Runner::Npm => vec![
                "npm".into(),
                "run".into(),
                "-w".into(),
                workspace.into(),
                script.into(),
            ],
            Runner::Yarn => vec![
                "yarn".into(),
                "workspace".into(),
                workspace.into(),
                script.into(),
            ],
            Runner::Pnpm => vec![
                "pnpm".into(),
                "--filter".into(),
                workspace.into(),
                script.into(),
            ],
            Runner::Bun => vec![
                "bun".into(),
                "run".into(),
                "--filter".into(),
                workspace.into(),
                script.into(),
            ],
        }
    }

    /// Get the command to run a workspace script with additional arguments.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace/package name
    /// * `script` - The script name to run
    /// * `args` - Additional arguments to pass to the script
    pub fn workspace_command_with_args(
        &self,
        workspace: &str,
        script: &str,
        args: &[String],
    ) -> Vec<String> {
        let mut cmd = self.workspace_command(workspace, script);

        if !args.is_empty() {
            // npm and pnpm require -- before args
            if matches!(self, Runner::Npm | Runner::Pnpm) {
                cmd.push("--".into());
            }
            cmd.extend(args.iter().cloned());
        }

        cmd
    }

    /// Format the workspace run command as a string for display.
    pub fn format_workspace_command(&self, workspace: &str, script: &str) -> String {
        self.workspace_command(workspace, script).join(" ")
    }

    /// Format the workspace run command with arguments as a string for display.
    pub fn format_workspace_command_with_args(
        &self,
        workspace: &str,
        script: &str,
        args: &[String],
    ) -> String {
        self.workspace_command_with_args(workspace, script, args)
            .join(" ")
    }
}

impl std::fmt::Display for Runner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for Runner {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "npm" => Ok(Runner::Npm),
            "yarn" => Ok(Runner::Yarn),
            "pnpm" => Ok(Runner::Pnpm),
            "bun" => Ok(Runner::Bun),
            _ => Err(format!(
                "Unknown package manager: '{s}'. Valid options are: npm, yarn, pnpm, bun"
            )),
        }
    }
}

/// Detect the package manager for a project.
///
/// Detection priority:
/// 1. `packageManager` field in package.json (e.g., "pnpm@8.0.0")
/// 2. Lock file detection:
///    - `bun.lockb` → Bun
///    - `pnpm-lock.yaml` → pnpm
///    - `yarn.lock` → Yarn
///    - `package-lock.json` → npm
/// 3. Fallback to npm
///
/// # Arguments
///
/// * `project_dir` - Path to the project directory
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use nrs::package::detect_runner;
///
/// let runner = detect_runner(Path::new("/path/to/project"));
/// println!("Using: {}", runner);
/// ```
pub fn detect_runner(project_dir: &Path) -> Runner {
    detect_runner_reason(project_dir).0
}

/// Detect the package manager for a project and return the reason for the detection.
///
/// Returns a tuple of (Runner, reason_string) where reason_string explains
/// why this package manager was selected.
///
/// # Arguments
///
/// * `project_dir` - Path to the project directory
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use nrs::package::detect_runner_reason;
///
/// let (runner, reason) = detect_runner_reason(Path::new("/path/to/project"));
/// println!("Using: {} ({})", runner, reason);
/// ```
pub fn detect_runner_reason(project_dir: &Path) -> (Runner, String) {
    // Priority 1: Check packageManager field in package.json
    if let Some(runner) = detect_from_package_json(project_dir) {
        let package_json = project_dir.join("package.json");
        return (
            runner,
            format!("packageManager field in {}", package_json.display()),
        );
    }

    // Priority 2: Check lock files (in order of specificity)
    // Bun first as it's the most specific (binary format)
    let bun_lock = project_dir.join("bun.lockb");
    if bun_lock.exists() {
        return (Runner::Bun, format!("found {}", bun_lock.display()));
    }

    let pnpm_lock = project_dir.join("pnpm-lock.yaml");
    if pnpm_lock.exists() {
        return (Runner::Pnpm, format!("found {}", pnpm_lock.display()));
    }

    let yarn_lock = project_dir.join("yarn.lock");
    if yarn_lock.exists() {
        return (Runner::Yarn, format!("found {}", yarn_lock.display()));
    }

    let npm_lock = project_dir.join("package-lock.json");
    if npm_lock.exists() {
        return (Runner::Npm, format!("found {}", npm_lock.display()));
    }

    // Priority 3: Fallback to npm
    (Runner::Npm, "default (no lock file found)".to_string())
}

/// Detect the package manager from the packageManager field in package.json.
///
/// The packageManager field can be in formats like:
/// - "pnpm@8.0.0"
/// - "yarn@4.0.0"
/// - "npm@10.0.0"
/// - "pnpm" (without version)
fn detect_from_package_json(project_dir: &Path) -> Option<Runner> {
    let package_json = project_dir.join("package.json");
    let content = std::fs::read_to_string(package_json).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let pm = json.get("packageManager")?.as_str()?;

    // Parse "pnpm@8.0.0" or "pnpm" format
    parse_package_manager_field(pm)
}

/// Parse the packageManager field value to extract the runner.
///
/// Handles formats like:
/// - "pnpm@8.0.0"
/// - "yarn@4.0.0+sha256.abc123"
/// - "npm"
fn parse_package_manager_field(value: &str) -> Option<Runner> {
    // Split on @ to get the package manager name
    let name = value.split('@').next()?;
    name.parse().ok()
}

/// Check if a specific lock file exists in the project directory.
pub fn has_lock_file(project_dir: &Path, runner: Runner) -> bool {
    project_dir.join(runner.lock_file()).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ==================== Runner enum tests ====================

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
        assert_eq!("Pnpm".parse::<Runner>().unwrap(), Runner::Pnpm);
        assert_eq!("BUN".parse::<Runner>().unwrap(), Runner::Bun);
    }

    #[test]
    fn test_runner_from_str_invalid() {
        let result = "invalid".parse::<Runner>();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown package manager"));
    }

    #[test]
    fn test_runner_display() {
        assert_eq!(format!("{}", Runner::Npm), "npm");
        assert_eq!(format!("{}", Runner::Yarn), "yarn");
        assert_eq!(format!("{}", Runner::Pnpm), "pnpm");
        assert_eq!(format!("{}", Runner::Bun), "bun");
    }

    #[test]
    fn test_runner_executable() {
        assert_eq!(Runner::Npm.executable(), "npm");
        assert_eq!(Runner::Yarn.executable(), "yarn");
        assert_eq!(Runner::Pnpm.executable(), "pnpm");
        assert_eq!(Runner::Bun.executable(), "bun");
    }

    #[test]
    fn test_runner_lock_file() {
        assert_eq!(Runner::Npm.lock_file(), "package-lock.json");
        assert_eq!(Runner::Yarn.lock_file(), "yarn.lock");
        assert_eq!(Runner::Pnpm.lock_file(), "pnpm-lock.yaml");
        assert_eq!(Runner::Bun.lock_file(), "bun.lockb");
    }

    #[test]
    fn test_runner_all() {
        let all = Runner::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&Runner::Npm));
        assert!(all.contains(&Runner::Yarn));
        assert!(all.contains(&Runner::Pnpm));
        assert!(all.contains(&Runner::Bun));
    }

    // ==================== Run command tests ====================

    #[test]
    fn test_run_command_npm() {
        assert_eq!(Runner::Npm.run_command("dev"), vec!["npm", "run", "dev"]);
        assert_eq!(
            Runner::Npm.run_command("build:prod"),
            vec!["npm", "run", "build:prod"]
        );
    }

    #[test]
    fn test_run_command_yarn() {
        assert_eq!(Runner::Yarn.run_command("dev"), vec!["yarn", "dev"]);
        assert_eq!(Runner::Yarn.run_command("test"), vec!["yarn", "test"]);
    }

    #[test]
    fn test_run_command_pnpm() {
        assert_eq!(Runner::Pnpm.run_command("dev"), vec!["pnpm", "dev"]);
        assert_eq!(Runner::Pnpm.run_command("build"), vec!["pnpm", "build"]);
    }

    #[test]
    fn test_run_command_bun() {
        assert_eq!(Runner::Bun.run_command("dev"), vec!["bun", "run", "dev"]);
        assert_eq!(
            Runner::Bun.run_command("start"),
            vec!["bun", "run", "start"]
        );
    }

    // ==================== Run command with args tests ====================

    #[test]
    fn test_run_command_with_args_npm() {
        let args = vec!["--watch".to_string()];
        assert_eq!(
            Runner::Npm.run_command_with_args("test", &args),
            vec!["npm", "run", "test", "--", "--watch"]
        );

        let args = vec!["--coverage".to_string(), "--verbose".to_string()];
        assert_eq!(
            Runner::Npm.run_command_with_args("test", &args),
            vec!["npm", "run", "test", "--", "--coverage", "--verbose"]
        );
    }

    #[test]
    fn test_run_command_with_args_yarn() {
        let args = vec!["--watch".to_string()];
        assert_eq!(
            Runner::Yarn.run_command_with_args("test", &args),
            vec!["yarn", "test", "--watch"]
        );
    }

    #[test]
    fn test_run_command_with_args_pnpm() {
        let args = vec!["--watch".to_string()];
        assert_eq!(
            Runner::Pnpm.run_command_with_args("test", &args),
            vec!["pnpm", "test", "--", "--watch"]
        );
    }

    #[test]
    fn test_run_command_with_args_bun() {
        let args = vec!["--watch".to_string()];
        assert_eq!(
            Runner::Bun.run_command_with_args("test", &args),
            vec!["bun", "run", "test", "--watch"]
        );
    }

    #[test]
    fn test_run_command_with_empty_args() {
        let args: Vec<String> = vec![];
        assert_eq!(
            Runner::Npm.run_command_with_args("dev", &args),
            vec!["npm", "run", "dev"]
        );
        assert_eq!(
            Runner::Yarn.run_command_with_args("dev", &args),
            vec!["yarn", "dev"]
        );
    }

    // ==================== Format command tests ====================

    #[test]
    fn test_format_command() {
        assert_eq!(Runner::Npm.format_command("dev"), "npm run dev");
        assert_eq!(Runner::Yarn.format_command("build"), "yarn build");
        assert_eq!(Runner::Pnpm.format_command("test"), "pnpm test");
        assert_eq!(Runner::Bun.format_command("start"), "bun run start");
    }

    #[test]
    fn test_format_command_with_args() {
        let args = vec!["--watch".to_string(), "--coverage".to_string()];
        assert_eq!(
            Runner::Npm.format_command_with_args("test", &args),
            "npm run test -- --watch --coverage"
        );
        assert_eq!(
            Runner::Yarn.format_command_with_args("test", &args),
            "yarn test --watch --coverage"
        );
    }

    // ==================== Workspace command tests ====================

    #[test]
    fn test_workspace_command_npm() {
        assert_eq!(
            Runner::Npm.workspace_command("@app/web", "build"),
            vec!["npm", "run", "-w", "@app/web", "build"]
        );
    }

    #[test]
    fn test_workspace_command_yarn() {
        assert_eq!(
            Runner::Yarn.workspace_command("@app/web", "build"),
            vec!["yarn", "workspace", "@app/web", "build"]
        );
    }

    #[test]
    fn test_workspace_command_pnpm() {
        assert_eq!(
            Runner::Pnpm.workspace_command("@app/web", "build"),
            vec!["pnpm", "--filter", "@app/web", "build"]
        );
    }

    #[test]
    fn test_workspace_command_bun() {
        assert_eq!(
            Runner::Bun.workspace_command("@app/web", "build"),
            vec!["bun", "run", "--filter", "@app/web", "build"]
        );
    }

    // ==================== Detection tests ====================

    #[test]
    fn test_detect_from_package_manager_field() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("package.json"),
            r#"{"packageManager": "pnpm@8.15.0"}"#,
        )
        .unwrap();

        assert_eq!(detect_runner(temp.path()), Runner::Pnpm);
    }

    #[test]
    fn test_detect_from_package_manager_field_with_hash() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("package.json"),
            r#"{"packageManager": "yarn@4.0.0+sha256.abc123"}"#,
        )
        .unwrap();

        assert_eq!(detect_runner(temp.path()), Runner::Yarn);
    }

    #[test]
    fn test_detect_from_package_manager_field_no_version() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("package.json"),
            r#"{"packageManager": "bun"}"#,
        )
        .unwrap();

        assert_eq!(detect_runner(temp.path()), Runner::Bun);
    }

    #[test]
    fn test_detect_from_bun_lock() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        fs::write(temp.path().join("bun.lockb"), "binary content").unwrap();

        assert_eq!(detect_runner(temp.path()), Runner::Bun);
    }

    #[test]
    fn test_detect_from_pnpm_lock() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        fs::write(temp.path().join("pnpm-lock.yaml"), "lockfileVersion: 5.4").unwrap();

        assert_eq!(detect_runner(temp.path()), Runner::Pnpm);
    }

    #[test]
    fn test_detect_from_yarn_lock() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        fs::write(temp.path().join("yarn.lock"), "# yarn lockfile v1").unwrap();

        assert_eq!(detect_runner(temp.path()), Runner::Yarn);
    }

    #[test]
    fn test_detect_from_npm_lock() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        fs::write(
            temp.path().join("package-lock.json"),
            r#"{"lockfileVersion": 3}"#,
        )
        .unwrap();

        assert_eq!(detect_runner(temp.path()), Runner::Npm);
    }

    #[test]
    fn test_detect_fallback_to_npm() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        assert_eq!(detect_runner(temp.path()), Runner::Npm);
    }

    #[test]
    fn test_detect_priority_package_manager_over_lock_file() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("package.json"),
            r#"{"packageManager": "pnpm@8.0.0"}"#,
        )
        .unwrap();
        // Create a yarn lock file that should be ignored
        fs::write(temp.path().join("yarn.lock"), "").unwrap();

        assert_eq!(detect_runner(temp.path()), Runner::Pnpm);
    }

    #[test]
    fn test_detect_bun_priority_over_other_lock_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        // Create multiple lock files
        fs::write(temp.path().join("bun.lockb"), "").unwrap();
        fs::write(temp.path().join("yarn.lock"), "").unwrap();
        fs::write(temp.path().join("package-lock.json"), "{}").unwrap();

        // Bun should win as it's checked first
        assert_eq!(detect_runner(temp.path()), Runner::Bun);
    }

    #[test]
    fn test_detect_no_package_json() {
        let temp = TempDir::new().unwrap();
        // No package.json, no lock files
        assert_eq!(detect_runner(temp.path()), Runner::Npm);
    }

    // ==================== Parse package manager field tests ====================

    #[test]
    fn test_parse_package_manager_field_with_version() {
        assert_eq!(
            parse_package_manager_field("pnpm@8.15.0"),
            Some(Runner::Pnpm)
        );
        assert_eq!(
            parse_package_manager_field("yarn@4.0.0"),
            Some(Runner::Yarn)
        );
        assert_eq!(parse_package_manager_field("npm@10.2.0"), Some(Runner::Npm));
        assert_eq!(parse_package_manager_field("bun@1.0.0"), Some(Runner::Bun));
    }

    #[test]
    fn test_parse_package_manager_field_without_version() {
        assert_eq!(parse_package_manager_field("pnpm"), Some(Runner::Pnpm));
        assert_eq!(parse_package_manager_field("yarn"), Some(Runner::Yarn));
        assert_eq!(parse_package_manager_field("npm"), Some(Runner::Npm));
        assert_eq!(parse_package_manager_field("bun"), Some(Runner::Bun));
    }

    #[test]
    fn test_parse_package_manager_field_with_hash() {
        assert_eq!(
            parse_package_manager_field("yarn@4.0.0+sha256.abc123def"),
            Some(Runner::Yarn)
        );
    }

    #[test]
    fn test_parse_package_manager_field_invalid() {
        assert_eq!(parse_package_manager_field("unknown@1.0.0"), None);
        assert_eq!(parse_package_manager_field(""), None);
    }

    // ==================== Has lock file tests ====================

    #[test]
    fn test_has_lock_file() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("yarn.lock"), "").unwrap();

        assert!(has_lock_file(temp.path(), Runner::Yarn));
        assert!(!has_lock_file(temp.path(), Runner::Npm));
        assert!(!has_lock_file(temp.path(), Runner::Pnpm));
        assert!(!has_lock_file(temp.path(), Runner::Bun));
    }
}
