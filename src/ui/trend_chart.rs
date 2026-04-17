use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    symbols,
    text::Line,
    widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, GraphType},
};
use rust_i18n::t;

use super::theme::*;

/// Render a pair of token + cost charts stacked vertically.
/// `compact` = true uses 2 Y-axis labels (for overview cards), false uses 3 (for detail).
pub fn render_with_data(
    frame: &mut Frame,
    token_data: &[(f64, f64)],
    cost_data: &[(f64, f64)],
    area: Rect,
    compact: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let y_max_tok = y_max_of(token_data);
    let y_max_cost = y_max_of(cost_data);

    // Compute unified Y-axis label width so both charts align
    let label_w = if compact {
        format_y(y_max_tok)
            .len()
            .max(format_y(y_max_cost).len())
            .max(1)
    } else {
        [
            format_y(y_max_tok).len(),
            format_y(y_max_tok / 2.0).len(),
            format_y(y_max_cost).len(),
            format_y(y_max_cost / 2.0).len(),
            1,
        ]
        .into_iter()
        .max()
        .unwrap()
    };

    let tok_title = format!(" {} ", t!("detail.chart_tokens"));
    let cost_title = format!(" {} ", t!("detail.chart_cost"));

    render_chart(
        frame, token_data, &tok_title, SKY, chunks[0], label_w, compact,
    );
    render_chart(
        frame,
        cost_data,
        &cost_title,
        GREEN,
        chunks[1],
        label_w,
        compact,
    );
}

fn render_chart(
    frame: &mut Frame,
    data: &[(f64, f64)],
    title: &str,
    color: Color,
    area: Rect,
    y_label_width: usize,
    compact: bool,
) {
    let block = Block::default()
        .title(Line::styled(title, Style::default().fg(OVERLAY0)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(SURFACE1))
        .style(Style::default().bg(BASE));

    if data.is_empty() || area.height < 3 {
        frame.render_widget(block, area);
        return;
    }

    let x_min = data.first().map(|&(x, _)| x).unwrap_or(0.0);
    let x_max = data.last().map(|&(x, _)| x).unwrap_or(1.0).max(x_min + 1.0);
    let y_max = y_max_of(data);

    let datasets = vec![
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(color))
            .data(data),
    ];

    let w = y_label_width;
    let y_labels: Vec<Line> = if compact {
        vec![
            Line::styled(format!("{:>w$}", "0"), Style::default().fg(SURFACE1)),
            Line::styled(
                format!("{:>w$}", format_y(y_max)),
                Style::default().fg(OVERLAY0),
            ),
        ]
    } else {
        vec![
            Line::styled(format!("{:>w$}", "0"), Style::default().fg(SURFACE1)),
            Line::styled(
                format!("{:>w$}", format_y(y_max / 2.0)),
                Style::default().fg(OVERLAY0),
            ),
            Line::styled(
                format!("{:>w$}", format_y(y_max)),
                Style::default().fg(TEXT),
            ),
        ]
    };

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

fn y_max_of(data: &[(f64, f64)]) -> f64 {
    data.iter()
        .map(|&(_, y)| y)
        .fold(0.0_f64, f64::max)
        .max(0.001)
}

fn format_y(n: f64) -> String {
    if n >= 1_000_000.0 {
        format!("{:.1}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.1}k", n / 1_000.0)
    } else if n >= 0.01 {
        format!("${n:.2}")
    } else {
        format!("{n:.0}")
    }
}
