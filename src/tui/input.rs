//! Input handling for the TUI.

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::app::{App, AppMode};

/// Handle a terminal event.
///
/// Returns `Ok(true)` if the app should quit, `Ok(false)` to continue.
pub fn handle_event(app: &mut App, event: Event) -> Result<bool> {
    match event {
        Event::Key(key) => Ok(handle_key(app, key)),
        Event::Resize(width, _height) => {
            app.update_columns(width);
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle a key event.
///
/// Returns true if the app should quit.
fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    // Global quit shortcuts (except in text input modes)
    if matches!(
        (key.code, key.modifiers),
        (KeyCode::Char('c'), KeyModifiers::CONTROL)
    ) && !matches!(app.mode(), AppMode::Filter { .. } | AppMode::Args { .. })
    {
        app.quit();
        return true;
    }

    match app.mode().clone() {
        AppMode::Normal => handle_normal_mode(app, key),
        AppMode::Filter { query } => handle_filter_mode(app, key, &query),
        AppMode::Help => handle_help_mode(app, key),
        AppMode::Error { .. } => handle_error_mode(app, key),
        AppMode::MultiSelect { selected } => handle_multiselect_mode(app, key, &selected),
        AppMode::Args {
            script_index,
            input,
        } => handle_args_mode(app, key, script_index, &input),
        AppMode::WorkspaceSelect => handle_workspace_select_mode(app, key),
    }

    app.should_quit()
}

/// Handle keys in normal mode.
///
/// Navigation:
/// - ↑/k: move up
/// - ↓/j: move down
/// - ←/h: move left
/// - →/l: move right
/// - Home/g: move to first
/// - End/G: move to last
///
/// Actions:
/// - Enter/o: run selected script
/// - 1-9: run numbered script
/// - /: enter filter mode
/// - s: cycle sort mode
/// - a: enter args mode
/// - m: enter multi-select mode
/// - ?: toggle help
/// - q/Ctrl+C: quit
fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
        KeyCode::Left | KeyCode::Char('h') => app.move_left(),
        KeyCode::Right | KeyCode::Char('l') => app.move_right(),
        KeyCode::Home | KeyCode::Char('g') => app.move_to_first(),
        KeyCode::End | KeyCode::Char('G') => app.move_to_last(),

        // Run selected script
        KeyCode::Enter | KeyCode::Char('o') => {
            app.run_selected();
        }

        // Quick select (1-9)
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let num = c.to_digit(10).unwrap() as usize;
            app.run_numbered(num);
        }

        // Enter filter mode
        KeyCode::Char('/') => {
            app.set_mode(AppMode::Filter {
                query: String::new(),
            });
        }

        // Cycle sort mode
        KeyCode::Char('s') => {
            app.cycle_sort_mode();
        }

        // Enter args mode
        KeyCode::Char('a') => {
            app.enter_args_mode();
        }

        // Enter multi-select mode
        KeyCode::Char('m') => {
            app.toggle_multi_select();
        }

        // Help
        KeyCode::Char('?') => {
            app.toggle_help();
        }

        // Quit
        KeyCode::Char('q') => {
            app.quit();
        }

        // Back to workspace selection (for monorepos)
        KeyCode::Char('w') if app.is_monorepo() => {
            app.back_to_workspace_select();
        }

        _ => {}
    }
}

/// Handle keys in workspace selection mode.
///
/// Navigation:
/// - ↑/k: move up
/// - ↓/j: move down
/// - ←/h: move left
/// - →/l: move right
///
/// Actions:
/// - Enter: select workspace and show its scripts
/// - 1-9: quick select workspace
/// - q/Esc: quit
fn handle_workspace_select_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => app.workspace_move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.workspace_move_down(),
        KeyCode::Left | KeyCode::Char('h') => app.workspace_move_left(),
        KeyCode::Right | KeyCode::Char('l') => app.workspace_move_right(),

        // Select workspace
        KeyCode::Enter => {
            app.select_current_workspace();
        }

        // Quick select (1-9)
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let num = c.to_digit(10).unwrap() as usize;
            app.select_workspace_by_number(num);
        }

        // Help
        KeyCode::Char('?') => {
            app.toggle_help();
        }

        // Quit
        KeyCode::Char('q') | KeyCode::Esc => {
            app.quit();
        }

        _ => {}
    }
}

/// Handle keys in filter mode.
///
/// - Printable characters: append to filter
/// - Backspace: remove last character
/// - Escape: clear filter and exit filter mode
/// - Enter: run first visible script
/// - Navigation keys still work while filtering
fn handle_filter_mode(app: &mut App, key: KeyEvent, current_query: &str) {
    match key.code {
        // Exit filter mode
        KeyCode::Esc => {
            app.clear_filter();
            app.set_mode(AppMode::Normal);
        }

        // Run selected script
        KeyCode::Enter => {
            app.run_selected();
        }

        // Remove last character
        KeyCode::Backspace => {
            let mut query = current_query.to_string();
            query.pop();
            if query.is_empty() {
                app.clear_filter();
                app.set_mode(AppMode::Normal);
            } else {
                app.set_filter(query);
            }
        }

        // Navigation in filter mode (arrow keys)
        KeyCode::Up => app.move_up(),
        KeyCode::Down => app.move_down(),
        KeyCode::Left => app.move_left(),
        KeyCode::Right => app.move_right(),

        // Append character (this must come after arrow keys)
        KeyCode::Char(c) => {
            // Quick select still works with empty filter and digits
            if c.is_ascii_digit() && c != '0' && current_query.is_empty() {
                let num = c.to_digit(10).unwrap() as usize;
                app.run_numbered(num);
            } else {
                let mut query = current_query.to_string();
                query.push(c);
                app.set_filter(query);
            }
        }

        _ => {}
    }
}

/// Handle keys in help mode.
///
/// Any key closes help.
fn handle_help_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc
        | KeyCode::Char('?')
        | KeyCode::Char('q')
        | KeyCode::Enter
        | KeyCode::Char(_) => {
            app.set_mode(AppMode::Normal);
        }
        _ => {
            // Any other key also dismisses help
            app.set_mode(AppMode::Normal);
        }
    }
}

/// Handle keys in error mode.
///
/// Any key dismisses the error.
fn handle_error_mode(app: &mut App, _key: KeyEvent) {
    // Any key dismisses the error
    app.set_mode(AppMode::Normal);
}

/// Handle keys in multi-select mode.
///
/// - Space: toggle current item selection
/// - Enter: run all selected scripts in order
/// - a: select all visible
/// - n: select none
/// - Escape: exit multi-select mode
fn handle_multiselect_mode(
    app: &mut App,
    key: KeyEvent,
    current_selected: &std::collections::HashSet<usize>,
) {
    match key.code {
        // Exit multi-select mode
        KeyCode::Esc => {
            app.set_mode(AppMode::Normal);
        }

        // Toggle current item
        KeyCode::Char(' ') => {
            app.toggle_current_selection();
        }

        // Run all selected
        KeyCode::Enter => {
            app.run_multi_selected();
        }

        // Select all visible
        KeyCode::Char('a') => {
            let mut selected = current_selected.clone();
            for i in 0..app.visible_count() {
                selected.insert(i);
            }
            app.set_mode(AppMode::MultiSelect { selected });
        }

        // Select none
        KeyCode::Char('n') => {
            app.set_mode(AppMode::MultiSelect {
                selected: std::collections::HashSet::new(),
            });
        }

        // Navigation
        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
        KeyCode::Left | KeyCode::Char('h') => app.move_left(),
        KeyCode::Right | KeyCode::Char('l') => app.move_right(),
        KeyCode::Home | KeyCode::Char('g') => app.move_to_first(),
        KeyCode::End | KeyCode::Char('G') => app.move_to_last(),

        _ => {}
    }
}

/// Handle keys in args input mode.
///
/// - Printable characters: append to args input
/// - Backspace: remove last character
/// - Enter: run script with args
/// - Escape: cancel and return to normal mode
fn handle_args_mode(app: &mut App, key: KeyEvent, script_index: usize, current_input: &str) {
    match key.code {
        // Cancel and return to normal mode
        KeyCode::Esc => {
            app.set_mode(AppMode::Normal);
        }

        // Run script with args
        KeyCode::Enter => {
            app.run_with_args(current_input.to_string());
        }

        // Remove last character
        KeyCode::Backspace => {
            let mut input = current_input.to_string();
            input.pop();
            app.set_mode(AppMode::Args {
                script_index,
                input,
            });
        }

        // Append character
        KeyCode::Char(c) => {
            let mut input = current_input.to_string();
            input.push(c);
            app.set_mode(AppMode::Args {
                script_index,
                input,
            });
        }

        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::history::History;
    use crate::package::{Runner, Script, Scripts};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use std::path::PathBuf;

    fn create_test_scripts() -> Scripts {
        let mut scripts = Scripts::new();
        scripts.add(Script::new("dev", "vite"));
        scripts.add(Script::new("build", "vite build"));
        scripts.add(Script::new("test", "vitest"));
        scripts.add(Script::new("lint", "eslint ."));
        scripts.add(Script::new("format", "prettier --write ."));
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

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn key_event_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    // ==================== Normal Mode Tests ====================

    #[test]
    fn test_normal_mode_navigation_arrows() {
        let mut app = create_test_app();
        app.update_columns(100); // Multiple columns

        handle_normal_mode(&mut app, key_event(KeyCode::Down));
        assert!(app.selected_index() > 0 || app.visible_count() <= 1);

        handle_normal_mode(&mut app, key_event(KeyCode::Up));
        assert_eq!(app.selected_index(), 0);
    }

    #[test]
    fn test_normal_mode_navigation_vim() {
        let mut app = create_test_app();
        app.update_columns(100);

        handle_normal_mode(&mut app, key_event(KeyCode::Char('j')));
        let after_j = app.selected_index();

        handle_normal_mode(&mut app, key_event(KeyCode::Char('k')));
        assert_eq!(app.selected_index(), 0);

        // Left/right with h/l
        handle_normal_mode(&mut app, key_event(KeyCode::Char('l')));
        assert_eq!(app.selected_index(), 1);

        handle_normal_mode(&mut app, key_event(KeyCode::Char('h')));
        assert_eq!(app.selected_index(), 0);

        // Verify j moved down
        assert!(after_j > 0 || app.columns() == 1);
    }

    #[test]
    fn test_normal_mode_navigation_home_end() {
        let mut app = create_test_app();

        handle_normal_mode(&mut app, key_event(KeyCode::End));
        assert_eq!(app.selected_index(), app.visible_count() - 1);

        handle_normal_mode(&mut app, key_event(KeyCode::Home));
        assert_eq!(app.selected_index(), 0);
    }

    #[test]
    fn test_normal_mode_navigation_g_shift_g() {
        let mut app = create_test_app();

        handle_normal_mode(&mut app, key_event(KeyCode::Char('G')));
        assert_eq!(app.selected_index(), app.visible_count() - 1);

        handle_normal_mode(&mut app, key_event(KeyCode::Char('g')));
        assert_eq!(app.selected_index(), 0);
    }

    #[test]
    fn test_normal_mode_run_selected() {
        let mut app = create_test_app();

        handle_normal_mode(&mut app, key_event(KeyCode::Enter));
        assert!(app.should_quit());
        assert!(app.script_to_run().is_some());
    }

    #[test]
    fn test_normal_mode_run_selected_o() {
        let mut app = create_test_app();

        handle_normal_mode(&mut app, key_event(KeyCode::Char('o')));
        assert!(app.should_quit());
        assert!(app.script_to_run().is_some());
    }

    #[test]
    fn test_normal_mode_quick_select() {
        let mut app = create_test_app();

        handle_normal_mode(&mut app, key_event(KeyCode::Char('3')));
        assert!(app.should_quit());
        assert!(app.script_to_run().is_some());
        assert_eq!(app.selected_index(), 2);
    }

    #[test]
    fn test_normal_mode_enter_filter() {
        let mut app = create_test_app();

        handle_normal_mode(&mut app, key_event(KeyCode::Char('/')));
        assert!(matches!(app.mode(), AppMode::Filter { .. }));
    }

    #[test]
    fn test_normal_mode_cycle_sort() {
        let mut app = create_test_app();
        let initial_sort = app.sort_mode();

        handle_normal_mode(&mut app, key_event(KeyCode::Char('s')));
        assert_ne!(app.sort_mode(), initial_sort);
    }

    #[test]
    fn test_normal_mode_enter_args() {
        let mut app = create_test_app();

        handle_normal_mode(&mut app, key_event(KeyCode::Char('a')));
        assert!(matches!(app.mode(), AppMode::Args { .. }));
    }

    #[test]
    fn test_normal_mode_enter_multiselect() {
        let mut app = create_test_app();

        handle_normal_mode(&mut app, key_event(KeyCode::Char('m')));
        assert!(matches!(app.mode(), AppMode::MultiSelect { .. }));
    }

    #[test]
    fn test_normal_mode_toggle_help() {
        let mut app = create_test_app();

        handle_normal_mode(&mut app, key_event(KeyCode::Char('?')));
        assert!(matches!(app.mode(), AppMode::Help));
    }

    #[test]
    fn test_normal_mode_quit_q() {
        let mut app = create_test_app();

        handle_normal_mode(&mut app, key_event(KeyCode::Char('q')));
        assert!(app.should_quit());
    }

    #[test]
    fn test_normal_mode_quit_ctrl_c() {
        let mut app = create_test_app();

        let result = handle_key(
            &mut app,
            key_event_with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(result);
        assert!(app.should_quit());
    }

    // ==================== Filter Mode Tests ====================

    #[test]
    fn test_filter_mode_type_character() {
        let mut app = create_test_app();
        app.set_mode(AppMode::Filter {
            query: String::new(),
        });

        handle_filter_mode(&mut app, key_event(KeyCode::Char('t')), "");
        assert_eq!(app.filter_text(), "t");
    }

    #[test]
    fn test_filter_mode_type_multiple_characters() {
        let mut app = create_test_app();
        app.set_mode(AppMode::Filter {
            query: String::new(),
        });

        handle_filter_mode(&mut app, key_event(KeyCode::Char('t')), "");
        handle_filter_mode(&mut app, key_event(KeyCode::Char('e')), "t");
        handle_filter_mode(&mut app, key_event(KeyCode::Char('s')), "te");
        assert_eq!(app.filter_text(), "tes");
    }

    #[test]
    fn test_filter_mode_backspace() {
        let mut app = create_test_app();
        app.set_filter("test".to_string());

        handle_filter_mode(&mut app, key_event(KeyCode::Backspace), "test");
        assert_eq!(app.filter_text(), "tes");
    }

    #[test]
    fn test_filter_mode_backspace_exits_when_empty() {
        let mut app = create_test_app();
        app.set_filter("t".to_string());

        handle_filter_mode(&mut app, key_event(KeyCode::Backspace), "t");
        assert!(matches!(app.mode(), AppMode::Normal));
        assert_eq!(app.filter_text(), "");
    }

    #[test]
    fn test_filter_mode_escape_clears_and_exits() {
        let mut app = create_test_app();
        app.set_filter("test".to_string());

        handle_filter_mode(&mut app, key_event(KeyCode::Esc), "test");
        assert!(matches!(app.mode(), AppMode::Normal));
        assert_eq!(app.filter_text(), "");
    }

    #[test]
    fn test_filter_mode_enter_runs_script() {
        let mut app = create_test_app();
        app.set_filter("dev".to_string());

        handle_filter_mode(&mut app, key_event(KeyCode::Enter), "dev");
        assert!(app.should_quit());
        assert!(app.script_to_run().is_some());
    }

    #[test]
    fn test_filter_mode_navigation() {
        let mut app = create_test_app();
        app.update_columns(100);
        app.set_mode(AppMode::Filter {
            query: String::new(),
        });

        handle_filter_mode(&mut app, key_event(KeyCode::Down), "");
        let pos = app.selected_index();
        assert!(pos > 0 || app.columns() == 1);

        handle_filter_mode(&mut app, key_event(KeyCode::Up), "");
        assert_eq!(app.selected_index(), 0);
    }

    #[test]
    fn test_filter_mode_quick_select_empty_query() {
        let mut app = create_test_app();
        app.set_mode(AppMode::Filter {
            query: String::new(),
        });

        handle_filter_mode(&mut app, key_event(KeyCode::Char('2')), "");
        assert!(app.should_quit());
        assert_eq!(app.selected_index(), 1);
    }

    // ==================== Multi-Select Mode Tests ====================

    #[test]
    fn test_multiselect_toggle_selection() {
        let mut app = create_test_app();
        app.toggle_multi_select();

        handle_multiselect_mode(
            &mut app,
            key_event(KeyCode::Char(' ')),
            &std::collections::HashSet::new(),
        );
        let selected = app.multi_selected_indices().unwrap();
        assert!(selected.contains(&0));
    }

    #[test]
    fn test_multiselect_run_selected() {
        let mut app = create_test_app();
        app.toggle_multi_select();
        app.toggle_current_selection();

        let selected = app.multi_selected_indices().unwrap().clone();
        handle_multiselect_mode(&mut app, key_event(KeyCode::Enter), &selected);
        assert!(app.should_quit());
    }

    #[test]
    fn test_multiselect_select_all() {
        let mut app = create_test_app();
        app.toggle_multi_select();

        handle_multiselect_mode(
            &mut app,
            key_event(KeyCode::Char('a')),
            &std::collections::HashSet::new(),
        );
        let selected = app.multi_selected_indices().unwrap();
        assert_eq!(selected.len(), app.visible_count());
    }

    #[test]
    fn test_multiselect_select_none() {
        let mut app = create_test_app();
        app.toggle_multi_select();
        app.toggle_current_selection();
        app.move_right();
        app.toggle_current_selection();

        let current_selected = app.multi_selected_indices().unwrap().clone();
        handle_multiselect_mode(&mut app, key_event(KeyCode::Char('n')), &current_selected);
        let selected = app.multi_selected_indices().unwrap();
        assert!(selected.is_empty());
    }

    #[test]
    fn test_multiselect_escape() {
        let mut app = create_test_app();
        app.toggle_multi_select();

        handle_multiselect_mode(
            &mut app,
            key_event(KeyCode::Esc),
            &std::collections::HashSet::new(),
        );
        assert!(matches!(app.mode(), AppMode::Normal));
    }

    #[test]
    fn test_multiselect_navigation() {
        let mut app = create_test_app();
        app.update_columns(100);
        app.toggle_multi_select();

        handle_multiselect_mode(
            &mut app,
            key_event(KeyCode::Char('j')),
            &std::collections::HashSet::new(),
        );
        assert!(app.selected_index() > 0 || app.columns() == 1);

        handle_multiselect_mode(
            &mut app,
            key_event(KeyCode::Char('k')),
            &std::collections::HashSet::new(),
        );
        assert_eq!(app.selected_index(), 0);
    }

    // ==================== Args Mode Tests ====================

    #[test]
    fn test_args_mode_type_character() {
        let mut app = create_test_app();
        app.enter_args_mode();

        handle_args_mode(&mut app, key_event(KeyCode::Char('-')), 0, "");

        if let AppMode::Args { input, .. } = app.mode() {
            assert_eq!(input, "-");
        } else {
            panic!("Expected Args mode");
        }
    }

    #[test]
    fn test_args_mode_type_multiple() {
        let mut app = create_test_app();
        app.enter_args_mode();

        handle_args_mode(&mut app, key_event(KeyCode::Char('-')), 0, "");
        handle_args_mode(&mut app, key_event(KeyCode::Char('-')), 0, "-");
        handle_args_mode(&mut app, key_event(KeyCode::Char('w')), 0, "--");

        if let AppMode::Args { input, .. } = app.mode() {
            assert_eq!(input, "--w");
        } else {
            panic!("Expected Args mode");
        }
    }

    #[test]
    fn test_args_mode_backspace() {
        let mut app = create_test_app();
        app.set_mode(AppMode::Args {
            script_index: 0,
            input: "--watch".to_string(),
        });

        handle_args_mode(&mut app, key_event(KeyCode::Backspace), 0, "--watch");

        if let AppMode::Args { input, .. } = app.mode() {
            assert_eq!(input, "--watc");
        } else {
            panic!("Expected Args mode");
        }
    }

    #[test]
    fn test_args_mode_enter_runs_with_args() {
        let mut app = create_test_app();
        app.set_mode(AppMode::Args {
            script_index: 0,
            input: "--watch".to_string(),
        });

        handle_args_mode(&mut app, key_event(KeyCode::Enter), 0, "--watch");
        assert!(app.should_quit());

        let run = app.script_to_run().unwrap();
        assert_eq!(run.args, Some("--watch".to_string()));
    }

    #[test]
    fn test_args_mode_escape_cancels() {
        let mut app = create_test_app();
        app.set_mode(AppMode::Args {
            script_index: 0,
            input: "--watch".to_string(),
        });

        handle_args_mode(&mut app, key_event(KeyCode::Esc), 0, "--watch");
        assert!(matches!(app.mode(), AppMode::Normal));
        assert!(!app.should_quit());
    }

    // ==================== Help Mode Tests ====================

    #[test]
    fn test_help_mode_any_key_closes() {
        let mut app = create_test_app();
        app.set_mode(AppMode::Help);

        handle_help_mode(&mut app, key_event(KeyCode::Char('x')));
        assert!(matches!(app.mode(), AppMode::Normal));
    }

    #[test]
    fn test_help_mode_escape_closes() {
        let mut app = create_test_app();
        app.set_mode(AppMode::Help);

        handle_help_mode(&mut app, key_event(KeyCode::Esc));
        assert!(matches!(app.mode(), AppMode::Normal));
    }

    // ==================== Error Mode Tests ====================

    #[test]
    fn test_error_mode_any_key_dismisses() {
        let mut app = create_test_app();
        app.set_mode(AppMode::Error {
            message: "Test error".to_string(),
        });

        handle_error_mode(&mut app, key_event(KeyCode::Enter));
        assert!(matches!(app.mode(), AppMode::Normal));
    }

    // ==================== Resize Tests ====================

    #[test]
    fn test_resize_updates_columns() {
        let mut app = create_test_app();
        app.update_columns(50);
        assert_eq!(app.columns(), 1);

        let result = handle_event(&mut app, Event::Resize(100, 50)).unwrap();
        assert!(!result);
        assert_eq!(app.columns(), 3);
    }

    // ==================== handle_event Tests ====================

    #[test]
    fn test_handle_event_key() {
        let mut app = create_test_app();

        let result = handle_event(&mut app, Event::Key(key_event(KeyCode::Char('q')))).unwrap();
        assert!(result);
        assert!(app.should_quit());
    }

    #[test]
    fn test_handle_event_unknown() {
        let mut app = create_test_app();

        // FocusGained is an unknown event
        let result = handle_event(&mut app, Event::FocusGained).unwrap();
        assert!(!result);
        assert!(!app.should_quit());
    }
}
