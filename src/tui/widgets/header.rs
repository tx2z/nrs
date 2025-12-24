//! Header widget for the TUI.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::config::AppearanceConfig;
use crate::package::Runner;
use crate::tui::theme::Theme;

/// Header widget showing project name and runner info.
pub struct Header<'a> {
    project_name: &'a str,
    runner: Runner,
    theme: &'a Theme,
    show_icons: bool,
}

impl<'a> Header<'a> {
    /// Create a new header widget.
    pub fn new(
        project_name: &'a str,
        runner: Runner,
        theme: &'a Theme,
        config: &AppearanceConfig,
    ) -> Self {
        Self {
            project_name,
            runner,
            theme,
            show_icons: config.icons,
        }
    }

    /// Build the header line.
    fn build_line(&self, width: u16) -> Line<'a> {
        let icon = if self.show_icons {
            self.runner.icon()
        } else {
            ""
        };

        let help_hint = "[?]";

        // Calculate available space for project name
        let icon_len = if self.show_icons { 2 } else { 0 }; // icon + space
        let runner_part = format!(" {} ", self.runner.display_name());
        let help_len = help_hint.len() + 2; // help + spaces
        let fixed_parts = icon_len + runner_part.len() + help_len + 4; // padding/separators

        let max_project_len = (width as usize).saturating_sub(fixed_parts);
        let project_display = truncate_with_ellipsis(self.project_name, max_project_len);

        // Build spans
        let mut spans = Vec::new();

        // Left side: icon + project name
        spans.push(Span::raw(" "));
        if self.show_icons && !icon.is_empty() {
            spans.push(Span::styled(
                format!("{} ", icon),
                self.theme.header_project(),
            ));
        }
        spans.push(Span::styled(project_display, self.theme.header_project()));

        // Calculate padding to right-align runner info
        let left_len = spans.iter().map(|s| s.content.len()).sum::<usize>();
        let right_content = format!("{} {} ", runner_part, help_hint);
        let padding_len = (width as usize).saturating_sub(left_len + right_content.len());

        if padding_len > 0 {
            spans.push(Span::styled(" ".repeat(padding_len), self.theme.header()));
        }

        // Right side: runner + help
        spans.push(Span::styled(runner_part, self.theme.header_runner()));
        spans.push(Span::styled(help_hint, self.theme.header()));
        spans.push(Span::raw(" "));

        Line::from(spans)
    }
}

impl Widget for Header<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let line = self.build_line(area.width);
        let paragraph = Paragraph::new(line).style(self.theme.header());
        paragraph.render(area, buf);
    }
}

/// Truncate a string with ellipsis if it exceeds max length.
///
/// Handles Unicode characters properly by counting characters, not bytes.
/// Uses the Unicode ellipsis character (…) which is more compact.
pub fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();

    if char_count <= max_len {
        s.to_string()
    } else if max_len == 0 {
        String::new()
    } else if max_len <= 3 {
        // For very short lengths, just truncate without ellipsis
        s.chars().take(max_len).collect()
    } else {
        // Leave room for ellipsis (1 character)
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{}…", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_with_ellipsis() {
        assert_eq!(truncate_with_ellipsis("hello", 10), "hello");
        assert_eq!(truncate_with_ellipsis("hello world", 8), "hello w…");
        assert_eq!(truncate_with_ellipsis("hi", 2), "hi");
        assert_eq!(truncate_with_ellipsis("hello", 5), "hello");
        assert_eq!(truncate_with_ellipsis("hello", 4), "hel…");
    }

    #[test]
    fn test_truncate_very_short() {
        // When max_len <= 3, just truncate without ellipsis
        assert_eq!(truncate_with_ellipsis("hello", 3), "hel");
        assert_eq!(truncate_with_ellipsis("hello", 2), "he");
        assert_eq!(truncate_with_ellipsis("hello", 1), "h");
    }

    #[test]
    fn test_truncate_zero_length() {
        assert_eq!(truncate_with_ellipsis("hello", 0), "");
    }

    #[test]
    fn test_truncate_unicode() {
        // Japanese text - each character is one code point (5 chars)
        assert_eq!(truncate_with_ellipsis("こんにちは", 6), "こんにちは"); // 5 chars fits in 6
        assert_eq!(truncate_with_ellipsis("こんにちは", 5), "こんにちは"); // 5 chars fits exactly
        assert_eq!(truncate_with_ellipsis("こんにちは", 4), "こんに…"); // 5 > 4, truncate with ellipsis

        // Emoji - each emoji is one code point
        assert_eq!(truncate_with_ellipsis("🚀🎉🔥", 4), "🚀🎉🔥"); // 3 chars fits in 4
        assert_eq!(truncate_with_ellipsis("🚀🎉🔥", 3), "🚀🎉🔥"); // 3 chars fits exactly
        assert_eq!(truncate_with_ellipsis("🚀🎉🔥🎯", 4), "🚀🎉🔥🎯"); // 4 chars fits exactly
                                                                       // For max_len <= 3, we truncate without ellipsis (no room for ellipsis)
        assert_eq!(truncate_with_ellipsis("🚀🎉🔥🎯", 3), "🚀🎉🔥"); // 4 > 3, but max_len <= 3
        assert_eq!(truncate_with_ellipsis("🚀🎉🔥🎯🌟", 4), "🚀🎉🔥…"); // 5 > 4, truncate with ellipsis

        // Mixed ASCII and Unicode (7 chars total)
        assert_eq!(truncate_with_ellipsis("hello世界", 8), "hello世界"); // 7 chars fits in 8
        assert_eq!(truncate_with_ellipsis("hello世界", 7), "hello世界"); // 7 chars fits exactly
        assert_eq!(truncate_with_ellipsis("hello世界", 6), "hello…"); // 7 > 6, truncate with ellipsis
    }

    #[test]
    fn test_header_build_line() {
        let theme = Theme::default();
        let config = AppearanceConfig::default();
        let header = Header::new("my-project", Runner::Npm, &theme, &config);

        let line = header.build_line(80);
        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();

        assert!(content.contains("my-project"));
        assert!(content.contains("npm"));
        assert!(content.contains("[?]"));
    }

    #[test]
    fn test_header_unicode_project_name() {
        let theme = Theme::default();
        let config = AppearanceConfig::default();
        let header = Header::new("日本語プロジェクト", Runner::Npm, &theme, &config);

        let line = header.build_line(80);
        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();

        assert!(content.contains("日本語"));
        assert!(content.contains("npm"));
    }
}
