//! Description panel widget for the TUI.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::config::AppearanceConfig;
use crate::package::{get_description, Script};
use crate::tui::theme::Theme;

/// Description panel widget.
pub struct Description<'a> {
    script: Option<&'a Script>,
    theme: &'a Theme,
    show_command: bool,
    compact: bool,
}

impl<'a> Description<'a> {
    /// Create a new description widget.
    pub fn new(script: Option<&'a Script>, theme: &'a Theme, config: &AppearanceConfig) -> Self {
        Self {
            script,
            theme,
            show_command: config.icons, // Reusing icons flag for command preview
            compact: config.compact,
        }
    }

    /// Create description with explicit command preview setting.
    pub fn with_command_preview(mut self, show: bool) -> Self {
        self.show_command = show;
        self
    }

    /// Build lines for the description panel.
    fn build_lines(&self, width: u16) -> Vec<Line<'a>> {
        let Some(script) = self.script else {
            return vec![Line::from(Span::styled(
                "No script selected",
                self.theme.description(),
            ))];
        };

        let mut lines = Vec::new();

        // Description text
        let desc = get_description(script);
        let desc_text = if desc.is_empty() {
            "No description".to_string()
        } else {
            desc.to_string()
        };
        lines.push(Line::from(Span::styled(
            desc_text,
            self.theme.description(),
        )));

        // Separator (only in non-compact mode)
        if !self.compact && self.show_command {
            let sep_width = (width as usize).min(60);
            let separator = "─".repeat(sep_width);
            lines.push(Line::from(Span::styled(separator, self.theme.separator())));
        }

        // Command preview
        if self.show_command {
            let command = script.command();
            let cmd_display = if command.len() > width as usize - 4 {
                // Truncate long commands
                let truncated: String = command.chars().take(width as usize - 7).collect();
                format!("$ {}...", truncated)
            } else {
                format!("$ {}", command)
            };
            lines.push(Line::from(Span::styled(cmd_display, self.theme.command())));
        }

        lines
    }
}

impl Widget for Description<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let lines = self.build_lines(area.width);
        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        paragraph.render(area, buf);
    }
}

/// Error display widget.
pub struct ErrorDisplay<'a> {
    message: &'a str,
    theme: &'a Theme,
}

impl<'a> ErrorDisplay<'a> {
    /// Create a new error display widget.
    pub fn new(message: &'a str, theme: &'a Theme) -> Self {
        Self { message, theme }
    }

    /// Build lines for the error display.
    fn build_lines(&self) -> Vec<Line<'a>> {
        vec![
            Line::from(Span::styled("Error", self.theme.error())),
            Line::from(Span::styled(self.message, self.theme.description())),
        ]
    }
}

impl Widget for ErrorDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let lines = self.build_lines();
        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_description_no_script() {
        let theme = Theme::default();
        let config = AppearanceConfig::default();
        let desc = Description::new(None, &theme, &config);

        let lines = desc.build_lines(80);
        let content: String = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.to_string())
            .collect();

        assert!(content.contains("No script selected"));
    }

    #[test]
    fn test_description_with_script() {
        let theme = Theme::default();
        let config = AppearanceConfig::default();
        let script = Script::new("dev", "vite --mode development");

        let desc = Description::new(Some(&script), &theme, &config);
        let lines = desc.build_lines(80);

        let content: String = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.to_string())
            .collect();

        assert!(content.contains("vite"));
    }

    #[test]
    fn test_description_compact() {
        let theme = Theme::default();
        let config = AppearanceConfig {
            compact: true,
            ..Default::default()
        };
        let script = Script::new("dev", "vite");

        let desc = Description::new(Some(&script), &theme, &config);
        let lines = desc.build_lines(80);

        // Compact mode should have fewer lines (no separator)
        assert!(lines.len() <= 2);
    }

    #[test]
    fn test_description_long_command() {
        let theme = Theme::default();
        let config = AppearanceConfig::default();
        let long_cmd =
            "webpack --config webpack.config.js --mode production --env NODE_ENV=production";
        let script = Script::new("build", long_cmd);

        let desc = Description::new(Some(&script), &theme, &config);
        let lines = desc.build_lines(40);

        let content: String = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.to_string())
            .collect();

        // Should be truncated
        assert!(content.contains("..."));
    }

    #[test]
    fn test_error_display() {
        let theme = Theme::default();
        let error = ErrorDisplay::new("Something went wrong", &theme);

        let lines = error.build_lines();
        let content: String = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.to_string())
            .collect();

        assert!(content.contains("Error"));
        assert!(content.contains("Something went wrong"));
    }
}
