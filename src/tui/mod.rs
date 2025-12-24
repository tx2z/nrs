//! TUI module for nrs.
//!
//! Provides the terminal user interface for interactive script selection.

mod app;
mod input;
mod layout;
mod theme;
mod ui;
pub mod widgets;

pub use app::{calculate_column_width, calculate_columns, App, AppMode, ScriptRun};
pub use input::handle_event;
pub use layout::{
    centered_rect, centered_rect_fixed, GridLayout, MainLayout, MIN_HEIGHT, MIN_WIDTH,
};
pub use theme::Theme;
pub use ui::{render, restore_terminal, run_tui, TerminalGuard};
