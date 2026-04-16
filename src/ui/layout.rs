use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::Block,
};

use crate::app::{ActiveTab, App};

const BASE: Color = Color::Rgb(30, 30, 46);

/// Render the entire UI.
pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Fill entire area with base background
    frame.render_widget(Block::default().style(Style::default().bg(BASE)), size);

    if app.sessions.is_empty() {
        // No sessions: full-screen welcome (overview handles it), no header/footer
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
