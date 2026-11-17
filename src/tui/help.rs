use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

use super::theme::{AppTheme, color_to_hex};

fn wrap_to_lines(text: &str, width: usize) -> Vec<String> {
    let w = width.max(1);
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut out: Vec<String> = vec![];
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
            if current.chars().count() > w {
                // hard-break very long token
                let chars: Vec<char> = current.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    let end = (i + w).min(chars.len());
                    out.push(chars[i..end].iter().collect());
                    i = end;
                }
                current.clear();
            }
            continue;
        }
        let with_space = format!("{} {}", current, word);
        if with_space.chars().count() <= w {
            current = with_space;
        } else {
            if !current.is_empty() {
                out.push(current);
            }
            current = word.to_string();
            if current.chars().count() > w {
                let chars: Vec<char> = current.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    let end = (i + w).min(chars.len());
                    out.push(chars[i..end].iter().collect());
                    i = end;
                }
                current.clear();
            }
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

pub(crate) fn build_help_text(theme: &AppTheme, width: usize, in_tmux: bool) -> Text<'static> {
    let h_style = Style::default()
        .fg(theme.modal_header)
        .add_modifier(Modifier::BOLD);
    let k_style = Style::default().fg(theme.text_active_focus);
    let div_style = Style::default().fg(theme.text_secondary);

    const KEY_COL: usize = 22;

    fn add_section(
        lines: &mut Vec<Line<'static>>,
        title: &str,
        items: &[(&str, &str)],
        icon: &str,
        h_style: Style,
        k_style: Style,
        div_style: Style,
        width: usize,
    ) {
        lines.push(Line::from(Span::styled(title.to_string(), h_style)));
        lines.push(Line::from("")); // padding top inside section
        for (k, a) in items {
            let prefix = format!("  {} {:<18}", icon, k);
            let avail = width.saturating_sub(KEY_COL);
            let chunks = wrap_to_lines(a, avail);
            if chunks.is_empty() || (chunks.len() == 1 && chunks[0].is_empty()) {
                lines.push(Line::from(Span::styled(prefix, k_style)));
            } else {
                for (i, chunk) in chunks.iter().enumerate() {
                    if i == 0 {
                        lines.push(Line::from(vec![
                            Span::styled(prefix.clone(), k_style),
                            Span::raw(chunk.clone()),
                        ]));
                    } else {
                        let cont = format!("{}{}", " ".repeat(KEY_COL), chunk);
                        lines.push(Line::from(Span::raw(cont)));
                    }
                }
            }
        }
        lines.push(Line::from("")); // padding bottom inside section

        // visible divider between sections (in muted)
        let rule_len = width.saturating_sub(4).clamp(12, 50);
        let rule = "─".repeat(rule_len);
        lines.push(Line::from(Span::styled(format!("  {}", rule), div_style)));
        lines.push(Line::from("")); // breathing room after divider
    }

    let mut lines: Vec<Line<'static>> = Vec::new();

    let mut sections: Vec<(&str, Vec<(&str, &str)>)> = Vec::new();
    for row in dd_siteforge_core::commands::shortcut_sheet() {
        if sections.last().map(|(name, _)| *name) != Some(row.section) {
            sections.push((row.section, Vec::new()));
        }
        sections.last_mut().expect("section").1.push((
            dd_siteforge_core::commands::tui_help_key(row.tui, in_tmux),
            row.action,
        ));
    }
    for (title, items) in &sections {
        add_section(
            &mut lines, title, items, "•", h_style, k_style, div_style, width,
        );
    }

    // Autosave is not a chord. The .backup is the manual checkpoint (Ctrl+S).
    lines.push(Line::from(Span::styled("Autosave", h_style)));
    lines.push(Line::from(""));
    let note = "2s after a change, the active site JSON is rewritten. The .backup is only refreshed by a manual Ctrl+S checkpoint; on next load the TUI surfaces a toast when site.json and site.json.backup differ.";
    for chunk in wrap_to_lines(note, width.saturating_sub(2)) {
        lines.push(Line::from(Span::raw(format!("  {}", chunk))));
    }
    lines.push(Line::from(""));
    let rule_len = width.saturating_sub(4).clamp(12, 50);
    let rule = "─".repeat(rule_len);
    lines.push(Line::from(Span::styled(format!("  {}", rule), div_style)));
    lines.push(Line::from(""));

    // Mouse (using safe bullet icon)
    add_section(
        &mut lines,
        "Mouse controls",
        &[
            (
                "Click panel/list",
                "Select the row/item (Regions/Pages/Layout/Details); Details click focuses and selects the matching tree grain",
            ),
            (
                "Double-click item",
                "Edit (unified modal; works on page-head, header/footer roots, sections, columns, components)",
            ),
            (
                "Click modal field",
                "Focus that input (click-to-focus in all FormEdit + legacy)",
            ),
            (
                "Click text field",
                "Place the caret at the click (single-line, compact textarea, or expanded)",
            ),
            (
                "Drag in text field",
                "Select a range; Shift+click extends from the caret",
            ),
            (
                "Double-click in text",
                "Select the word (or whitespace run) under the pointer",
            ),
            (
                "Wheel over pane",
                "Scroll the pane under the cursor (Regions/Pages/Layout/Details)",
            ),
            (
                "Scrollbar track/thumb",
                "Jump/scroll via custom painted │/█ scrollbar",
            ),
            (
                "Wheel / drag scroll",
                "Scroll lists, Details, help modal, long form content",
            ),
        ],
        "•",
        h_style,
        k_style,
        div_style,
        width,
    );

    Text::from(lines)
}

pub(crate) fn build_theme_text(
    theme: &AppTheme,
    source: &str,
    status: &Option<String>,
    width: usize,
) -> Text<'static> {
    let h_style = Style::default()
        .fg(theme.modal_header)
        .add_modifier(Modifier::BOLD);
    let k_style = Style::default().fg(theme.text_active_focus);
    let div_style = Style::default().fg(theme.text_secondary);

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Theme section
    lines.push(Line::from(Span::styled("Theme", h_style)));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  App: ", k_style),
        Span::raw(format!("dd_siteforge v{}", env!("CARGO_PKG_VERSION"))),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Source: ", k_style),
        Span::raw(format!(
            "{}   (./dd_siteforge_theme.yml or equivalent)",
            source
        )),
    ]));
    let status_str = status.as_deref().unwrap_or("OK (loaded cleanly)");
    lines.push(Line::from(vec![
        Span::styled("  Status: ", k_style),
        Span::raw(status_str.to_string()),
    ]));
    lines.push(Line::from(""));

    // divider
    let rule_len = width.saturating_sub(4).clamp(12, 50);
    let rule = "─".repeat(rule_len);
    lines.push(Line::from(Span::styled(format!("  {}", rule), div_style)));
    lines.push(Line::from(""));

    // Color tokens section
    lines.push(Line::from(Span::styled(
        "Loaded color tokens (sampled)",
        h_style,
    )));
    lines.push(Line::from(""));

    let tokens: Vec<(&str, Color, &str)> = vec![
        ("base_background", theme.base_background, "app_shell base"),
        ("body_background", theme.body_background, "content panes"),
        (
            "modal_background",
            theme.modal_background,
            "modals & popups",
        ),
        ("text_primary", theme.text_primary, "primary text"),
        ("modal_header", theme.modal_header, "section titles bold"),
        ("text_labels", theme.text_labels, "labels default"),
        ("text_active_focus", theme.text_active_focus, "focus + keys"),
        (
            "input_border_focus",
            theme.input_border_focus,
            "focused inputs",
        ),
        ("cursor", theme.cursor, "caret overlay"),
        ("selection", theme.selection, "text field highlight"),
        ("success", theme.success, "success toasts"),
        ("warning", theme.warning, "warning toasts"),
        ("error", theme.error, "error toasts"),
        ("info", theme.info, "info toasts"),
        ("folders", theme.folders, "image picker folders"),
        ("files", theme.files, "image picker files"),
        ("links", theme.links, "image picker links"),
        ("scrollbar", theme.scrollbar, "scrollbars"),
        ("scrollbar_hover", theme.scrollbar_hover, "scrollbar thumb"),
    ];

    for (name, color, role) in tokens {
        let hex = color_to_hex(color);
        let line = format!("  {:<18} {}   ({})", name, hex, role);
        lines.push(Line::from(Span::raw(line)));
    }

    lines.push(Line::from(""));

    // final divider
    lines.push(Line::from(Span::styled(format!("  {}", rule), div_style)));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::raw(
        "  (All colors from self.theme.*. No hardcodes.)",
    )));

    Text::from(lines)
}

pub(crate) fn count_lines(text: &Text, _width: usize) -> usize {
    text.lines.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(in_tmux: bool) -> String {
        let text = build_help_text(&AppTheme::default(), 80, in_tmux);
        text.lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn f1_outside_tmux_keeps_ctrl_b_and_ctrl_shift_n() {
        let text = plain(false);
        assert!(text.contains("Ctrl B"), "{text}");
        assert!(text.contains("Ctrl Shift N"), "{text}");
        assert!(!text.contains("Shift+B"), "{text}");
    }

    #[test]
    fn f1_inside_tmux_names_shift_b_and_the_palette() {
        let text = plain(true);
        assert!(text.contains("Shift+B"), "{text}");
        assert!(text.contains("palette"), "{text}");
        assert!(!text.contains("Ctrl B"), "{text}");
        assert!(!text.contains("Ctrl Shift N"), "{text}");
        assert!(text.contains("Ctrl E"), "{text}");
        assert!(text.contains("Ctrl R"), "{text}");
    }
}
