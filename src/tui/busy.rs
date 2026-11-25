//! Family working loader used by dd TUI apps: ▃ d_d ▃ → ▅ d_d ▅ → █ d_d █.

use ratatui::style::{Modifier, Style};
use ratatui::text::Span;

use super::theme::AppTheme;

const BUSY_BARS: [char; 4] = ['▃', '▅', '█', '▅'];
pub(super) const BUSY_FRAME_MS: u128 = 150;

pub(super) fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub(super) fn bar(now_ms: u128) -> char {
    BUSY_BARS[(now_ms / BUSY_FRAME_MS) as usize % BUSY_BARS.len()]
}

#[cfg(test)]
pub(super) fn label(now_ms: u128) -> String {
    let b = bar(now_ms);
    format!("{b} d_d {b}")
}

pub(super) fn spans(now_ms: u128, theme: &AppTheme) -> Vec<Span<'static>> {
    let b = bar(now_ms).to_string();
    vec![
        Span::styled(b.clone(), Style::default().fg(theme.info)),
        Span::styled(
            " d_d ",
            Style::default()
                .fg(theme.text_active_focus)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(b, Style::default().fg(theme.info)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulses_through_the_four_frames() {
        assert_eq!(label(0), "▃ d_d ▃");
        assert_eq!(label(149), "▃ d_d ▃");
        assert_eq!(label(150), "▅ d_d ▅");
        assert_eq!(label(300), "█ d_d █");
        assert_eq!(label(450), "▅ d_d ▅");
        assert_eq!(label(600), "▃ d_d ▃");
    }
}
