use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use rust_i18n::t;

/// Render the help popup overlay.
pub fn render(frame: &mut Frame, area: Rect) {
    let help_width = 52.min(area.width.saturating_sub(4));
    let help_height = 16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(help_width)) / 2;
    let y = (area.height.saturating_sub(help_height)) / 2;
    let popup_area = Rect::new(x, y, help_width, help_height);

    frame.render_widget(Clear, popup_area);

    let lines = vec![
        Line::from(Span::styled(
            format!(" {}", t!("help.subtitle")),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" 1-9      ", Style::default().fg(Color::Yellow)),
            Span::raw(t!("help.key_jump_tab").to_string()),
        ]),
        Line::from(vec![
            Span::styled(" Tab      ", Style::default().fg(Color::Yellow)),
            Span::raw(t!("help.key_next_tab").to_string()),
        ]),
        Line::from(vec![
            Span::styled(" S-Tab    ", Style::default().fg(Color::Yellow)),
            Span::raw(t!("help.key_prev_tab").to_string()),
        ]),
        Line::from(vec![
            Span::styled(" j/k ↑/↓  ", Style::default().fg(Color::Yellow)),
            Span::raw(t!("help.key_move_ud").to_string()),
        ]),
        Line::from(vec![
            Span::styled(" h/l ←/→  ", Style::default().fg(Color::Yellow)),
            Span::raw(t!("help.key_move_lr").to_string()),
        ]),
        Line::from(vec![
            Span::styled(" Enter    ", Style::default().fg(Color::Yellow)),
            Span::raw(t!("help.key_open_session").to_string()),
        ]),
        Line::from(vec![
            Span::styled(" Esc      ", Style::default().fg(Color::Yellow)),
            Span::raw(t!("help.key_back").to_string()),
        ]),
        Line::from(vec![
            Span::styled(" ?        ", Style::default().fg(Color::Yellow)),
            Span::raw(t!("help.key_toggle_help").to_string()),
        ]),
        Line::from(vec![
            Span::styled(" q Ctrl+C ", Style::default().fg(Color::Yellow)),
            Span::raw(t!("help.key_quit").to_string()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!(" {}", t!("help.close_hint")),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(format!(" {} ", t!("help.title")))
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
