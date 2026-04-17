pub mod alert_bar;
pub mod detail_panel;
pub mod header;
pub mod help;
pub mod history;
pub mod layout;
pub mod overview;
pub mod session_tab;
pub mod shared;
pub mod theme;
pub mod trend_chart;

pub use layout::render;

use unicode_width::UnicodeWidthStr;

/// Right-pad a string to exactly `width` display columns.
/// Respects CJK double-width characters.
/// Truncates with '…' if the string exceeds `width`.
pub fn pad_r(s: &str, width: usize) -> String {
    let display_w = UnicodeWidthStr::width(s);
    if display_w >= width {
        truncate_to_width(s, width)
    } else {
        let padding = width - display_w;
        format!("{s}{}", " ".repeat(padding))
    }
}

/// Truncate a string to fit within `max_width` display columns.
fn truncate_to_width(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let mut current_w = 0;
    let mut result = String::new();
    for ch in s.chars() {
        let ch_w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_w + ch_w > max_width {
            break;
        }
        result.push(ch);
        current_w += ch_w;
    }
    let remaining = max_width - current_w;
    result.extend(std::iter::repeat_n(' ', remaining));
    result
}
