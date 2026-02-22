use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::api::types::ProblemSummary;

use super::status_bar::render_status_bar;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DifficultyFilter {
    All,
    Easy,
    Medium,
    Hard,
}

impl DifficultyFilter {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Easy,
            Self::Easy => Self::Medium,
            Self::Medium => Self::Hard,
            Self::Hard => Self::All,
        }
    }

    pub fn as_api_str(&self) -> Option<&str> {
        match self {
            Self::All => None,
            Self::Easy => Some("EASY"),
            Self::Medium => Some("MEDIUM"),
            Self::Hard => Some("HARD"),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::All => "All",
            Self::Easy => "Easy",
            Self::Medium => "Medium",
            Self::Hard => "Hard",
        }
    }
}

pub struct BrowserState {
    pub table_state: TableState,
    pub problems: Vec<ProblemSummary>,
    pub filtered_indices: Vec<usize>,
    pub search_query: String,
    pub search_mode: bool,
    pub difficulty_filter: DifficultyFilter,
    pub loading: bool,
    pub total_problems: i32,
    pub error_message: Option<String>,
    pub spinner_frame: usize,
}

impl BrowserState {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default(),
            problems: Vec::new(),
            filtered_indices: Vec::new(),
            search_query: String::new(),
            search_mode: false,
            difficulty_filter: DifficultyFilter::All,
            loading: true,
            total_problems: 0,
            error_message: None,
            spinner_frame: 0,
        }
    }

    pub fn rebuild_filter(&mut self) {
        let query = self.search_query.to_lowercase();
        self.filtered_indices = self
            .problems
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if query.is_empty() {
                    return true;
                }
                p.title.to_lowercase().contains(&query)
                    || p.frontend_question_id == query
            })
            .map(|(i, _)| i)
            .collect();

        // Keep selection in bounds
        if self.filtered_indices.is_empty() {
            self.table_state.select(None);
        } else if let Some(selected) = self.table_state.selected() {
            if selected >= self.filtered_indices.len() {
                self.table_state.select(Some(self.filtered_indices.len() - 1));
            }
        } else {
            self.table_state.select(Some(0));
        }
    }

    pub fn selected_problem(&self) -> Option<&ProblemSummary> {
        let selected = self.table_state.selected()?;
        let idx = *self.filtered_indices.get(selected)?;
        self.problems.get(idx)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> BrowserAction {
        if self.search_mode {
            return self.handle_search_key(key);
        }

        match key.code {
            KeyCode::Char('q') => BrowserAction::Quit,
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_selection(1);
                BrowserAction::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_selection(-1);
                BrowserAction::None
            }
            KeyCode::Char('g') => {
                if !self.filtered_indices.is_empty() {
                    self.table_state.select(Some(0));
                }
                BrowserAction::None
            }
            KeyCode::Char('G') => {
                if !self.filtered_indices.is_empty() {
                    self.table_state
                        .select(Some(self.filtered_indices.len() - 1));
                }
                BrowserAction::None
            }
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_query.clear();
                BrowserAction::None
            }
            KeyCode::Char('d') => {
                self.difficulty_filter = self.difficulty_filter.next();
                BrowserAction::FilterChanged
            }
            KeyCode::Enter => {
                if let Some(problem) = self.selected_problem() {
                    BrowserAction::OpenDetail(problem.title_slug.clone())
                } else {
                    BrowserAction::None
                }
            }
            KeyCode::Char('o') => {
                if let Some(problem) = self.selected_problem() {
                    BrowserAction::Scaffold(problem.title_slug.clone())
                } else {
                    BrowserAction::None
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                BrowserAction::Quit
            }
            _ => BrowserAction::None,
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> BrowserAction {
        match key.code {
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_query.clear();
                self.rebuild_filter();
                BrowserAction::None
            }
            KeyCode::Enter => {
                self.search_mode = false;
                BrowserAction::None
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.rebuild_filter();
                BrowserAction::None
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.rebuild_filter();
                BrowserAction::None
            }
            _ => BrowserAction::None,
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let current = self.table_state.selected().unwrap_or(0) as i32;
        let max = self.filtered_indices.len() as i32 - 1;
        let next = (current + delta).clamp(0, max) as usize;
        self.table_state.select(Some(next));
    }
}

pub enum BrowserAction {
    None,
    Quit,
    OpenDetail(String),
    Scaffold(String),
    FilterChanged,
}

pub fn render_browser(frame: &mut Frame, area: Rect, state: &mut BrowserState) {
    let layout = Layout::vertical([
        Constraint::Length(1), // title bar
        Constraint::Min(3),   // table
        Constraint::Length(1), // status bar
    ])
    .split(area);

    // Title bar
    render_title_bar(frame, layout[0], state);

    // Problem table
    if state.loading {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let s = spinner[state.spinner_frame % spinner.len()];
        let loading = Paragraph::new(format!(" {s} Loading problems..."))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(loading, layout[1]);
    } else if let Some(ref err) = state.error_message {
        let error = Paragraph::new(format!(" Error: {err}"))
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error, layout[1]);
    } else {
        render_table(frame, layout[1], state);
    }

    // Status bar
    let hints = if state.search_mode {
        vec![
            ("Enter", "Apply"),
            ("Esc", "Cancel"),
            ("type", "Filter"),
        ]
    } else {
        vec![
            ("j/k", "Navigate"),
            ("Enter", "View"),
            ("o", "Open"),
            ("/", "Search"),
            ("d", "Difficulty"),
            ("q", "Quit"),
        ]
    };
    render_status_bar(frame, layout[2], &hints);
}

fn render_title_bar(frame: &mut Frame, area: Rect, state: &BrowserState) {
    let mut spans = vec![
        Span::styled(
            " LeetCode ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];

    if state.difficulty_filter != DifficultyFilter::All {
        let (label, color) = match state.difficulty_filter {
            DifficultyFilter::Easy => ("Easy", Color::Green),
            DifficultyFilter::Medium => ("Medium", Color::Yellow),
            DifficultyFilter::Hard => ("Hard", Color::Red),
            DifficultyFilter::All => unreachable!(),
        };
        spans.push(Span::styled(
            format!("[{label}] "),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    }

    spans.push(Span::styled(
        format!(
            "{} / {} problems",
            state.filtered_indices.len(),
            state.total_problems
        ),
        Style::default().fg(Color::DarkGray),
    ));

    if state.search_mode || !state.search_query.is_empty() {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("/{}", state.search_query),
            Style::default().fg(Color::Cyan),
        ));
        if state.search_mode {
            spans.push(Span::styled("▎", Style::default().fg(Color::Cyan)));
        }
    }

    let title = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));
    frame.render_widget(title, area);
}

fn render_table(frame: &mut Frame, area: Rect, state: &mut BrowserState) {
    let header = Row::new([
        Cell::from(" # "),
        Cell::from("Title"),
        Cell::from("Difficulty"),
        Cell::from("AC Rate"),
    ])
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(0);

    let rows: Vec<Row> = state
        .filtered_indices
        .iter()
        .map(|&idx| {
            let p = &state.problems[idx];
            let diff_color = match p.difficulty.as_str() {
                "Easy" => Color::Green,
                "Medium" => Color::Yellow,
                "Hard" => Color::Red,
                _ => Color::White,
            };
            let paid = if p.is_paid_only { " 🔒" } else { "" };
            Row::new([
                Cell::from(format!(" {}", p.frontend_question_id)),
                Cell::from(format!("{}{}", p.title, paid)),
                Cell::from(Span::styled(
                    p.difficulty.clone(),
                    Style::default().fg(diff_color),
                )),
                Cell::from(format!("{:.1}%", p.ac_rate)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Min(20),
        Constraint::Length(10),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(table, area, &mut state.table_state);
}
