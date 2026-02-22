use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::api::types::QuestionDetail;

use super::status_bar::render_status_bar;

pub struct DetailState {
    pub detail: QuestionDetail,
    pub rendered_content: String,
    pub scroll_offset: u16,
    pub content_height: u16,
}

impl DetailState {
    pub fn new(detail: QuestionDetail) -> Self {
        let rendered_content = if detail.is_paid_only && detail.content.is_none() {
            "Premium content — not available without authentication.".to_string()
        } else if let Some(ref html) = detail.content {
            html2text::from_read(html.as_bytes(), 100)
                .unwrap_or_else(|_| "Failed to render content.".to_string())
        } else {
            "No content available.".to_string()
        };

        Self {
            detail,
            rendered_content,
            scroll_offset: 0,
            content_height: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> DetailAction {
        match key.code {
            KeyCode::Char('b') | KeyCode::Esc => DetailAction::Back,
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll(1);
                DetailAction::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll(-1);
                DetailAction::None
            }
            KeyCode::Char('d') => {
                self.scroll(self.content_height as i32 / 2);
                DetailAction::None
            }
            KeyCode::Char('u') => {
                self.scroll(-(self.content_height as i32 / 2));
                DetailAction::None
            }
            KeyCode::Char('o') => {
                DetailAction::Scaffold(self.detail.title_slug.clone())
            }
            KeyCode::Char('q') => DetailAction::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                DetailAction::Quit
            }
            _ => DetailAction::None,
        }
    }

    fn scroll(&mut self, delta: i32) {
        let new_offset = self.scroll_offset as i32 + delta;
        self.scroll_offset = new_offset.max(0) as u16;
    }
}

pub enum DetailAction {
    None,
    Back,
    Quit,
    Scaffold(String),
}

pub fn render_detail(frame: &mut Frame, area: Rect, state: &mut DetailState) {
    let layout = Layout::vertical([
        Constraint::Length(3), // title bar
        Constraint::Min(3),   // content
        Constraint::Length(1), // status bar
    ])
    .split(area);

    // Title bar
    render_detail_title(frame, layout[0], state);

    // Content area
    state.content_height = layout[1].height;
    let content_lines: Vec<Line> = state
        .rendered_content
        .lines()
        .map(|l| Line::from(l.to_string()))
        .collect();

    let total_lines = content_lines.len() as u16;
    let max_scroll = total_lines.saturating_sub(state.content_height);
    if state.scroll_offset > max_scroll {
        state.scroll_offset = max_scroll;
    }

    let content = Paragraph::new(content_lines)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default()),
        )
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_offset, 0));

    frame.render_widget(content, layout[1]);

    // Status bar
    render_status_bar(
        frame,
        layout[2],
        &[
            ("j/k", "Scroll"),
            ("d/u", "Half page"),
            ("o", "Open"),
            ("b/Esc", "Back"),
            ("q", "Quit"),
        ],
    );
}

fn render_detail_title(frame: &mut Frame, area: Rect, state: &DetailState) {
    let d = &state.detail;
    let diff_color = match d.difficulty.as_str() {
        "Easy" => Color::Green,
        "Medium" => Color::Yellow,
        "Hard" => Color::Red,
        _ => Color::White,
    };

    let title_line = Line::from(vec![
        Span::styled(
            format!(" {}. {} ", d.frontend_question_id, d.title),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("[{}]", d.difficulty),
            Style::default()
                .fg(diff_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let tags: String = d
        .topic_tags
        .iter()
        .map(|t| t.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let tags_line = Line::from(vec![
        Span::styled(" Tags: ", Style::default().fg(Color::DarkGray)),
        Span::styled(tags, Style::default().fg(Color::Gray)),
    ]);

    let title_block = Paragraph::new(vec![title_line, tags_line])
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(title_block, area);
}
