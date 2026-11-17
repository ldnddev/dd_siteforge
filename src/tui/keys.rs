//! Crossterm key events as core chords. The chord table decides what they do.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use dd_siteforge_core::commands::Chord;

/// Owned key name so a [`Chord`] can borrow it for [`dd_siteforge_core::commands::lookup`].
pub(super) struct OwnedChord {
    pub key: String,
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
}

impl OwnedChord {
    pub fn as_chord(&self) -> Chord<'_> {
        Chord {
            key: &self.key,
            control: self.control,
            alt: self.alt,
            shift: self.shift,
        }
    }
}

/// Translate one crossterm key. Letters are lowercase, with Shift as a flag.
/// Punctuation keeps the typed character (`?`, `:`, `/`) and does not also
/// set Shift, because that is how the core table spells those chords.
pub(super) fn event_chord(key: &KeyEvent) -> Option<OwnedChord> {
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let shift_mod = key.modifiers.contains(KeyModifiers::SHIFT);
    let (name, shift) = match key.code {
        KeyCode::Char(ch) if ch.is_ascii_alphabetic() => (
            ch.to_ascii_lowercase().to_string(),
            shift_mod || ch.is_ascii_uppercase(),
        ),
        KeyCode::Char(ch) => (ch.to_string(), false),
        KeyCode::F(n) => (format!("f{n}"), shift_mod),
        KeyCode::Delete => ("delete".to_string(), shift_mod),
        KeyCode::Backspace => ("backspace".to_string(), shift_mod),
        KeyCode::Enter => ("enter".to_string(), shift_mod),
        KeyCode::Esc => ("escape".to_string(), shift_mod),
        KeyCode::Tab => ("tab".to_string(), false),
        KeyCode::BackTab => ("tab".to_string(), true),
        KeyCode::Up => ("up".to_string(), shift_mod),
        KeyCode::Down => ("down".to_string(), shift_mod),
        KeyCode::Left => ("left".to_string(), shift_mod),
        KeyCode::Right => ("right".to_string(), shift_mod),
        KeyCode::Home => ("home".to_string(), shift_mod),
        KeyCode::End => ("end".to_string(), shift_mod),
        KeyCode::PageUp => ("pageup".to_string(), shift_mod),
        KeyCode::PageDown => ("pagedown".to_string(), shift_mod),
        _ => return None,
    };
    Some(OwnedChord {
        key: name,
        control,
        alt,
        shift,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;
    use dd_siteforge_core::commands::{
        BindingKind, Command, Hit, InputMode, Query, Surface, VimAction, lookup,
    };

    fn press(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    fn hit(key: &KeyEvent, mode: InputMode, vim: bool) -> Option<Hit> {
        let owned = event_chord(key).expect("chord");
        lookup(Query {
            surface: Surface::Tui,
            mode,
            vim,
            in_text: false,
            chord: owned.as_chord(),
        })
    }

    fn command(key: &KeyEvent, mode: InputMode) -> Command {
        match hit(key, mode, true) {
            Some(Hit::Command { command, .. }) => command,
            other => panic!("expected a command, got {other:?}"),
        }
    }

    #[test]
    fn legacy_ctrl_backslash_is_ctrl_4_and_means_regions() {
        let key = press(KeyCode::Char('4'), KeyModifiers::CONTROL);
        let owned = event_chord(&key).unwrap();
        assert_eq!(owned.key, "4");
        assert!(owned.control);
        assert!(!owned.alt);
        assert_eq!(command(&key, InputMode::Legacy), Command::ToggleRegions);
        assert_eq!(command(&key, InputMode::Tmux), Command::ToggleRegions);
        assert_eq!(command(&key, InputMode::Kitty), Command::ToggleIssues);
    }

    #[test]
    fn legacy_ctrl_alt_backslash_is_alt_ctrl_4() {
        let key = press(
            KeyCode::Char('4'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );
        let owned = event_chord(&key).unwrap();
        assert!(owned.control && owned.alt);
        assert_eq!(command(&key, InputMode::Legacy), Command::ToggleDetails);
        assert_eq!(command(&key, InputMode::Tmux), Command::ToggleDetails);
    }

    #[test]
    fn kitty_backslash_chords_toggle_the_panes() {
        let regions = press(KeyCode::Char('\\'), KeyModifiers::CONTROL);
        let details = press(
            KeyCode::Char('\\'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );
        assert_eq!(command(&regions, InputMode::Kitty), Command::ToggleRegions);
        assert_eq!(command(&details, InputMode::Kitty), Command::ToggleDetails);
        assert!(hit(&regions, InputMode::Legacy, true).is_none());
    }

    #[test]
    fn aliases_carry_a_one_time_hint() {
        let shift_b = press(KeyCode::Char('B'), KeyModifiers::SHIFT);
        match hit(&shift_b, InputMode::Tmux, true) {
            Some(Hit::Command {
                command: Command::BuildAssets,
                kind: BindingKind::Alias,
                hint: Some("Build is now Ctrl+B"),
            }) => {}
            other => panic!("{other:?}"),
        }
        let save = press(KeyCode::Char('s'), KeyModifiers::NONE);
        match hit(&save, InputMode::Legacy, false) {
            Some(Hit::Command {
                command: Command::Save,
                kind: BindingKind::Alias,
                hint: Some("Save is now Ctrl+S"),
            }) => {}
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn tmux_drops_ctrl_b_and_ctrl_shift_letters() {
        let ctrl_b = press(KeyCode::Char('b'), KeyModifiers::CONTROL);
        assert!(hit(&ctrl_b, InputMode::Tmux, true).is_none());
        assert_eq!(command(&ctrl_b, InputMode::Kitty), Command::BuildAssets);
        assert_eq!(command(&ctrl_b, InputMode::Legacy), Command::BuildAssets);

        let redo = press(
            KeyCode::Char('Z'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert!(hit(&redo, InputMode::Tmux, true).is_none());
        assert!(hit(&redo, InputMode::Legacy, true).is_none());
        assert_eq!(command(&redo, InputMode::Kitty), Command::Redo);

        let new_site = press(
            KeyCode::Char('N'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert!(hit(&new_site, InputMode::Tmux, true).is_none());
        assert!(hit(&new_site, InputMode::Legacy, true).is_none());
        assert_eq!(command(&new_site, InputMode::Kitty), Command::NewSite);

        let ctrl_r = press(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert_eq!(command(&ctrl_r, InputMode::Tmux), Command::Redo);
    }

    #[test]
    fn vim_setting_gates_letter_keys_from_the_adapter() {
        let delete = press(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(matches!(
            hit(&delete, InputMode::Legacy, true),
            Some(Hit::Vim(VimAction::Delete))
        ));
        assert!(hit(&delete, InputMode::Legacy, false).is_none());
        let row = press(KeyCode::Delete, KeyModifiers::NONE);
        assert_eq!(command(&row, InputMode::Legacy), Command::DeleteRow);
        assert_eq!(command(&row, InputMode::Tmux), Command::DeleteRow);
    }
}
