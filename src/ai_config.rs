//! AI configuration TUI with ratatui.

use crate::api::{self, AssistantConfig, AVAILABLE_MODELS};
use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io;

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Enabled,
    Model,
    ApiKey,
    BaseUrl,
    Test,
    Save,
}

const FIELDS: &[Field] = &[
    Field::Enabled,
    Field::Model,
    Field::ApiKey,
    Field::BaseUrl,
    Field::Test,
    Field::Save,
];

struct App {
    config: AssistantConfig,
    selected: usize,
    editing: bool,
    edit_buffer: String,
    model_idx: usize,
    status_msg: String,
    status_ok: bool,
}

impl App {
    fn new(config: AssistantConfig) -> Self {
        let current_model = config.model.as_deref().unwrap_or(api::DEFAULT_MODEL);
        let model_idx = AVAILABLE_MODELS
            .iter()
            .position(|m| *m == current_model)
            .unwrap_or(0);

        Self {
            config,
            selected: 0,
            editing: false,
            edit_buffer: String::new(),
            model_idx,
            status_msg: String::new(),
            status_ok: true,
        }
    }

    fn current_field(&self) -> Field {
        FIELDS[self.selected]
    }

    fn start_edit(&mut self) {
        self.editing = true;
        self.edit_buffer = match self.current_field() {
            Field::ApiKey => self.config.api_key.clone().unwrap_or_default(),
            Field::BaseUrl => self.config.base_url.clone().unwrap_or_else(|| api::DEFAULT_BASE_URL.to_string()),
            _ => String::new(),
        };
    }

    fn commit_edit(&mut self) {
        self.editing = false;
        match self.current_field() {
            Field::ApiKey => self.config.api_key = Some(self.edit_buffer.clone()),
            Field::BaseUrl => self.config.base_url = Some(self.edit_buffer.clone()),
            _ => {}
        }
        self.edit_buffer.clear();
    }

    fn cancel_edit(&mut self) {
        self.editing = false;
        self.edit_buffer.clear();
    }

    fn toggle_enabled(&mut self) {
        let current = self.config.enabled.unwrap_or(true);
        self.config.enabled = Some(!current);
    }

    fn cycle_model(&mut self, forward: bool) {
        if forward {
            self.model_idx = (self.model_idx + 1) % AVAILABLE_MODELS.len();
        } else if self.model_idx > 0 {
            self.model_idx -= 1;
        } else {
            self.model_idx = AVAILABLE_MODELS.len() - 1;
        }
        self.config.model = Some(AVAILABLE_MODELS[self.model_idx].to_string());
    }
}

pub fn run() -> anyhow::Result<()> {
    let config = api::load_config()?;
    let mut app = App::new(config);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.editing {
                match key.code {
                    KeyCode::Enter => app.commit_edit(),
                    KeyCode::Esc => app.cancel_edit(),
                    KeyCode::Backspace => { app.edit_buffer.pop(); }
                    KeyCode::Char(c) => app.edit_buffer.push(c),
                    _ => {}
                }
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.selected > 0 { app.selected -= 1; }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.selected + 1 < FIELDS.len() { app.selected += 1; }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    match app.current_field() {
                        Field::Enabled => app.toggle_enabled(),
                        Field::Model => app.cycle_model(true),
                        Field::ApiKey | Field::BaseUrl => app.start_edit(),
                        Field::Test => {
                            app.status_msg = "Testing...".into();
                            app.status_ok = true;
                            // Quick sync redraw
                            terminal.draw(|f| ui(f, app))?;
                            match test_connection(&app.config) {
                                Ok(msg) => {
                                    app.status_msg = format!("OK: {msg}");
                                    app.status_ok = true;
                                }
                                Err(e) => {
                                    app.status_msg = format!("FAIL: {e:#}");
                                    app.status_ok = false;
                                }
                            }
                        }
                        Field::Save => {
                            match api::save_config(&app.config) {
                                Ok(()) => {
                                    app.status_msg = "Saved!".into();
                                    app.status_ok = true;
                                }
                                Err(e) => {
                                    app.status_msg = format!("Save failed: {e:#}");
                                    app.status_ok = false;
                                }
                            }
                        }
                    }
                }
                KeyCode::Left | KeyCode::Char('h') if app.current_field() == Field::Model => {
                    app.cycle_model(false);
                }
                KeyCode::Right | KeyCode::Char('l') if app.current_field() == Field::Model => {
                    app.cycle_model(true);
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // title
            Constraint::Length(2),  // spacer
            Constraint::Min(10),   // fields
            Constraint::Length(3),  // status
            Constraint::Length(2),  // help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Kaku AI Assistant Configuration")
        .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Fields
    let field_area = chunks[2];
    let field_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(FIELDS.iter().map(|_| Constraint::Length(2)).collect::<Vec<_>>())
        .split(field_area);

    for (i, field) in FIELDS.iter().enumerate() {
        let is_selected = i == app.selected;
        let arrow = if is_selected { "> " } else { "  " };
        let highlight = if is_selected {
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let line = match field {
            Field::Enabled => {
                let v = if app.config.enabled.unwrap_or(true) { "ON" } else { "OFF" };
                let color = if app.config.enabled.unwrap_or(true) { Color::Green } else { Color::Red };
                Line::from(vec![
                    Span::styled(arrow, highlight),
                    Span::raw("Enabled:   "),
                    Span::styled(v, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                ])
            }
            Field::Model => {
                let model = app.config.model.as_deref().unwrap_or(api::DEFAULT_MODEL);
                Line::from(vec![
                    Span::styled(arrow, highlight),
                    Span::raw("Model:     "),
                    Span::styled(format!("< {model} >"), Style::default().fg(Color::Cyan)),
                ])
            }
            Field::ApiKey => {
                let display = if app.editing && app.current_field() == Field::ApiKey {
                    format!("{}_", app.edit_buffer)
                } else {
                    match &app.config.api_key {
                        Some(k) if !k.is_empty() => {
                            let masked = if k.len() > 8 {
                                format!("{}...{}", &k[..4], &k[k.len()-4..])
                            } else {
                                "****".into()
                            };
                            masked
                        }
                        _ => "(not set)".into(),
                    }
                };
                let color = if app.config.api_key.as_deref().filter(|k| !k.is_empty()).is_some() {
                    Color::Green
                } else {
                    Color::Yellow
                };
                Line::from(vec![
                    Span::styled(arrow, highlight),
                    Span::raw("API Key:   "),
                    Span::styled(display, Style::default().fg(color)),
                ])
            }
            Field::BaseUrl => {
                let display = if app.editing && app.current_field() == Field::BaseUrl {
                    format!("{}_", app.edit_buffer)
                } else {
                    app.config.base_url.clone().unwrap_or_else(|| api::DEFAULT_BASE_URL.into())
                };
                Line::from(vec![
                    Span::styled(arrow, highlight),
                    Span::raw("Base URL:  "),
                    Span::styled(display, Style::default().fg(Color::Cyan)),
                ])
            }
            Field::Test => {
                Line::from(vec![
                    Span::styled(arrow, highlight),
                    Span::styled("[ Test Connection ]", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                ])
            }
            Field::Save => {
                Line::from(vec![
                    Span::styled(arrow, highlight),
                    Span::styled("[ Save ]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ])
            }
        };

        f.render_widget(Paragraph::new(line), field_chunks[i]);
    }

    // Status bar
    if !app.status_msg.is_empty() {
        let color = if app.status_ok { Color::Green } else { Color::Red };
        let status = Paragraph::new(app.status_msg.as_str())
            .style(Style::default().fg(color))
            .alignment(Alignment::Center);
        f.render_widget(status, chunks[3]);
    }

    // Help
    let help_text = if app.editing {
        "Enter: confirm | Esc: cancel | type to edit"
    } else {
        "arrows/jk: navigate | Enter/Space: edit | </>: cycle model | q: quit"
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[4]);
}

fn test_connection(config: &AssistantConfig) -> anyhow::Result<String> {
    let rt = tokio::runtime::Runtime::new()?;
    let response = rt.block_on(api::chat(
        config,
        "You are a test assistant. Respond with only: OK",
        "ping",
    )).context("connection test")?;
    Ok(response.trim().to_string())
}
