use chrono::Local;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Row, Table, TableState},
};
use rust_i18n::t;

use super::shared::fmt_tok;
use super::theme::*;
use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let done_count = app.done_sessions().len();

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(
                format!(" {} ", t!("history.title")),
                Style::default().fg(SUBTEXT0).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {} ", t!("history.sessions_count", count = done_count)),
                Style::default().fg(OVERLAY0),
            ),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(SURFACE1))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(BASE));

    if done_count == 0 {
        let empty = ratatui::widgets::Paragraph::new(format!("  {}", t!("history.empty")))
            .style(Style::default().fg(OVERLAY0))
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Update visible rows for scrolling
    let row_height = 1_u16;
    let header_height = 1_u16;
    let visible = ((inner.height.saturating_sub(header_height)) / row_height) as usize;
    app.history_visible_rows = visible.max(1);
    app.history_ensure_visible();

    // Clamp selected
    if app.history_selected >= done_count {
        app.history_selected = done_count.saturating_sub(1);
    }

    // Re-borrow after mutations
    let done = app.done_sessions();

    // Header
    let header = Row::new(vec![
        format!("  {}", t!("history.col_provider")),
        t!("history.col_session").to_string(),
        t!("history.col_model").to_string(),
        t!("history.col_in").to_string(),
        t!("history.col_out").to_string(),
        t!("history.col_cost").to_string(),
        t!("history.col_dir").to_string(),
        t!("history.col_time").to_string(),
    ])
    .style(Style::default().fg(OVERLAY0).add_modifier(Modifier::BOLD));

    let widths = [
        ratatui::layout::Constraint::Length(14),
        ratatui::layout::Constraint::Length(10),
        ratatui::layout::Constraint::Length(26),
        ratatui::layout::Constraint::Length(10),
        ratatui::layout::Constraint::Length(10),
        ratatui::layout::Constraint::Length(10),
        ratatui::layout::Constraint::Length(20),
        ratatui::layout::Constraint::Min(16),
    ];

    let rows: Vec<Row> = done
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let id_short = &s.session_id[..s.session_id.len().min(8)];
            let model_short = s.model.clone();
            let cost = if let Some(c) = s.cost_reported {
                format!("${c:.2}")
            } else {
                let est = app.pricing.estimate_cost(s);
                format!("~${est:.2}")
            };
            let dir = s.work_dir.as_deref().unwrap_or("—");
            let dir_short = if dir.len() > 18 {
                format!("…{}", &dir[dir.len() - 17..])
            } else {
                dir.to_string()
            };
            let time = s
                .timestamp
                .with_timezone(&Local)
                .format("%m-%d %H:%M")
                .to_string();

            let style = if i == app.history_selected {
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(SUBTEXT0)
            };

            Row::new(vec![
                Line::styled(format!("  {}", s.provider), style),
                Line::styled(format!("#{id_short}"), style),
                Line::styled(model_short, Style::default().fg(PEACH)),
                Line::styled(fmt_tok(s.input_tokens), Style::default().fg(SKY)),
                Line::styled(fmt_tok(s.output_tokens), Style::default().fg(MAUVE)),
                Line::styled(cost, Style::default().fg(GREEN)),
                Line::styled(dir_short, Style::default().fg(OVERLAY0)),
                Line::styled(time, Style::default().fg(OVERLAY0)),
            ])
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().bg(SURFACE0).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    let mut state = TableState::default();
    state.select(Some(app.history_selected));
    state.offset_mut().clone_from(&app.history_scroll);
    frame.render_stateful_widget(table, inner, &mut state);
}
