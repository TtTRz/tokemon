use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};
use rust_i18n::t;

use super::theme::*;
use crate::app::App;

/// Bottom bar — model pricing (left) + keybindings (right).
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();

    if let Some(s) = app.focused_session() {
        if let Some(pricing) = app.pricing.get_price(&s.model) {
            let mut parts = vec![
                Span::styled(" ", Style::default()),
                Span::styled(&s.model, Style::default().fg(PEACH)),
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("in ${}/M", fmt_price(pricing.input_per_mtok)),
                    Style::default().fg(SKY),
                ),
                Span::styled(" · ", Style::default().fg(SURFACE1)),
                Span::styled(
                    format!("out ${}/M", fmt_price(pricing.output_per_mtok)),
                    Style::default().fg(MAUVE),
                ),
            ];
            if let Some(cw) = pricing.cache_write_per_mtok {
                parts.push(Span::styled(" · ", Style::default().fg(SURFACE1)));
                parts.push(Span::styled(
                    format!("cache_w ${}/M", fmt_price(cw)),
                    Style::default().fg(LAVENDER),
                ));
            }
            if let Some(cr) = pricing.cache_read_per_mtok {
                parts.push(Span::styled(" · ", Style::default().fg(SURFACE1)));
                parts.push(Span::styled(
                    format!("cache_r ${}/M", fmt_price(cr)),
                    Style::default().fg(LAVENDER),
                ));
            }
            spans.extend(parts);
        } else {
            spans.push(Span::styled(" ", Style::default()));
            spans.push(Span::styled(&s.model, Style::default().fg(PEACH)));
            spans.push(Span::styled("  ", Style::default()));
            spans.push(Span::styled(
                t!("alert.no_pricing").to_string(),
                Style::default().fg(YELLOW),
            ));
        }
    }

    let keys = t!("alert.keys").to_string();
    let left_len: usize = spans.iter().map(|s| s.width()).sum();
    let right_len = unicode_width::UnicodeWidthStr::width(keys.as_str()) + 2;
    let padding = area
        .width
        .saturating_sub(left_len as u16 + right_len as u16);
    spans.push(Span::raw(" ".repeat(padding as usize)));
    spans.push(Span::styled(
        format!(" {keys} "),
        Style::default().fg(OVERLAY0),
    ));

    let line = Paragraph::new(Line::from(spans)).style(Style::default().bg(BASE));
    frame.render_widget(line, area);
}

fn fmt_price(v: f64) -> String {
    if v >= 1.0 && v == v.floor() {
        format!("{:.0}", v)
    } else {
        format!("{:.2}", v)
    }
}
