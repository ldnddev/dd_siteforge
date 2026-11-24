//! Key names the shell handles.
//!
//! Jared, Nov 16 (J4): Ctrl+\ toggles the Sidebar and Ctrl+Alt+\ toggles the
//! Inspector. Ctrl+B stays free because that chord is Build CSS/JS.

pub const TOGGLE_SIDEBAR_KEY: &str = "ctrl-\\";
pub const TOGGLE_INSPECTOR_KEY: &str = "ctrl-alt-\\";

/// Kbd hint on the "Show sidebar" tooltip.
pub const SIDEBAR_KBD: &str = "Ctrl \\";
/// Kbd hint on the "Show inspector" tooltip.
pub const INSPECTOR_KBD: &str = "Ctrl Alt \\";

pub const NEW_SITE_KEY: &str = "ctrl-shift-n";
#[cfg(test)]
pub const ADD_PAGE_KEY: &str = "ctrl-n";
/// F2 opens the same Settings panel as F1.
#[cfg(test)]
pub const THEME_KEY: &str = "f2";
pub const REDO_KEY: &str = "ctrl-r";
#[cfg(test)]
pub const QUIT_KEY: &str = "ctrl-q";
pub const OPEN_KEY: &str = "ctrl-o";

pub fn show_sidebar_tooltip() -> String {
    format!("Show sidebar · {SIDEBAR_KBD}")
}

pub fn show_inspector_tooltip() -> String {
    format!("Show inspector · {INSPECTOR_KBD}")
}
