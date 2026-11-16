//! GUI undo history. The TUI keeps its own stacks.
//!
//! Decision 9: one history type for the GUI, depth 20, session only.
//! A snapshot is the whole `Site` from before a committed edit.

use crate::model::Site;

/// Oldest entries fall off the front once the stack is full.
pub const GUI_UNDO_DEPTH: usize = 20;

#[derive(Clone, Debug, Default)]
pub struct GuiUndoHistory {
    undo: Vec<Site>,
    redo: Vec<Site>,
}

impl GuiUndoHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Remember `before` and drop the redo branch.
    pub fn push(&mut self, before: Site) {
        self.redo.clear();
        self.undo.push(before);
        if self.undo.len() > GUI_UNDO_DEPTH {
            self.undo.remove(0);
        }
    }

    /// Swap `current` for the previous snapshot. The displaced site can be redone.
    pub fn undo(&mut self, current: Site) -> Option<Site> {
        let previous = self.undo.pop()?;
        self.redo.push(current);
        if self.redo.len() > GUI_UNDO_DEPTH {
            self.redo.remove(0);
        }
        Some(previous)
    }

    pub fn redo(&mut self, current: Site) -> Option<Site> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        if self.undo.len() > GUI_UNDO_DEPTH {
            self.undo.remove(0);
        }
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site_named(name: &str) -> Site {
        let mut site = Site::starter();
        site.name = name.to_string();
        site
    }

    #[test]
    fn depth_stops_at_twenty_and_undo_walks_back() {
        let mut history = GuiUndoHistory::new();
        for n in 0..25 {
            history.push(site_named(&format!("s{n}")));
        }
        assert_eq!(history.undo_len(), GUI_UNDO_DEPTH);
        assert_eq!(history.undo[0].name, "s5");
        assert_eq!(history.undo[19].name, "s24");

        let mut current = site_named("now");
        for expected in (5..25).rev() {
            current = history.undo(current).expect("undo entry");
            assert_eq!(current.name, format!("s{expected}"));
        }
        assert!(history.undo(current).is_none());
        assert_eq!(history.redo_len(), GUI_UNDO_DEPTH);
    }

    #[test]
    fn redo_restores_the_displaced_site_until_a_new_edit() {
        let mut history = GuiUndoHistory::new();
        history.push(site_named("first"));
        let undone = history.undo(site_named("second")).unwrap();
        assert_eq!(undone.name, "first");
        let redone = history.redo(undone).unwrap();
        assert_eq!(redone.name, "second");
        history.push(site_named("branch"));
        assert_eq!(history.redo_len(), 0);
        assert!(history.undo(site_named("later")).unwrap().name == "branch");
    }
}
