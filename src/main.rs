//! ClickUp TUI - Personal task organizer
//!
//! A terminal UI for organizing ClickUp tasks with a personal workflow overlay.

mod api;
mod app;
mod config;
mod models;
mod theme;
mod ui;

use anyhow::Result;
use app::{App, FocusedPane, InputMode};
use config::Config;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use models::TaskGroup;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            eprintln!();
            eprintln!("Please ensure your config file exists and contains:");
            eprintln!("  api_token = \"your_clickup_api_token\"");
            eprintln!("  user_id = \"your_user_id\"");
            eprintln!();
            if let Ok(path) = Config::config_path() {
                eprintln!("Config file location: {}", path.display());
            }
            std::process::exit(1);
        }
    };

    // Initialize app
    let mut app = App::new();
    app.set_user_id(&config.user_id);

    // Load local state
    if let Err(e) = app.load_local_state() {
        eprintln!("Warning: Could not load local state: {}", e);
    }

    // Try to load cached tasks first
    let _ = app.load_cached_tasks();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initial refresh if auto_refresh enabled or no cached tasks
    if config.auto_refresh || app.tasks.is_empty() {
        app.is_loading = true;
        terminal.draw(|f| ui::render(f, &app))?;

        match fetch_tasks(&config).await {
            Ok(tasks) => {
                app.set_tasks(tasks);
                app.is_loading = false;
                app.status_message = Some(format!("Loaded {} tasks", app.tasks.len()));
                let _ = app.save_tasks_cache();
                let _ = app.save_local_state();
            }
            Err(e) => {
                app.is_loading = false;
                app.status_message = Some(format!("Failed to load: {}", e));
            }
        }
    }

    // Run event loop
    let res = run_app(&mut terminal, &mut app, &config).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

/// Main event loop
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    config: &Config,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Poll for events with timeout to allow status message clearing
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (not release)
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Clear status message on any key press
                app.clear_status();

                match app.input_mode {
                    InputMode::Normal => {
                        match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            match app.focused_pane {
                                FocusedPane::TaskList => {
                                    app.select_next();
                                    app.reset_preview_scroll();
                                }
                                FocusedPane::Preview => app.scroll_preview_down(),
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            match app.focused_pane {
                                FocusedPane::TaskList => {
                                    app.select_prev();
                                    app.reset_preview_scroll();
                                }
                                FocusedPane::Preview => app.scroll_preview_up(),
                            }
                        }
                        KeyCode::Char('1') => {
                            app.switch_group(TaskGroup::MyAction);
                        }
                        KeyCode::Char('2') => {
                            app.switch_group(TaskGroup::Waiting);
                        }
                        KeyCode::Char('3') => {
                            app.switch_group(TaskGroup::Backlog);
                        }
                        KeyCode::Char('4') => {
                            app.switch_group(TaskGroup::Done);
                        }
                        KeyCode::Char('5') => {
                            app.switch_group(TaskGroup::Snoozed);
                        }
                        KeyCode::Char('6') => {
                            app.switch_group(TaskGroup::Person);
                        }
                        KeyCode::Tab => {
                            app.focus_next_pane();
                        }
                        KeyCode::BackTab => {
                            app.focus_prev_pane();
                        }
                        KeyCode::Char('l') => {
                            app.next_tab();
                        }
                        KeyCode::Char('h') => {
                            app.prev_tab();
                        }
                        KeyCode::Char('p') => {
                            app.toggle_pin();
                        }
                        KeyCode::Char('s') => {
                            app.start_snooze();
                        }
                        KeyCode::Char('S') => {
                            app.unsnooze();
                        }
                        KeyCode::Char('o') | KeyCode::Enter => {
                            app.open_in_browser();
                        }
                        KeyCode::Char('y') => {
                            app.copy_to_clipboard();
                        }
                        KeyCode::Char('/') => {
                            app.start_search();
                        }
                        KeyCode::Char('r') => {
                            // Refresh tasks
                            app.is_loading = true;
                            app.status_message = Some("Refreshing...".to_string());
                            terminal.draw(|f| ui::render(f, app))?;

                            match fetch_tasks(config).await {
                                Ok(tasks) => {
                                    app.set_tasks(tasks);
                                    app.is_loading = false;
                                    app.status_message =
                                        Some(format!("Loaded {} tasks", app.tasks.len()));
                                    let _ = app.save_tasks_cache();
                                    let _ = app.save_local_state();
                                }
                                Err(e) => {
                                    app.is_loading = false;
                                    app.status_message = Some(format!("Failed: {}", e));
                                }
                            }
                        }
                        KeyCode::Char('?') => {
                            app.show_help = true;
                            app.input_mode = InputMode::Help;
                        }
                        _ => {}
                    }
                    }
                    InputMode::Search => match key.code {
                        KeyCode::Esc => {
                            app.cancel_input();
                        }
                        KeyCode::Enter => {
                            // Open selected search result in browser
                            if let Some(task) = app.selected_search_result() {
                                if let Err(e) = open::that(&task.task.url) {
                                    app.status_message = Some(format!("Failed to open: {}", e));
                                } else {
                                    app.status_message = Some("Opened in browser".to_string());
                                }
                            }
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() || app.search_query.is_empty() => {
                            // j navigates when query is empty, otherwise types
                            if app.search_query.is_empty() || key.code == KeyCode::Down {
                                app.search_select_next();
                            } else {
                                app.handle_char('j');
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() || app.search_query.is_empty() => {
                            if app.search_query.is_empty() || key.code == KeyCode::Up {
                                app.search_select_prev();
                            } else {
                                app.handle_char('k');
                            }
                        }
                        KeyCode::Backspace => {
                            app.handle_backspace();
                        }
                        KeyCode::Char(c) => {
                            app.handle_char(c);
                        }
                        _ => {}
                    },
                    InputMode::Snooze => match key.code {
                        KeyCode::Esc => {
                            app.cancel_input();
                        }
                        KeyCode::Enter => {
                            app.confirm_snooze();
                        }
                        KeyCode::Backspace => {
                            app.handle_backspace();
                        }
                        KeyCode::Char(c) => {
                            app.handle_char(c);
                        }
                        _ => {}
                    },
                    InputMode::Help => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                            app.show_help = false;
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Fetch tasks from ClickUp API
async fn fetch_tasks(config: &Config) -> Result<Vec<models::Task>> {
    let client = api::ClickUpClient::new(config.api_token.clone());
    let team_id = client.get_team_id().await?;
    client.fetch_tasks(&team_id, &config.user_id).await
}
