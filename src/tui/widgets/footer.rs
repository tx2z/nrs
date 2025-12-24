//! Footer widget for the TUI.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::tui::app::AppMode;
use crate::tui::theme::Theme;

/// Footer widget showing keybinding hints.
pub struct Footer<'a> {
    mode: &'a AppMode,
    theme: &'a Theme,
}

impl<'a> Footer<'a> {
    /// Create a new footer widget.
    pub fn new(mode: &'a AppMode, theme: &'a Theme) -> Self {
        Self { mode, theme }
    }

    /// Get keybinding hints for the current mode.
    fn get_hints(&self) -> Vec<(&'static str, &'static str)> {
        match self.mode {
            AppMode::Normal => vec![
                ("j/k", "move"),
                ("Enter", "run"),
                ("1-9", "quick"),
                ("/", "filter"),
                ("?", "help"),
                ("q", "quit"),
            ],
            AppMode::Filter { .. } => vec![("j/k", "move"), ("Enter", "run"), ("Esc", "cancel")],
            AppMode::Help => vec![("any key", "close")],
            AppMode::Error { .. } => vec![("any key", "dismiss")],
            AppMode::MultiSelect { .. } => {
                vec![("Space", "toggle"), ("Enter", "run"), ("Esc", "cancel")]
            }
            AppMode::Args { .. } => vec![("Enter", "run"), ("Esc", "cancel")],
            AppMode::WorkspaceSelect => vec![
                ("j/k", "move"),
                ("Enter", "select"),
                ("1-9", "quick"),
                ("q", "quit"),
            ],
        }
    }

    /// Build the footer line with adaptive width.
    fn build_line(&self, width: u16) -> Line<'a> {
        let hints = self.get_hints();

        // Calculate total width needed for full hints
        let full_width: usize = hints
            .iter()
            .map(|(key, action)| key.len() + action.len() + 3) // "key action  "
            .sum();

        // Determine display mode based on available width
        let mut spans = vec![Span::raw(" ")];

        if (width as usize) >= full_width + 2 {
            // Full display
            for (i, (key, action)) in hints.iter().enumerate() {
                spans.push(Span::styled(*key, self.theme.key()));
                spans.push(Span::styled(format!(" {} ", action), self.theme.footer()));
                if i < hints.len() - 1 {
                    spans.push(Span::styled(" ", self.theme.footer()));
                }
            }
        } else if (width as usize) >= hints.len() * 4 {
            // Compact display (just keys)
            for (i, (key, _)) in hints.iter().enumerate() {
                spans.push(Span::styled(*key, self.theme.key()));
                if i < hints.len() - 1 {
                    spans.push(Span::styled(" ", self.theme.footer()));
                }
            }
        } else {
            // Ultra compact - just show first few hints
            let max_hints = ((width as usize) / 6).max(1).min(hints.len());
            for (i, (key, _)) in hints.iter().take(max_hints).enumerate() {
                spans.push(Span::styled(*key, self.theme.key()));
                if i < max_hints - 1 {
                    spans.push(Span::styled(" ", self.theme.footer()));
                }
            }
        }

        Line::from(spans)
    }
}

impl Widget for Footer<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let line = self.build_line(area.width);
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}

/// Simple message footer for status messages.
pub struct MessageFooter<'a> {
    message: &'a str,
    theme: &'a Theme,
    is_error: bool,
}

impl<'a> MessageFooter<'a> {
    /// Create a new message footer.
    pub fn new(message: &'a str, theme: &'a Theme) -> Self {
        Self {
            message,
            theme,
            is_error: false,
        }
    }

    /// Mark this as an error message.
    pub fn error(mut self) -> Self {
        self.is_error = true;
        self
    }
}

impl Widget for MessageFooter<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let style = if self.is_error {
            self.theme.error()
        } else {
            self.theme.footer()
        };

        let line = Line::from(vec![Span::raw(" "), Span::styled(self.message, style)]);
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_normal_mode() {
        let mode = AppMode::Normal;
        let theme = Theme::default();
        let footer = Footer::new(&mode, &theme);

        let hints = footer.get_hints();
        assert!(!hints.is_empty());
        assert!(hints.iter().any(|(k, _)| *k == "q"));
    }

    #[test]
    fn test_footer_filter_mode() {
        let mode = AppMode::Filter {
            query: String::new(),
        };
        let theme = Theme::default();
        let footer = Footer::new(&mode, &theme);

        let hints = footer.get_hints();
        assert!(hints.iter().any(|(k, _)| *k == "Esc"));
    }

    #[test]
    fn test_footer_help_mode() {
        let mode = AppMode::Help;
        let theme = Theme::default();
        let footer = Footer::new(&mode, &theme);

        let hints = footer.get_hints();
        assert!(hints.iter().any(|(_, a)| *a == "close"));
    }

    #[test]
    fn test_footer_adaptive_width() {
        let mode = AppMode::Normal;
        let theme = Theme::default();
        let footer = Footer::new(&mode, &theme);

        // Full width
        let line = footer.build_line(100);
        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(content.contains("move")); // Full text

        // Narrow width
        let footer = Footer::new(&mode, &theme);
        let line = footer.build_line(20);
        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        // Should still have something
        assert!(!content.trim().is_empty());
    }

    #[test]
    fn test_message_footer() {
        let theme = Theme::default();
        let _footer = MessageFooter::new("Script completed", &theme);
    }

    #[test]
    fn test_message_footer_error() {
        let theme = Theme::default();
        let footer = MessageFooter::new("Error occurred", &theme).error();
        assert!(footer.is_error);
    }
}
