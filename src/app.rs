use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::path::PathBuf;
use std::process::Command;
use tokio::sync::mpsc;

use crate::api::client::LeetCodeClient;
use crate::api::types::{ProblemSummary, QuestionDetail};
use crate::config::Config;
use crate::event::{Event, EventHandler};
use crate::scaffold;
use crate::ui::browser::{self, BrowserAction, BrowserState};
use crate::ui::detail::{self, DetailAction, DetailState};
use crate::ui::setup::{self, SetupAction, SetupState};

pub enum Screen {
    Setup(SetupState),
    Browse(BrowserState),
    Detail(DetailState),
}

pub enum ApiResult {
    Problems(Result<(Vec<ProblemSummary>, i32)>),
    Detail(Result<QuestionDetail>),
}

pub struct App {
    pub screen: Screen,
    pub config: Option<Config>,
    pub should_quit: bool,
    pub error_overlay: Option<String>,
    pub last_opened_dir: Option<PathBuf>,
    api_client: LeetCodeClient,
    api_tx: mpsc::UnboundedSender<ApiResult>,
    api_rx: mpsc::UnboundedReceiver<ApiResult>,
}

impl App {
    pub fn new(config: Option<Config>) -> Result<Self> {
        let (api_tx, api_rx) = mpsc::unbounded_channel();
        let api_client = LeetCodeClient::new()?;

        let screen = if config.is_some() {
            Screen::Browse(BrowserState::new())
        } else {
            Screen::Setup(SetupState::new())
        };

        Ok(Self {
            screen,
            config,
            should_quit: false,
            error_overlay: None,
            last_opened_dir: None,
            api_client,
            api_tx,
            api_rx,
        })
    }

    pub async fn run(
        &mut self,
        terminal: &mut ratatui::DefaultTerminal,
        events: &mut EventHandler,
    ) -> Result<()> {
        // If starting on Browse screen, kick off initial fetch
        if matches!(self.screen, Screen::Browse(_)) {
            self.start_fetch_problems();
        }

        loop {
            terminal.draw(|f| self.render(f))?;

            if self.should_quit {
                break;
            }

            tokio::select! {
                event = events.next() => {
                    match event? {
                        Event::Key(key) => self.handle_key(key, terminal)?,
                        Event::Tick => self.handle_tick(),
                        Event::Resize(_, _) => {}
                    }
                }
                Some(api_result) = self.api_rx.recv() => {
                    self.handle_api_result(api_result);
                }
            }
        }

        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        match &mut self.screen {
            Screen::Setup(state) => setup::render_setup(frame, state),
            Screen::Browse(state) => browser::render_browser(frame, area, state),
            Screen::Detail(state) => detail::render_detail(frame, area, state),
        }

        // Error overlay
        if let Some(ref msg) = self.error_overlay {
            let overlay_width = 50u16.min(area.width.saturating_sub(4));
            let overlay_height = 8u16.min(area.height.saturating_sub(4));
            let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
            let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
            let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

            frame.render_widget(Clear, overlay_area);
            let error_block = Paragraph::new(format!("\n{msg}\n\nPress Esc to dismiss"))
                .block(
                    Block::default()
                        .title(" Error ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red)),
                )
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: true });
            frame.render_widget(error_block, overlay_area);
        }
    }

    fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        terminal: &mut ratatui::DefaultTerminal,
    ) -> Result<()> {
        // Global quit: Ctrl+C always exits
        if key.code == KeyCode::Char('c')
            && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
        {
            self.should_quit = true;
            return Ok(());
        }

        // Dismiss error overlay on Esc or q
        if self.error_overlay.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => self.error_overlay = None,
                _ => {}
            }
            return Ok(());
        }

        match &mut self.screen {
            Screen::Setup(state) => match state.handle_key(key) {
                SetupAction::Submit => {
                    let config = Config {
                        workspace_dir: state.fields[0].clone(),
                        language: state.fields[1].clone(),
                        editor: state.fields[2].clone(),
                    };
                    if let Err(e) = config.save() {
                        self.error_overlay = Some(format!("Failed to save config: {e}"));
                    } else {
                        self.config = Some(config);
                        self.screen = Screen::Browse(BrowserState::new());
                        self.start_fetch_problems();
                    }
                }
                SetupAction::Quit => self.should_quit = true,
                SetupAction::None => {}
            },
            Screen::Browse(state) => match state.handle_key(key) {
                BrowserAction::Quit => self.should_quit = true,
                BrowserAction::OpenDetail(slug) => {
                    self.start_fetch_detail(&slug);
                }
                BrowserAction::Scaffold(slug) => {
                    self.start_fetch_detail_for_scaffold(&slug, terminal)?;
                }
                BrowserAction::FilterChanged => {
                    self.start_fetch_problems();
                }
                BrowserAction::None => {}
            },
            Screen::Detail(state) => {
                let action = state.handle_key(key);
                match action {
                    DetailAction::Back => {
                        self.screen = Screen::Browse(BrowserState::new());
                        self.start_fetch_problems();
                    }
                    DetailAction::Quit => self.should_quit = true,
                    DetailAction::Scaffold(_) => {
                        let detail = if let Screen::Detail(s) = &self.screen {
                            s.detail.clone()
                        } else {
                            unreachable!()
                        };
                        self.do_scaffold_and_edit(&detail, terminal)?;
                    }
                    DetailAction::None => {}
                }
            }
        }

        Ok(())
    }

    fn handle_tick(&mut self) {
        if let Screen::Browse(ref mut state) = self.screen {
            state.spinner_frame = state.spinner_frame.wrapping_add(1);
        }
    }

    fn handle_api_result(&mut self, result: ApiResult) {
        match result {
            ApiResult::Problems(Ok((problems, total))) => {
                if let Screen::Browse(ref mut state) = self.screen {
                    state.problems = problems;
                    state.total_problems = total;
                    state.loading = false;
                    state.error_message = None;
                    state.rebuild_filter();
                }
            }
            ApiResult::Problems(Err(e)) => {
                if let Screen::Browse(ref mut state) = self.screen {
                    state.loading = false;
                    state.error_message = Some(format!("{e}"));
                }
            }
            ApiResult::Detail(Ok(detail)) => {
                self.screen = Screen::Detail(DetailState::new(detail));
            }
            ApiResult::Detail(Err(e)) => {
                self.error_overlay = Some(format!("Failed to load problem: {e}"));
            }
        }
    }

    fn start_fetch_problems(&mut self) {
        if let Screen::Browse(ref mut state) = self.screen {
            state.loading = true;
            state.error_message = None;

            let client = self.api_client.clone();
            let tx = self.api_tx.clone();
            let difficulty = state.difficulty_filter.as_api_str().map(String::from);

            tokio::spawn(async move {
                let result = client
                    .fetch_problems(50, 0, difficulty.as_deref())
                    .await;
                let _ = tx.send(ApiResult::Problems(result));
            });
        }
    }

    fn start_fetch_detail(&self, slug: &str) {
        let client = self.api_client.clone();
        let tx = self.api_tx.clone();
        let slug = slug.to_string();

        tokio::spawn(async move {
            let result = client.fetch_problem_detail(&slug).await;
            let _ = tx.send(ApiResult::Detail(result));
        });
    }

    fn start_fetch_detail_for_scaffold(
        &mut self,
        slug: &str,
        _terminal: &mut ratatui::DefaultTerminal,
    ) -> Result<()> {
        // Fetch detail first, then user presses 'o' from detail view to scaffold
        let client = self.api_client.clone();
        let tx = self.api_tx.clone();
        let slug = slug.to_string();

        tokio::spawn(async move {
            let result = client.fetch_problem_detail(&slug).await;
            let _ = tx.send(ApiResult::Detail(result));
        });
        Ok(())
    }

    fn do_scaffold_and_edit(
        &mut self,
        detail: &QuestionDetail,
        terminal: &mut ratatui::DefaultTerminal,
    ) -> Result<()> {
        let config = match &self.config {
            Some(c) => c.clone(),
            None => {
                self.error_overlay = Some("No config loaded".to_string());
                return Ok(());
            }
        };

        let workspace = config.expanded_workspace();
        std::fs::create_dir_all(&workspace).ok();

        match scaffold::scaffold_problem(&workspace, detail, &config.language) {
            Ok(file_path) => {
                let project_dir = file_path.parent().and_then(|p| p.parent()).unwrap_or(&workspace);
                self.last_opened_dir = Some(project_dir.to_path_buf());

                // Suspend TUI for editor
                ratatui::restore();

                let status = Command::new(&config.editor)
                    .arg(&file_path)
                    .current_dir(project_dir)
                    .status();

                // Re-init TUI
                *terminal = ratatui::init();

                match status {
                    Ok(s) if s.success() => {}
                    Ok(s) => {
                        self.error_overlay =
                            Some(format!("Editor exited with status: {}", s));
                    }
                    Err(e) => {
                        self.error_overlay =
                            Some(format!("Failed to launch editor '{}': {}", config.editor, e));
                    }
                }
            }
            Err(e) => {
                self.error_overlay = Some(format!("Scaffold failed: {e}"));
            }
        }

        Ok(())
    }
}
