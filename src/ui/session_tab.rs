use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::app::App;

/// Single-session tab: full detail panel (top) + per-session charts (bottom).
pub fn render(frame: &mut Frame, app: &App, session_id: &str, area: Rect) {
    let Some(session_idx) = app.sessions.iter().position(|s| s.session_id == session_id) else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // detail info (6 rows + border + padding)
            Constraint::Min(6),     // charts
        ])
        .split(area);

    super::detail_panel::render_session(frame, app, session_idx, chunks[0]);

    let token_data = app
        .session_token_series
        .get(session_id)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let cost_data = app
        .session_cost_series
        .get(session_id)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    super::trend_chart::render_with_data(frame, token_data, cost_data, chunks[1]);
}
