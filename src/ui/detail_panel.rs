use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
};

use super::pad_r;
use super::shared::{ctx_color, fmt_speed, fmt_tok, label_col_width, render_status_badge};
use super::theme::*;
use crate::app::App;
use crate::model::SessionSnapshot;
use rust_i18n::t;

pub fn render_session(frame: &mut Frame, app: &App, session_idx: usize, area: Rect) {
    let block = Block::default()
        .title(Line::styled(
            format!(" {} ", t!("detail.title")),
            Style::default().fg(SUBTEXT0),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(SURFACE1))
        .padding(Padding::new(1, 1, 1, 0))
        .style(Style::default().bg(BASE));

    let Some(s) = app.sessions.get(session_idx) else {
        let empty = Paragraph::new(format!("  {}", t!("overview.session_not_found")))
            .style(Style::default().fg(OVERLAY0))
            .block(block);
        frame.render_widget(empty, area);
        return;
    };

    render_status_badge(frame, s.status, area);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // 0=model, 1=ctx, 2=tokens, 3=speed, 4=cost, 5=branch
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let lbl = Style::default().fg(OVERLAY0);
    let sep = Span::styled(" · ", Style::default().fg(SURFACE1));

    // Compute dynamic label width for i18n
    let lw = label_col_width(&[
        &t!("label.model"),
        &t!("label.ctx"),
        &t!("label.in"),
        &t!("label.out"),
        &t!("label.cached"),
        &t!("label.cost"),
        &t!("label.est"),
        &t!("label.total"),
        &t!("label.branch"),
        &t!("label.dir"),
    ]);

    // Row 0: Model + subagent count
    let mut model_spans = vec![
        Span::styled(pad_r(&t!("label.model"), lw), lbl),
        Span::styled(
            &s.model,
            Style::default().fg(PEACH).add_modifier(Modifier::BOLD),
        ),
    ];
    if s.subagent_count > 0 {
        model_spans.push(Span::styled(
            format!("  ⑂ {}", t!("detail.subagents", count = s.subagent_count)),
            Style::default().fg(OVERLAY0),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(model_spans)), rows[0]);

    // Row 1: Context bar
    let ctx_spans = build_context_line(s, rows[1].width, lw);
    frame.render_widget(Paragraph::new(Line::from(ctx_spans)), rows[1]);

    // Row 2: In / Out / Cached
    let cached = s.cache_read_tokens + s.cache_creation_tokens;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(pad_r(&t!("label.in"), lw), lbl),
            Span::styled(
                pad_r(&fmt_tok(s.input_tokens), 10),
                Style::default().fg(SKY),
            ),
            sep.clone(),
            Span::styled(pad_r(&t!("label.out"), lw), lbl),
            Span::styled(
                pad_r(&fmt_tok(s.output_tokens), 10),
                Style::default().fg(MAUVE),
            ),
            sep.clone(),
            Span::styled(pad_r(&t!("label.cached"), lw), lbl),
            Span::styled(fmt_tok(cached), Style::default().fg(LAVENDER)),
        ])),
        rows[2],
    );

    // Row 3: Speed + Total
    let total = s.input_tokens + s.output_tokens;
    let in_tps = s
        .input_tps
        .map(|v| format!("{} t/s", fmt_speed(v)))
        .unwrap_or_else(|| "—".into());
    let out_tps = s
        .output_tps
        .map(|v| format!("{} t/s", fmt_speed(v)))
        .unwrap_or_else(|| "—".into());
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(pad_r(&t!("label.in"), lw), lbl),
            Span::styled(pad_r(&in_tps, 10), Style::default().fg(TEAL)),
            sep.clone(),
            Span::styled(pad_r(&t!("label.out"), lw), lbl),
            Span::styled(pad_r(&out_tps, 10), Style::default().fg(TEAL)),
            sep.clone(),
            Span::styled(pad_r(&t!("label.total"), lw), lbl),
            Span::styled(fmt_tok(total), Style::default().fg(TEXT)),
        ])),
        rows[3],
    );

    // Row 4: Cost
    let est = app.pricing.estimate_cost(s);
    let cost_str = if let Some(r) = s.cost_reported {
        format!("${r:.2}")
    } else {
        format!("~${est:.2}")
    };
    let mut cost_spans = vec![
        Span::styled(pad_r(&t!("label.cost"), lw), lbl),
        Span::styled(pad_r(&cost_str, 10), Style::default().fg(GREEN)),
    ];
    if s.cost_reported.is_some() {
        cost_spans.push(sep.clone());
        cost_spans.push(Span::styled(pad_r(&t!("label.est"), lw), lbl));
        cost_spans.push(Span::styled(
            format!("${est:.2}"),
            Style::default().fg(OVERLAY0),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(cost_spans)), rows[4]);

    // Row 5: Branch + Dir
    let branch = s.git_branch.as_deref().unwrap_or("—");
    let dir = s.work_dir.as_deref().unwrap_or("—");
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(pad_r(&t!("label.branch"), lw), lbl),
            Span::styled(pad_r(branch, 10), Style::default().fg(TEAL)),
            sep,
            Span::styled(pad_r(&t!("label.dir"), lw), lbl),
            Span::styled(dir.to_string(), Style::default().fg(SUBTEXT0)),
        ])),
        rows[5],
    );
}

fn build_context_line(s: &SessionSnapshot, width: u16, lw: usize) -> Vec<Span<'static>> {
    let lbl = Style::default().fg(OVERLAY0);
    let mut spans = vec![Span::styled(pad_r(&t!("label.ctx"), lw), lbl)];

    match (s.context_tokens, s.context_max, s.context_window_pct) {
        (Some(used), Some(max), Some(pct)) => {
            let bar_width = (width as usize).saturating_sub(40).clamp(8, 28);
            let filled = ((pct / 100.0) * bar_width as f64).round() as usize;
            let empty = bar_width.saturating_sub(filled);
            let color = ctx_color(pct);

            spans.push(Span::styled("[", Style::default().fg(SURFACE1)));
            spans.push(Span::styled("━".repeat(filled), Style::default().fg(color)));
            spans.push(Span::styled(
                "─".repeat(empty),
                Style::default().fg(SURFACE0),
            ));
            spans.push(Span::styled("] ", Style::default().fg(SURFACE1)));
            spans.push(Span::styled(
                format!("{}/{}", fmt_tok(used), fmt_tok(max)),
                Style::default().fg(SUBTEXT0),
            ));
            spans.push(Span::styled(
                format!(" ({pct:.0}%)"),
                Style::default().fg(color),
            ));
        }
        (None, None, Some(pct)) => {
            let bar_width = 20_usize;
            let filled = ((pct / 100.0) * bar_width as f64).round() as usize;
            let empty = bar_width.saturating_sub(filled);
            let color = ctx_color(pct);
            spans.push(Span::styled("[", Style::default().fg(SURFACE1)));
            spans.push(Span::styled("━".repeat(filled), Style::default().fg(color)));
            spans.push(Span::styled(
                "─".repeat(empty),
                Style::default().fg(SURFACE0),
            ));
            spans.push(Span::styled("] ", Style::default().fg(SURFACE1)));
            spans.push(Span::styled(
                format!("{pct:.0}%"),
                Style::default().fg(color),
            ));
        }
        _ => {
            spans.push(Span::styled("—", Style::default().fg(SURFACE1)));
        }
    }

    spans
}
