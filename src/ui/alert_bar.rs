use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

/// Bottom bar — alert + keybindings on a subtle background.
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();

    if let Some(alert) = app.alerts.last() {
        let (icon, color) = match alert {
            crate::model::Alert::ContextHigh { pct, .. } => {
                if *pct >= 95.0 {
                    (" ▲ ", Color::Rgb(243, 139, 168)) // red
                } else {
                    (" ▲ ", Color::Rgb(249, 226, 175)) // yellow
                }
            }
            crate::model::Alert::CostThreshold { .. } => (" $ ", Color::Rgb(249, 226, 175)),
            crate::model::Alert::ProviderDisconnected { .. } => (" ✗ ", Color::Rgb(243, 139, 168)),
        };
        spans.push(Span::styled(icon, Style::default().fg(color)));
        spans.push(Span::styled(format!("{alert}"), Style::default().fg(color)));
    }

    let keys = "1-9 tabs · j/k nav · Enter open · Esc back · ? help · q quit";
    let left_len: usize = spans.iter().map(|s| s.width()).sum();
    let right_len = keys.len() + 2;
    let padding = area
        .width
        .saturating_sub(left_len as u16 + right_len as u16);
    spans.push(Span::raw(" ".repeat(padding as usize)));
    spans.push(Span::styled(
        format!(" {keys} "),
        Style::default().fg(Color::Rgb(88, 91, 112)),
    ));

    let bg = Style::default().bg(Color::Rgb(30, 30, 46));
    let line = Paragraph::new(Line::from(spans)).style(bg);
    frame.render_widget(line, area);
}
