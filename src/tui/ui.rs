//! Main UI rendering and TUI loop.

use std::io::{self, stdout, Stdout, Write};
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::{
    cursor, event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};

use super::app::{App, AppMode, ScriptRun};
use super::input::handle_event;
use super::layout::{centered_rect_fixed, MainLayout};
use super::theme::Theme;
use super::widgets::{ArgsFilter, Description, EmptyScripts, Filter, Footer, Header, ScriptsGrid};

/// Blink interval for cursor (in milliseconds).
const CURSOR_BLINK_MS: u64 = 530;

/// Global flag to track if terminal is in raw mode.
static TERMINAL_RAW_MODE: AtomicBool = AtomicBool::new(false);

/// RAII guard for terminal state.
/// Ensures terminal is properly restored even on panic.
pub struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    /// Create a new terminal guard, setting up the terminal for TUI.
    pub fn new() -> Result<Self> {
        // Set up panic hook before entering raw mode
        setup_panic_hook();

        enable_raw_mode().context("Failed to enable raw mode")?;
        TERMINAL_RAW_MODE.store(true, Ordering::SeqCst);

        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)
            .context("Failed to enter alternate screen")?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("Failed to create terminal")?;

        Ok(Self { terminal })
    }

    /// Get a mutable reference to the terminal.
    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Restore terminal state
        let _ = disable_raw_mode();
        TERMINAL_RAW_MODE.store(false, Ordering::SeqCst);
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            cursor::Show
        );
    }
}

/// Set up a panic hook that restores the terminal.
fn setup_panic_hook() {
    let original_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal
        if TERMINAL_RAW_MODE.load(Ordering::SeqCst) {
            let _ = disable_raw_mode();
            let _ = execute!(stdout(), LeaveAlternateScreen, cursor::Show);
        }

        // Call the original panic hook
        original_hook(panic_info);
    }));
}

/// Restore terminal to normal state.
/// Call this before running external commands.
pub fn restore_terminal() -> Result<()> {
    if TERMINAL_RAW_MODE.load(Ordering::SeqCst) {
        disable_raw_mode().context("Failed to disable raw mode")?;
        execute!(stdout(), LeaveAlternateScreen, cursor::Show)
            .context("Failed to leave alternate screen")?;
        TERMINAL_RAW_MODE.store(false, Ordering::SeqCst);
    }
    io::stdout().flush()?;
    Ok(())
}

/// Run the TUI application.
///
/// Returns the scripts to run after TUI exits, along with their arguments.
pub fn run_tui(mut app: App) -> Result<Vec<ScriptRun>> {
    let mut guard = TerminalGuard::new()?;

    // Main loop
    let result = run_loop(guard.terminal(), &mut app);

    // Guard will restore terminal on drop
    drop(guard);

    result?;

    // Return all scripts to run
    if let Some(script_run) = app.script_to_run() {
        Ok(vec![script_run.clone()])
    } else {
        Ok(vec![])
    }
}

/// Main TUI loop.
fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    let theme = Theme::new(&app.config().appearance.theme);
    let mut last_blink = Instant::now();
    let mut blink_state = true;

    loop {
        // Update blink state
        if last_blink.elapsed() >= Duration::from_millis(CURSOR_BLINK_MS) {
            blink_state = !blink_state;
            last_blink = Instant::now();
        }

        // Update columns based on terminal size
        let size = terminal.size()?;
        app.update_columns(size.width);

        // Draw UI
        terminal.draw(|frame| render(frame, app, &theme, blink_state))?;

        // Handle events
        if event::poll(Duration::from_millis(50))? {
            let event = event::read()?;
            if handle_event(app, event)? {
                break;
            }
            // Reset blink on input
            blink_state = true;
            last_blink = Instant::now();
        }

        if app.should_quit() {
            break;
        }
    }

    Ok(())
}

/// Render the complete UI.
pub fn render(frame: &mut Frame, app: &App, theme: &Theme, blink_state: bool) {
    let config = &app.config().appearance;
    let layout = MainLayout::with_config(frame.area(), config);

    // Render main components
    render_header(frame, app, theme, layout.header);
    render_filter(frame, app, theme, layout.filter, blink_state);
    render_scripts(frame, app, theme, layout.scripts);
    render_description(frame, app, theme, layout.description);

    if config.show_footer {
        render_footer(frame, app, theme, layout.footer);
    }

    // Render overlays
    match app.mode() {
        AppMode::Help => render_help_overlay(frame, theme),
        AppMode::Error { message } => render_error_overlay(frame, theme, message),
        AppMode::WorkspaceSelect => render_workspace_selector(frame, app, theme, layout.scripts),
        _ => {}
    }
}

/// Render the header.
fn render_header(frame: &mut Frame, app: &App, theme: &Theme, area: ratatui::layout::Rect) {
    let config = &app.config().appearance;
    // Use breadcrumb if in workspace context
    let title = app.breadcrumb();
    let header = Header::new(&title, app.runner(), theme, config);
    frame.render_widget(header, area);
}

/// Render the filter bar.
fn render_filter(
    frame: &mut Frame,
    app: &App,
    theme: &Theme,
    area: ratatui::layout::Rect,
    blink_state: bool,
) {
    let config = &app.config().appearance;

    match app.mode() {
        AppMode::Filter { query } => {
            let filter = Filter::new(query, true, theme, config).blink(blink_state);
            frame.render_widget(filter, area);
        }
        AppMode::Args { input, .. } => {
            let script_name = app.selected_script().map(|s| s.name()).unwrap_or("script");
            let args_filter = ArgsFilter::new(script_name, input, theme).blink(blink_state);
            frame.render_widget(args_filter, area);
        }
        _ => {
            let query = app.filter_text();
            let filter = Filter::new(query, false, theme, config);
            frame.render_widget(filter, area);
        }
    }
}

/// Render the scripts grid.
fn render_scripts(frame: &mut Frame, app: &App, theme: &Theme, area: ratatui::layout::Rect) {
    let visible = app.visible_scripts();

    if visible.is_empty() {
        let empty = if app.filter_text().is_empty() {
            EmptyScripts::no_scripts(theme)
        } else {
            EmptyScripts::no_matches(theme)
        };
        frame.render_widget(empty, area);
        return;
    }

    let mut grid =
        ScriptsGrid::new(&visible, app.selected_index(), theme).scroll_offset(app.scroll_offset());

    // Add multi-select state if in that mode
    if let AppMode::MultiSelect { selected } = app.mode() {
        grid = grid.multi_selected(selected);
    }

    frame.render_widget(grid, area);
}

/// Render the description panel.
fn render_description(frame: &mut Frame, app: &App, theme: &Theme, area: ratatui::layout::Rect) {
    let config = &app.config().appearance;
    let script = app.selected_script();
    let desc = Description::new(script, theme, config)
        .with_command_preview(app.config().general.show_command_preview);
    frame.render_widget(desc, area);
}

/// Render the footer.
fn render_footer(frame: &mut Frame, app: &App, theme: &Theme, area: ratatui::layout::Rect) {
    let footer = Footer::new(app.mode(), theme);
    frame.render_widget(footer, area);
}

/// Render the workspace selector.
fn render_workspace_selector(
    frame: &mut Frame,
    app: &App,
    theme: &Theme,
    area: ratatui::layout::Rect,
) {
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

    let workspaces = app.workspaces();
    let selected = app.workspace_selected();

    // Build list items: [root, workspace1, workspace2, ...]
    let mut items: Vec<ListItem> = Vec::with_capacity(workspaces.len() + 1);

    // Root item
    let root_label = format!(" 1  {} (root)", app.project_name());
    let root_style = if selected == 0 {
        theme.selected()
    } else {
        theme.script()
    };
    items.push(ListItem::new(Line::from(Span::styled(
        root_label, root_style,
    ))));

    // Workspace items
    for (i, ws) in workspaces.iter().enumerate() {
        let num = i + 2; // 2-indexed since root is 1
        let label = if num <= 9 {
            format!(" {}  {}", num, ws.name())
        } else {
            format!("    {}", ws.name())
        };

        let style = if selected == i + 1 {
            theme.selected()
        } else {
            theme.script()
        };
        items.push(ListItem::new(Line::from(Span::styled(label, style))));
    }

    // Create the list widget
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Workspace ")
                .title_style(theme.bold())
                .border_style(theme.separator()),
        )
        .highlight_style(theme.selected());

    // Render with state
    let mut state = ListState::default();
    state.select(Some(selected));

    frame.render_stateful_widget(list, area, &mut state);
}

/// Render the help overlay.
fn render_help_overlay(frame: &mut Frame, theme: &Theme) {
    let area = frame.area();
    let help_area = centered_rect_fixed(50, 18, area);

    // Clear the area
    frame.render_widget(Clear, help_area);

    // Help content
    let help_lines = vec![
        Line::from(Span::styled("Keyboard Shortcuts", theme.bold())),
        Line::from(""),
        Line::from(vec![
            Span::styled("  j/k     ", theme.key()),
            Span::styled("Move up/down", theme.description()),
        ]),
        Line::from(vec![
            Span::styled("  h/l     ", theme.key()),
            Span::styled("Move left/right (in grid)", theme.description()),
        ]),
        Line::from(vec![
            Span::styled("  g/G     ", theme.key()),
            Span::styled("First/last item", theme.description()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter   ", theme.key()),
            Span::styled("Run selected script", theme.description()),
        ]),
        Line::from(vec![
            Span::styled("  1-9     ", theme.key()),
            Span::styled("Quick run numbered script", theme.description()),
        ]),
        Line::from(vec![
            Span::styled("  /       ", theme.key()),
            Span::styled("Filter scripts", theme.description()),
        ]),
        Line::from(vec![
            Span::styled("  s       ", theme.key()),
            Span::styled("Cycle sort mode", theme.description()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ?       ", theme.key()),
            Span::styled("Toggle this help", theme.description()),
        ]),
        Line::from(vec![
            Span::styled("  q/Esc   ", theme.key()),
            Span::styled("Quit", theme.description()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            theme.filter_placeholder(),
        )),
    ];

    let help = Paragraph::new(help_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .style(theme.description()),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(help, help_area);
}

/// Render an error overlay.
fn render_error_overlay(frame: &mut Frame, theme: &Theme, message: &str) {
    let area = frame.area();
    let error_area = centered_rect_fixed(60, 8, area);

    // Clear the area
    frame.render_widget(Clear, error_area);

    let error_lines = vec![
        Line::from(Span::styled("Error", theme.error())),
        Line::from(""),
        Line::from(Span::styled(message, theme.description())),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to dismiss",
            theme.filter_placeholder(),
        )),
    ];

    let error = Paragraph::new(error_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Error ")
                .border_style(theme.error()),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(error, error_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::history::History;
    use crate::package::{Runner, Script, Scripts};
    use std::path::PathBuf;

    fn create_test_app() -> App {
        let mut scripts = Scripts::new();
        scripts.add(Script::new("dev", "vite"));
        scripts.add(Script::new("build", "vite build"));
        scripts.add(Script::new("test", "vitest"));

        App::new(
            scripts,
            Config::default(),
            History::new(),
            "test-project".to_string(),
            PathBuf::from("/test"),
            Runner::Npm,
        )
    }

    #[test]
    fn test_render_creates_layout() {
        // This is a basic test to ensure the render function doesn't panic
        // Full rendering tests would require a mock terminal
        let _app = create_test_app();
        let _theme = Theme::default();
        // Can't easily test render without a terminal, but ensure it compiles
    }
}
