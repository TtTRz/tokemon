use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
};

use crate::app::App;
use crate::model::{SessionSnapshot, SessionStatus};

// Catppuccin Mocha
const BASE: Color = Color::Rgb(30, 30, 46);
const SURFACE0: Color = Color::Rgb(49, 50, 68);
const SURFACE1: Color = Color::Rgb(69, 71, 90);
const OVERLAY0: Color = Color::Rgb(108, 112, 134);
const SUBTEXT0: Color = Color::Rgb(166, 173, 200);
const TEXT: Color = Color::Rgb(205, 214, 244);
const TEAL: Color = Color::Rgb(148, 226, 213);
const GREEN: Color = Color::Rgb(166, 227, 161);
const YELLOW: Color = Color::Rgb(249, 226, 175);
const PEACH: Color = Color::Rgb(250, 179, 135);
const RED: Color = Color::Rgb(243, 139, 168);
const MAUVE: Color = Color::Rgb(203, 166, 247);
const LAVENDER: Color = Color::Rgb(180, 190, 254);
const SKY: Color = Color::Rgb(137, 220, 235);

pub fn render_session(frame: &mut Frame, app: &App, session_idx: usize, area: Rect) {
    let block = Block::default()
        .title(Line::styled(" Detail ", Style::default().fg(SUBTEXT0)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(SURFACE1))
        .padding(Padding::new(1, 1, 1, 0))
        .style(Style::default().bg(BASE));

    let Some(s) = app.sessions.get(session_idx) else {
        let empty = Paragraph::new("  Session not found")
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

    // Row 0: Model
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(pad_r("Model:", 8), lbl),
            Span::styled(
                &s.model,
                Style::default().fg(PEACH).add_modifier(Modifier::BOLD),
            ),
        ])),
        rows[0],
    );

    // Row 1: Context bar
    let ctx_spans = build_context_line(s, rows[1].width);
    frame.render_widget(Paragraph::new(Line::from(ctx_spans)), rows[1]);

    // Row 2: In / Out / Cached
    let cached = s.cache_read_tokens + s.cache_creation_tokens;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(pad_r("In:", 8), lbl),
            Span::styled(
                pad_r(&fmt_tok(s.input_tokens), 10),
                Style::default().fg(SKY),
            ),
            sep.clone(),
            Span::styled(pad_r("Out:", 8), lbl),
            Span::styled(
                pad_r(&fmt_tok(s.output_tokens), 10),
                Style::default().fg(MAUVE),
            ),
            sep.clone(),
            Span::styled(pad_r("Cached:", 8), lbl),
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
            Span::styled(pad_r("In:", 8), lbl),
            Span::styled(pad_r(&in_tps, 10), Style::default().fg(TEAL)),
            sep.clone(),
            Span::styled(pad_r("Out:", 8), lbl),
            Span::styled(pad_r(&out_tps, 10), Style::default().fg(TEAL)),
            sep.clone(),
            Span::styled(pad_r("Total:", 8), lbl),
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
        Span::styled(pad_r("Cost:", 8), lbl),
        Span::styled(pad_r(&cost_str, 10), Style::default().fg(GREEN)),
    ];
    if s.cost_reported.is_some() {
        cost_spans.push(sep.clone());
        cost_spans.push(Span::styled(pad_r("Est:", 8), lbl));
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
            Span::styled(pad_r("Branch:", 8), lbl),
            Span::styled(pad_r(branch, 10), Style::default().fg(TEAL)),
            sep,
            Span::styled(pad_r("Dir:", 8), lbl),
            Span::styled(dir.to_string(), Style::default().fg(SUBTEXT0)),
        ])),
        rows[5],
    );
}

fn build_context_line(s: &SessionSnapshot, width: u16) -> Vec<Span<'static>> {
    let lbl = Style::default().fg(OVERLAY0);
    let mut spans = vec![Span::styled(pad_r("Ctx:", 8), lbl)];

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

fn render_status_badge(frame: &mut Frame, status: SessionStatus, area: Rect) {
    let (dot_color, label_color, label, pill_bg) = match status {
        SessionStatus::Active => (
            Color::Rgb(116, 199, 136),
            Color::Rgb(186, 230, 190),
            "Active",
            Color::Rgb(28, 48, 36),
        ),
        SessionStatus::Idle => (
            Color::Rgb(229, 200, 120),
            Color::Rgb(240, 220, 170),
            "Idle",
            Color::Rgb(55, 50, 30),
        ),
        SessionStatus::Done => (
            Color::Rgb(120, 200, 190),
            Color::Rgb(170, 210, 206),
            "Done",
            Color::Rgb(30, 44, 46),
        ),
        SessionStatus::Disconnected => (
            Color::Rgb(220, 110, 130),
            Color::Rgb(240, 160, 170),
            "Offline",
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

fn fmt_tok(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        format!("{n}")
    }
}

fn fmt_speed(v: f64) -> String {
    if v >= 1_000.0 {
        format!("{:.1}k", v / 1_000.0)
    } else if v >= 100.0 {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    }
}

fn pad_r(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        format!("{s:<width$}")
    }
}

fn ctx_color(pct: f64) -> Color {
    if pct >= 95.0 {
        RED
    } else if pct >= 80.0 {
        YELLOW
    } else {
        GREEN
    }
}
