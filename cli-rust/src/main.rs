#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::too_many_arguments,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::match_same_arms,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::as_conversions,
    clippy::unused_async,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::format_push_string,
    clippy::uninlined_format_args,
    clippy::unreadable_literal,
    clippy::items_after_statements,
    clippy::single_match_else,
    clippy::if_not_else,
    clippy::wildcard_imports,
    clippy::semicolon_if_nothing_returned,
    clippy::return_self_not_must_use,
    clippy::needless_pass_by_value,
    clippy::implicit_hasher,
    clippy::from_over_into,
    clippy::default_trait_access,
    clippy::redundant_closure_for_method_calls
)]

mod app;
pub mod config;
pub mod deploy;
mod deploy_state;
mod ui;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

use app::{App, DeployTarget, Screen};
use deploy::StepUpdate;
use deploy_state::DeployState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let project_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let mut deploy_rx: Option<tokio::sync::mpsc::UnboundedReceiver<StepUpdate>> = None;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if let Some(ref mut rx) = deploy_rx {
            while let Ok(update) = rx.try_recv() {
                if let Some(step) = app
                    .deploy_steps
                    .iter_mut()
                    .find(|s| s.name == update.step_name)
                {
                    step.status = update.status;
                } else {
                    app.deploy_steps.push(app::DeployStep {
                        name: update.step_name.clone(),
                        status: update.status,
                    });
                }
                if let Some(err) = update.error {
                    app.deploy_error = Some(err);
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            if app.screen == Screen::Done || app.screen == Screen::Migrate {
                                break;
                            }
                        }
                        KeyCode::Esc => {
                            if app.screen == Screen::Migrate {
                                app.screen = Screen::Done;
                                app.selected = 0;
                            } else if app.screen != Screen::Welcome {
                                app.prev_screen();
                            }
                        }
                        KeyCode::Enter => match app.screen {
                            Screen::Welcome => app.next_screen(),
                            Screen::DeployTarget => {
                                app.deploy_target = match app.selected {
                                    0 => DeployTarget::DockerCompose,
                                    1 => DeployTarget::AwsLambda,
                                    2 => DeployTarget::GcpCloudRun,
                                    3 => DeployTarget::AzureContainerApps,
                                    _ => DeployTarget::Kubernetes,
                                };
                                app.next_screen();
                            }
                            Screen::Services => app.next_screen(),
                            Screen::Database => app.next_screen(),
                            Screen::Storage => {
                                app.storage_provider = match app.selected {
                                    0 => "local".into(),
                                    1 => "minio".into(),
                                    2 => "s3".into(),
                                    3 => "gcs".into(),
                                    _ => "azure".into(),
                                };
                                app.next_screen();
                            }
                            Screen::Email => {
                                app.mail_provider = match app.selected {
                                    0 => "ses".into(),
                                    1 => "sendgrid".into(),
                                    2 => "smtp".into(),
                                    _ => "disabled".into(),
                                };
                                app.next_screen();
                            }
                            Screen::Secrets => app.next_screen(),
                            Screen::CostEstimate => app.next_screen(),
                            Screen::Review => {
                                app.screen = Screen::Progress;
                                config::generate_all(&app, &project_dir)
                                    .unwrap_or_else(|e| eprintln!("Config generation error: {e}"));
                                app.deploy_steps = deploy::build_steps(&app);
                                app.deploy_state = Some(DeployState::from_app(&app));
                                let state_path = DeployState::path_for_project(&project_dir);
                                if let Some(ref state) = app.deploy_state {
                                    let _ = state.save(&state_path);
                                }
                                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                                deploy_rx = Some(rx);
                                let app_clone = app.clone();
                                let pd = project_dir.clone();
                                rt.spawn(async move {
                                    let _ = deploy::deploy(&app_clone, &pd, &tx).await;
                                });
                            }
                            Screen::Migrate => {
                                if let Some(ref state) = app.deploy_state {
                                    if state.migration_count() > 0 {
                                        let migrated = state.migrated_services();
                                        app.deploy_steps = deploy::build_migration_steps(&migrated);
                                        app.screen = Screen::Progress;
                                        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                                        deploy_rx = Some(rx);
                                        let app_clone = app.clone();
                                        let pd = project_dir.clone();
                                        rt.spawn(async move {
                                            let _ = deploy::migrate(&app_clone, &pd, &tx).await;
                                        });
                                    }
                                }
                            }
                            _ => {}
                        },
                        KeyCode::Up => {
                            if app.screen == Screen::Migrate {
                                if app.migrate_selected > 0 {
                                    app.migrate_selected -= 1;
                                }
                            } else if app.selected > 0 {
                                app.selected -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if app.screen == Screen::Migrate {
                                let count = app.migrate_service_keys().len();
                                if app.migrate_selected + 1 < count {
                                    app.migrate_selected += 1;
                                }
                            } else {
                                app.selected += 1;
                            }
                        }
                        KeyCode::Char(' ') => {
                            if app.screen == Screen::Services {
                                app.toggle_service(app.selected);
                            } else if app.screen == Screen::Migrate {
                                if let Some(ref mut state) = app.deploy_state {
                                    let keys: Vec<String> = state
                                        .service_keys_ordered()
                                        .iter()
                                        .map(|k| (*k).to_string())
                                        .collect();
                                    if let Some(key) = keys.get(app.migrate_selected) {
                                        state.toggle_service_target(key);
                                    }
                                }
                            }
                        }
                        KeyCode::Tab => {
                            app.editing = (app.editing + 1) % app.field_count();
                        }
                        KeyCode::Char('r') => {
                            if app.screen == Screen::Secrets && app.selected == 0 {
                                app.jwt_secret = App::generate_jwt_secret();
                            }
                        }
                        KeyCode::Char('m') => {
                            if app.screen == Screen::Done {
                                app.enter_migrate_screen(&project_dir);
                            }
                        }
                        KeyCode::Char('s') => {
                            if app.screen == Screen::Review {
                                config::generate_all(&app, &project_dir)
                                    .unwrap_or_else(|e| eprintln!("Config generation error: {e}"));
                                app.screen = Screen::Done;
                            }
                        }
                        KeyCode::Char(c) => {
                            if app.editing_text {
                                app.input_buffer.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if app.editing_text {
                                app.input_buffer.pop();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.screen == Screen::Done && deploy_rx.is_none() {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
