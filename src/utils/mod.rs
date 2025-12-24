//! Utility module for nrs.
//!
//! Common utilities for paths, terminal handling, and other helpers.

mod paths;
mod terminal;

pub use paths::{
    config_dir, find_package_json, find_project_root, global_config_file, history_file,
    local_config_file, MAX_SEARCH_DEPTH,
};
pub use terminal::{
    check_terminal_size, cleanup_terminal, disable_raw_mode, enable_raw_mode,
    enter_alternate_screen, hide_cursor, is_raw_mode_enabled, leave_alternate_screen,
    prepare_for_script_execution, restore_for_tui, show_cursor, TerminalSize, MIN_HEIGHT,
    MIN_WIDTH,
};
