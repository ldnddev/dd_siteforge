//! Shortcut sheet. Both columns are generated from the core chord table.

pub use dd_siteforge_core::commands::SheetRow as Shortcut;

pub fn rows() -> &'static [Shortcut] {
    dd_siteforge_core::commands::shortcut_sheet()
}

pub fn lines() -> Vec<String> {
    let mut out = vec![
        "Shortcut sheet".to_string(),
        "Ctrl+R in the main window is redo. F5 refreshes the preview. Ctrl+R in the preview companion refreshes the preview.".to_string(),
    ];
    let mut section = "";
    for row in rows() {
        if row.section != section {
            section = row.section;
            out.push(section.to_string());
        }
        out.push(format!("{} · {} · {}", row.action, row.gui, row.tui));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheet_uses_the_decided_keys() {
        let text = lines().join("\n");
        assert!(text.contains("Add page · Ctrl N · Ctrl N"));
        assert!(text.contains("New site · Ctrl Shift N · Ctrl Shift N"));
        assert!(text.contains("Theme · F2 · F2"));
        assert!(text.contains("Redo · Ctrl R · Ctrl R"));
        assert!(text.contains("Refresh preview · F5 · F5"));
        assert!(text.contains("Find in site · Ctrl F"));
        assert!(!text.contains("J2"));
        assert!(!text.contains("J3"));
        assert!(!text.contains("J5"));
    }
}
