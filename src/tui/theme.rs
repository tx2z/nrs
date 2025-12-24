//! Color theme for the TUI.

use ratatui::style::{Color, Modifier, Style};

use crate::config::Theme as ThemeConfig;

/// Color theme for the TUI.
#[derive(Debug, Clone)]
pub struct Theme {
    // Header
    header_bg: Color,
    header_fg: Color,

    // Filter
    filter_fg: Color,
    filter_placeholder_fg: Color,

    // Scripts
    number_fg: Color,
    script_fg: Color,
    selected_bg: Color,
    selected_fg: Color,
    cursor_fg: Color,
    multiselect_fg: Color,

    // Description
    description_fg: Color,
    command_fg: Color,
    separator_fg: Color,

    // Footer
    footer_fg: Color,
    key_fg: Color,

    // Status
    error_fg: Color,
    success_fg: Color,
    warning_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(&ThemeConfig::Default)
    }
}

impl Theme {
    /// Create a theme from configuration.
    pub fn new(config: &ThemeConfig) -> Self {
        match config {
            ThemeConfig::Default => Self::default_theme(),
            ThemeConfig::Minimal => Self::minimal_theme(),
            ThemeConfig::None => Self::no_color_theme(),
        }
    }

    /// Default full-color theme.
    fn default_theme() -> Self {
        Self {
            header_bg: Color::Blue,
            header_fg: Color::White,

            filter_fg: Color::Yellow,
            filter_placeholder_fg: Color::DarkGray,

            number_fg: Color::Cyan,
            script_fg: Color::White,
            selected_bg: Color::Blue,
            selected_fg: Color::White,
            cursor_fg: Color::Green,
            multiselect_fg: Color::Magenta,

            description_fg: Color::Gray,
            command_fg: Color::DarkGray,
            separator_fg: Color::DarkGray,

            footer_fg: Color::DarkGray,
            key_fg: Color::Cyan,

            error_fg: Color::Red,
            success_fg: Color::Green,
            warning_fg: Color::Yellow,
        }
    }

    /// Minimal color theme (fewer colors, less bold).
    fn minimal_theme() -> Self {
        Self {
            header_bg: Color::Reset,
            header_fg: Color::White,

            filter_fg: Color::White,
            filter_placeholder_fg: Color::DarkGray,

            number_fg: Color::Gray,
            script_fg: Color::White,
            selected_bg: Color::Reset,
            selected_fg: Color::Cyan,
            cursor_fg: Color::White,
            multiselect_fg: Color::White,

            description_fg: Color::Gray,
            command_fg: Color::DarkGray,
            separator_fg: Color::DarkGray,

            footer_fg: Color::DarkGray,
            key_fg: Color::Gray,

            error_fg: Color::Red,
            success_fg: Color::Green,
            warning_fg: Color::Yellow,
        }
    }

    /// No-color theme (monochrome).
    fn no_color_theme() -> Self {
        Self {
            header_bg: Color::Reset,
            header_fg: Color::Reset,

            filter_fg: Color::Reset,
            filter_placeholder_fg: Color::Reset,

            number_fg: Color::Reset,
            script_fg: Color::Reset,
            selected_bg: Color::Reset,
            selected_fg: Color::Reset,
            cursor_fg: Color::Reset,
            multiselect_fg: Color::Reset,

            description_fg: Color::Reset,
            command_fg: Color::Reset,
            separator_fg: Color::Reset,

            footer_fg: Color::Reset,
            key_fg: Color::Reset,

            error_fg: Color::Reset,
            success_fg: Color::Reset,
            warning_fg: Color::Reset,
        }
    }

    // ==================== Header Styles ====================

    /// Get the header style.
    pub fn header(&self) -> Style {
        Style::default()
            .fg(self.header_fg)
            .bg(self.header_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get the header project name style.
    pub fn header_project(&self) -> Style {
        Style::default()
            .fg(self.header_fg)
            .bg(self.header_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get the header runner style.
    pub fn header_runner(&self) -> Style {
        Style::default().fg(self.header_fg).bg(self.header_bg)
    }

    // ==================== Filter Styles ====================

    /// Get the filter text style.
    pub fn filter(&self) -> Style {
        Style::default().fg(self.filter_fg)
    }

    /// Get the filter active style (when in filter mode).
    pub fn filter_active(&self) -> Style {
        Style::default()
            .fg(self.filter_fg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get the filter placeholder style.
    pub fn filter_placeholder(&self) -> Style {
        Style::default()
            .fg(self.filter_placeholder_fg)
            .add_modifier(Modifier::ITALIC)
    }

    // ==================== Script Styles ====================

    /// Get the script number style.
    pub fn number(&self) -> Style {
        Style::default()
            .fg(self.number_fg)
            .add_modifier(Modifier::DIM)
    }

    /// Get the script name style.
    pub fn script(&self) -> Style {
        Style::default().fg(self.script_fg)
    }

    /// Get the selected script style.
    pub fn selected(&self) -> Style {
        if self.selected_bg == Color::Reset {
            Style::default()
                .fg(self.selected_fg)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default()
                .fg(self.selected_fg)
                .bg(self.selected_bg)
                .add_modifier(Modifier::BOLD)
        }
    }

    /// Get the cursor style.
    pub fn cursor(&self) -> Style {
        Style::default()
            .fg(self.cursor_fg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get the multiselect marker style.
    pub fn multiselect(&self) -> Style {
        Style::default()
            .fg(self.multiselect_fg)
            .add_modifier(Modifier::BOLD)
    }

    // ==================== Description Styles ====================

    /// Get the description style.
    pub fn description(&self) -> Style {
        Style::default().fg(self.description_fg)
    }

    /// Get the command preview style.
    pub fn command(&self) -> Style {
        Style::default()
            .fg(self.command_fg)
            .add_modifier(Modifier::ITALIC)
    }

    /// Get the separator style.
    pub fn separator(&self) -> Style {
        Style::default().fg(self.separator_fg)
    }

    // ==================== Footer Styles ====================

    /// Get the footer style.
    pub fn footer(&self) -> Style {
        Style::default().fg(self.footer_fg)
    }

    /// Get the keybinding style.
    pub fn key(&self) -> Style {
        Style::default()
            .fg(self.key_fg)
            .add_modifier(Modifier::BOLD)
    }

    // ==================== Status Styles ====================

    /// Get the error style.
    pub fn error(&self) -> Style {
        Style::default()
            .fg(self.error_fg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get the success style.
    pub fn success(&self) -> Style {
        Style::default().fg(self.success_fg)
    }

    /// Get the warning style.
    pub fn warning(&self) -> Style {
        Style::default().fg(self.warning_fg)
    }

    // ==================== Utility Styles ====================

    /// Get style for dimmed/muted text.
    pub fn dim(&self) -> Style {
        Style::default().add_modifier(Modifier::DIM)
    }

    /// Get bold style.
    pub fn bold(&self) -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.header_bg, Color::Blue);
        assert_eq!(theme.header_fg, Color::White);
    }

    #[test]
    fn test_theme_from_config() {
        let theme = Theme::new(&ThemeConfig::Default);
        assert_eq!(theme.header_bg, Color::Blue);

        let theme = Theme::new(&ThemeConfig::Minimal);
        assert_eq!(theme.header_bg, Color::Reset);

        let theme = Theme::new(&ThemeConfig::None);
        assert_eq!(theme.header_fg, Color::Reset);
    }

    #[test]
    fn test_header_style() {
        let theme = Theme::default();
        let style = theme.header();

        // Should have bold modifier
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_selected_style_minimal() {
        let theme = Theme::new(&ThemeConfig::Minimal);
        let style = theme.selected();

        // Minimal theme uses underline instead of background
        assert!(style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn test_no_color_theme() {
        let theme = Theme::new(&ThemeConfig::None);

        // All colors should be Reset
        assert_eq!(theme.header_bg, Color::Reset);
        assert_eq!(theme.filter_fg, Color::Reset);
        assert_eq!(theme.number_fg, Color::Reset);
    }
}
