use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::Line,
    widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, GraphType},
};

// Catppuccin Mocha
const BASE: Color = Color::Rgb(30, 30, 46);
const SURFACE0: Color = Color::Rgb(49, 50, 68);
const SURFACE1: Color = Color::Rgb(69, 71, 90);
const OVERLAY0: Color = Color::Rgb(108, 112, 134);
const SKY: Color = Color::Rgb(137, 220, 235);
const GREEN: Color = Color::Rgb(166, 227, 161);
const TEXT: Color = Color::Rgb(205, 214, 244);

/// Render token rate + cost charts from given data slices.
pub fn render_with_data(
    frame: &mut Frame,
    token_data: &[(f64, f64)],
    cost_data: &[(f64, f64)],
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_chart(frame, token_data, " Token Rate ", SKY, chunks[0]);
    render_chart(frame, cost_data, " Cumulative Cost ", GREEN, chunks[1]);
}

fn render_chart(frame: &mut Frame, data: &[(f64, f64)], title: &str, color: Color, area: Rect) {
    let block = Block::default()
        .title(Line::styled(title, Style::default().fg(OVERLAY0)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(SURFACE1))
        .style(Style::default().bg(BASE));

    if data.is_empty() {
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

    let y_labels: Vec<Line> = vec![
        Line::styled("0", Style::default().fg(SURFACE1)),
        Line::styled(format_y(y_max / 2.0), Style::default().fg(OVERLAY0)),
        Line::styled(format_y(y_max), Style::default().fg(TEXT)),
    ];

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
