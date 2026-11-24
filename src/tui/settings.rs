//! Preference pages on the settings sheet.
//!
//! Themes and Help are drawn by the sheet. Window holds the prefs both
//! frontends share: density and Vim keys, stored in `gui.json`.

use super::App;

pub const NAV: [&str; 3] = ["Themes", "Window", "Help"];
pub const LAST: u8 = 2;
pub const WINDOW: u8 = 1;
pub const HELP: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefId {
    Density,
    Vim,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pref {
    pub title: &'static str,
    pub keys: &'static str,
    pub id: PrefId,
}

pub fn window_items() -> &'static [Pref] {
    &[
        Pref {
            title: "Density",
            keys: "",
            id: PrefId::Density,
        },
        Pref {
            title: "Vim keys",
            keys: "",
            id: PrefId::Vim,
        },
    ]
}

pub fn value(app: &App, id: PrefId) -> String {
    match id {
        PrefId::Density => {
            if app.density_compact {
                "compact".to_string()
            } else {
                "comfortable".to_string()
            }
        }
        PrefId::Vim => {
            if app.vim_keys {
                "on".to_string()
            } else {
                "off".to_string()
            }
        }
    }
}

pub fn move_row(app: &mut App, delta: isize) {
    let count = window_items().len();
    if count == 0 {
        return;
    }
    let last = count - 1;
    let next = match delta {
        isize::MAX => last,
        isize::MIN => 0,
        step => {
            let cur = app.settings_row.min(last) as isize;
            (cur + step).clamp(0, last as isize) as usize
        }
    };
    app.settings_row = next;
    app.settings_follow = true;
}

pub fn activate(app: &mut App) {
    let Some(row) = window_items().get(app.settings_row) else {
        return;
    };
    match row.id {
        PrefId::Density => {
            app.density_compact = !app.density_compact;
            app.save_gui_window();
        }
        PrefId::Vim => {
            app.vim_keys = !app.vim_keys;
            app.save_gui_window();
        }
    }
}
