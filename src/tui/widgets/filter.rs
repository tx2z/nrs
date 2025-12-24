//! Filter widget for the TUI.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::config::AppearanceConfig;
use crate::tui::theme::Theme;

/// Filter bar widget.
pub struct Filter<'a> {
    query: &'a str,
    is_active: bool,
    theme: &'a Theme,
    show_icon: bool,
    blink_state: bool,
}

impl<'a> Filter<'a> {
    /// Create a new filter widget.
    pub fn new(
        query: &'a str,
        is_active: bool,
        theme: &'a Theme,
        config: &AppearanceConfig,
    ) -> Self {
        Self {
            query,
            is_active,
            theme,
            show_icon: config.icons,
            blink_state: true, // Default to cursor visible
        }
    }

    /// Set the blink state for the cursor.
    pub fn blink(mut self, state: bool) -> Self {
        self.blink_state = state;
        self
    }

    /// Build the filter line.
    fn build_line(&self) -> Line<'a> {
        let icon = if self.show_icon { " " } else { "" }; // magnifying glass

        if self.is_active {
            // Active filter mode
            let mut spans = vec![
                Span::styled(format!("{} / ", icon), self.theme.filter_active()),
                Span::styled(self.query.to_string(), self.theme.filter_active()),
            ];

            // Add blinking cursor
            if self.blink_state {
                spans.push(Span::styled("_", self.theme.filter_active()));
            } else {
                spans.push(Span::raw(" "));
            }

            Line::from(spans)
        } else if !self.query.is_empty() {
            // Has filter but not active (showing results)
            Line::from(vec![
                Span::styled(format!("{} / ", icon), self.theme.filter()),
                Span::styled(self.query.to_string(), self.theme.filter()),
            ])
        } else {
            // Placeholder
            let placeholder = format!("{} Type / to filter...", icon);
            Line::from(vec![Span::styled(
                placeholder,
                self.theme.filter_placeholder(),
            )])
        }
    }
}

impl Widget for Filter<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let line = self.build_line();
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}

/// Filter bar for Args mode (entering arguments).
pub struct ArgsFilter<'a> {
    script_name: &'a str,
    args: &'a str,
    theme: &'a Theme,
    blink_state: bool,
}

impl<'a> ArgsFilter<'a> {
    /// Create a new args filter widget.
    pub fn new(script_name: &'a str, args: &'a str, theme: &'a Theme) -> Self {
        Self {
            script_name,
            args,
            theme,
            blink_state: true,
        }
    }

    /// Set the blink state for the cursor.
    pub fn blink(mut self, state: bool) -> Self {
        self.blink_state = state;
        self
    }

    /// Build the args input line.
    fn build_line(&self) -> Line<'a> {
        let mut spans = vec![
            Span::styled(" Args for ", self.theme.filter_placeholder()),
            Span::styled(self.script_name.to_string(), self.theme.filter()),
            Span::styled(": ", self.theme.filter_placeholder()),
            Span::styled(self.args.to_string(), self.theme.filter_active()),
        ];

        // Add blinking cursor
        if self.blink_state {
            spans.push(Span::styled("_", self.theme.filter_active()));
        } else {
            spans.push(Span::raw(" "));
        }

        Line::from(spans)
    }
}

impl Widget for ArgsFilter<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let line = self.build_line();
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_placeholder() {
        let theme = Theme::default();
        let config = AppearanceConfig::default();
        let filter = Filter::new("", false, &theme, &config);

        let line = filter.build_line();
        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();

        assert!(content.contains("filter"));
    }

    #[test]
    fn test_filter_active() {
        let theme = Theme::default();
        let config = AppearanceConfig::default();
        let filter = Filter::new("test", true, &theme, &config);

        let line = filter.build_line();
        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();

        assert!(content.contains("test"));
        assert!(content.contains("_")); // cursor
    }

    #[test]
    fn test_filter_with_query_inactive() {
        let theme = Theme::default();
        let config = AppearanceConfig::default();
        let filter = Filter::new("dev", false, &theme, &config);

        let line = filter.build_line();
        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();

        assert!(content.contains("dev"));
        assert!(!content.contains("_")); // no cursor when inactive
    }

    #[test]
    fn test_args_filter() {
        let theme = Theme::default();
        let args_filter = ArgsFilter::new("dev", "--watch", &theme);

        let line = args_filter.build_line();
        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();

        assert!(content.contains("dev"));
        assert!(content.contains("--watch"));
    }
}
