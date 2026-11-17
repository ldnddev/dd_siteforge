//! Thin GUI adapter over [`dd_siteforge_core::commands`].
//!
//! Chords and palette labels live in core. This module turns a shell key
//! token into that chord, and keeps Vim motions and single-key verbs here.
//! Those letters are not command chords.

use dd_siteforge_core::commands::{self as core, Chord};

pub use core::Command as CommandId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    pub id: CommandId,
    pub title: &'static str,
    pub detail: &'static str,
    pub kbd: &'static str,
}

/// Title-bar buttons. Init site kit and Init templates are not here (J17).
pub const TITLE_BAR_ACTIONS: &[&str] = &["Export", "Preview"];

pub fn registry() -> impl Iterator<Item = Command> {
    core::registry()
        .iter()
        .filter(|spec| spec.in_palette)
        .map(view)
}

fn view(spec: &core::CommandSpec) -> Command {
    Command {
        id: spec.command,
        title: spec.title,
        detail: spec.detail,
        kbd: spec.kbd,
    }
}

pub fn command(id: CommandId) -> Command {
    view(core::spec(id))
}

/// Map a shell key token onto the core chord table.
pub fn command_for_key(key: &str) -> Option<CommandId> {
    let chord = Chord::parse(key.trim())?;
    core::command_for_chord(chord)
}

pub fn filter_commands(query: &str) -> Vec<Command> {
    let q = query.trim();
    let mut scored: Vec<(Command, i32)> = core::registry()
        .iter()
        .filter(|spec| spec.in_palette)
        .filter_map(|spec| {
            let hay = format!("{} {} {}", spec.title, spec.detail, spec.kbd);
            fuzzy_score(q, &hay).map(|score| (view(spec), score))
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.title.cmp(b.0.title)));
    scored.into_iter().map(|(cmd, _)| cmd).collect()
}

/// Detail shown on a palette row. Build and Export keep their running sentence.
pub fn palette_detail(
    cmd: &Command,
    build_running: bool,
    build_text: &str,
    export_running: bool,
    export_text: &str,
) -> String {
    if cmd.id == CommandId::BuildAssets && build_running && !build_text.is_empty() {
        return build_text.to_string();
    }
    if cmd.id == CommandId::Export && export_running && !export_text.is_empty() {
        return export_text.to_string();
    }
    cmd.detail.to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Outline, sidebar, welcome, and lists.
    List,
    /// A text field. Vim motions never apply here.
    Text,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Up,
    Down,
    First,
    Last,
    Collapse,
    Expand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Insert,
    Repeat,
    Copy,
    Paste,
    Delete,
    Undo,
    Rename,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyHit {
    Command(CommandId),
    Motion(Motion),
    Verb(Verb),
    Type(char),
    Ignore,
}

/// Resolve one key. Command chords come from core. Vim motions stay here.
pub fn resolve(key: &str, vim: bool, scope: Scope) -> KeyHit {
    let key = key.to_ascii_lowercase();
    if let Some(id) = command_for_key(&key) {
        return KeyHit::Command(id);
    }
    match (scope, vim, key.as_str()) {
        (Scope::List, true, "j" | "down") => KeyHit::Motion(Motion::Down),
        (Scope::List, true, "k" | "up") => KeyHit::Motion(Motion::Up),
        (Scope::List, false, "down") => KeyHit::Motion(Motion::Down),
        (Scope::List, false, "up") => KeyHit::Motion(Motion::Up),
        (Scope::List, true, "g") => KeyHit::Motion(Motion::First),
        (Scope::List, true, "shift-g") => KeyHit::Motion(Motion::Last),
        (Scope::List, true, "h") => KeyHit::Motion(Motion::Collapse),
        (Scope::List, true, "l") => KeyHit::Motion(Motion::Expand),
        (_, _, "down") => KeyHit::Motion(Motion::Down),
        (_, _, "up") => KeyHit::Motion(Motion::Up),
        (Scope::List, _, "/") => KeyHit::Verb(Verb::Insert),
        (Scope::List, _, ".") => KeyHit::Verb(Verb::Repeat),
        (Scope::List, _, "y") => KeyHit::Verb(Verb::Copy),
        (Scope::List, _, "p") => KeyHit::Verb(Verb::Paste),
        (Scope::List, _, "d") => KeyHit::Verb(Verb::Delete),
        (Scope::List, _, "u") => KeyHit::Verb(Verb::Undo),
        (Scope::List, _, "r") => KeyHit::Verb(Verb::Rename),
        (_, _, other) if is_typeable(other) => KeyHit::Type(other.chars().next().unwrap_or('?')),
        _ => KeyHit::Ignore,
    }
}

fn is_typeable(key: &str) -> bool {
    key.chars().count() == 1 && !key.chars().any(|ch| ch.is_control())
}

/// Same scoring as the TUI palette (`src/tui/util.rs`).
pub fn fuzzy_score(query: &str, text: &str) -> Option<i32> {
    let q = query.to_ascii_lowercase();
    let t = text.to_ascii_lowercase();
    if q.is_empty() {
        return Some(0);
    }
    if t.contains(&q) {
        return Some(1000 - (t.find(&q).unwrap_or(0) as i32));
    }
    let mut score = 0i32;
    let mut t_chars = t.chars().enumerate();
    let mut last_idx: Option<usize> = None;
    for qc in q.chars() {
        let mut found = None;
        for (idx, tc) in t_chars.by_ref() {
            if tc == qc {
                found = Some(idx);
                break;
            }
        }
        let Some(idx) = found else {
            return None;
        };
        score += 10;
        if let Some(prev) = last_idx {
            if idx == prev + 1 {
                score += 8;
            }
        }
        if idx == 0 {
            score += 6;
        }
        last_idx = Some(idx);
    }
    Some(score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys;

    #[test]
    fn adapter_reads_the_core_chord_table() {
        for spec in core::registry() {
            for chord in spec.chords {
                let token = core::chord_token(*chord);
                assert_eq!(command_for_key(&token), Some(spec.command), "{token}");
            }
        }
        assert_eq!(
            command_for_key(keys::TOGGLE_SIDEBAR_KEY),
            Some(CommandId::ToggleSidebar)
        );
        assert_eq!(
            command_for_key(keys::TOGGLE_INSPECTOR_KEY),
            Some(CommandId::ToggleInspector)
        );
        assert_eq!(command_for_key(keys::THEME_KEY), Some(CommandId::Theme));
        assert_eq!(
            command_for_key(keys::ADD_PAGE_KEY),
            Some(CommandId::AddPage)
        );
        assert_eq!(
            command_for_key(keys::NEW_SITE_KEY),
            Some(CommandId::NewSite)
        );
        assert_eq!(command_for_key(keys::REDO_KEY), Some(CommandId::Redo));
        assert_eq!(command_for_key(keys::QUIT_KEY), Some(CommandId::Quit));
        assert_eq!(command_for_key(keys::OPEN_KEY), Some(CommandId::Open));
    }

    #[test]
    fn palette_lists_init_commands_and_the_title_bar_does_not() {
        let init = filter_commands("init");
        let titles: Vec<_> = init.iter().map(|cmd| cmd.title).collect();
        assert!(titles.contains(&"Init site kit"), "{titles:?}");
        assert!(titles.contains(&"Init templates"), "{titles:?}");
        assert!(!TITLE_BAR_ACTIONS.contains(&"Init site kit"));
        assert!(!TITLE_BAR_ACTIONS.contains(&"Init templates"));
        assert!(TITLE_BAR_ACTIONS.contains(&"Export"));
        assert!(TITLE_BAR_ACTIONS.contains(&"Preview"));
    }

    #[test]
    fn registry_chords_keep_the_phase_4_bindings() {
        assert_eq!(command_for_key("ctrl-\\"), Some(CommandId::ToggleSidebar));
        assert_eq!(
            command_for_key("ctrl-alt-\\"),
            Some(CommandId::ToggleInspector)
        );
        assert_eq!(command_for_key("f2"), Some(CommandId::Theme));
        assert_eq!(command_for_key("ctrl-n"), Some(CommandId::AddPage));
        assert_eq!(command_for_key("ctrl-shift-n"), Some(CommandId::NewSite));
        assert_eq!(command_for_key("ctrl-r"), Some(CommandId::Redo));
        assert_eq!(command_for_key("f5"), Some(CommandId::RefreshPreview));
        assert_eq!(command_for_key("ctrl-b"), Some(CommandId::BuildAssets));
        assert_eq!(command_for_key("ctrl-e"), Some(CommandId::Export));
        assert_eq!(command_for_key("f3"), Some(CommandId::Validate));
        assert_eq!(command_for_key("f4"), Some(CommandId::PageHealth));
        assert_eq!(command_for_key("ctrl-k"), Some(CommandId::Palette));
        assert_eq!(resolve(":", true, Scope::List), KeyHit::Type(':'));
        assert_eq!(resolve("?", true, Scope::List), KeyHit::Type('?'));
        assert_eq!(resolve("s", true, Scope::List), KeyHit::Type('s'));
    }

    #[test]
    fn vim_on_moves_and_vim_off_types() {
        assert_eq!(
            resolve("j", true, Scope::List),
            KeyHit::Motion(Motion::Down)
        );
        assert_eq!(resolve("k", true, Scope::List), KeyHit::Motion(Motion::Up));
        assert_eq!(
            resolve("g", true, Scope::List),
            KeyHit::Motion(Motion::First)
        );
        assert_eq!(
            resolve("shift-g", true, Scope::List),
            KeyHit::Motion(Motion::Last)
        );
        assert_eq!(
            resolve("h", true, Scope::List),
            KeyHit::Motion(Motion::Collapse)
        );
        assert_eq!(
            resolve("l", true, Scope::List),
            KeyHit::Motion(Motion::Expand)
        );
        assert_eq!(resolve("j", false, Scope::List), KeyHit::Type('j'));
        assert_eq!(resolve("k", false, Scope::List), KeyHit::Type('k'));
        assert_eq!(resolve("g", false, Scope::List), KeyHit::Type('g'));
        assert_eq!(resolve("h", false, Scope::List), KeyHit::Type('h'));
        assert_eq!(resolve("l", false, Scope::List), KeyHit::Type('l'));
        assert_eq!(
            resolve("down", false, Scope::List),
            KeyHit::Motion(Motion::Down)
        );
        assert_eq!(
            resolve("up", false, Scope::List),
            KeyHit::Motion(Motion::Up)
        );
    }

    #[test]
    fn vim_motions_never_fire_inside_text_and_verbs_stay() {
        assert_eq!(resolve("j", true, Scope::Text), KeyHit::Type('j'));
        assert_eq!(resolve("k", true, Scope::Text), KeyHit::Type('k'));
        assert_eq!(resolve("g", true, Scope::Text), KeyHit::Type('g'));
        assert_eq!(resolve("h", true, Scope::Text), KeyHit::Type('h'));
        assert_eq!(
            resolve("down", true, Scope::Text),
            KeyHit::Motion(Motion::Down)
        );
        for (key, verb) in [
            ("/", Verb::Insert),
            (".", Verb::Repeat),
            ("y", Verb::Copy),
            ("p", Verb::Paste),
            ("d", Verb::Delete),
            ("u", Verb::Undo),
            ("r", Verb::Rename),
        ] {
            assert_eq!(
                resolve(key, false, Scope::List),
                KeyHit::Verb(verb),
                "{key} with vim off"
            );
            assert_eq!(
                resolve(key, true, Scope::List),
                KeyHit::Verb(verb),
                "{key} with vim on"
            );
        }
    }

    #[test]
    fn running_rows_replace_the_description() {
        let build = command(CommandId::BuildAssets);
        assert_eq!(
            palette_detail(&build, true, "Running `npx grunt build`…", false, ""),
            "Running `npx grunt build`…"
        );
        let export = command(CommandId::Export);
        assert_eq!(
            palette_detail(&export, false, "", true, "Exporting 1 page(s) to web/…"),
            "Exporting 1 page(s) to web/…"
        );
        assert_eq!(palette_detail(&export, false, "", false, ""), export.detail);
    }

    #[test]
    fn empty_query_lists_every_palette_command_including_find() {
        let all = filter_commands("");
        assert_eq!(all.len(), registry().count());
        assert!(all.iter().any(|cmd| cmd.id == CommandId::Find));
        assert!(all.iter().any(|cmd| cmd.title == "Find in site"));
        assert!(all.iter().all(|cmd| cmd.id != CommandId::Open));
    }
}
