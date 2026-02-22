use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::status_bar::render_status_bar;

const FIELD_LABELS: [&str; 3] = ["Workspace Directory", "Language", "Editor"];
const FIELD_DEFAULTS: [&str; 3] = ["~/leetcode", "rust", "vim"];
const FIELD_HINTS: [&str; 3] = [
    "Directory where problem projects will be created",
    "Default language for code snippets (rust, python3, cpp, java, ...)",
    "Editor command to open files (vim, nvim, code, ...)",
];

pub struct SetupState {
    pub fields: [String; 3],
    pub active_field: usize,
}

impl SetupState {
    pub fn new() -> Self {
        Self {
            fields: [
                FIELD_DEFAULTS[0].to_string(),
                FIELD_DEFAULTS[1].to_string(),
                FIELD_DEFAULTS[2].to_string(),
            ],
            active_field: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> SetupAction {
        match key.code {
            KeyCode::Tab | KeyCode::Down => {
                self.active_field = (self.active_field + 1) % 3;
                SetupAction::None
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.active_field = (self.active_field + 2) % 3;
                SetupAction::None
            }
            KeyCode::Char(c) => {
                self.fields[self.active_field].push(c);
                SetupAction::None
            }
            KeyCode::Backspace => {
                self.fields[self.active_field].pop();
                SetupAction::None
            }
            KeyCode::Enter => SetupAction::Submit,
            KeyCode::Esc => SetupAction::Quit,
            _ => SetupAction::None,
        }
    }
}

pub enum SetupAction {
    None,
    Submit,
    Quit,
}

pub fn render_setup(frame: &mut Frame, state: &SetupState) {
    let area = frame.area();

    // Center the form
    let form_width = 60u16.min(area.width.saturating_sub(4));
    let form_height = 16u16.min(area.height.saturating_sub(2));
    let form_area = centered_rect(form_width, form_height, area);

    let block = Block::default()
        .title(" LeetCode CLI — Setup ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(Clear, form_area);
    frame.render_widget(block, form_area);

    let inner = form_area.inner(Margin::new(2, 1));

    let layout = Layout::vertical([
        Constraint::Length(1), // welcome text
        Constraint::Length(1), // spacer
        Constraint::Length(3), // field 0
        Constraint::Length(3), // field 1
        Constraint::Length(3), // field 2
        Constraint::Length(1), // spacer
        Constraint::Length(1), // status bar
    ])
    .split(inner);

    let welcome = Paragraph::new("Configure your LeetCode CLI settings:")
        .style(Style::default().fg(Color::White));
    frame.render_widget(welcome, layout[0]);

    for i in 0..3 {
        render_field(frame, layout[i + 2], i, state);
    }

    render_status_bar(
        frame,
        layout[6],
        &[
            ("Tab/↓", "Next"),
            ("Shift+Tab/↑", "Prev"),
            ("Enter", "Save"),
            ("Esc", "Quit"),
        ],
    );
}

fn render_field(frame: &mut Frame, area: Rect, index: usize, state: &SetupState) {
    let is_active = state.active_field == index;
    let label_style = if is_active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let value = &state.fields[index];
    let cursor = if is_active { "▎" } else { "" };

    let layout = Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let label = Line::from(vec![
        Span::styled(FIELD_LABELS[index], label_style),
        Span::styled(format!("  {}", FIELD_HINTS[index]), Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(label), layout[0]);

    let input_style = if is_active {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    };

    let input = Line::from(vec![
        Span::styled(format!(" {value}"), input_style),
        Span::styled(cursor, Style::default().fg(Color::Cyan)),
    ]);
    let input_block = Paragraph::new(input).style(
        Style::default().bg(if is_active {
            Color::DarkGray
        } else {
            Color::Black
        }),
    );
    frame.render_widget(input_block, layout[1]);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
