use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::Block,
};

use super::theme::BASE;
use crate::app::{ActiveTab, App};
use crate::model::SessionStatus;

/// Render the entire UI.
pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Fill entire area with base background
    frame.render_widget(Block::default().style(Style::default().bg(BASE)), size);

    let has_live = app
        .sessions
        .iter()
        .any(|s| matches!(s.status, SessionStatus::Active | SessionStatus::Idle));

    if !has_live && app.active_tab == ActiveTab::Overview {
        // No live sessions on overview: full-screen welcome, no header/footer
        super::overview::render(frame, app, size);
        return;
    }

    // Has sessions: Header(5) | gap(1) | Body | gap(1) | AlertBar(1)
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // header (4-line logo + 1 gap + 1-line tabs)
            Constraint::Length(1), // gap
            Constraint::Min(10),   // body
            Constraint::Length(1), // gap
            Constraint::Length(1), // alert bar
        ])
        .split(size);

    super::header::render(frame, app, outer[0]);
    super::alert_bar::render(frame, app, outer[4]);

    match app.active_tab.clone() {
        ActiveTab::Overview => {
            super::overview::render(frame, app, outer[2]);
        }
        ActiveTab::History => {
            super::history::render(frame, app, outer[2]);
        }
        ActiveTab::Session(session_id) => {
            super::session_tab::render(frame, app, &session_id, outer[2]);
        }
    }
}
