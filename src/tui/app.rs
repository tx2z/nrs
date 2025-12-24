//! Application state for the TUI.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::{Config, SortMode};
use crate::history::History;
use crate::package::{Runner, Script, Scripts, Workspace};

/// Minimum column width for script items.
const MIN_COLUMN_WIDTH: u16 = 28;

/// Application mode/state.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AppMode {
    /// Normal navigation mode.
    #[default]
    Normal,
    /// Filter/search mode.
    Filter { query: String },
    /// Multi-select mode.
    MultiSelect { selected: HashSet<usize> },
    /// Help overlay.
    Help,
    /// Error display.
    Error { message: String },
    /// Arguments input mode.
    Args { script_index: usize, input: String },
    /// Workspace selection mode (for monorepos).
    WorkspaceSelect,
}

/// Currently selected workspace context.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceContext {
    /// Root scripts (no specific workspace selected).
    Root,
    /// A specific workspace is selected.
    Workspace(usize),
}

/// Information about a script to run after TUI exits.
#[derive(Debug, Clone)]
pub struct ScriptRun {
    /// The script to run.
    pub script: Script,
    /// Optional arguments to pass to the script.
    pub args: Option<String>,
    /// Workspace name if running from a specific workspace.
    pub workspace: Option<String>,
    /// Workspace path if running from a specific workspace.
    pub workspace_path: Option<PathBuf>,
}

impl std::fmt::Display for ScriptRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = if let Some(ws) = &self.workspace {
            format!("{} > ", ws)
        } else {
            String::new()
        };

        if let Some(args) = &self.args {
            write!(f, "{}{} {}", prefix, self.script.name(), args)
        } else {
            write!(f, "{}{}", prefix, self.script.name())
        }
    }
}

/// Main application state.
pub struct App {
    // Data
    /// All available scripts.
    scripts: Scripts,
    /// Current configuration.
    config: Config,
    /// Execution history.
    history: History,
    /// Detected package manager.
    runner: Runner,
    /// Project name.
    project_name: String,
    /// Project path.
    project_path: PathBuf,

    // Workspace data
    /// Whether this is a monorepo.
    is_monorepo: bool,
    /// Available workspaces (if monorepo).
    workspaces: Vec<Workspace>,
    /// Currently selected workspace context.
    workspace_context: WorkspaceContext,
    /// Selected workspace index (for workspace selector).
    workspace_selected: usize,

    // UI State
    /// Current application mode.
    mode: AppMode,
    /// Currently selected script index (within visible_indices).
    selected: usize,
    /// Scroll offset for the scripts list.
    scroll_offset: usize,
    /// Current filter text.
    filter_text: String,
    /// Current sort mode.
    sort_mode: SortMode,

    // Computed (cached)
    /// Indices of visible scripts (after filtering and sorting).
    visible_indices: Vec<usize>,
    /// Number of columns in the grid.
    columns: usize,
    /// Should the app quit.
    should_quit: bool,
    /// Script to run after exit.
    script_to_run: Option<ScriptRun>,
}

impl App {
    /// Create a new application.
    pub fn new(
        scripts: Scripts,
        config: Config,
        history: History,
        project_name: String,
        project_path: PathBuf,
        runner: Runner,
    ) -> Self {
        Self::with_workspaces(
            scripts,
            config,
            history,
            project_name,
            project_path,
            runner,
            Vec::new(),
        )
    }

    /// Create a new application with workspace support.
    pub fn with_workspaces(
        scripts: Scripts,
        config: Config,
        history: History,
        project_name: String,
        project_path: PathBuf,
        runner: Runner,
        workspaces: Vec<Workspace>,
    ) -> Self {
        let sort_mode = config.general.default_sort;
        let visible_indices: Vec<usize> = (0..scripts.len()).collect();
        let is_monorepo = !workspaces.is_empty();

        // Start in workspace select mode if this is a monorepo with workspaces
        let initial_mode = if is_monorepo {
            AppMode::WorkspaceSelect
        } else {
            AppMode::Normal
        };

        let mut app = Self {
            scripts,
            config,
            history,
            runner,
            project_name,
            project_path,
            is_monorepo,
            workspaces,
            workspace_context: WorkspaceContext::Root,
            workspace_selected: 0,
            mode: initial_mode,
            selected: 0,
            scroll_offset: 0,
            filter_text: String::new(),
            sort_mode,
            visible_indices,
            columns: 1,
            should_quit: false,
            script_to_run: None,
        };

        // Initial sort based on default sort mode
        app.update_visible_scripts();
        app
    }

    // ==================== Getters ====================

    /// Get the current mode.
    pub fn mode(&self) -> &AppMode {
        &self.mode
    }

    /// Check if the app should quit.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Get the script to run after exit.
    pub fn script_to_run(&self) -> Option<&ScriptRun> {
        self.script_to_run.as_ref()
    }

    /// Get the project name.
    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    /// Get the project path.
    pub fn project_path(&self) -> &PathBuf {
        &self.project_path
    }

    /// Get the runner.
    pub fn runner(&self) -> Runner {
        self.runner
    }

    /// Get all scripts.
    pub fn scripts(&self) -> &Scripts {
        &self.scripts
    }

    /// Get the current filter text.
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Get the current sort mode.
    pub fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    /// Get the number of columns.
    pub fn columns(&self) -> usize {
        self.columns
    }

    /// Get the selected index.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Get the scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get the number of visible scripts.
    pub fn visible_count(&self) -> usize {
        self.visible_indices.len()
    }

    /// Get the config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    // ==================== Workspace Getters ====================

    /// Check if this is a monorepo.
    pub fn is_monorepo(&self) -> bool {
        self.is_monorepo
    }

    /// Get the available workspaces.
    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }

    /// Get the current workspace context.
    pub fn workspace_context(&self) -> &WorkspaceContext {
        &self.workspace_context
    }

    /// Get the selected workspace index in the workspace selector.
    pub fn workspace_selected(&self) -> usize {
        self.workspace_selected
    }

    /// Get the currently selected workspace (if any).
    pub fn current_workspace(&self) -> Option<&Workspace> {
        match &self.workspace_context {
            WorkspaceContext::Root => None,
            WorkspaceContext::Workspace(idx) => self.workspaces.get(*idx),
        }
    }

    /// Get the breadcrumb path for display.
    /// Returns something like "monorepo > packages/web > scripts"
    pub fn breadcrumb(&self) -> String {
        match &self.workspace_context {
            WorkspaceContext::Root => self.project_name.clone(),
            WorkspaceContext::Workspace(idx) => {
                if let Some(ws) = self.workspaces.get(*idx) {
                    format!("{} > {}", self.project_name, ws.name())
                } else {
                    self.project_name.clone()
                }
            }
        }
    }

    // ==================== Script Access ====================

    /// Get visible scripts after filtering and sorting.
    pub fn visible_scripts(&self) -> Vec<&Script> {
        self.visible_indices
            .iter()
            .filter_map(|&i| self.scripts.iter().nth(i))
            .collect()
    }

    /// Get the currently selected script.
    pub fn selected_script(&self) -> Option<&Script> {
        self.visible_indices
            .get(self.selected)
            .and_then(|&i| self.scripts.iter().nth(i))
    }

    /// Get a visible script by its display index (0-based).
    pub fn get_visible_script(&self, index: usize) -> Option<&Script> {
        self.visible_indices
            .get(index)
            .and_then(|&i| self.scripts.iter().nth(i))
    }

    // ==================== Mode Management ====================

    /// Set the application mode.
    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    /// Toggle filter mode.
    pub fn toggle_filter_mode(&mut self) {
        match &self.mode {
            AppMode::Filter { .. } => {
                self.mode = AppMode::Normal;
                self.filter_text.clear();
                self.update_visible_scripts();
            }
            AppMode::Normal => {
                self.mode = AppMode::Filter {
                    query: String::new(),
                };
            }
            _ => {}
        }
    }

    /// Enter args input mode for the selected script.
    pub fn enter_args_mode(&mut self) {
        if self.selected < self.visible_indices.len() {
            self.mode = AppMode::Args {
                script_index: self.selected,
                input: String::new(),
            };
        }
    }

    /// Toggle multi-select mode.
    pub fn toggle_multi_select(&mut self) {
        match &self.mode {
            AppMode::MultiSelect { .. } => {
                self.mode = AppMode::Normal;
            }
            AppMode::Normal => {
                self.mode = AppMode::MultiSelect {
                    selected: HashSet::new(),
                };
            }
            _ => {}
        }
    }

    /// Toggle help overlay.
    pub fn toggle_help(&mut self) {
        match self.mode {
            AppMode::Help => {
                self.mode = AppMode::Normal;
            }
            _ => {
                self.mode = AppMode::Help;
            }
        }
    }

    // ==================== Workspace Management ====================

    /// Enter workspace selection mode.
    pub fn enter_workspace_select(&mut self) {
        if self.is_monorepo {
            self.mode = AppMode::WorkspaceSelect;
            self.workspace_selected = 0;
        }
    }

    /// Exit workspace selection and go to scripts.
    pub fn exit_workspace_select(&mut self) {
        self.mode = AppMode::Normal;
        self.selected = 0;
        self.update_visible_scripts();
    }

    /// Select a workspace and show its scripts.
    pub fn select_workspace(&mut self, index: usize) {
        // Index 0 is "root", indices 1+ are workspaces
        if index == 0 {
            self.workspace_context = WorkspaceContext::Root;
            // Keep root scripts
        } else if let Some(workspace) = self.workspaces.get(index - 1) {
            self.workspace_context = WorkspaceContext::Workspace(index - 1);
            // Load workspace scripts
            self.scripts = Scripts::from_vec(workspace.scripts().to_vec());
        }

        self.selected = 0;
        self.mode = AppMode::Normal;
        self.update_visible_scripts();
    }

    /// Select the currently highlighted workspace.
    pub fn select_current_workspace(&mut self) {
        self.select_workspace(self.workspace_selected);
    }

    /// Go back to workspace selection from script view.
    pub fn back_to_workspace_select(&mut self) {
        if self.is_monorepo {
            self.mode = AppMode::WorkspaceSelect;
        }
    }

    /// Move workspace selection up.
    pub fn workspace_move_up(&mut self) {
        if self.workspace_selected > 0 {
            self.workspace_selected -= 1;
        }
    }

    /// Move workspace selection down.
    pub fn workspace_move_down(&mut self) {
        // +1 for the "root" option
        let max_index = self.workspaces.len();
        if self.workspace_selected < max_index {
            self.workspace_selected += 1;
        }
    }

    /// Move workspace selection left (in grid).
    pub fn workspace_move_left(&mut self) {
        if self.workspace_selected > 0 {
            self.workspace_selected -= 1;
        }
    }

    /// Move workspace selection right (in grid).
    pub fn workspace_move_right(&mut self) {
        let max_index = self.workspaces.len();
        if self.workspace_selected < max_index {
            self.workspace_selected += 1;
        }
    }

    /// Select workspace by number (1-9).
    pub fn select_workspace_by_number(&mut self, num: usize) {
        // num 1 = root (index 0), num 2 = first workspace (index 1), etc.
        if num > 0 && num <= self.workspaces.len() + 1 {
            self.select_workspace(num - 1);
        }
    }

    /// Get the count of items in workspace selector (root + workspaces).
    pub fn workspace_count(&self) -> usize {
        self.workspaces.len() + 1 // +1 for root
    }

    // ==================== Filter Management ====================

    /// Set the filter text.
    pub fn set_filter(&mut self, text: String) {
        self.filter_text = text.clone();
        self.mode = AppMode::Filter { query: text };
        self.update_visible_scripts();
    }

    /// Append a character to the filter text.
    pub fn push_filter_char(&mut self, c: char) {
        self.filter_text.push(c);
        self.update_visible_scripts();
    }

    /// Remove the last character from the filter text.
    pub fn pop_filter_char(&mut self) {
        self.filter_text.pop();
        self.update_visible_scripts();
    }

    /// Clear the filter text.
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.update_visible_scripts();
    }

    // ==================== Sort Management ====================

    /// Cycle through sort modes: Recent -> Alphabetical -> Category -> Recent.
    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Recent => SortMode::Alpha,
            SortMode::Alpha => SortMode::Category,
            SortMode::Category => SortMode::Recent,
        };
        self.update_visible_scripts();
    }

    /// Set the sort mode.
    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
        self.update_visible_scripts();
    }

    // ==================== Visibility Update ====================

    /// Update the visible scripts based on current filter and sort mode.
    pub fn update_visible_scripts(&mut self) {
        // Step 1: Filter
        let filtered_indices: Vec<usize> = if self.filter_text.is_empty() {
            (0..self.scripts.len()).collect()
        } else {
            // Use the optimized filter_scripts that returns (index, score) pairs
            let matches = crate::filter::filter_scripts(
                &self.filter_text,
                self.scripts.as_slice(),
                self.config.filter.search_descriptions,
            );

            matches.into_iter().map(|(idx, _score)| idx).collect()
        };

        // Step 2: Sort
        self.visible_indices = self.sort_indices(filtered_indices);

        // Adjust selection if needed
        if self.selected >= self.visible_indices.len() {
            self.selected = self.visible_indices.len().saturating_sub(1);
        }
    }

    /// Sort indices based on current sort mode.
    fn sort_indices(&self, mut indices: Vec<usize>) -> Vec<usize> {
        match self.sort_mode {
            SortMode::Recent => {
                // Sort by history score (recent/frequent first)
                // Clone scripts so we can pass them to get_sorted_by_recent
                let scripts_owned: Vec<Script> = indices
                    .iter()
                    .filter_map(|&i| self.scripts.iter().nth(i).cloned())
                    .collect();

                let sorted = self
                    .history
                    .get_sorted_by_recent(&self.project_path, &scripts_owned);

                // Map back to indices
                sorted
                    .iter()
                    .filter_map(|s| {
                        self.scripts
                            .iter()
                            .position(|script| script.name() == s.name())
                    })
                    .filter(|i| indices.contains(i))
                    .collect()
            }
            SortMode::Alpha => {
                // Sort alphabetically by name
                indices.sort_by(|&a, &b| {
                    let name_a = self.scripts.iter().nth(a).map(|s| s.name()).unwrap_or("");
                    let name_b = self.scripts.iter().nth(b).map(|s| s.name()).unwrap_or("");
                    name_a.cmp(name_b)
                });
                indices
            }
            SortMode::Category => {
                // Sort by category (prefix before colon) then alphabetically
                indices.sort_by(|&a, &b| {
                    let name_a = self.scripts.iter().nth(a).map(|s| s.name()).unwrap_or("");
                    let name_b = self.scripts.iter().nth(b).map(|s| s.name()).unwrap_or("");

                    let category_a = name_a.split(':').next().unwrap_or(name_a);
                    let category_b = name_b.split(':').next().unwrap_or(name_b);

                    category_a.cmp(category_b).then_with(|| name_a.cmp(name_b))
                });
                indices
            }
        }
    }

    // ==================== Column Management ====================

    /// Update the number of columns based on terminal width.
    pub fn update_columns(&mut self, width: u16) {
        self.columns = calculate_columns(width);
    }

    // ==================== Navigation ====================

    /// Calculate the number of rows based on visible items and columns.
    fn row_count(&self) -> usize {
        let count = self.visible_indices.len();
        if count == 0 || self.columns == 0 {
            return 0;
        }
        (count + self.columns - 1) / self.columns
    }

    /// Get the current row and column from the selected index.
    fn current_position(&self) -> (usize, usize) {
        let row = self.selected / self.columns;
        let col = self.selected % self.columns;
        (row, col)
    }

    /// Move selection up by one row.
    pub fn move_up(&mut self) {
        if self.visible_indices.is_empty() || self.columns == 0 {
            return;
        }

        let (row, col) = self.current_position();
        if row > 0 {
            let new_index = (row - 1) * self.columns + col;
            if new_index < self.visible_indices.len() {
                self.selected = new_index;
            }
        }
    }

    /// Move selection down by one row.
    pub fn move_down(&mut self) {
        if self.visible_indices.is_empty() || self.columns == 0 {
            return;
        }

        let (row, col) = self.current_position();
        let new_index = (row + 1) * self.columns + col;

        if new_index < self.visible_indices.len() {
            self.selected = new_index;
        } else {
            // Try to go to last row, same column or last item
            let last_row = self.row_count().saturating_sub(1);
            if row < last_row {
                // Go to last item if the target column doesn't exist in last row
                self.selected = self.visible_indices.len().saturating_sub(1);
            }
        }
    }

    /// Move selection left by one column.
    pub fn move_left(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection right by one column.
    pub fn move_right(&mut self) {
        if self.selected < self.visible_indices.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Move selection to the first item.
    pub fn move_to_first(&mut self) {
        self.selected = 0;
    }

    /// Move selection to the last item.
    pub fn move_to_last(&mut self) {
        self.selected = self.visible_indices.len().saturating_sub(1);
    }

    /// Select a script by number (1-9).
    pub fn select_by_number(&mut self, num: usize) {
        if num > 0 && num <= self.visible_indices.len() {
            self.selected = num - 1;
        }
    }

    // ==================== Navigation Aliases (for compatibility) ====================

    /// Move selection up (alias for move_up).
    pub fn select_prev(&mut self) {
        self.move_up();
    }

    /// Move selection down (alias for move_down).
    pub fn select_next(&mut self) {
        self.move_down();
    }

    /// Move to first item (alias for move_to_first).
    pub fn select_first(&mut self) {
        self.move_to_first();
    }

    /// Move to last item (alias for move_to_last).
    pub fn select_last(&mut self) {
        self.move_to_last();
    }

    // ==================== Actions ====================

    /// Request the app to quit.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Run the currently selected script.
    pub fn run_selected(&mut self) -> Option<ScriptRun> {
        if let Some(script) = self.selected_script() {
            let (workspace, workspace_path) = self.get_workspace_info();
            let run = ScriptRun {
                script: script.clone(),
                args: None,
                workspace,
                workspace_path,
            };
            self.script_to_run = Some(run.clone());
            self.should_quit = true;
            Some(run)
        } else {
            None
        }
    }

    /// Get workspace info for script execution.
    fn get_workspace_info(&self) -> (Option<String>, Option<PathBuf>) {
        match &self.workspace_context {
            WorkspaceContext::Root => (None, None),
            WorkspaceContext::Workspace(idx) => {
                if let Some(ws) = self.workspaces.get(*idx) {
                    (Some(ws.name().to_string()), Some(ws.path().to_path_buf()))
                } else {
                    (None, None)
                }
            }
        }
    }

    /// Run a script by number (1-9).
    pub fn run_numbered(&mut self, num: usize) -> Option<ScriptRun> {
        if num > 0 && num <= self.visible_indices.len() {
            self.selected = num - 1;
            self.run_selected()
        } else {
            None
        }
    }

    /// Run a script by number (alias for run_numbered, for compatibility).
    pub fn run_by_number(&mut self, num: usize) {
        self.run_numbered(num);
    }

    /// Run the selected script with arguments.
    pub fn run_with_args(&mut self, args: String) -> Option<ScriptRun> {
        if let Some(script) = self.selected_script() {
            let (workspace, workspace_path) = self.get_workspace_info();
            let run = ScriptRun {
                script: script.clone(),
                args: if args.is_empty() { None } else { Some(args) },
                workspace,
                workspace_path,
            };
            self.script_to_run = Some(run.clone());
            self.should_quit = true;
            Some(run)
        } else {
            None
        }
    }

    /// Toggle selection of current item in multi-select mode.
    pub fn toggle_current_selection(&mut self) {
        if let AppMode::MultiSelect { ref mut selected } = self.mode {
            if selected.contains(&self.selected) {
                selected.remove(&self.selected);
            } else {
                selected.insert(self.selected);
            }
        }
    }

    /// Get selected indices in multi-select mode.
    pub fn multi_selected_indices(&self) -> Option<&HashSet<usize>> {
        if let AppMode::MultiSelect { ref selected } = self.mode {
            Some(selected)
        } else {
            None
        }
    }

    /// Run all selected scripts in multi-select mode.
    pub fn run_multi_selected(&mut self) -> Vec<ScriptRun> {
        let (workspace, workspace_path) = self.get_workspace_info();
        let runs: Vec<ScriptRun> = if let AppMode::MultiSelect { ref selected } = self.mode {
            selected
                .iter()
                .filter_map(|&idx| {
                    self.get_visible_script(idx).map(|script| ScriptRun {
                        script: script.clone(),
                        args: None,
                        workspace: workspace.clone(),
                        workspace_path: workspace_path.clone(),
                    })
                })
                .collect()
        } else {
            vec![]
        };

        if !runs.is_empty() {
            // Set first script to run, the rest would need to be handled by the caller
            self.script_to_run = runs.first().cloned();
            self.should_quit = true;
        }

        runs
    }
}

/// Calculate grid columns based on terminal width.
pub fn calculate_columns(width: u16) -> usize {
    if width < 60 {
        1
    } else if width < 90 {
        2
    } else if width < 120 {
        3
    } else if width < 160 {
        4
    } else {
        5
    }
}

/// Calculate column width for the scripts grid.
pub fn calculate_column_width(total_width: u16, columns: usize) -> u16 {
    if columns == 0 {
        return total_width;
    }
    let padding = 2; // Left and right padding
    let available = total_width.saturating_sub(padding * 2);
    (available / columns as u16).max(MIN_COLUMN_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_scripts() -> Scripts {
        let mut scripts = Scripts::new();
        scripts.add(Script::new("dev", "vite"));
        scripts.add(Script::new("build", "vite build"));
        scripts.add(Script::new("test", "vitest"));
        scripts.add(Script::new("lint", "eslint ."));
        scripts.add(Script::new("format", "prettier --write ."));
        scripts.add(Script::new("typecheck", "tsc --noEmit"));
        scripts.add(Script::new("build:prod", "vite build --mode production"));
        scripts.add(Script::new("build:dev", "vite build --mode development"));
        scripts.add(Script::new("test:unit", "vitest unit"));
        scripts
    }

    fn create_test_app() -> App {
        let scripts = create_test_scripts();
        let config = Config::default();
        let history = History::new();
        App::new(
            scripts,
            config,
            history,
            "test-project".to_string(),
            PathBuf::from("/test/project"),
            Runner::Npm,
        )
    }

    // ==================== Basic Tests ====================

    #[test]
    fn test_app_new() {
        let app = create_test_app();
        assert_eq!(app.project_name(), "test-project");
        assert_eq!(app.runner(), Runner::Npm);
        assert!(!app.should_quit());
        assert!(app.script_to_run().is_none());
        assert_eq!(app.mode(), &AppMode::Normal);
    }

    #[test]
    fn test_visible_scripts() {
        let app = create_test_app();
        let visible = app.visible_scripts();
        assert_eq!(visible.len(), 9);
    }

    #[test]
    fn test_selected_script() {
        let mut app = create_test_app();
        app.set_sort_mode(SortMode::Alpha); // Ensure predictable order
        let script = app.selected_script().unwrap();
        // First alphabetically should be "build"
        assert_eq!(script.name(), "build");
    }

    // ==================== Navigation Tests ====================

    #[test]
    fn test_move_left_right() {
        let mut app = create_test_app();
        app.update_columns(100); // 3 columns
        app.set_sort_mode(SortMode::Alpha);

        assert_eq!(app.selected_index(), 0);

        app.move_right();
        assert_eq!(app.selected_index(), 1);

        app.move_right();
        assert_eq!(app.selected_index(), 2);

        app.move_left();
        assert_eq!(app.selected_index(), 1);

        app.move_left();
        assert_eq!(app.selected_index(), 0);

        // Should not go below 0
        app.move_left();
        assert_eq!(app.selected_index(), 0);
    }

    #[test]
    fn test_move_up_down_single_column() {
        let mut app = create_test_app();
        app.update_columns(50); // 1 column
        app.set_sort_mode(SortMode::Alpha);

        assert_eq!(app.selected_index(), 0);

        app.move_down();
        assert_eq!(app.selected_index(), 1);

        app.move_down();
        assert_eq!(app.selected_index(), 2);

        app.move_up();
        assert_eq!(app.selected_index(), 1);

        app.move_up();
        assert_eq!(app.selected_index(), 0);

        // Should not go above 0
        app.move_up();
        assert_eq!(app.selected_index(), 0);
    }

    #[test]
    fn test_move_up_down_multi_column() {
        let mut app = create_test_app();
        app.update_columns(100); // 3 columns
        app.set_sort_mode(SortMode::Alpha);

        // Grid layout (9 items, 3 columns):
        // 0 1 2
        // 3 4 5
        // 6 7 8

        assert_eq!(app.selected_index(), 0);

        app.move_down(); // 0 -> 3
        assert_eq!(app.selected_index(), 3);

        app.move_down(); // 3 -> 6
        assert_eq!(app.selected_index(), 6);

        app.move_right(); // 6 -> 7
        assert_eq!(app.selected_index(), 7);

        app.move_up(); // 7 -> 4
        assert_eq!(app.selected_index(), 4);

        app.move_up(); // 4 -> 1
        assert_eq!(app.selected_index(), 1);
    }

    #[test]
    fn test_move_to_first_last() {
        let mut app = create_test_app();
        app.set_sort_mode(SortMode::Alpha);

        app.move_to_last();
        assert_eq!(app.selected_index(), 8); // 9 items, 0-indexed

        app.move_to_first();
        assert_eq!(app.selected_index(), 0);
    }

    #[test]
    fn test_select_by_number() {
        let mut app = create_test_app();
        app.set_sort_mode(SortMode::Alpha);

        app.select_by_number(5);
        assert_eq!(app.selected_index(), 4); // 5 -> index 4

        app.select_by_number(1);
        assert_eq!(app.selected_index(), 0);

        app.select_by_number(9);
        assert_eq!(app.selected_index(), 8);

        // Invalid numbers should not change selection
        app.select_by_number(0);
        assert_eq!(app.selected_index(), 8);

        app.select_by_number(100);
        assert_eq!(app.selected_index(), 8);
    }

    // ==================== Filter Tests ====================

    #[test]
    fn test_filter_updates_visible() {
        let mut app = create_test_app();

        app.set_filter("build".to_string());
        let visible = app.visible_scripts();

        // Should match: build, build:prod, build:dev
        assert_eq!(visible.len(), 3);
        assert!(visible.iter().all(|s| s.name().contains("build")));
    }

    #[test]
    fn test_filter_adjusts_selection() {
        let mut app = create_test_app();
        app.set_sort_mode(SortMode::Alpha);

        // Select last item
        app.move_to_last();
        assert_eq!(app.selected_index(), 8);

        // Filter to fewer items
        app.set_filter("test".to_string());

        // Selection should be adjusted to be within bounds
        assert!(app.selected_index() < app.visible_count());
    }

    #[test]
    fn test_filter_clear() {
        let mut app = create_test_app();

        app.set_filter("dev".to_string());
        assert!(app.visible_count() < 9);

        app.clear_filter();
        assert_eq!(app.visible_count(), 9);
    }

    #[test]
    fn test_filter_char_operations() {
        let mut app = create_test_app();

        app.push_filter_char('t');
        assert_eq!(app.filter_text(), "t");

        app.push_filter_char('e');
        assert_eq!(app.filter_text(), "te");

        app.pop_filter_char();
        assert_eq!(app.filter_text(), "t");

        app.pop_filter_char();
        assert_eq!(app.filter_text(), "");
    }

    // ==================== Sort Mode Tests ====================

    #[test]
    fn test_cycle_sort_mode() {
        let mut app = create_test_app();

        assert_eq!(app.sort_mode(), SortMode::Recent); // Default

        app.cycle_sort_mode();
        assert_eq!(app.sort_mode(), SortMode::Alpha);

        app.cycle_sort_mode();
        assert_eq!(app.sort_mode(), SortMode::Category);

        app.cycle_sort_mode();
        assert_eq!(app.sort_mode(), SortMode::Recent);
    }

    #[test]
    fn test_sort_mode_alpha() {
        let mut app = create_test_app();
        app.set_sort_mode(SortMode::Alpha);

        let visible = app.visible_scripts();
        let names: Vec<&str> = visible.iter().map(|s| s.name()).collect();

        // Should be alphabetically sorted
        let mut sorted_names = names.clone();
        sorted_names.sort();
        assert_eq!(names, sorted_names);
    }

    #[test]
    fn test_sort_mode_category() {
        let mut app = create_test_app();
        app.set_sort_mode(SortMode::Category);

        let visible = app.visible_scripts();
        let names: Vec<&str> = visible.iter().map(|s| s.name()).collect();

        // Scripts with "build" prefix should be together
        let build_indices: Vec<usize> = names
            .iter()
            .enumerate()
            .filter(|(_, n)| n.starts_with("build"))
            .map(|(i, _)| i)
            .collect();

        // Build scripts should be consecutive
        if build_indices.len() > 1 {
            for i in 1..build_indices.len() {
                assert!(build_indices[i] - build_indices[i - 1] <= 1);
            }
        }
    }

    // ==================== Action Tests ====================

    #[test]
    fn test_run_selected() {
        let mut app = create_test_app();
        app.set_sort_mode(SortMode::Alpha);

        let run = app.run_selected();
        assert!(run.is_some());

        let run = run.unwrap();
        assert_eq!(run.script.name(), "build"); // First alphabetically
        assert!(run.args.is_none());
        assert!(app.should_quit());
    }

    #[test]
    fn test_run_numbered() {
        let mut app = create_test_app();
        app.set_sort_mode(SortMode::Alpha);

        let run = app.run_numbered(3);
        assert!(run.is_some());

        // Verify the run was successful
        assert!(run.unwrap().script.name().len() > 0);
        assert_eq!(app.selected_index(), 2);
        assert!(app.should_quit());
    }

    #[test]
    fn test_run_with_args() {
        let mut app = create_test_app();
        app.set_sort_mode(SortMode::Alpha);

        let run = app.run_with_args("--watch".to_string());
        assert!(run.is_some());

        let run = run.unwrap();
        assert_eq!(run.args, Some("--watch".to_string()));
        assert!(app.should_quit());
    }

    #[test]
    fn test_quit() {
        let mut app = create_test_app();
        assert!(!app.should_quit());

        app.quit();
        assert!(app.should_quit());
    }

    // ==================== Mode Tests ====================

    #[test]
    fn test_toggle_filter_mode() {
        let mut app = create_test_app();
        assert_eq!(app.mode(), &AppMode::Normal);

        app.toggle_filter_mode();
        assert!(matches!(app.mode(), &AppMode::Filter { .. }));

        app.toggle_filter_mode();
        assert_eq!(app.mode(), &AppMode::Normal);
    }

    #[test]
    fn test_toggle_multi_select() {
        let mut app = create_test_app();
        assert_eq!(app.mode(), &AppMode::Normal);

        app.toggle_multi_select();
        assert!(matches!(app.mode(), &AppMode::MultiSelect { .. }));

        app.toggle_multi_select();
        assert_eq!(app.mode(), &AppMode::Normal);
    }

    #[test]
    fn test_toggle_help() {
        let mut app = create_test_app();
        assert_eq!(app.mode(), &AppMode::Normal);

        app.toggle_help();
        assert_eq!(app.mode(), &AppMode::Help);

        app.toggle_help();
        assert_eq!(app.mode(), &AppMode::Normal);
    }

    #[test]
    fn test_enter_args_mode() {
        let mut app = create_test_app();
        app.enter_args_mode();

        assert!(matches!(
            app.mode(),
            &AppMode::Args {
                script_index: 0,
                ..
            }
        ));
    }

    #[test]
    fn test_multi_select_toggle_selection() {
        let mut app = create_test_app();
        app.toggle_multi_select();

        app.toggle_current_selection();
        let selected = app.multi_selected_indices().unwrap();
        assert!(selected.contains(&0));

        app.move_right();
        app.toggle_current_selection();
        let selected = app.multi_selected_indices().unwrap();
        assert!(selected.contains(&0));
        assert!(selected.contains(&1));

        // Toggle off
        app.toggle_current_selection();
        let selected = app.multi_selected_indices().unwrap();
        assert!(!selected.contains(&1));
    }

    // ==================== Column Tests ====================

    #[test]
    fn test_calculate_columns() {
        assert_eq!(calculate_columns(50), 1);
        assert_eq!(calculate_columns(59), 1);
        assert_eq!(calculate_columns(60), 2);
        assert_eq!(calculate_columns(89), 2);
        assert_eq!(calculate_columns(90), 3);
        assert_eq!(calculate_columns(119), 3);
        assert_eq!(calculate_columns(120), 4);
        assert_eq!(calculate_columns(159), 4);
        assert_eq!(calculate_columns(160), 5);
        assert_eq!(calculate_columns(200), 5);
    }

    #[test]
    fn test_update_columns() {
        let mut app = create_test_app();

        app.update_columns(100);
        assert_eq!(app.columns(), 3);

        app.update_columns(50);
        assert_eq!(app.columns(), 1);

        app.update_columns(160);
        assert_eq!(app.columns(), 5);
    }

    #[test]
    fn test_calculate_column_width() {
        assert_eq!(calculate_column_width(100, 3), 32);
        assert_eq!(calculate_column_width(80, 2), 38);
        assert_eq!(calculate_column_width(60, 1), 56);
        assert_eq!(calculate_column_width(50, 0), 50); // Edge case
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_scripts() {
        let scripts = Scripts::new();
        let config = Config::default();
        let history = History::new();
        let app = App::new(
            scripts,
            config,
            history,
            "empty-project".to_string(),
            PathBuf::from("/test/empty"),
            Runner::Npm,
        );

        assert_eq!(app.visible_count(), 0);
        assert!(app.selected_script().is_none());
    }

    #[test]
    fn test_navigation_with_empty_scripts() {
        let scripts = Scripts::new();
        let config = Config::default();
        let history = History::new();
        let mut app = App::new(
            scripts,
            config,
            history,
            "empty-project".to_string(),
            PathBuf::from("/test/empty"),
            Runner::Npm,
        );

        // These should not panic
        app.move_up();
        app.move_down();
        app.move_left();
        app.move_right();
        app.move_to_first();
        app.move_to_last();

        assert_eq!(app.selected_index(), 0);
    }

    #[test]
    fn test_filter_no_matches() {
        let mut app = create_test_app();

        app.set_filter("nonexistent_script_xyz".to_string());
        assert_eq!(app.visible_count(), 0);
        assert!(app.selected_script().is_none());
    }

    #[test]
    fn test_navigation_last_row_partial() {
        // Test navigation when last row has fewer items than columns
        let mut scripts = Scripts::new();
        for i in 0..7 {
            scripts.add(Script::new(format!("script{}", i), format!("cmd{}", i)));
        }

        let config = Config::default();
        let history = History::new();
        let mut app = App::new(
            scripts,
            config,
            history,
            "test".to_string(),
            PathBuf::from("/test"),
            Runner::Npm,
        );

        app.update_columns(100); // 3 columns
        app.set_sort_mode(SortMode::Alpha);

        // Grid layout (7 items, 3 columns):
        // 0 1 2
        // 3 4 5
        // 6

        // Navigate to position 2 (top right)
        app.select_by_number(3);
        assert_eq!(app.selected_index(), 2);

        // Move down should go to 5
        app.move_down();
        assert_eq!(app.selected_index(), 5);

        // Move down again - should go to last item (6) since column 2 doesn't exist in row 2
        app.move_down();
        assert_eq!(app.selected_index(), 6);
    }
}
