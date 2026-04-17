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
use crate::model::{SessionSnapshot, SessionStatus};
use rust_i18n::t;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let live: Vec<usize> = app
        .sessions
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            matches!(
                s.status,
                crate::model::SessionStatus::Active | crate::model::SessionStatus::Idle
            )
        })
        .map(|(i, _)| i)
        .collect();

    if live.is_empty() {
        render_welcome(frame, area);
        return;
    }

    let cols = if area.width >= 110 { 2 } else { 1 };
    app.overview_cols = cols;
    let session_count = live.len();
    let rows_needed = session_count.div_ceil(cols);
    let overflows = rows_needed > 1;

    // Reserve 1 row top + 1 row bottom for scroll hints when content can overflow
    let hint_rows: u16 = if overflows { 2 } else { 0 };
    let cards_area = if overflows {
        Rect::new(
            area.x,
            area.y + 1,
            area.width,
            area.height.saturating_sub(hint_rows),
        )
    } else {
        area
    };

    let card_height: u16 = 21;
    let visible_rows = (cards_area.height / card_height).max(1) as usize;
    app.overview_visible_rows = visible_rows;

    // Clamp selected to live session count
    if app.overview_selected >= session_count && session_count > 0 {
        app.overview_selected = session_count - 1;
    }

    // Clamp scroll
    let max_scroll = rows_needed.saturating_sub(visible_rows);
    if app.overview_scroll > max_scroll {
        app.overview_scroll = max_scroll;
    }

    // Ensure selected is visible
    app.overview_ensure_visible();

    let start_row = app.overview_scroll;
    let end_row = (start_row + visible_rows).min(rows_needed);

    let row_constraints: Vec<Constraint> = (start_row..end_row)
        .map(|_| Constraint::Length(card_height))
        .collect();

    let row_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(cards_area);

    for (vi, row_area) in row_areas.iter().enumerate() {
        let grid_row = start_row + vi;
        let start = grid_row * cols;
        let end = (start + cols).min(session_count);
        let sessions_in_row = end - start;

        if sessions_in_row == 0 {
            break;
        }

        let col_constraints: Vec<Constraint> = (0..sessions_in_row)
            .map(|_| Constraint::Ratio(1, sessions_in_row as u32))
            .collect();

        let col_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(*row_area);

        for (col_idx, col_area) in col_areas.iter().enumerate() {
            let live_idx = start + col_idx;
            if live_idx >= live.len() {
                break;
            }
            let session_idx = live[live_idx];
            let selected = live_idx == app.overview_selected;
            render_card(frame, app, session_idx, selected, *col_area);
        }
    }

    // Scroll hints (if content overflows)
    if rows_needed > visible_rows {
        let has_above = app.overview_scroll > 0;
        let has_below = app.overview_scroll + visible_rows < rows_needed;

        // Top hint: ▲ more above
        if has_above {
            let hint = format!(
                " {} ",
                t!("overview.scroll_above", count = app.overview_scroll)
            );
            let hw = unicode_width::UnicodeWidthStr::width(hint.as_str()) as u16;
            let hx = area.x + (area.width.saturating_sub(hw)) / 2;
            let r = Rect::new(hx, area.y, hw, 1);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    hint,
                    Style::default().fg(OVERLAY0).bg(SURFACE0),
                ))),
                r,
            );
        }

        // Bottom hint: ▼ more below + page indicator
        if has_below {
            let remaining = rows_needed - app.overview_scroll - visible_rows;
            let hint = format!(
                " {} ",
                t!(
                    "overview.scroll_below",
                    count = remaining,
                    cur = app.overview_scroll + 1,
                    total = rows_needed
                )
            );
            let hw = unicode_width::UnicodeWidthStr::width(hint.as_str()) as u16;
            let hx = area.x + (area.width.saturating_sub(hw)) / 2;
            let hy = area.y + area.height.saturating_sub(1);
            let r = Rect::new(hx, hy, hw, 1);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    hint,
                    Style::default().fg(OVERLAY0).bg(SURFACE0),
                ))),
                r,
            );
        }
    }
}

fn render_welcome(frame: &mut Frame, area: Rect) {
    // ANSI Shadow style "TOKEMON"
    #[rustfmt::skip]
    let logo_lines = [
        "████████╗ ██████╗ ██╗  ██╗███████╗███╗   ███╗ ██████╗ ███╗   ██╗",
        "╚══██╔══╝██╔═══██╗██║ ██╔╝██╔════╝████╗ ████║██╔═══██╗████╗  ██║",
        "   ██║   ██║   ██║█████╔╝ █████╗  ██╔████╔██║██║   ██║██╔██╗ ██║",
        "   ██║   ██║   ██║██╔═██╗ ██╔══╝  ██║╚██╔╝██║██║   ██║██║╚██╗██║",
        "   ██║   ╚██████╔╝██║  ██╗███████╗██║ ╚═╝ ██║╚██████╔╝██║ ╚████║",
        "   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═══╝",
    ];

    let subtitle = t!("overview.welcome_subtitle").to_string();
    let hint = t!("overview.welcome_waiting").to_string();

    // Total block height: logo(6) + blank(1) + subtitle(1) + blank(1) + hint(1) = 10
    let block_height: u16 = 10;

    // Center vertically
    let y_offset = area.y + area.height.saturating_sub(block_height) / 2;

    // Render logo lines with gradient matching header
    for (i, line) in logo_lines.iter().enumerate() {
        let line_width = line.chars().count() as u16;
        let x = area.x + area.width.saturating_sub(line_width) / 2;
        let y = y_offset + i as u16;
        if y >= area.y + area.height {
            break;
        }
        let color = match i {
            0 => BLUE,
            1 => LAVENDER,
            2 | 3 => LAVENDER,
            _ => SURFACE1,
        };
        let style = if i == 0 {
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color)
        };
        let r = Rect::new(x, y, line_width.min(area.width), 1);
        frame.render_widget(Paragraph::new(Line::from(Span::styled(*line, style))), r);
    }

    // Subtitle
    let sub_y = y_offset + 7;
    if sub_y < area.y + area.height {
        let sub_w = unicode_width::UnicodeWidthStr::width(subtitle.as_str()) as u16;
        let sub_x = area.x + area.width.saturating_sub(sub_w) / 2;
        let r = Rect::new(sub_x, sub_y, sub_w.min(area.width), 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                subtitle,
                Style::default().fg(SUBTEXT0),
            ))),
            r,
        );
    }

    // Hint
    let hint_y = y_offset + 9;
    if hint_y < area.y + area.height {
        let hint_w = unicode_width::UnicodeWidthStr::width(hint.as_str()) as u16;
        let hint_x = area.x + area.width.saturating_sub(hint_w) / 2;
        let r = Rect::new(hint_x, hint_y, hint_w.min(area.width), 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                hint,
                Style::default().fg(OVERLAY0),
            ))),
            r,
        );
    }
}

fn render_card(frame: &mut Frame, app: &App, idx: usize, selected: bool, area: Rect) {
    let Some(s) = app.sessions.get(idx) else {
        return;
    };

    let id_short = &s.session_id[..s.session_id.len().min(6)];
    let (icon, icon_color) = status_display(s.status);
    let accent = if selected { BLUE } else { SURFACE1 };

    let title = Line::from(vec![
        Span::styled(format!(" {icon} "), Style::default().fg(icon_color)),
        Span::styled(
            format!("{} ", &s.provider),
            Style::default()
                .fg(if selected { LAVENDER } else { SUBTEXT0 })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("#{id_short} "),
            Style::default().fg(if selected { TEXT } else { OVERLAY0 }),
        ),
    ]);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent))
        .padding(Padding::new(1, 1, 1, 0))
        .style(Style::default().bg(BASE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Status badge — right-top corner, overlaid on the border
    render_status_badge(frame, s.status, area);

    if inner.height < 5 {
        return;
    }

    // 6 info rows (no spacers) + charts
    let info_height = 6;
    let chart_height = inner.height.saturating_sub(info_height as u16).min(10);
    let has_chart = chart_height >= 3;

    let mut constraints = vec![
        Constraint::Length(1), // row 0: model
        Constraint::Length(1), // row 1: context bar
        Constraint::Length(1), // row 2: in / out / cached
        Constraint::Length(1), // row 3: speed
        Constraint::Length(1), // row 4: cost
        Constraint::Length(1), // row 5: dir
    ];
    if has_chart {
        constraints.push(Constraint::Min(3)); // charts
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    render_info_rows(frame, app, s, &rows);

    if has_chart && rows.len() > 6 {
        let token_data = app
            .session_token_series
            .get(&s.session_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let cost_data = app
            .session_cost_series
            .get(&s.session_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        super::trend_chart::render_with_data(frame, token_data, cost_data, rows[6], true);
    }
}

fn render_info_rows(frame: &mut Frame, app: &App, s: &SessionSnapshot, rows: &[Rect]) {
    // Layout indices: 0=model, 1=ctx, 2=tokens, 3=speed, 4=cost, 5=dir
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
        &t!("label.dir"),
    ]);

    // Row 0: Model + subagent count
    let mut model_spans = vec![
        Span::styled(pad_r(&t!("label.model"), lw), lbl),
        Span::styled(
            s.model.clone(),
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
    let mut spans = vec![
        Span::styled(pad_r(&t!("label.cost"), lw), lbl),
        Span::styled(pad_r(&cost_str, 10), Style::default().fg(GREEN)),
    ];
    if s.cost_reported.is_some() {
        spans.push(sep);
        spans.push(Span::styled(pad_r(&t!("label.est"), lw), lbl));
        spans.push(Span::styled(
            format!("${est:.2}"),
            Style::default().fg(OVERLAY0),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), rows[4]);

    // Row 5: Dir
    if let Some(ref dir) = s.work_dir {
        let dir_label = t!("label.dir").to_string();
        let dir_label_w = unicode_width::UnicodeWidthStr::width(pad_r(&dir_label, lw).as_str());
        let avail = (rows[5].width as usize).saturating_sub(dir_label_w);
        let dir_display = if dir.len() <= avail || avail < 4 {
            dir.clone()
        } else {
            format!("…{}", &dir[dir.len() - avail + 1..])
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(pad_r(&dir_label, lw), lbl),
                Span::styled(dir_display, Style::default().fg(SUBTEXT0)),
            ])),
            rows[5],
        );
    }
}

fn build_context_line(s: &SessionSnapshot, width: u16, lw: usize) -> Vec<Span<'static>> {
    let mut spans = vec![Span::styled(
        pad_r(&t!("label.ctx"), lw),
        Style::default().fg(OVERLAY0),
    )];

    match (s.context_tokens, s.context_max, s.context_window_pct) {
        (Some(used), Some(max), Some(pct)) => {
            let bar_width = (width as usize).saturating_sub(40).clamp(8, 24);
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
            let bar_width = 16_usize;
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

fn status_display(status: SessionStatus) -> (&'static str, Color) {
    match status {
        SessionStatus::Active => ("●", GREEN),
        SessionStatus::Idle => ("◑", YELLOW),
        SessionStatus::Done => ("✓", OVERLAY0),
        SessionStatus::Disconnected => ("✗", RED),
    }
}
