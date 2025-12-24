//! Terminal utilities.

use std::io::{self, Write};

use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};

/// Minimum terminal dimensions.
pub const MIN_WIDTH: u16 = 40;
pub const MIN_HEIGHT: u16 = 10;

/// Terminal size information.
#[derive(Debug, Clone, Copy)]
pub struct TerminalSize {
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

impl TerminalSize {
    /// Check if the terminal size meets minimum requirements.
    pub fn is_valid(&self) -> bool {
        self.width >= MIN_WIDTH && self.height >= MIN_HEIGHT
    }

    /// Calculate the number of columns for the scripts grid.
    pub fn grid_columns(&self) -> usize {
        match self.width {
            0..60 => 1,
            60..90 => 2,
            90..120 => 3,
            120..160 => 4,
            _ => 5,
        }
    }
}

/// Check the terminal size.
///
/// Returns the current terminal size, or None if it cannot be determined.
pub fn check_terminal_size() -> Option<TerminalSize> {
    terminal::size()
        .ok()
        .map(|(width, height)| TerminalSize { width, height })
}

/// Enable raw mode for TUI.
///
/// This enables character-by-character input without echo,
/// which is required for TUI applications.
///
/// # Errors
///
/// Returns an error if raw mode cannot be enabled.
pub fn enable_raw_mode() -> io::Result<()> {
    terminal::enable_raw_mode()
}

/// Disable raw mode.
///
/// Restores the terminal to its normal state where input
/// is line-buffered and echoed.
///
/// # Errors
///
/// Returns an error if raw mode cannot be disabled.
pub fn disable_raw_mode() -> io::Result<()> {
    terminal::disable_raw_mode()
}

/// Check if the terminal is currently in raw mode.
pub fn is_raw_mode_enabled() -> bool {
    terminal::is_raw_mode_enabled().unwrap_or(false)
}

/// Enter the alternate screen buffer.
///
/// This preserves the current terminal content and provides
/// a clean screen for TUI rendering. When the TUI exits,
/// the original content can be restored.
///
/// # Errors
///
/// Returns an error if the alternate screen cannot be entered.
pub fn enter_alternate_screen() -> io::Result<()> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    Ok(())
}

/// Leave the alternate screen buffer.
///
/// Restores the original terminal content that was saved
/// when entering the alternate screen.
///
/// # Errors
///
/// Returns an error if the alternate screen cannot be left.
pub fn leave_alternate_screen() -> io::Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// Show the cursor.
///
/// # Errors
///
/// Returns an error if the cursor cannot be shown.
pub fn show_cursor() -> io::Result<()> {
    execute!(io::stdout(), cursor::Show)?;
    Ok(())
}

/// Hide the cursor.
///
/// # Errors
///
/// Returns an error if the cursor cannot be hidden.
pub fn hide_cursor() -> io::Result<()> {
    execute!(io::stdout(), cursor::Hide)?;
    Ok(())
}

/// Prepare terminal for script execution.
///
/// This function should be called before running a script to ensure
/// the terminal is in a normal state:
/// - Disables raw mode
/// - Leaves alternate screen
/// - Shows cursor
///
/// # Errors
///
/// Returns an error if terminal cleanup fails.
pub fn prepare_for_script_execution() -> io::Result<()> {
    // Only disable raw mode if it's enabled
    if is_raw_mode_enabled() {
        disable_raw_mode()?;
    }

    // Leave alternate screen
    leave_alternate_screen()?;

    // Show cursor
    show_cursor()?;

    // Flush stdout
    io::stdout().flush()?;

    Ok(())
}

/// Restore terminal for TUI.
///
/// This function should be called after a script finishes to restore
/// the terminal to TUI mode:
/// - Enables raw mode
/// - Enters alternate screen
/// - Hides cursor
///
/// # Errors
///
/// Returns an error if terminal setup fails.
pub fn restore_for_tui() -> io::Result<()> {
    // Enter alternate screen
    enter_alternate_screen()?;

    // Enable raw mode
    enable_raw_mode()?;

    // Hide cursor
    hide_cursor()?;

    // Flush stdout
    io::stdout().flush()?;

    Ok(())
}

/// Cleanup terminal completely.
///
/// This function should be called when the application exits to ensure
/// the terminal is in a clean state:
/// - Disables raw mode
/// - Leaves alternate screen
/// - Shows cursor
///
/// This is similar to `prepare_for_script_execution` but doesn't check
/// raw mode status first - it always tries to clean up.
///
/// # Errors
///
/// Returns an error if terminal cleanup fails.
pub fn cleanup_terminal() -> io::Result<()> {
    // Always try to disable raw mode
    let _ = disable_raw_mode();

    // Leave alternate screen
    let _ = leave_alternate_screen();

    // Show cursor
    let _ = show_cursor();

    // Flush stdout
    io::stdout().flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_size_validity() {
        let valid = TerminalSize {
            width: 80,
            height: 24,
        };
        assert!(valid.is_valid());

        let too_small = TerminalSize {
            width: 30,
            height: 5,
        };
        assert!(!too_small.is_valid());
    }

    #[test]
    fn test_grid_columns() {
        assert_eq!(
            TerminalSize {
                width: 50,
                height: 24
            }
            .grid_columns(),
            1
        );
        assert_eq!(
            TerminalSize {
                width: 80,
                height: 24
            }
            .grid_columns(),
            2
        );
        assert_eq!(
            TerminalSize {
                width: 100,
                height: 24
            }
            .grid_columns(),
            3
        );
        assert_eq!(
            TerminalSize {
                width: 140,
                height: 24
            }
            .grid_columns(),
            4
        );
        assert_eq!(
            TerminalSize {
                width: 200,
                height: 24
            }
            .grid_columns(),
            5
        );
    }
}
