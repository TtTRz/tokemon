use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::pad_r;
use super::theme::*;
use crate::app::App;
use rust_i18n::t;

/// 6-line header: 4-line ANSI Shadow logo + 1-line gap + 1-line tab bar.
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // logo
            Constraint::Length(1), // gap
            Constraint::Length(1), // tabs
        ])
        .split(area);

    render_logo(frame, app, rows[0]);
    render_tab_row(frame, app, rows[2]);
}

/// 4-line ANSI Shadow compressed logo with right-aligned stats on last line.
fn render_logo(frame: &mut Frame, app: &App, area: Rect) {
    #[rustfmt::skip]
    let logo_lines = [
        "████████╗ ██████╗ ██╗  ██╗███████╗███╗   ███╗ ██████╗ ███╗   ██╗",
        "   ██║   ██║   ██║█████╔╝ ██╔════╝████╗ ████║██║   ██║████╗  ██║",
        "   ██║   ██║   ██║██╔═██╗ █████╗  ██╔████╔██║██║   ██║██╔██╗ ██║",
        "   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═══╝",
    ];

    // Summary stats — right-aligned, one per logo line
    let total_tokens: u64 = app
        .sessions
        .iter()
        .map(|s| s.input_tokens + s.output_tokens)
        .sum();
    let total_cost: f64 = app
        .sessions
        .iter()
        .map(|s| {
            s.cost_reported
                .unwrap_or_else(|| app.pricing.estimate_cost(s))
        })
        .sum();
    let active_count = app
        .sessions
        .iter()
        .filter(|s| matches!(s.status, crate::model::SessionStatus::Active))
        .count();
    let total_count = app.sessions.len();
    let total_cached: u64 = app
        .sessions
        .iter()
        .map(|s| s.cache_read_tokens + s.cache_creation_tokens)
        .sum();

    // Each logo line gets a stat label on the right, table-aligned
    let tok_val = fmt_compact(total_tokens);
    let cost_val = format!("${total_cost:.2}");
    let cache_val = fmt_compact(total_cached);
    let sess_val = t!(
        "header.sessions_summary",
        active = active_count,
        total = total_count
    )
    .to_string();

    // Fixed-width: label(10) + value(right-aligned in 22 chars)
    let stat_lines: Vec<Vec<Span>> = vec![
        vec![
            Span::styled(
                pad_r(&t!("header.tokens"), 10),
                Style::default().fg(OVERLAY0),
            ),
            Span::styled(pad_r(&tok_val, 22), Style::default().fg(SKY)),
        ],
        vec![
            Span::styled(pad_r(&t!("header.cost"), 10), Style::default().fg(OVERLAY0)),
            Span::styled(pad_r(&cost_val, 22), Style::default().fg(GREEN)),
        ],
        vec![
            Span::styled(
                pad_r(&t!("header.cached"), 10),
                Style::default().fg(OVERLAY0),
            ),
            Span::styled(pad_r(&cache_val, 22), Style::default().fg(LAVENDER)),
        ],
        vec![
            Span::styled(
                pad_r(&t!("header.sessions"), 10),
                Style::default().fg(OVERLAY0),
            ),
            Span::styled(pad_r(&sess_val, 22), Style::default().fg(SUBTEXT0)),
        ],
    ];

    for (i, logo) in logo_lines.iter().enumerate() {
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            break;
        }

        let row_area = Rect::new(area.x, y, area.width, 1);

        let color = match i {
            0 => BLUE,
            1 => LAVENDER,
            _ => SURFACE1,
        };
        let style = if i == 0 {
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color)
        };

        let logo_w = logo.chars().count() as u16;
        let stat_row = &stat_lines[i];
        let stat_w: u16 = stat_row.iter().map(|s| s.width() as u16).sum();
        let pad = area.width.saturating_sub(logo_w + stat_w + 2);

        let mut spans = vec![
            Span::styled(" ", Style::default()),
            Span::styled(*logo, style),
            Span::raw(" ".repeat(pad as usize)),
        ];
        spans.extend(stat_row.clone());

        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(BASE)),
            row_area,
        );
    }
}

/// Pill-style tab bar.
fn render_tab_row(frame: &mut Frame, app: &App, area: Rect) {
    let labels = app.tab_labels();
    let active = app.active_tab_index();
    let mut spans = vec![Span::styled(" ", Style::default())];

    for (i, label) in labels.iter().enumerate() {
        let key = i + 1;
        if i == active {
            spans.push(Span::styled("\u{2590}", Style::default().fg(BLUE).bg(BASE)));
            spans.push(Span::styled(
                format!(" {key} {label} "),
                Style::default()
                    .fg(Color::Rgb(30, 30, 46))
                    .bg(BLUE)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled("\u{258C}", Style::default().fg(BLUE).bg(BASE)));
        } else {
            spans.push(Span::styled(
                "\u{2590}",
                Style::default().fg(SURFACE0).bg(BASE),
            ));
            spans.push(Span::styled(
                format!(" {key}"),
                Style::default().fg(OVERLAY0).bg(SURFACE0),
            ));
            spans.push(Span::styled(
                format!(" {label} "),
                Style::default().fg(SUBTEXT0).bg(SURFACE0),
            ));
            spans.push(Span::styled(
                "\u{258C}",
                Style::default().fg(SURFACE0).bg(BASE),
            ));
        }
        spans.push(Span::styled(" ", Style::default()));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(BASE)),
        area,
    );
}

fn fmt_compact(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        format!("{n}")
    }
}
