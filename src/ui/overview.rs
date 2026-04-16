use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, GraphType, Padding, Paragraph},
};

use crate::app::App;
use crate::model::{SessionSnapshot, SessionStatus};

// --- Catppuccin Mocha palette ---
const BASE: Color = Color::Rgb(30, 30, 46);
const SURFACE0: Color = Color::Rgb(49, 50, 68);
const SURFACE1: Color = Color::Rgb(69, 71, 90);
const OVERLAY0: Color = Color::Rgb(108, 112, 134);
const SUBTEXT0: Color = Color::Rgb(166, 173, 200);
const TEXT: Color = Color::Rgb(205, 214, 244);
const BLUE: Color = Color::Rgb(137, 180, 250);
const TEAL: Color = Color::Rgb(148, 226, 213);
const GREEN: Color = Color::Rgb(166, 227, 161);
const YELLOW: Color = Color::Rgb(249, 226, 175);
const PEACH: Color = Color::Rgb(250, 179, 135);
const RED: Color = Color::Rgb(243, 139, 168);
const MAUVE: Color = Color::Rgb(203, 166, 247);
const LAVENDER: Color = Color::Rgb(180, 190, 254);
const SKY: Color = Color::Rgb(137, 220, 235);

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

    let card_height: u16 = 20;
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
            let hint = format!(" ▲ {} more above ", app.overview_scroll);
            let hw = hint.len() as u16;
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
                " ▼ {} more below  {}/{} ",
                remaining,
                app.overview_scroll + 1,
                rows_needed,
            );
            let hw = hint.len() as u16;
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

    let subtitle = "Token Monitor for AI Coding Tools";
    let hint = "Waiting for sessions...";

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
        let sub_w = subtitle.len() as u16;
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
        let hint_w = hint.len() as u16;
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

    // 5 info rows (no spacers) + charts
    let info_height = 5;
    let chart_height = inner.height.saturating_sub(info_height as u16).min(10);
    let has_chart = chart_height >= 3;

    let mut constraints = vec![
        Constraint::Length(1), // row 0: model
        Constraint::Length(1), // row 1: context bar
        Constraint::Length(1), // row 2: in / out / cached
        Constraint::Length(1), // row 3: speed
        Constraint::Length(1), // row 4: cost
    ];
    if has_chart {
        constraints.push(Constraint::Min(3)); // charts
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    render_info_rows(frame, app, s, &rows);

    if has_chart && rows.len() > 5 {
        render_card_charts(frame, app, &s.session_id, rows[5]);
    }
}

fn render_info_rows(frame: &mut Frame, app: &App, s: &SessionSnapshot, rows: &[Rect]) {
    // Layout indices: 0=model, 1=ctx, 2=tokens, 3=speed, 4=cost
    let lbl = Style::default().fg(OVERLAY0);
    let sep = Span::styled(" · ", Style::default().fg(SURFACE1));

    // Row 0: Model
    let model_name = display_model(&s.model);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(pad_r("Model:", 8), lbl),
            Span::styled(
                model_name,
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
    let mut spans = vec![
        Span::styled(pad_r("Cost:", 8), lbl),
        Span::styled(pad_r(&cost_str, 10), Style::default().fg(GREEN)),
    ];
    if s.cost_reported.is_some() {
        spans.push(sep);
        spans.push(Span::styled(pad_r("Est:", 8), lbl));
        spans.push(Span::styled(
            format!("${est:.2}"),
            Style::default().fg(OVERLAY0),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), rows[4]);
}

/// Right-pad a string to exactly `width` chars.
fn pad_r(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        format!("{s:<width$}")
    }
}

fn build_context_line(s: &SessionSnapshot, width: u16) -> Vec<Span<'static>> {
    let mut spans = vec![Span::styled(
        pad_r("Ctx:", 8),
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

    // Append dir at end of ctx line
    if let Some(ref dir) = s.work_dir {
        let sep = Span::styled("  ·  ", Style::default().fg(SURFACE1));
        spans.push(sep);
        spans.push(Span::styled("Dir: ", Style::default().fg(OVERLAY0)));
        // Truncate front if too long, keeping the meaningful tail
        let used: usize = spans.iter().map(|s| s.width()).sum();
        let avail = (width as usize).saturating_sub(used);
        let dir_display = if dir.len() <= avail || avail < 4 {
            dir.clone()
        } else {
            format!("…{}", &dir[dir.len() - avail + 1..])
        };
        spans.push(Span::styled(dir_display, Style::default().fg(SUBTEXT0)));
    }

    spans
}

fn render_card_charts(frame: &mut Frame, app: &App, session_id: &str, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

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

    render_mini_chart(frame, token_data, "tokens", SKY, chunks[0]);
    render_mini_chart(frame, cost_data, "cost", GREEN, chunks[1]);
}

fn render_mini_chart(
    frame: &mut Frame,
    data: &[(f64, f64)],
    label: &str,
    color: Color,
    area: Rect,
) {
    let block = Block::default()
        .title(Line::styled(
            format!(" {label} "),
            Style::default().fg(OVERLAY0),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(SURFACE0))
        .style(Style::default().bg(BASE));

    if data.is_empty() || area.height < 3 {
        frame.render_widget(block, area);
        return;
    }

    let x_min = data.first().map(|&(x, _)| x).unwrap_or(0.0);
    let x_max = data.last().map(|&(x, _)| x).unwrap_or(1.0).max(x_min + 1.0);
    let y_max = data
        .iter()
        .map(|&(_, y)| y)
        .fold(0.0_f64, f64::max)
        .max(0.001);

    let datasets = vec![
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(color))
            .data(data),
    ];

    // Y-axis labels
    let y_labels: Vec<Line> = vec![
        Line::styled("0", Style::default().fg(SURFACE1)),
        Line::styled(fmt_axis(y_max), Style::default().fg(OVERLAY0)),
    ];

    // X-axis labels: just show data point count
    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(
            Axis::default()
                .bounds([x_min, x_max])
                .style(Style::default().fg(SURFACE0)),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, y_max * 1.1])
                .labels(y_labels)
                .style(Style::default().fg(SURFACE0)),
        );

    frame.render_widget(chart, area);
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

fn fmt_axis(n: f64) -> String {
    if n >= 1_000_000.0 {
        format!("{:.0}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.0}k", n / 1_000.0)
    } else if n < 1.0 {
        format!("{n:.2}")
    } else {
        format!("{n:.0}")
    }
}

fn display_model(model: &str) -> String {
    model.to_string()
}

fn status_display(status: SessionStatus) -> (&'static str, Color) {
    match status {
        SessionStatus::Active => ("●", GREEN),
        SessionStatus::Idle => ("◑", YELLOW),
        SessionStatus::Done => ("✓", OVERLAY0),
        SessionStatus::Disconnected => ("✗", RED),
    }
}

/// Render a rounded pill status badge at the top-right corner.
/// Uses half-block chars (▐ ▌) to simulate radius on standard terminals.
fn render_status_badge(frame: &mut Frame, status: SessionStatus, card_area: Rect) {
    let (dot_color, label_color, label, pill_bg) = match status {
        SessionStatus::Active => (
            Color::Rgb(116, 199, 136), // bright dot
            Color::Rgb(186, 230, 190), // bright text
            "Active",
            Color::Rgb(28, 48, 36), // deep green
        ),
        SessionStatus::Idle => (
            Color::Rgb(229, 200, 120),
            Color::Rgb(240, 220, 170),
            "Idle",
            Color::Rgb(48, 42, 24), // deep amber
        ),
        SessionStatus::Done => (
            Color::Rgb(120, 200, 190),
            Color::Rgb(170, 210, 206),
            "Done",
            Color::Rgb(30, 44, 46), // deep teal
        ),
        SessionStatus::Disconnected => (
            Color::Rgb(220, 110, 130),
            Color::Rgb(240, 160, 170),
            "Offline",
            Color::Rgb(52, 26, 32), // deep rose
        ),
    };

    // ▐(fg=pill, bg=base) + content(bg=pill) + ▌(fg=pill, bg=base)
    let pill_spans = vec![
        Span::styled("\u{2590}", Style::default().fg(pill_bg).bg(BASE)), // ▐ left cap
        Span::styled(" ● ", Style::default().fg(dot_color).bg(pill_bg)),
        Span::styled(
            format!("{label} "),
            Style::default().fg(label_color).bg(pill_bg),
        ),
        Span::styled("\u{258C}", Style::default().fg(pill_bg).bg(BASE)), // ▌ right cap
    ];

    let badge_width: u16 = pill_spans.iter().map(|s| s.width() as u16).sum();

    if badge_width + 3 > card_area.width {
        return;
    }

    let x = card_area.x + card_area.width - badge_width - 1;
    let y = card_area.y;
    let badge_area = Rect::new(x, y, badge_width, 1);

    frame.render_widget(Paragraph::new(Line::from(pill_spans)), badge_area);
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
