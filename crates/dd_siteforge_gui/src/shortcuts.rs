//! Shortcut sheet. GUI key first, TUI key second. Decided keys from Nov 16
//! (Ctrl+N, Ctrl+Shift+N, F2, Ctrl+R, F5) replace the sheet's old question tags.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub section: &'static str,
    pub action: &'static str,
    pub gui: &'static str,
    pub tui: &'static str,
}

pub fn rows() -> &'static [Shortcut] {
    &[
        Shortcut {
            section: "Global",
            action: "Help",
            gui: "F1",
            tui: "F1",
        },
        Shortcut {
            section: "Global",
            action: "Theme",
            gui: "F2",
            tui: "F2",
        },
        Shortcut {
            section: "Global",
            action: "Validate site",
            gui: "F3",
            tui: "F3",
        },
        Shortcut {
            section: "Global",
            action: "Page health",
            gui: "F4",
            tui: "F4",
        },
        Shortcut {
            section: "Global",
            action: "Export",
            gui: "Ctrl E",
            tui: "Shift+E",
        },
        Shortcut {
            section: "Global",
            action: "Preview",
            gui: "Ctrl P",
            tui: "Shift+P",
        },
        Shortcut {
            section: "Global",
            action: "Refresh preview",
            gui: "F5",
            tui: "—",
        },
        Shortcut {
            section: "Global",
            action: "Build CSS/JS",
            gui: "Ctrl B",
            tui: "Shift+B",
        },
        Shortcut {
            section: "Global",
            action: "Find in site",
            gui: "Ctrl F",
            tui: "? / Ctrl+F",
        },
        Shortcut {
            section: "Global",
            action: "Command palette",
            gui: "Ctrl K",
            tui: ": / Ctrl+K",
        },
        Shortcut {
            section: "Global",
            action: "Redo",
            gui: "Ctrl R",
            tui: "Ctrl+R",
        },
        Shortcut {
            section: "Global",
            action: "Checkpoint (site.json + .backup)",
            gui: "Ctrl S",
            tui: "s",
        },
        Shortcut {
            section: "Global",
            action: "Quit",
            gui: "Ctrl Q",
            tui: "Ctrl+Q",
        },
        Shortcut {
            section: "Global",
            action: "New site",
            gui: "Ctrl Shift N",
            tui: "—",
        },
        Shortcut {
            section: "Outline",
            action: "Move",
            gui: "↑↓ or j k",
            tui: "Up/Down or j/k",
        },
        Shortcut {
            section: "Outline",
            action: "First / last",
            gui: "g G",
            tui: "g G",
        },
        Shortcut {
            section: "Outline",
            action: "Collapse / expand",
            gui: "h l",
            tui: "Space / h / l",
        },
        Shortcut {
            section: "Outline",
            action: "Insert",
            gui: "/",
            tui: "/",
        },
        Shortcut {
            section: "Outline",
            action: "Delete",
            gui: "Backspace",
            tui: "d",
        },
        Shortcut {
            section: "Outline",
            action: "Copy",
            gui: "y",
            tui: "y",
        },
        Shortcut {
            section: "Outline",
            action: "Paste",
            gui: "p",
            tui: "p",
        },
        Shortcut {
            section: "Outline",
            action: "Undo",
            gui: "Ctrl Z",
            tui: "u",
        },
        Shortcut {
            section: "Outline",
            action: "Move row",
            gui: "Shift J / Shift K",
            tui: "J / K",
        },
        Shortcut {
            section: "Sidebar · pages",
            action: "Add page",
            gui: "Ctrl N",
            tui: "Shift+A",
        },
        Shortcut {
            section: "Sidebar · pages",
            action: "New site",
            gui: "Ctrl Shift N",
            tui: "—",
        },
        Shortcut {
            section: "Sidebar · pages",
            action: "Rename page",
            gui: "r",
            tui: "r",
        },
        Shortcut {
            section: "Inspector",
            action: "Next / previous field",
            gui: "Tab / Shift Tab",
            tui: "Tab / Shift+Tab",
        },
        Shortcut {
            section: "Inspector",
            action: "Checkpoint (site.json + .backup)",
            gui: "Ctrl S",
            tui: "Ctrl+S",
        },
        Shortcut {
            section: "Inspector",
            action: "Cancel",
            gui: "Esc",
            tui: "Esc",
        },
    ]
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
        assert!(text.contains("Add page · Ctrl N · Shift+A"));
        assert!(text.contains("New site · Ctrl Shift N · —"));
        assert!(text.contains("Theme · F2 · F2"));
        assert!(text.contains("Redo · Ctrl R · Ctrl+R"));
        assert!(text.contains("Refresh preview · F5 · —"));
        assert!(text.contains("Find in site · Ctrl F"));
        assert!(!text.contains("J2"));
        assert!(!text.contains("J3"));
        assert!(!text.contains("J5"));
    }
}
