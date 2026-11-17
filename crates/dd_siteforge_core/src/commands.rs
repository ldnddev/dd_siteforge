//! One command registry and chord table for the GUI and the TUI.
//!
//! Frontends translate a key event into a [`Chord`] and call [`lookup`].
//! They do not keep a second binding list. GUI primary chords are the ones
//! [`command_for_chord`] returns; terminal fallbacks, deprecated aliases, and
//! Vim letters live in the same table and are filtered by surface and mode.

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
    /// TUI pages pane. Bare `2`, and Ctrl+2 when the Kitty protocol is on.
    FocusPages,
    /// TUI layout pane. Bare `3`.
    FocusLayouts,
    /// TUI details pane. Bare `4`.
    FocusDetails,
    /// TUI Ctrl+\ (J19). Legacy terminals report that byte as Ctrl+4.
    ToggleRegions,
    /// TUI Ctrl+Alt+\ (J19). Legacy terminals report that as Alt+Ctrl+4.
    ToggleDetails,
    /// Ctrl+Z. The Delete key is [`Command::DeleteRow`].
    Undo,
    /// The Delete key, outside a text field.
    DeleteRow,
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

    pub const fn shift(key: &'a str) -> Self {
        Self {
            key,
            control: false,
            alt: false,
            shift: true,
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
    /// False for chords that are not GUI palette rows.
    pub in_palette: bool,
    /// TUI palette row. The GUI palette ignores this.
    pub tui_palette: bool,
}

const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        command: Command::Export,
        title: "Export HTML",
        detail: "Validate, then write the site",
        kbd: "Ctrl E",
        chords: &[Chord::ctrl("e")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::Preview,
        title: "Preview in browser",
        detail: "Open the preview companion and the browser",
        kbd: "Ctrl P",
        chords: &[Chord::ctrl("p")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::RefreshPreview,
        title: "Refresh preview",
        detail: "Reload the preview from the main window",
        kbd: "F5",
        chords: &[Chord::plain("f5")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::Validate,
        title: "Validate site",
        detail: "Open the Issues list",
        kbd: "F3",
        chords: &[Chord::plain("f3")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::Save,
        title: "Save",
        detail: "Checkpoint site.json and the .backup",
        kbd: "Ctrl S",
        chords: &[Chord::ctrl("s")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::Help,
        title: "Help",
        detail: "Shortcut sheet",
        kbd: "F1",
        chords: &[Chord::plain("f1"), Chord::ctrl("/")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::Theme,
        title: "Theme",
        detail: "Settings › Appearance",
        kbd: "F2",
        chords: &[Chord::plain("f2"), Chord::ctrl(",")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::Find,
        title: "Find in site",
        detail: "Search every page",
        kbd: "Ctrl F",
        chords: &[Chord::ctrl("f")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::Insert,
        title: "Insert component",
        detail: "Insert after the selected row",
        kbd: "/",
        chords: &[Chord::ctrl("i")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::AddPage,
        title: "Add page",
        detail: "Choose a page template",
        kbd: "Ctrl N",
        chords: &[Chord::ctrl("n")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::BuildAssets,
        title: "Build CSS/JS",
        detail: "lando grunt build, or npx grunt build",
        kbd: "Ctrl B",
        chords: &[Chord::ctrl("b")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::PageHealth,
        title: "Page health",
        detail: "SEO, alt text, headings, links",
        kbd: "F4",
        chords: &[Chord::plain("f4")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::Redo,
        title: "Redo",
        detail: "Redo in the main window",
        kbd: "Ctrl R",
        chords: &[Chord::ctrl("r")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::Quit,
        title: "Quit",
        detail: "Quit dd_siteforge",
        kbd: "Ctrl Q",
        chords: &[Chord::ctrl("q")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::InitKit,
        title: "Init site kit",
        detail: "Write the starter kit into this site",
        kbd: "—",
        chords: &[],
        in_palette: true,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::InitTemplates,
        title: "Init templates",
        detail: "Write source/templates into this site",
        kbd: "—",
        chords: &[],
        in_palette: true,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::NewSite,
        title: "New site",
        detail: "Create a starter site.json",
        kbd: "Ctrl Shift N",
        chords: &[Chord::ctrl_shift("n")],
        in_palette: true,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::ToggleSidebar,
        title: "Toggle sidebar",
        detail: "Show or hide the sidebar",
        kbd: "Ctrl \\",
        chords: &[Chord::ctrl("\\")],
        in_palette: true,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::ToggleInspector,
        title: "Toggle inspector",
        detail: "Show or hide the inspector",
        kbd: "Ctrl Alt \\",
        chords: &[Chord::ctrl_alt("\\")],
        in_palette: true,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::Palette,
        title: "Command palette",
        detail: "Run a command",
        kbd: "Ctrl K",
        chords: &[Chord::ctrl("k")],
        in_palette: true,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::Open,
        title: "Open site",
        detail: "Choose a site.json file",
        kbd: "Ctrl O",
        chords: &[Chord::ctrl("o")],
        in_palette: false,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::ToggleIssues,
        title: "Toggle issues",
        detail: "Show or hide the Issues drawer",
        kbd: "Ctrl 4",
        chords: &[Chord::ctrl("4")],
        in_palette: false,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::FocusSidebar,
        title: "Focus sidebar",
        detail: "Move focus to the sidebar",
        kbd: "Ctrl 1",
        chords: &[Chord::ctrl("1")],
        in_palette: false,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::FocusOutline,
        title: "Focus outline",
        detail: "Move focus to the outline",
        kbd: "Ctrl 2",
        chords: &[Chord::ctrl("2")],
        in_palette: false,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::FocusPages,
        title: "Focus pages",
        detail: "Move focus to the Pages pane",
        kbd: "2",
        chords: &[],
        in_palette: false,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::FocusLayouts,
        title: "Focus layout",
        detail: "Move focus to the Layout pane",
        kbd: "3",
        chords: &[],
        in_palette: false,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::FocusDetails,
        title: "Focus details",
        detail: "Move focus to the Details pane",
        kbd: "4",
        chords: &[],
        in_palette: false,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::ToggleRegions,
        title: "Toggle Regions",
        detail: "Show or hide the Regions pane",
        kbd: "Ctrl \\",
        chords: &[],
        in_palette: false,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::ToggleDetails,
        title: "Toggle Details",
        detail: "Show or hide the Details pane",
        kbd: "Ctrl Alt \\",
        chords: &[],
        in_palette: false,
        tui_palette: true,
    },
    CommandSpec {
        command: Command::Undo,
        title: "Undo",
        detail: "Undo the last edit",
        kbd: "Ctrl Z",
        chords: &[],
        in_palette: false,
        tui_palette: false,
    },
    CommandSpec {
        command: Command::DeleteRow,
        title: "Delete row",
        detail: "Delete the selected row",
        kbd: "Delete",
        chords: &[],
        in_palette: false,
        tui_palette: false,
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

/// Which app is asking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Gui,
    Tui,
}

/// How the terminal delivers keys. The GUI always uses [`InputMode::Kitty`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    /// Kitty keyboard protocol. Ctrl+Shift+letter and Ctrl+\ are distinct.
    Kitty,
    /// Legacy bytes. Ctrl+\ arrives as Ctrl+4. Ctrl+Shift+letter does not.
    Legacy,
    /// tmux without extended-keys. No Ctrl+B, and no Ctrl+Shift+letter.
    Tmux,
}

/// `supports_keyboard_enhancement()` is true. tmux stays on the tmux map
/// even when the protocol query succeeds, because Ctrl+B is its prefix.
pub fn input_mode(keyboard_enhanced: bool, in_tmux: bool) -> InputMode {
    if in_tmux {
        InputMode::Tmux
    } else if keyboard_enhanced {
        InputMode::Kitty
    } else {
        InputMode::Legacy
    }
}

/// Push the Kitty keyboard protocol only when the terminal says it can.
pub fn push_keyboard_enhancement(supported: bool) -> bool {
    supported
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingKind {
    Primary,
    /// Used where the primary chord cannot be delivered.
    Fallback,
    /// Kept for one release. [`Hit::Command::hint`] is shown once.
    Alias,
}

/// Letter keys that follow the Vim keys setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VimAction {
    Up,
    Down,
    First,
    Last,
    Collapse,
    Expand,
    ToggleExpand,
    Repeat,
    Copy,
    Paste,
    Delete,
    Undo,
    Rename,
    AddItem,
    RemoveItem,
    MoveDown,
    MoveUp,
    PrevColumn,
    NextColumn,
    AddColumn,
    RemoveColumn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Command(Command),
    Vim(VimAction),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Command {
        command: Command,
        kind: BindingKind,
        hint: Option<&'static str>,
    },
    Vim(VimAction),
}

#[derive(Clone, Copy, Debug)]
pub struct Binding {
    pub target: Target,
    pub chord: Chord<'static>,
    pub kind: BindingKind,
    pub gui: bool,
    pub tui: bool,
    pub kitty: bool,
    pub legacy: bool,
    pub tmux: bool,
    pub vim_only: bool,
    pub outside_text: bool,
    pub hint: Option<&'static str>,
}

impl Binding {
    const fn on(command: Command, chord: Chord<'static>) -> Self {
        Self {
            target: Target::Command(command),
            chord,
            kind: BindingKind::Primary,
            gui: true,
            tui: true,
            kitty: true,
            legacy: true,
            tmux: true,
            vim_only: false,
            outside_text: false,
            hint: None,
        }
    }

    const fn vim(action: VimAction, chord: Chord<'static>) -> Self {
        Self {
            target: Target::Vim(action),
            chord,
            kind: BindingKind::Primary,
            gui: true,
            tui: true,
            kitty: true,
            legacy: true,
            tmux: true,
            vim_only: true,
            outside_text: true,
            hint: None,
        }
    }

    const fn gui_only(mut self) -> Self {
        self.gui = true;
        self.tui = false;
        self
    }

    const fn tui_only(mut self) -> Self {
        self.gui = false;
        self.tui = true;
        self
    }

    const fn kitty_only(mut self) -> Self {
        self.kitty = true;
        self.legacy = false;
        self.tmux = false;
        self
    }

    const fn not_tmux(mut self) -> Self {
        self.tmux = false;
        self
    }

    const fn legacy_tmux(mut self) -> Self {
        self.kitty = false;
        self.legacy = true;
        self.tmux = true;
        self
    }

    const fn alias(mut self, hint: &'static str) -> Self {
        self.kind = BindingKind::Alias;
        self.hint = Some(hint);
        self
    }

    const fn outside_text(mut self) -> Self {
        self.outside_text = true;
        self
    }
}

const BINDINGS: &[Binding] = &[
    Binding::on(Command::Export, Chord::ctrl("e")).gui_only(),
    Binding::on(Command::Export, Chord::ctrl("e"))
        .tui_only()
        .outside_text(),
    Binding::on(Command::Export, Chord::shift("e"))
        .tui_only()
        .alias("Export is now Ctrl+E"),
    Binding::on(Command::Preview, Chord::ctrl("p")),
    Binding::on(Command::Preview, Chord::shift("p"))
        .tui_only()
        .alias("Preview is now Ctrl+P"),
    Binding::on(Command::RefreshPreview, Chord::plain("f5")),
    Binding::on(Command::Validate, Chord::plain("f3")),
    Binding::on(Command::Save, Chord::ctrl("s")),
    Binding::on(Command::Save, Chord::plain("s"))
        .tui_only()
        .outside_text()
        .alias("Save is now Ctrl+S"),
    Binding::on(Command::Help, Chord::plain("f1")),
    Binding::on(Command::Help, Chord::ctrl("/")),
    Binding::on(Command::Theme, Chord::plain("f2")),
    Binding::on(Command::Theme, Chord::ctrl(",")),
    Binding::on(Command::Find, Chord::ctrl("f")),
    Binding::on(Command::Find, Chord::plain("?"))
        .tui_only()
        .outside_text(),
    Binding::on(Command::Insert, Chord::ctrl("i")).gui_only(),
    Binding::on(Command::Insert, Chord::plain("/")).outside_text(),
    Binding::on(Command::AddPage, Chord::ctrl("n")),
    Binding::on(Command::BuildAssets, Chord::ctrl("b")).not_tmux(),
    Binding::on(Command::BuildAssets, Chord::shift("b"))
        .tui_only()
        .alias("Build is now Ctrl+B"),
    Binding::on(Command::PageHealth, Chord::plain("f4")),
    Binding::on(Command::Redo, Chord::ctrl("r")),
    Binding::on(Command::Redo, Chord::ctrl_shift("z")).kitty_only(),
    Binding::on(Command::Quit, Chord::ctrl("q")),
    Binding::on(Command::NewSite, Chord::ctrl_shift("n")).kitty_only(),
    Binding::on(Command::ToggleSidebar, Chord::ctrl("\\")).gui_only(),
    Binding::on(Command::ToggleInspector, Chord::ctrl_alt("\\")).gui_only(),
    Binding::on(Command::ToggleRegions, Chord::ctrl("\\"))
        .tui_only()
        .kitty_only(),
    Binding::on(Command::ToggleRegions, Chord::ctrl("4"))
        .tui_only()
        .legacy_tmux(),
    Binding::on(Command::ToggleDetails, Chord::ctrl_alt("\\"))
        .tui_only()
        .kitty_only(),
    Binding::on(Command::ToggleDetails, Chord::ctrl_alt("4"))
        .tui_only()
        .legacy_tmux(),
    Binding::on(Command::Palette, Chord::ctrl("k")),
    Binding::on(Command::Palette, Chord::plain(":"))
        .tui_only()
        .outside_text(),
    Binding::on(Command::Open, Chord::ctrl("o")),
    Binding::on(Command::ToggleIssues, Chord::ctrl("4")).gui_only(),
    Binding::on(Command::ToggleIssues, Chord::ctrl("4"))
        .tui_only()
        .kitty_only(),
    Binding::on(Command::FocusSidebar, Chord::ctrl("1")).kitty_only(),
    Binding::on(Command::FocusOutline, Chord::ctrl("2"))
        .gui_only()
        .kitty_only(),
    Binding::on(Command::FocusPages, Chord::ctrl("2"))
        .tui_only()
        .kitty_only(),
    Binding::on(Command::FocusSidebar, Chord::plain("1"))
        .tui_only()
        .outside_text(),
    Binding::on(Command::FocusPages, Chord::plain("2"))
        .tui_only()
        .outside_text(),
    Binding::on(Command::FocusLayouts, Chord::plain("3"))
        .tui_only()
        .outside_text(),
    Binding::on(Command::FocusDetails, Chord::plain("4"))
        .tui_only()
        .outside_text(),
    Binding::on(Command::Undo, Chord::ctrl("z")),
    Binding::on(Command::DeleteRow, Chord::plain("delete")).outside_text(),
    Binding::vim(VimAction::Down, Chord::plain("j")),
    Binding::vim(VimAction::Up, Chord::plain("k")),
    Binding::vim(VimAction::First, Chord::plain("g")),
    Binding::vim(VimAction::Last, Chord::shift("g")),
    Binding::vim(VimAction::Collapse, Chord::plain("h")),
    Binding::vim(VimAction::Expand, Chord::plain("l")),
    Binding::vim(VimAction::ToggleExpand, Chord::plain(" ")).tui_only(),
    Binding::vim(VimAction::ToggleExpand, Chord::plain("space")).gui_only(),
    Binding::vim(VimAction::Repeat, Chord::plain(".")),
    Binding::vim(VimAction::Copy, Chord::plain("y")),
    Binding::vim(VimAction::Paste, Chord::plain("p")),
    Binding::vim(VimAction::Delete, Chord::plain("d")),
    Binding::vim(VimAction::Undo, Chord::plain("u")),
    Binding::vim(VimAction::Rename, Chord::plain("r")),
    Binding::vim(VimAction::MoveDown, Chord::shift("j")),
    Binding::vim(VimAction::MoveUp, Chord::shift("k")),
    Binding::vim(VimAction::AddItem, Chord::shift("a")),
    Binding::vim(VimAction::RemoveItem, Chord::shift("x")),
    Binding::vim(VimAction::PrevColumn, Chord::plain("c")),
    Binding::vim(VimAction::NextColumn, Chord::plain("v")),
    Binding::vim(VimAction::AddColumn, Chord::shift("c")),
    Binding::vim(VimAction::RemoveColumn, Chord::shift("v")),
];

pub fn bindings() -> &'static [Binding] {
    BINDINGS
}

#[derive(Clone, Copy, Debug)]
pub struct Query<'a> {
    pub surface: Surface,
    pub mode: InputMode,
    pub vim: bool,
    pub in_text: bool,
    pub chord: Chord<'a>,
}

/// Resolve one chord. Primary wins over an alias on the same chord.
pub fn lookup(query: Query<'_>) -> Option<Hit> {
    let mut found: Option<&Binding> = None;
    for binding in BINDINGS {
        if !binding_matches(binding, &query) {
            continue;
        }
        match found {
            None => found = Some(binding),
            Some(current) if kind_rank(binding.kind) < kind_rank(current.kind) => {
                found = Some(binding);
            }
            Some(_) => {}
        }
    }
    found.map(|binding| match binding.target {
        Target::Command(command) => Hit::Command {
            command,
            kind: binding.kind,
            hint: binding.hint,
        },
        Target::Vim(action) => Hit::Vim(action),
    })
}

fn kind_rank(kind: BindingKind) -> u8 {
    match kind {
        BindingKind::Primary => 0,
        BindingKind::Fallback => 1,
        BindingKind::Alias => 2,
    }
}

fn binding_matches(binding: &Binding, query: &Query<'_>) -> bool {
    let surface_ok = match query.surface {
        Surface::Gui => binding.gui,
        Surface::Tui => binding.tui,
    };
    if !surface_ok || !same_mods(binding.chord, query.chord) || binding.chord.key != query.chord.key
    {
        return false;
    }
    let mode_ok = match query.mode {
        InputMode::Kitty => binding.kitty,
        InputMode::Legacy => binding.legacy,
        InputMode::Tmux => binding.tmux,
    };
    mode_ok && !(binding.vim_only && !query.vim) && !(binding.outside_text && query.in_text)
}

/// One shortcut-sheet row. Both apps render this list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SheetRow {
    pub section: &'static str,
    pub action: &'static str,
    pub gui: &'static str,
    pub tui: &'static str,
}

pub fn shortcut_sheet() -> &'static [SheetRow] {
    SHEET
}

const SHEET: &[SheetRow] = &[
    SheetRow {
        section: "Global",
        action: "Help",
        gui: "F1",
        tui: "F1",
    },
    SheetRow {
        section: "Global",
        action: "Theme",
        gui: "F2",
        tui: "F2",
    },
    SheetRow {
        section: "Global",
        action: "Validate site",
        gui: "F3",
        tui: "F3",
    },
    SheetRow {
        section: "Global",
        action: "Page health",
        gui: "F4",
        tui: "F4",
    },
    SheetRow {
        section: "Global",
        action: "Export",
        gui: "Ctrl E",
        tui: "Ctrl E",
    },
    SheetRow {
        section: "Global",
        action: "Preview",
        gui: "Ctrl P",
        tui: "Ctrl P",
    },
    SheetRow {
        section: "Global",
        action: "Refresh preview",
        gui: "F5",
        tui: "F5",
    },
    SheetRow {
        section: "Global",
        action: "Build CSS/JS",
        gui: "Ctrl B",
        tui: "Ctrl B",
    },
    SheetRow {
        section: "Global",
        action: "Find in site",
        gui: "Ctrl F",
        tui: "Ctrl F",
    },
    SheetRow {
        section: "Global",
        action: "Command palette",
        gui: "Ctrl K",
        tui: "Ctrl K",
    },
    SheetRow {
        section: "Global",
        action: "Redo",
        gui: "Ctrl R",
        tui: "Ctrl R",
    },
    SheetRow {
        section: "Global",
        action: "Undo",
        gui: "Ctrl Z",
        tui: "Ctrl Z",
    },
    SheetRow {
        section: "Global",
        action: "Checkpoint (site.json + .backup)",
        gui: "Ctrl S",
        tui: "Ctrl S",
    },
    SheetRow {
        section: "Global",
        action: "Quit",
        gui: "Ctrl Q",
        tui: "Ctrl Q",
    },
    SheetRow {
        section: "Global",
        action: "New site",
        gui: "Ctrl Shift N",
        tui: "Ctrl Shift N",
    },
    SheetRow {
        section: "Global",
        action: "Toggle sidebar / Regions",
        gui: "Ctrl \\",
        tui: "Ctrl \\",
    },
    SheetRow {
        section: "Global",
        action: "Toggle inspector / Details",
        gui: "Ctrl Alt \\",
        tui: "Ctrl Alt \\",
    },
    SheetRow {
        section: "Outline",
        action: "Move",
        gui: "j k",
        tui: "j k",
    },
    SheetRow {
        section: "Outline",
        action: "First / last",
        gui: "g G",
        tui: "g G",
    },
    SheetRow {
        section: "Outline",
        action: "Collapse / expand",
        gui: "h l",
        tui: "Space",
    },
    SheetRow {
        section: "Outline",
        action: "Insert",
        gui: "/",
        tui: "/",
    },
    SheetRow {
        section: "Outline",
        action: "Delete",
        gui: "Delete",
        tui: "Delete",
    },
    SheetRow {
        section: "Outline",
        action: "Copy",
        gui: "y",
        tui: "y",
    },
    SheetRow {
        section: "Outline",
        action: "Paste",
        gui: "p",
        tui: "p",
    },
    SheetRow {
        section: "Outline",
        action: "Move row",
        gui: "J K",
        tui: "J K",
    },
    SheetRow {
        section: "Sidebar · pages",
        action: "Add page",
        gui: "Ctrl N",
        tui: "Ctrl N",
    },
    SheetRow {
        section: "Sidebar · pages",
        action: "New site",
        gui: "Ctrl Shift N",
        tui: "Ctrl Shift N",
    },
    SheetRow {
        section: "Sidebar · pages",
        action: "Rename page",
        gui: "r",
        tui: "r",
    },
    SheetRow {
        section: "Sidebar · pages",
        action: "Focus panes",
        gui: "Ctrl 1 / Ctrl 2",
        tui: "1 2 3 4",
    },
    SheetRow {
        section: "Inspector",
        action: "Next / previous field",
        gui: "Tab / Shift Tab",
        tui: "Tab / Shift Tab",
    },
    SheetRow {
        section: "Inspector",
        action: "Checkpoint (site.json + .backup)",
        gui: "Ctrl S",
        tui: "Ctrl S",
    },
    SheetRow {
        section: "Inspector",
        action: "Open picker",
        gui: "Ctrl O",
        tui: "Ctrl O",
    },
    SheetRow {
        section: "Inspector",
        action: "Cancel",
        gui: "Esc",
        tui: "Esc",
    },
];

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

    fn q(surface: Surface, mode: InputMode, vim: bool, in_text: bool, token: &str) -> Option<Hit> {
        let chord = Chord::parse(token).unwrap_or_else(|| panic!("parse {token}"));
        lookup(Query {
            surface,
            mode,
            vim,
            in_text,
            chord,
        })
    }

    fn cmd(token: &str, mode: InputMode) -> Hit {
        q(Surface::Tui, mode, true, false, token).unwrap_or_else(|| panic!("no hit {token}"))
    }

    fn command_of(hit: Hit) -> Command {
        match hit {
            Hit::Command { command, .. } => command,
            Hit::Vim(_) => panic!("expected a command"),
        }
    }

    #[test]
    fn every_gui_primary_chord_is_in_the_shared_table() {
        for spec in registry() {
            for chord in spec.chords {
                let hit = lookup(Query {
                    surface: Surface::Gui,
                    mode: InputMode::Kitty,
                    vim: false,
                    in_text: false,
                    chord: *chord,
                });
                assert_eq!(
                    hit,
                    Some(Hit::Command {
                        command: spec.command,
                        kind: BindingKind::Primary,
                        hint: None,
                    }),
                    "{}",
                    chord_token(*chord)
                );
            }
        }
    }

    #[test]
    fn kitty_legacy_and_tmux_cover_every_fallback() {
        for mode in [InputMode::Kitty, InputMode::Legacy, InputMode::Tmux] {
            assert_eq!(command_of(cmd("f1", mode)), Command::Help);
            assert_eq!(command_of(cmd("f2", mode)), Command::Theme);
            assert_eq!(command_of(cmd("f3", mode)), Command::Validate);
            assert_eq!(command_of(cmd("f4", mode)), Command::PageHealth);
            assert_eq!(command_of(cmd("f5", mode)), Command::RefreshPreview);
            assert_eq!(command_of(cmd("ctrl-e", mode)), Command::Export);
            assert_eq!(command_of(cmd("ctrl-p", mode)), Command::Preview);
            assert_eq!(command_of(cmd("ctrl-s", mode)), Command::Save);
            assert_eq!(command_of(cmd("ctrl-z", mode)), Command::Undo);
            assert_eq!(command_of(cmd("ctrl-r", mode)), Command::Redo);
            assert_eq!(command_of(cmd("ctrl-n", mode)), Command::AddPage);
            assert_eq!(command_of(cmd("ctrl-q", mode)), Command::Quit);
            assert_eq!(command_of(cmd("ctrl-o", mode)), Command::Open);
            assert_eq!(command_of(cmd("ctrl-k", mode)), Command::Palette);
            assert_eq!(command_of(cmd(":", mode)), Command::Palette);
            assert_eq!(command_of(cmd("ctrl-f", mode)), Command::Find);
            assert_eq!(command_of(cmd("?", mode)), Command::Find);
            assert_eq!(command_of(cmd("/", mode)), Command::Insert);
            assert_eq!(command_of(cmd("delete", mode)), Command::DeleteRow);
            assert_eq!(command_of(cmd("1", mode)), Command::FocusSidebar);
            assert_eq!(command_of(cmd("2", mode)), Command::FocusPages);
            assert_eq!(command_of(cmd("3", mode)), Command::FocusLayouts);
            assert_eq!(command_of(cmd("4", mode)), Command::FocusDetails);
        }

        assert_eq!(
            command_of(cmd("ctrl-b", InputMode::Kitty)),
            Command::BuildAssets
        );
        assert_eq!(
            command_of(cmd("ctrl-b", InputMode::Legacy)),
            Command::BuildAssets
        );
        assert!(q(Surface::Tui, InputMode::Tmux, true, false, "ctrl-b").is_none());
        assert_eq!(
            command_of(cmd("shift-b", InputMode::Tmux)),
            Command::BuildAssets
        );

        assert_eq!(
            command_of(cmd("ctrl-shift-z", InputMode::Kitty)),
            Command::Redo
        );
        assert!(q(Surface::Tui, InputMode::Legacy, true, false, "ctrl-shift-z").is_none());
        assert!(q(Surface::Tui, InputMode::Tmux, true, false, "ctrl-shift-z").is_none());
        assert_eq!(
            command_of(cmd("ctrl-shift-n", InputMode::Kitty)),
            Command::NewSite
        );
        assert!(q(Surface::Tui, InputMode::Legacy, true, false, "ctrl-shift-n").is_none());
        assert!(q(Surface::Tui, InputMode::Tmux, true, false, "ctrl-shift-n").is_none());
        assert!(spec(Command::NewSite).tui_palette);

        assert!(q(Surface::Tui, InputMode::Legacy, true, false, "ctrl-1").is_none());
        assert!(q(Surface::Tui, InputMode::Tmux, true, false, "ctrl-2").is_none());
        assert_eq!(
            command_of(cmd("ctrl-1", InputMode::Kitty)),
            Command::FocusSidebar
        );
        assert_eq!(
            command_of(cmd("ctrl-2", InputMode::Kitty)),
            Command::FocusPages
        );
    }

    #[test]
    fn aliases_are_marked_and_name_the_new_chord() {
        for (token, hint) in [
            ("shift-b", "Build is now Ctrl+B"),
            ("shift-e", "Export is now Ctrl+E"),
            ("shift-p", "Preview is now Ctrl+P"),
            ("s", "Save is now Ctrl+S"),
        ] {
            match cmd(token, InputMode::Legacy) {
                Hit::Command {
                    kind: BindingKind::Alias,
                    hint: Some(got),
                    ..
                } => assert_eq!(got, hint),
                other => panic!("{token} should be an alias, got {other:?}"),
            }
        }
        assert!(bindings().iter().any(|binding| {
            binding.kind == BindingKind::Alias && binding.hint == Some("Build is now Ctrl+B")
        }));
    }

    #[test]
    fn j19_regions_and_details_in_kitty_and_legacy() {
        assert_eq!(
            command_of(cmd("ctrl-\\", InputMode::Kitty)),
            Command::ToggleRegions
        );
        assert_eq!(
            command_of(cmd("ctrl-alt-\\", InputMode::Kitty)),
            Command::ToggleDetails
        );
        assert_eq!(
            command_of(cmd("ctrl-4", InputMode::Kitty)),
            Command::ToggleIssues
        );
        assert_eq!(
            command_of(cmd("ctrl-4", InputMode::Legacy)),
            Command::ToggleRegions
        );
        assert_eq!(
            command_of(cmd("ctrl-alt-4", InputMode::Legacy)),
            Command::ToggleDetails
        );
        assert_eq!(
            command_of(cmd("ctrl-4", InputMode::Tmux)),
            Command::ToggleRegions
        );
        assert_eq!(
            command_of(cmd("ctrl-alt-4", InputMode::Tmux)),
            Command::ToggleDetails
        );
        assert!(q(Surface::Tui, InputMode::Legacy, true, false, "ctrl-\\").is_none());
        assert_eq!(
            q(Surface::Gui, InputMode::Kitty, false, false, "ctrl-\\"),
            Some(Hit::Command {
                command: Command::ToggleSidebar,
                kind: BindingKind::Primary,
                hint: None,
            })
        );
        assert_eq!(
            q(Surface::Gui, InputMode::Kitty, false, false, "ctrl-4"),
            Some(Hit::Command {
                command: Command::ToggleIssues,
                kind: BindingKind::Primary,
                hint: None,
            })
        );
        assert!(spec(Command::ToggleRegions).tui_palette);
        assert!(spec(Command::ToggleDetails).tui_palette);
        assert!(!spec(Command::ToggleRegions).in_palette);
    }

    #[test]
    fn vim_letters_follow_the_setting_in_both_apps_and_skip_text() {
        let letters = [
            "u", "d", "y", "p", ".", "j", "k", "g", "shift-g", "r", "h", "l", "shift-j", "shift-k",
            "shift-a", "shift-x", "c", "v", "shift-c", "shift-v",
        ];
        for surface in [Surface::Gui, Surface::Tui] {
            for token in letters {
                assert!(
                    q(surface, InputMode::Kitty, true, false, token).is_some(),
                    "{surface:?} {token} on"
                );
                assert!(
                    q(surface, InputMode::Kitty, false, false, token).is_none(),
                    "{surface:?} {token} off"
                );
                assert!(
                    q(surface, InputMode::Kitty, true, true, token).is_none(),
                    "{surface:?} {token} in text"
                );
            }
        }
        let space = Chord::plain(" ");
        assert!(matches!(
            lookup(Query {
                surface: Surface::Tui,
                mode: InputMode::Legacy,
                vim: true,
                in_text: false,
                chord: space,
            }),
            Some(Hit::Vim(VimAction::ToggleExpand))
        ));
        assert!(
            lookup(Query {
                surface: Surface::Tui,
                mode: InputMode::Legacy,
                vim: false,
                in_text: false,
                chord: space,
            })
            .is_none()
        );
        assert!(matches!(
            q(Surface::Gui, InputMode::Kitty, true, false, "space"),
            Some(Hit::Vim(VimAction::ToggleExpand))
        ));
        assert_eq!(
            q(Surface::Gui, InputMode::Kitty, false, false, "/"),
            Some(Hit::Command {
                command: Command::Insert,
                kind: BindingKind::Primary,
                hint: None,
            })
        );
        assert!(q(Surface::Gui, InputMode::Kitty, true, true, "/").is_none());
    }

    #[test]
    fn shortcut_sheet_comes_from_this_module_and_keeps_gui_chords() {
        let text = shortcut_sheet()
            .iter()
            .map(|row| format!("{} · {} · {}", row.action, row.gui, row.tui))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("Add page · Ctrl N · Ctrl N"));
        assert!(text.contains("New site · Ctrl Shift N · Ctrl Shift N"));
        assert!(text.contains("Theme · F2 · F2"));
        assert!(text.contains("Redo · Ctrl R · Ctrl R"));
        assert!(text.contains("Refresh preview · F5 · F5"));
        assert!(text.contains("Undo · Ctrl Z · Ctrl Z"));
        assert!(!text.contains("J2"));
        assert!(!text.contains("Shift+A"));
    }

    #[test]
    fn input_mode_treats_tmux_as_its_own_map() {
        assert_eq!(input_mode(true, false), InputMode::Kitty);
        assert_eq!(input_mode(false, false), InputMode::Legacy);
        assert_eq!(input_mode(true, true), InputMode::Tmux);
        assert_eq!(input_mode(false, true), InputMode::Tmux);
        assert!(!push_keyboard_enhancement(false));
        assert!(push_keyboard_enhancement(true));
    }
}
