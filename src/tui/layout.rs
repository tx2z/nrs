//! Layout calculations for the TUI.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::config::AppearanceConfig;

/// Minimum terminal dimensions.
pub const MIN_WIDTH: u16 = 40;
pub const MIN_HEIGHT: u16 = 10;

/// Main layout areas.
#[derive(Debug, Clone, Copy)]
pub struct MainLayout {
    /// Header area.
    pub header: Rect,
    /// Filter bar area.
    pub filter: Rect,
    /// Scripts grid area.
    pub scripts: Rect,
    /// Description panel area.
    pub description: Rect,
    /// Footer area.
    pub footer: Rect,
}

impl MainLayout {
    /// Calculate the main layout for the given area with default settings.
    pub fn new(area: Rect) -> Self {
        Self::with_config(area, &AppearanceConfig::default())
    }

    /// Calculate the main layout with configuration options.
    pub fn with_config(area: Rect, config: &AppearanceConfig) -> Self {
        // Handle minimum terminal size
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            return Self::minimal_layout(area);
        }

        let (header_height, filter_height, desc_height, footer_height) = if config.compact {
            (1, 1, 2, if config.show_footer { 1 } else { 0 })
        } else {
            (1, 1, 4, if config.show_footer { 1 } else { 0 })
        };

        let constraints = if config.show_footer {
            vec![
                Constraint::Length(header_height),
                Constraint::Length(filter_height),
                Constraint::Min(3), // Scripts (flexible, minimum 3 rows)
                Constraint::Length(desc_height),
                Constraint::Length(footer_height),
            ]
        } else {
            vec![
                Constraint::Length(header_height),
                Constraint::Length(filter_height),
                Constraint::Min(3),
                Constraint::Length(desc_height),
                Constraint::Length(0), // No footer
            ]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        Self {
            header: chunks[0],
            filter: chunks[1],
            scripts: chunks[2],
            description: chunks[3],
            footer: chunks[4],
        }
    }

    /// Create minimal layout for small terminals.
    fn minimal_layout(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Length(1), // Filter
                Constraint::Min(1),    // Scripts
                Constraint::Length(1), // Description
                Constraint::Length(1), // Footer
            ])
            .split(area);

        Self {
            header: chunks[0],
            filter: chunks[1],
            scripts: chunks[2],
            description: chunks[3],
            footer: chunks[4],
        }
    }

    /// Calculate available rows for scripts.
    pub fn script_rows(&self) -> usize {
        self.scripts.height as usize
    }

    /// Check if the terminal is too small.
    pub fn is_too_small(&self) -> bool {
        self.scripts.height < 1
    }
}

/// Calculate grid columns based on terminal width.
pub fn calculate_columns(width: u16) -> usize {
    match width {
        0..=59 => 1,
        60..=89 => 2,
        90..=119 => 3,
        120..=159 => 4,
        _ => 5,
    }
}

/// Calculate column width for the scripts grid.
pub fn calculate_column_width(total_width: u16, columns: usize) -> u16 {
    if columns == 0 {
        return total_width;
    }
    let padding = 2; // Left and right padding
    let available = total_width.saturating_sub(padding);
    let gap_count = columns.saturating_sub(1) as u16;
    let gaps_width = gap_count; // 1 char gap between columns
    available.saturating_sub(gaps_width) / columns as u16
}

/// Calculate grid layout for scripts.
#[derive(Debug, Clone, Copy)]
pub struct GridLayout {
    /// Number of columns.
    pub columns: usize,
    /// Number of rows.
    pub rows: usize,
    /// Width of each column.
    pub column_width: u16,
    /// Total visible items.
    pub visible_items: usize,
}

impl GridLayout {
    /// Create a new grid layout.
    pub fn new(area: Rect, total_items: usize) -> Self {
        let columns = calculate_columns(area.width);
        let column_width = calculate_column_width(area.width, columns);
        let rows = area.height as usize;
        let visible_items = (rows * columns).min(total_items);

        Self {
            columns,
            rows,
            column_width,
            visible_items,
        }
    }

    /// Get the row and column for a given index.
    pub fn position(&self, index: usize) -> (usize, usize) {
        let row = index / self.columns;
        let col = index % self.columns;
        (row, col)
    }

    /// Get the index for a given row and column.
    pub fn index(&self, row: usize, col: usize) -> usize {
        row * self.columns + col
    }

    /// Check if an index is visible.
    pub fn is_visible(&self, index: usize, scroll_offset: usize) -> bool {
        index >= scroll_offset && index < scroll_offset + self.visible_items
    }

    /// Get the display position for an index (accounting for scroll).
    pub fn display_position(&self, index: usize, scroll_offset: usize) -> Option<(usize, usize)> {
        if self.is_visible(index, scroll_offset) {
            let display_index = index - scroll_offset;
            Some(self.position(display_index))
        } else {
            None
        }
    }
}

/// Create a centered rectangle for popups/overlays.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Create a fixed-size centered rectangle for popups/overlays.
pub fn centered_rect_fixed(width: u16, height: u16, area: Rect) -> Rect {
    let actual_width = width.min(area.width);
    let actual_height = height.min(area.height);

    let x = area.x + (area.width.saturating_sub(actual_width)) / 2;
    let y = area.y + (area.height.saturating_sub(actual_height)) / 2;

    Rect::new(x, y, actual_width, actual_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_layout_default() {
        let area = Rect::new(0, 0, 100, 30);
        let layout = MainLayout::new(area);

        assert_eq!(layout.header.height, 1);
        assert_eq!(layout.filter.height, 1);
        assert!(layout.scripts.height >= 3);
        assert_eq!(layout.description.height, 4);
        assert_eq!(layout.footer.height, 1);
    }

    #[test]
    fn test_main_layout_compact() {
        let area = Rect::new(0, 0, 100, 30);
        let config = AppearanceConfig {
            compact: true,
            ..Default::default()
        };
        let layout = MainLayout::with_config(area, &config);

        assert_eq!(layout.description.height, 2);
    }

    #[test]
    fn test_main_layout_no_footer() {
        let area = Rect::new(0, 0, 100, 30);
        let config = AppearanceConfig {
            show_footer: false,
            ..Default::default()
        };
        let layout = MainLayout::with_config(area, &config);

        assert_eq!(layout.footer.height, 0);
    }

    #[test]
    fn test_main_layout_small_terminal() {
        let area = Rect::new(0, 0, 30, 8);
        let layout = MainLayout::new(area);

        // Should create minimal layout
        assert_eq!(layout.header.height, 1);
        assert_eq!(layout.filter.height, 1);
        assert_eq!(layout.description.height, 1);
        assert_eq!(layout.footer.height, 1);
    }

    #[test]
    fn test_calculate_columns() {
        assert_eq!(calculate_columns(40), 1);
        assert_eq!(calculate_columns(59), 1);
        assert_eq!(calculate_columns(60), 2);
        assert_eq!(calculate_columns(89), 2);
        assert_eq!(calculate_columns(90), 3);
        assert_eq!(calculate_columns(119), 3);
        assert_eq!(calculate_columns(120), 4);
        assert_eq!(calculate_columns(159), 4);
        assert_eq!(calculate_columns(160), 5);
    }

    #[test]
    fn test_calculate_column_width() {
        assert_eq!(calculate_column_width(100, 2), 48);
        assert_eq!(calculate_column_width(120, 3), 38);
        assert_eq!(calculate_column_width(60, 1), 58);
        assert_eq!(calculate_column_width(50, 0), 50);
    }

    #[test]
    fn test_grid_layout_position() {
        let area = Rect::new(0, 0, 100, 10);
        let grid = GridLayout::new(area, 30);

        assert_eq!(grid.columns, 3);
        assert_eq!(grid.rows, 10);

        assert_eq!(grid.position(0), (0, 0));
        assert_eq!(grid.position(1), (0, 1));
        assert_eq!(grid.position(2), (0, 2));
        assert_eq!(grid.position(3), (1, 0));
        assert_eq!(grid.position(5), (1, 2));
    }

    #[test]
    fn test_grid_layout_index() {
        let area = Rect::new(0, 0, 100, 10);
        let grid = GridLayout::new(area, 30);

        assert_eq!(grid.index(0, 0), 0);
        assert_eq!(grid.index(0, 2), 2);
        assert_eq!(grid.index(1, 0), 3);
        assert_eq!(grid.index(2, 1), 7);
    }

    #[test]
    fn test_grid_layout_visibility() {
        let area = Rect::new(0, 0, 60, 5); // 2 columns, 5 rows = 10 visible
        let grid = GridLayout::new(area, 20);

        assert_eq!(grid.visible_items, 10);
        assert!(grid.is_visible(0, 0));
        assert!(grid.is_visible(9, 0));
        assert!(!grid.is_visible(10, 0));

        // With scroll offset
        assert!(!grid.is_visible(0, 5));
        assert!(grid.is_visible(5, 5));
        assert!(grid.is_visible(14, 5));
    }

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect(50, 50, area);

        // Should be roughly centered
        assert!(centered.x >= 20 && centered.x <= 30);
        assert!(centered.y >= 10 && centered.y <= 15);
        assert!(centered.width >= 45 && centered.width <= 55);
        assert!(centered.height >= 22 && centered.height <= 28);
    }

    #[test]
    fn test_centered_rect_fixed() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect_fixed(40, 20, area);

        assert_eq!(centered.width, 40);
        assert_eq!(centered.height, 20);
        assert_eq!(centered.x, 30); // (100 - 40) / 2
        assert_eq!(centered.y, 15); // (50 - 20) / 2
    }

    #[test]
    fn test_centered_rect_fixed_clamps_to_area() {
        let area = Rect::new(0, 0, 30, 20);
        let centered = centered_rect_fixed(100, 50, area);

        assert_eq!(centered.width, 30);
        assert_eq!(centered.height, 20);
        assert_eq!(centered.x, 0);
        assert_eq!(centered.y, 0);
    }
}
