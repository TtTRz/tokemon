use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::App;
use crate::model::SessionStatus;

/// Left panel — session navigation list.
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let (indicator, color) = match s.status {
                SessionStatus::Active => ("●", Color::Green),
                SessionStatus::Idle => ("○", Color::Yellow),
                SessionStatus::Done => ("✓", Color::DarkGray),
                SessionStatus::Disconnected => ("✗", Color::Red),
            };

            // Truncate session_id to fit
            let id_short = if s.session_id.len() > 6 {
                &s.session_id[..6]
            } else {
                &s.session_id
            };

            let provider_label = match s.provider.as_str() {
                "Claude Code" => "CC",
                "Codex" => "CDX",
                "CodeBuddy" => "CB",
                other => &other[..other.len().min(3)],
            };

            let style = if Some(i) == app.selected_index {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(format!("{indicator} "), Style::default().fg(color)),
                Span::styled(format!("{provider_label:<3} "), Style::default().fg(Color::Blue)),
                Span::styled(format!("#{id_short}"), style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let block = Block::default()
        .title(" Sessions ")
        .borders(Borders::RIGHT);

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    let mut state = ListState::default();
    state.select(app.selected_index);
    frame.render_stateful_widget(list, area, &mut state);
}
