pub mod app;
pub mod api;
pub mod presets;
pub mod ui;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

use app::ThemeApp;

/// Run the standalone theme editor TUI.
///
/// Expected Behavior:
///   Enters raw mode and alternate screen, initializes a ThemeApp,
///   attempts to load the current hackathon config from the local API
///   (http://localhost:8000/api/core/info). If the API is unreachable,
///   starts in offline mode with default values. Runs an event loop that
///   draws the UI and handles keyboard input for tab navigation, field
///   editing, saving, and quitting. On exit, restores the terminal to
///   cooked mode and leaves the alternate screen.
///
/// Returns:
///   Ok(()) on clean exit, or an error if terminal setup fails.
///
/// Side Effects:
///   - Switches terminal to raw mode and alternate screen on entry.
///   - May send HTTP GET/PUT requests to localhost:8000.
///   - Restores terminal state on exit or panic (best-effort via Drop on
///     backend). Logs API errors to stderr only.
pub fn run_theme_editor() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    let mut app = ThemeApp::new();

    // Try to load existing config
    match rt.block_on(api::fetch_config(&app.api_url)) {
        Ok(config) => {
            app.config = config;
            app.api_connected = true;
        }
        Err(e) => {
            app.status_message = Some(format!("Offline: {e}"));
            app.api_connected = false;
        }
    }

    let result = run_loop(&mut terminal, &mut app, &rt);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut ThemeApp,
    rt: &tokio::runtime::Runtime,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let handled = handle_key(app, key.code, rt);
                    if !handled {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Handle a key press. Returns true to continue the loop, false to exit.
fn handle_key(app: &mut ThemeApp, code: KeyCode, rt: &tokio::runtime::Runtime) -> bool {
    if app.editing {
        match code {
            KeyCode::Esc => {
                app.editing = false;
                app.input_buffer.clear();
            }
            KeyCode::Enter => {
                app.confirm_edit();
                app.editing = false;
                app.input_buffer.clear();
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            _ => {}
        }
        return true;
    }

    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return false,
        KeyCode::Tab => app.next_tab(),
        KeyCode::BackTab => app.prev_tab(),
        KeyCode::Up => app.prev_field(),
        KeyCode::Down => app.next_field(),
        KeyCode::Enter => {
            app.editing = true;
            app.input_buffer = app.current_field_value();
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if app.api_connected {
                let token = app.auth_token.as_deref();
                match rt.block_on(api::save_config(&app.api_url, &app.config, token)) {
                    Ok(()) => {
                        app.set_status("Saved!".to_string());
                    }
                    Err(e) => {
                        app.set_status(format!("Save failed: {e}"));
                    }
                }
            } else {
                app.set_status("Offline — cannot save".to_string());
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.config = app::ThemeConfig::default();
            app.set_status("Reset to defaults".to_string());
        }
        _ => {}
    }
    true
}
