use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::api::types::{ProblemSummary, UserStats};

use super::status_bar::render_status_bar;

pub struct FilterState {
    pub easy: bool,
    pub medium: bool,
    pub hard: bool,
    pub hide_solved: bool,
    pub active_item: usize,
    pub open: bool,
}

impl FilterState {
    pub fn new() -> Self {
        Self {
            easy: true,
            medium: true,
            hard: true,
            hide_solved: false,
            active_item: 0,
            open: false,
        }
    }

    fn item_count(&self) -> usize {
        4
    }

    pub fn summary(&self) -> Option<String> {
        let all = self.easy && self.medium && self.hard && !self.hide_solved;
        if all {
            return None;
        }
        let mut parts = Vec::new();
        if self.easy { parts.push("E"); }
        if self.medium { parts.push("M"); }
        if self.hard { parts.push("H"); }
        let mut s = parts.join("+");
        if self.hide_solved {
            s.push_str(" -Solved");
        }
        Some(format!("[{s}]"))
    }
}

pub enum HomeFocus {
    Search,
    Table,
}

pub struct HomeState {
    pub table_state: TableState,
    pub problems: Vec<ProblemSummary>,
    pub filtered_indices: Vec<usize>,
    pub search_query: String,
    pub focus: HomeFocus,
    pub filter: FilterState,
    pub search_loading: bool,
    pub search_total: i32,
    pub error_message: Option<String>,
    pub spinner_frame: usize,
    pub user_stats: Option<UserStats>,
}

impl HomeState {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default(),
            problems: Vec::new(),
            filtered_indices: Vec::new(),
            search_query: String::new(),
            focus: HomeFocus::Search,
            filter: FilterState::new(),
            search_loading: false,
            search_total: 0,
            error_message: None,
            spinner_frame: 0,
            user_stats: None,
        }
    }

    pub fn rebuild_filter(&mut self) {
        self.filtered_indices = self
            .problems
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                let diff_ok = match p.difficulty.as_str() {
                    "Easy" => self.filter.easy,
                    "Medium" => self.filter.medium,
                    "Hard" => self.filter.hard,
                    _ => true,
                };
                if !diff_ok {
                    return false;
                }
                if self.filter.hide_solved && p.status.as_deref() == Some("ac") {
                    return false;
                }
                true
            })
            .map(|(i, _)| i)
            .collect();

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

    pub fn handle_key(&mut self, key: KeyEvent) -> HomeAction {
        if self.filter.open {
            return self.handle_filter_key(key);
        }
        match self.focus {
            HomeFocus::Search => self.handle_search_key(key),
            HomeFocus::Table => self.handle_table_key(key),
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                HomeAction::Quit
            }
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.problems.clear();
                    self.filtered_indices.clear();
                    self.search_total = 0;
                    self.error_message = None;
                }
                HomeAction::None
            }
            KeyCode::Enter => {
                if !self.search_query.is_empty() {
                    if !self.filtered_indices.is_empty() {
                        self.focus = HomeFocus::Table;
                    }
                    HomeAction::SearchFetch(self.search_query.clone())
                } else {
                    HomeAction::None
                }
            }
            KeyCode::Down | KeyCode::Tab => {
                if !self.filtered_indices.is_empty() {
                    self.focus = HomeFocus::Table;
                    if self.table_state.selected().is_none() {
                        self.table_state.select(Some(0));
                    }
                }
                HomeAction::None
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                HomeAction::SearchFetch(self.search_query.clone())
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                if self.search_query.is_empty() {
                    self.problems.clear();
                    self.filtered_indices.clear();
                    self.search_total = 0;
                    self.error_message = None;
                    HomeAction::None
                } else {
                    HomeAction::SearchFetch(self.search_query.clone())
                }
            }
            _ => HomeAction::None,
        }
    }

    fn handle_table_key(&mut self, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                HomeAction::Quit
            }
            KeyCode::Char('q') => HomeAction::Quit,
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_selection(1);
                HomeAction::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_selection(-1);
                HomeAction::None
            }
            KeyCode::Char('g') => {
                if !self.filtered_indices.is_empty() {
                    self.table_state.select(Some(0));
                }
                HomeAction::None
            }
            KeyCode::Char('G') => {
                if !self.filtered_indices.is_empty() {
                    self.table_state.select(Some(self.filtered_indices.len() - 1));
                }
                HomeAction::None
            }
            KeyCode::Char('/') | KeyCode::Esc => {
                self.focus = HomeFocus::Search;
                HomeAction::None
            }
            KeyCode::Char('f') => {
                self.filter.open = true;
                HomeAction::None
            }
            KeyCode::Enter => {
                if let Some(problem) = self.selected_problem() {
                    HomeAction::OpenDetail(problem.title_slug.clone())
                } else {
                    HomeAction::None
                }
            }
            KeyCode::Char('o') => {
                if let Some(problem) = self.selected_problem() {
                    HomeAction::Scaffold(problem.title_slug.clone())
                } else {
                    HomeAction::None
                }
            }
            KeyCode::Char('a') => {
                if let Some(problem) = self.selected_problem() {
                    HomeAction::AddToList(problem.frontend_question_id.clone())
                } else {
                    HomeAction::None
                }
            }
            KeyCode::Char('L') => HomeAction::Lists,
            KeyCode::Char('S') => HomeAction::Settings,
            _ => HomeAction::None,
        }
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.filter.active_item = (self.filter.active_item + 1) % self.filter.item_count();
                HomeAction::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.filter.active_item = (self.filter.active_item + self.filter.item_count() - 1)
                    % self.filter.item_count();
                HomeAction::None
            }
            KeyCode::Char(' ') => {
                match self.filter.active_item {
                    0 => self.filter.easy = !self.filter.easy,
                    1 => self.filter.medium = !self.filter.medium,
                    2 => self.filter.hard = !self.filter.hard,
                    3 => self.filter.hide_solved = !self.filter.hide_solved,
                    _ => {}
                }
                self.rebuild_filter();
                HomeAction::None
            }
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char('f') => {
                self.filter.open = false;
                HomeAction::None
            }
            _ => HomeAction::None,
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

pub enum HomeAction {
    None,
    Quit,
    OpenDetail(String),
    Scaffold(String),
    SearchFetch(String),
    AddToList(String),
    Settings,
    Lists,
}

pub fn render_home(frame: &mut Frame, area: Rect, state: &mut HomeState) {
    let has_stats = state.user_stats.is_some();
    let stats_height: u16 = if has_stats { 2 } else { 0 };

    let layout = Layout::vertical([
        Constraint::Length(1),            // title bar
        Constraint::Length(stats_height), // stats header
        Constraint::Length(1),            // search bar
        Constraint::Min(3),              // table / empty state
        Constraint::Length(1),           // status bar
    ])
    .split(area);

    render_title_bar(frame, layout[0], state);

    if let Some(ref stats) = state.user_stats {
        render_stats_header(frame, layout[1], stats);
    }

    render_search_bar(frame, layout[2], state);

    if state.search_loading && state.problems.is_empty() {
        let spinner = ["\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}", "\u{2826}", "\u{2827}", "\u{2807}", "\u{280f}"];
        let s = spinner[state.spinner_frame % spinner.len()];
        let loading = Paragraph::new(format!("  {s} Searching..."))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(loading, layout[3]);
    } else if let Some(ref err) = state.error_message {
        let error = Paragraph::new(format!("  Error: {err}"))
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error, layout[3]);
    } else if state.problems.is_empty() {
        let msg = if state.search_query.is_empty() {
            "  Type to search problems..."
        } else {
            "  No results found"
        };
        let p = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(p, layout[3]);
    } else {
        render_table(frame, layout[3], state);
    }

    let hints = match state.focus {
        HomeFocus::Search => vec![
            ("Enter", "Search"),
            ("Tab/\u{2193}", "Table"),
            ("Esc", "Clear"),
            ("?", "Help"),
        ],
        HomeFocus::Table => vec![
            ("j/k", "Navigate"),
            ("Enter", "View"),
            ("o", "Open"),
            ("a", "Add to List"),
            ("/", "Search"),
            ("f", "Filter"),
            ("L", "Lists"),
            ("S", "Settings"),
            ("q", "Quit"),
            ("?", "Help"),
        ],
    };
    render_status_bar(frame, layout[4], &hints);

    if state.filter.open {
        render_filter_popup(frame, area, &state.filter);
    }
}

fn render_search_bar(frame: &mut Frame, area: Rect, state: &HomeState) {
    let is_focused = matches!(state.focus, HomeFocus::Search);
    let cursor = if is_focused { "\u{258e}" } else { "" };
    let icon_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let line = Line::from(vec![
        Span::styled("  / ", icon_style),
        Span::styled(&state.search_query, Style::default().fg(Color::White)),
        Span::styled(cursor, Style::default().fg(Color::Cyan)),
    ]);
    let bar = Paragraph::new(line).style(Style::default().bg(Color::Black));
    frame.render_widget(bar, area);
}

fn render_stats_header(frame: &mut Frame, area: Rect, stats: &UserStats) {
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
    ]).split(area);

    let total_solved = stats.easy_solved + stats.medium_solved + stats.hard_solved;
    let total_all = stats.easy_total + stats.medium_total + stats.hard_total;

    let line0 = Line::from(vec![
        Span::styled(
            format!("  {} ", stats.username),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{total_solved}/{total_all} solved"),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line0), rows[0]);

    let line1 = Line::from(vec![
        Span::styled("  Easy ", Style::default().fg(Color::Green)),
        Span::styled(
            format!("{}/{}", stats.easy_solved, stats.easy_total),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled("Med ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{}/{}", stats.medium_solved, stats.medium_total),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled("Hard ", Style::default().fg(Color::Red)),
        Span::styled(
            format!("{}/{}", stats.hard_solved, stats.hard_total),
            Style::default().fg(Color::White),
        ),
    ]);
    frame.render_widget(Paragraph::new(line1), rows[1]);
}

fn render_title_bar(frame: &mut Frame, area: Rect, state: &HomeState) {
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

    if let Some(summary) = state.filter.summary() {
        spans.push(Span::styled(
            format!("{summary} "),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if !state.problems.is_empty() {
        spans.push(Span::styled(
            format!("{} / {} results", state.filtered_indices.len(), state.search_total),
            Style::default().fg(Color::DarkGray),
        ));
    }

    if state.search_loading {
        let spinner = ["\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}", "\u{2826}", "\u{2827}", "\u{2807}", "\u{280f}"];
        let s = spinner[state.spinner_frame % spinner.len()];
        spans.push(Span::styled(
            format!(" {s}"),
            Style::default().fg(Color::Yellow),
        ));
    }

    let title = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));
    frame.render_widget(title, area);
}

fn render_table(frame: &mut Frame, area: Rect, state: &mut HomeState) {
    let header = Row::new([
        Cell::from(" "),
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
            let paid = if p.is_paid_only { " \u{1f512}" } else { "" };
            let status_cell = match p.status.as_deref() {
                Some("ac") => Cell::from(Span::styled(" \u{2714}", Style::default().fg(Color::Green))),
                Some("notac") => Cell::from(Span::styled(" \u{25cf}", Style::default().fg(Color::Yellow))),
                _ => Cell::from("  "),
            };
            Row::new([
                status_cell,
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
        Constraint::Length(3),
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
        .highlight_symbol("\u{25b8} ");

    frame.render_stateful_widget(table, area, &mut state.table_state);
}

fn render_filter_popup(frame: &mut Frame, area: Rect, filter: &FilterState) {
    let popup_width = 30u16.min(area.width.saturating_sub(4));
    let popup_height = 9u16;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Filter ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    frame.render_widget(block, popup_area);

    let inner = Rect::new(popup_area.x + 2, popup_area.y + 1, popup_area.width.saturating_sub(4), popup_area.height.saturating_sub(2));
    let items = [
        ("Easy", filter.easy, Color::Green),
        ("Medium", filter.medium, Color::Yellow),
        ("Hard", filter.hard, Color::Red),
        ("Hide Solved", filter.hide_solved, Color::Cyan),
    ];

    let mut constraints: Vec<Constraint> = items.iter().map(|_| Constraint::Length(1)).collect();
    constraints.push(Constraint::Length(1));
    constraints.push(Constraint::Length(1));
    let rows = Layout::vertical(constraints).split(inner);

    for (i, ((label, checked, color), row)) in items.iter().zip(rows.iter()).enumerate() {
        let marker = if *checked { "\u{25c9}" } else { "\u{25cb}" };
        let highlight = i == filter.active_item;
        let style = if highlight {
            Style::default().fg(*color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(*color)
        };
        let prefix = if highlight { "\u{25b8} " } else { "  " };
        let line = Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("{marker} "), style),
            Span::styled(*label, style),
        ]);
        frame.render_widget(Paragraph::new(line), *row);
    }

    let hint = Paragraph::new(Line::from(Span::styled(
        "  Space: toggle  Esc: close",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(hint, rows[items.len() + 1]);
}
