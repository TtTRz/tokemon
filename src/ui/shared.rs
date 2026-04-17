use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};
use rust_i18n::t;

use super::theme::*;
use crate::model::SessionStatus;

pub fn fmt_tok(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        format!("{n}")
    }
}

pub fn fmt_speed(v: f64) -> String {
    if v >= 1_000.0 {
        format!("{:.1}k", v / 1_000.0)
    } else if v >= 100.0 {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    }
}

pub fn ctx_color(pct: f64) -> Color {
    if pct >= 95.0 {
        RED
    } else if pct >= 80.0 {
        YELLOW
    } else {
        GREEN
    }
}

pub fn label_col_width(labels: &[&str]) -> usize {
    use unicode_width::UnicodeWidthStr;
    labels
        .iter()
        .map(|l| UnicodeWidthStr::width(*l))
        .max()
        .unwrap_or(6)
        + 1
}

/// Render a rounded pill status badge at the top-right corner.
pub fn render_status_badge(frame: &mut Frame, status: SessionStatus, area: Rect) {
    let (dot_color, label_color, label, pill_bg) = match status {
        SessionStatus::Active => (
            Color::Rgb(116, 199, 136),
            Color::Rgb(186, 230, 190),
            t!("status.active").to_string(),
            Color::Rgb(28, 48, 36),
        ),
        SessionStatus::Idle => (
            Color::Rgb(229, 200, 120),
            Color::Rgb(240, 220, 170),
            t!("status.idle").to_string(),
            Color::Rgb(48, 42, 24),
        ),
        SessionStatus::Done => (
            Color::Rgb(120, 200, 190),
            Color::Rgb(170, 210, 206),
            t!("status.done").to_string(),
            Color::Rgb(30, 44, 46),
        ),
        SessionStatus::Disconnected => (
            Color::Rgb(220, 110, 130),
            Color::Rgb(240, 160, 170),
            t!("status.offline").to_string(),
            Color::Rgb(52, 26, 32),
        ),
    };

    let pill_spans = vec![
        Span::styled("\u{2590}", Style::default().fg(pill_bg).bg(BASE)),
        Span::styled(" ● ", Style::default().fg(dot_color).bg(pill_bg)),
        Span::styled(
            format!("{label} "),
            Style::default().fg(label_color).bg(pill_bg),
        ),
        Span::styled("\u{258C}", Style::default().fg(pill_bg).bg(BASE)),
    ];

    let badge_width: u16 = pill_spans.iter().map(|s| s.width() as u16).sum();
    if badge_width + 3 > area.width {
        return;
    }

    let x = area.x + area.width - badge_width - 1;
    let badge_area = Rect::new(x, area.y, badge_width, 1);
    frame.render_widget(Paragraph::new(Line::from(pill_spans)), badge_area);
}
