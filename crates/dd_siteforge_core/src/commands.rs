//! Command registry shared by frontends.
//!
//! The chord table is the GUI's bindings. It does not list TUI keys (`:`,
//! `?`, `s`, Shift+E, and the rest) or Vim letters (`j`, `k`, `g`, `h`, `l`).
//! Those stay with the frontend that owns them. The TUI does not read this
//! module.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    Export,
    Preview,
    RefreshPreview,
    Validate,
    Save,
    Help,
    Theme,
    Find,
    Insert,
    AddPage,
    BuildAssets,
    PageHealth,
    Redo,
    Quit,
    InitKit,
    InitTemplates,
    NewSite,
    ToggleSidebar,
    ToggleInspector,
    Palette,
    Open,
    ToggleIssues,
    FocusSidebar,
    FocusOutline,
}

/// One key chord, without a frontend's string spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chord<'a> {
    pub key: &'a str,
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
}

impl<'a> Chord<'a> {
    pub const fn plain(key: &'a str) -> Self {
        Self {
            key,
            control: false,
            alt: false,
            shift: false,
        }
    }

    pub const fn ctrl(key: &'a str) -> Self {
        Self {
            key,
            control: true,
            alt: false,
            shift: false,
        }
    }

    pub const fn ctrl_shift(key: &'a str) -> Self {
        Self {
            key,
            control: true,
            alt: false,
            shift: true,
        }
    }

    pub const fn ctrl_alt(key: &'a str) -> Self {
        Self {
            key,
            control: true,
            alt: true,
            shift: false,
        }
    }

    /// Parse a GUI key token (`ctrl-shift-n`, `f2`, `ctrl-\`).
    pub fn parse(token: &'a str) -> Option<Self> {
        let token = token.trim();
        if token.is_empty() {
            return None;
        }
        let mut parts: Vec<&str> = token.split('-').collect();
        let key = parts.pop()?;
        if key.is_empty() {
            return None;
        }
        let mut control = false;
        let mut alt = false;
        let mut shift = false;
        for part in parts {
            match part {
                "ctrl" => control = true,
                "alt" => alt = true,
                "shift" => shift = true,
                _ => return None,
            }
        }
        Some(Self {
            key,
            control,
            alt,
            shift,
        })
    }
}

/// GUI token for a chord, in the same order the shell builds key events.
pub fn chord_token(chord: Chord<'_>) -> String {
    let mut parts = Vec::new();
    if chord.control {
        parts.push("ctrl");
    }
    if chord.shift {
        parts.push("shift");
    }
    if chord.alt {
        parts.push("alt");
    }
    parts.push(chord.key);
    parts.join("-")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandSpec {
    pub command: Command,
    pub title: &'static str,
    pub detail: &'static str,
    /// Short label drawn on a palette row.
    pub kbd: &'static str,
    pub chords: &'static [Chord<'static>],
    /// False for chords that are not palette rows.
    pub in_palette: bool,
}

const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        command: Command::Export,
        title: "Export HTML",
        detail: "Validate, then write the site",
        kbd: "Ctrl E",
        chords: &[Chord::ctrl("e")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::Preview,
        title: "Preview in browser",
        detail: "Open the preview companion and the browser",
        kbd: "Ctrl P",
        chords: &[Chord::ctrl("p")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::RefreshPreview,
        title: "Refresh preview",
        detail: "Reload the preview from the main window",
        kbd: "F5",
        chords: &[Chord::plain("f5")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::Validate,
        title: "Validate site",
        detail: "Open the Issues list",
        kbd: "F3",
        chords: &[Chord::plain("f3")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::Save,
        title: "Save",
        detail: "Checkpoint site.json and the .backup",
        kbd: "Ctrl S",
        chords: &[Chord::ctrl("s")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::Help,
        title: "Help",
        detail: "Shortcut sheet",
        kbd: "F1",
        chords: &[Chord::plain("f1"), Chord::ctrl("/")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::Theme,
        title: "Theme",
        detail: "Settings › Appearance",
        kbd: "F2",
        chords: &[Chord::plain("f2"), Chord::ctrl(",")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::Find,
        title: "Find in site",
        detail: "Search every page",
        kbd: "Ctrl F",
        chords: &[Chord::ctrl("f")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::Insert,
        title: "Insert component",
        detail: "Insert after the selected row",
        kbd: "/",
        chords: &[Chord::ctrl("i")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::AddPage,
        title: "Add page",
        detail: "Choose a page template",
        kbd: "Ctrl N",
        chords: &[Chord::ctrl("n")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::BuildAssets,
        title: "Build CSS/JS",
        detail: "lando grunt build, or npx grunt build",
        kbd: "Ctrl B",
        chords: &[Chord::ctrl("b")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::PageHealth,
        title: "Page health",
        detail: "SEO, alt text, headings, links",
        kbd: "F4",
        chords: &[Chord::plain("f4")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::Redo,
        title: "Redo",
        detail: "Redo in the main window",
        kbd: "Ctrl R",
        chords: &[Chord::ctrl("r")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::Quit,
        title: "Quit",
        detail: "Quit dd_siteforge",
        kbd: "Ctrl Q",
        chords: &[Chord::ctrl("q")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::InitKit,
        title: "Init site kit",
        detail: "Write the starter kit into this site",
        kbd: "—",
        chords: &[],
        in_palette: true,
    },
    CommandSpec {
        command: Command::InitTemplates,
        title: "Init templates",
        detail: "Write source/templates into this site",
        kbd: "—",
        chords: &[],
        in_palette: true,
    },
    CommandSpec {
        command: Command::NewSite,
        title: "New site",
        detail: "Create a starter site.json",
        kbd: "Ctrl Shift N",
        chords: &[Chord::ctrl_shift("n")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::ToggleSidebar,
        title: "Toggle sidebar",
        detail: "Show or hide the sidebar",
        kbd: "Ctrl \\",
        chords: &[Chord::ctrl("\\")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::ToggleInspector,
        title: "Toggle inspector",
        detail: "Show or hide the inspector",
        kbd: "Ctrl Alt \\",
        chords: &[Chord::ctrl_alt("\\")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::Palette,
        title: "Command palette",
        detail: "Run a command",
        kbd: "Ctrl K",
        chords: &[Chord::ctrl("k")],
        in_palette: true,
    },
    CommandSpec {
        command: Command::Open,
        title: "Open site",
        detail: "Choose a site.json file",
        kbd: "Ctrl O",
        chords: &[Chord::ctrl("o")],
        in_palette: false,
    },
    CommandSpec {
        command: Command::ToggleIssues,
        title: "Toggle issues",
        detail: "Show or hide the Issues drawer",
        kbd: "Ctrl 4",
        chords: &[Chord::ctrl("4")],
        in_palette: false,
    },
    CommandSpec {
        command: Command::FocusSidebar,
        title: "Focus sidebar",
        detail: "Move focus to the sidebar",
        kbd: "Ctrl 1",
        chords: &[Chord::ctrl("1")],
        in_palette: false,
    },
    CommandSpec {
        command: Command::FocusOutline,
        title: "Focus outline",
        detail: "Move focus to the outline",
        kbd: "Ctrl 2",
        chords: &[Chord::ctrl("2")],
        in_palette: false,
    },
];

pub fn registry() -> &'static [CommandSpec] {
    COMMANDS
}

pub fn spec(command: Command) -> &'static CommandSpec {
    registry()
        .iter()
        .find(|spec| spec.command == command)
        .expect("command is registered")
}

pub fn command_for_chord(chord: Chord<'_>) -> Option<Command> {
    registry().iter().find_map(|spec| {
        spec.chords
            .iter()
            .any(|bound| bound.key == chord.key && same_mods(*bound, chord))
            .then_some(spec.command)
    })
}

fn same_mods(bound: Chord<'_>, chord: Chord<'_>) -> bool {
    bound.control == chord.control && bound.alt == chord.alt && bound.shift == chord.shift
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd_ord(left: Command, right: Command) -> std::cmp::Ordering {
        format!("{left:?}").cmp(&format!("{right:?}"))
    }

    /// The GUI shell's command chords, spelled the way `handle_key` sees them.
    const GUI_BINDINGS: &[(&str, Command)] = &[
        ("ctrl-\\", Command::ToggleSidebar),
        ("ctrl-alt-\\", Command::ToggleInspector),
        ("f2", Command::Theme),
        ("ctrl-,", Command::Theme),
        ("ctrl-n", Command::AddPage),
        ("ctrl-shift-n", Command::NewSite),
        ("ctrl-r", Command::Redo),
        ("f5", Command::RefreshPreview),
        ("ctrl-b", Command::BuildAssets),
        ("ctrl-e", Command::Export),
        ("f3", Command::Validate),
        ("f4", Command::PageHealth),
        ("ctrl-k", Command::Palette),
        ("ctrl-p", Command::Preview),
        ("ctrl-s", Command::Save),
        ("ctrl-q", Command::Quit),
        ("f1", Command::Help),
        ("ctrl-/", Command::Help),
        ("ctrl-f", Command::Find),
        ("ctrl-i", Command::Insert),
        ("ctrl-o", Command::Open),
        ("ctrl-4", Command::ToggleIssues),
        ("ctrl-1", Command::FocusSidebar),
        ("ctrl-2", Command::FocusOutline),
    ];

    #[test]
    fn chord_table_matches_the_gui_bindings() {
        let mut from_table: Vec<(String, Command)> = registry()
            .iter()
            .flat_map(|spec| {
                spec.chords
                    .iter()
                    .map(|chord| (chord_token(*chord), spec.command))
            })
            .collect();
        from_table.sort_by(|a, b| a.0.cmp(&b.0).then(cmd_ord(a.1, b.1)));
        let mut expected: Vec<(String, Command)> = GUI_BINDINGS
            .iter()
            .map(|(token, command)| ((*token).to_string(), *command))
            .collect();
        expected.sort_by(|a, b| a.0.cmp(&b.0).then(cmd_ord(a.1, b.1)));
        assert_eq!(from_table, expected);
        for (token, command) in GUI_BINDINGS {
            let chord = Chord::parse(token).unwrap_or_else(|| panic!("parse {token}"));
            assert_eq!(chord_token(chord), *token, "{token}");
            assert_eq!(command_for_chord(chord), Some(*command), "{token}");
        }
    }

    #[test]
    fn tui_and_vim_letters_are_not_gui_chords() {
        for token in [
            ":", "?", "s", "j", "k", "g", "h", "l", "shift-g", "shift-e", "shift-a", "shift-b",
            "shift-p",
        ] {
            let chord = Chord::parse(token).unwrap_or_else(|| panic!("parse {token}"));
            assert_eq!(
                command_for_chord(chord),
                None,
                "{token} is not a GUI command chord"
            );
        }
    }

    #[test]
    fn init_commands_have_no_chord() {
        assert!(spec(Command::InitKit).chords.is_empty());
        assert!(spec(Command::InitTemplates).chords.is_empty());
        assert!(spec(Command::InitKit).in_palette);
        assert!(spec(Command::InitTemplates).in_palette);
    }
}
