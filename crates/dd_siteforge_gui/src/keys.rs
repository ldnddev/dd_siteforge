//! Key names the shell handles.
//!
//! J4 (dock-toggle shortcuts) is still open. Nothing is bound for it, and
//! Ctrl+B stays free because that chord is Build CSS/JS.

/// Unassigned on purpose. Do not point this at `ctrl-b`.
pub const DOCK_TOGGLE_KEY: Option<&str> = None;

pub const NEW_SITE_KEY: &str = "ctrl-shift-n";
pub const ADD_PAGE_KEY: &str = "ctrl-n";
pub const THEME_KEY: &str = "f2";
pub const REDO_KEY: &str = "ctrl-r";
pub const QUIT_KEY: &str = "ctrl-q";
pub const OPEN_KEY: &str = "ctrl-o";
