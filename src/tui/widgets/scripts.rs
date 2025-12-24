//! Scripts grid widget for the TUI.

use std::collections::HashSet;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::Widget,
};

use crate::package::Script;
use crate::tui::layout::{calculate_column_width, calculate_columns};
use crate::tui::theme::Theme;
use crate::tui::widgets::header::truncate_with_ellipsis;

/// Scripts grid widget.
pub struct ScriptsGrid<'a> {
    scripts: &'a [&'a Script],
    selected: usize,
    scroll_offset: usize,
    theme: &'a Theme,
    multi_selected: Option<&'a HashSet<usize>>,
}

impl<'a> ScriptsGrid<'a> {
    /// Create a new scripts grid widget.
    pub fn new(scripts: &'a [&'a Script], selected: usize, theme: &'a Theme) -> Self {
        Self {
            scripts,
            selected,
            scroll_offset: 0,
            theme,
            multi_selected: None,
        }
    }

    /// Set the scroll offset.
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Set multi-selected items.
    pub fn multi_selected(mut self, selected: &'a HashSet<usize>) -> Self {
        self.multi_selected = Some(selected);
        self
    }

    /// Render a single script item.
    fn render_script(
        &self,
        script: &Script,
        index: usize,
        is_selected: bool,
        is_multi_selected: bool,
        max_width: u16,
    ) -> Vec<Span<'a>> {
        // Number prefix (1-9 for first 9 visible items)
        let num_str = if index < 9 {
            format!("{}", index + 1)
        } else {
            " ".to_string()
        };

        // Cursor/marker
        let marker = if is_selected {
            ">"
        } else if is_multi_selected {
            "*"
        } else {
            " "
        };

        // Calculate name width (accounting for num, marker, and padding)
        let prefix_len = 4; // " N > " or " N * " etc
        let name_width = (max_width as usize).saturating_sub(prefix_len);
        let name = truncate_with_ellipsis(script.name(), name_width);

        // Build spans
        let marker_style = if is_multi_selected {
            self.theme.multiselect()
        } else {
            self.theme.cursor()
        };

        let name_style = if is_selected {
            self.theme.selected()
        } else {
            self.theme.script()
        };

        vec![
            Span::styled(format!("{} ", num_str), self.theme.number()),
            Span::styled(format!("{} ", marker), marker_style),
            Span::styled(name, name_style),
        ]
    }
}

impl Widget for ScriptsGrid<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 || self.scripts.is_empty() {
            return;
        }

        let columns = calculate_columns(area.width);
        let column_width = calculate_column_width(area.width, columns);
        let rows = area.height as usize;

        // Calculate which items to show
        let total_visible = rows * columns;
        let start_idx = self.scroll_offset;
        let end_idx = (start_idx + total_visible).min(self.scripts.len());

        // Render items in horizontal-first order
        for display_idx in 0..(end_idx - start_idx) {
            let script_idx = start_idx + display_idx;

            // Calculate position in grid
            let row = display_idx / columns;
            let col = display_idx % columns;

            if row >= rows {
                break;
            }

            // Calculate screen position
            let x = area.x + (col as u16 * column_width);
            let y = area.y + row as u16;

            if y >= area.y + area.height {
                break;
            }

            // Get script and render state
            let script = self.scripts[script_idx];
            let is_selected = script_idx == self.selected;
            let is_multi = self
                .multi_selected
                .map(|m| m.contains(&script_idx))
                .unwrap_or(false);

            // Render the script item
            let spans =
                self.render_script(script, display_idx, is_selected, is_multi, column_width);
            let line = Line::from(spans);

            // Render to buffer
            let item_area = Rect::new(x, y, column_width, 1);
            buf.set_line(item_area.x, item_area.y, &line, item_area.width);
        }
    }
}

/// Empty state widget when no scripts are available.
pub struct EmptyScripts<'a> {
    message: &'a str,
    hint: Option<&'a str>,
    theme: &'a Theme,
}

impl<'a> EmptyScripts<'a> {
    /// Create a new empty scripts widget.
    pub fn new(message: &'a str, theme: &'a Theme) -> Self {
        Self {
            message,
            hint: None,
            theme,
        }
    }

    /// Create a new empty scripts widget with a hint.
    pub fn with_hint(message: &'a str, hint: &'a str, theme: &'a Theme) -> Self {
        Self {
            message,
            hint: Some(hint),
            theme,
        }
    }

    /// Create for no scripts found.
    pub fn no_scripts(theme: &'a Theme) -> Self {
        Self::with_hint(
            "No scripts found in package.json",
            "Add scripts to your package.json to get started",
            theme,
        )
    }

    /// Create for no filter matches.
    pub fn no_matches(theme: &'a Theme) -> Self {
        Self::with_hint(
            "No scripts match the filter",
            "Press Escape to clear the filter",
            theme,
        )
    }

    /// Create for filter with specific query that has no matches.
    pub fn no_matches_for(filter: &str, theme: &'a Theme) -> Self {
        // We can't store the dynamic string, so just use the static version
        let _ = filter;
        Self::no_matches(theme)
    }

    /// Create for all scripts excluded.
    pub fn all_excluded(theme: &'a Theme) -> Self {
        Self::with_hint(
            "All scripts are excluded",
            "Check your exclude patterns in config",
            theme,
        )
    }
}

impl Widget for EmptyScripts<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let has_hint = self.hint.is_some() && area.height >= 3;

        // Center the message vertically
        let y = if has_hint {
            area.y + area.height / 2 - 1
        } else {
            area.y + area.height / 2
        };

        if y >= area.y + area.height {
            return;
        }

        // Render main message
        let msg_len = self.message.chars().count() as u16;
        let x = area.x + (area.width.saturating_sub(msg_len)) / 2;
        let line = Line::from(vec![Span::styled(self.message, self.theme.description())]);
        buf.set_line(x, y, &line, area.width.saturating_sub(x - area.x));

        // Render hint if present
        if let Some(hint) = self.hint {
            let hint_y = y + 2;
            if hint_y < area.y + area.height {
                let hint_len = hint.chars().count() as u16;
                let hint_x = area.x + (area.width.saturating_sub(hint_len)) / 2;
                let hint_line =
                    Line::from(vec![Span::styled(hint, self.theme.filter_placeholder())]);
                buf.set_line(
                    hint_x,
                    hint_y,
                    &hint_line,
                    area.width.saturating_sub(hint_x - area.x),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_scripts() -> Vec<Script> {
        vec![
            Script::new("dev", "vite"),
            Script::new("build", "vite build"),
            Script::new("test", "vitest"),
            Script::new("lint", "eslint ."),
            Script::new("format", "prettier --write ."),
        ]
    }

    #[test]
    fn test_scripts_grid_creation() {
        let scripts = create_test_scripts();
        let script_refs: Vec<&Script> = scripts.iter().collect();
        let theme = Theme::default();

        let grid = ScriptsGrid::new(&script_refs, 0, &theme);
        assert_eq!(grid.selected, 0);
        assert_eq!(grid.scroll_offset, 0);
    }

    #[test]
    fn test_render_script() {
        let scripts = create_test_scripts();
        let script_refs: Vec<&Script> = scripts.iter().collect();
        let theme = Theme::default();

        let grid = ScriptsGrid::new(&script_refs, 0, &theme);
        let spans = grid.render_script(&scripts[0], 0, true, false, 30);

        let content: String = spans.iter().map(|s| s.content.to_string()).collect();
        assert!(content.contains("dev"));
        assert!(content.contains("1")); // First item should have number 1
    }

    #[test]
    fn test_render_script_multiselect() {
        let scripts = create_test_scripts();
        let script_refs: Vec<&Script> = scripts.iter().collect();
        let theme = Theme::default();

        let grid = ScriptsGrid::new(&script_refs, 0, &theme);
        let spans = grid.render_script(&scripts[0], 0, false, true, 30);

        let content: String = spans.iter().map(|s| s.content.to_string()).collect();
        assert!(content.contains("*")); // Multi-select marker
    }

    #[test]
    fn test_calculate_columns() {
        assert_eq!(calculate_columns(50), 1);
        assert_eq!(calculate_columns(80), 2);
        assert_eq!(calculate_columns(100), 3);
        assert_eq!(calculate_columns(130), 4);
        assert_eq!(calculate_columns(170), 5);
    }
}
