//! Headless GUI editor. The window calls this; tests do too.
//!
//! Drafts are `EditFormState` values. A valid draft commits through
//! `apply_edit_form_to_component`. An invalid draft stays on its node.
//! Structural edits are live: they snapshot the site, then Ctrl+Z restores it.

use std::collections::BTreeMap;

use crate::component_kind::{ComponentKind, InsertRegion};
use crate::controls::{MarkdownInsert, apply_markdown_insert};
use crate::cursor::{self, Cursor};
use crate::editform::{EditFormState, FieldKind, SubChange};
use crate::edits;
use crate::history::GuiUndoHistory;
use crate::model::{Site, slug_from_title};

#[derive(Clone, Debug)]
struct DrillFrame {
    parent: EditFormState,
    field_id: String,
    item_idx: usize,
    /// Item as it was before the drill, so Esc can put it back.
    original: EditFormState,
}

#[derive(Clone, Debug)]
pub struct Draft {
    pub state: EditFormState,
    pub cursor: Cursor,
    drill: Vec<DrillFrame>,
    /// Field ids that failed the last explicit commit.
    pub invalid: Vec<String>,
    /// One previous value for the field last typed in. Ctrl+Z uses it first.
    text_undo: Option<(String, String)>,
    pub expanded: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Commit {
    Applied,
    /// "{label} is required." The draft stays and the checkpoint must not run.
    Blocked(String),
    /// Drilled item written back to its parent. The site is unchanged.
    ItemSaved,
    Failed(String),
}

#[derive(Clone)]
pub struct GuiEdit {
    history: GuiUndoHistory,
    drafts: BTreeMap<String, Draft>,
    active: Option<String>,
    pub last_insert: Option<ComponentKind>,
    /// Set after a delete so the toast can offer Undo.
    pub delete_toast: Option<String>,
}

impl Default for GuiEdit {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiEdit {
    pub fn new() -> Self {
        Self {
            history: GuiUndoHistory::new(),
            drafts: BTreeMap::new(),
            active: None,
            last_insert: None,
            delete_toast: None,
        }
    }

    pub fn undo_len(&self) -> usize {
        self.history.undo_len()
    }

    pub fn redo_len(&self) -> usize {
        self.history.redo_len()
    }

    pub fn active_key(&self) -> Option<&str> {
        self.active.as_deref()
    }

    pub fn active(&self) -> Option<&Draft> {
        self.active.as_ref().and_then(|key| self.drafts.get(key))
    }

    pub fn draft(&self, key: &str) -> Option<&Draft> {
        self.drafts.get(key)
    }

    pub fn invalid_keys(&self) -> impl Iterator<Item = &str> {
        self.drafts.iter().filter_map(|(key, draft)| {
            if draft.invalid.is_empty() {
                None
            } else {
                Some(key.as_str())
            }
        })
    }

    pub fn has_invalid(&self) -> bool {
        self.drafts.values().any(|draft| !draft.invalid.is_empty())
    }

    pub fn activate(&mut self, key: impl Into<String>) -> bool {
        let key = key.into();
        if self.drafts.contains_key(&key) {
            self.active = Some(key);
            true
        } else {
            false
        }
    }

    pub fn back_label(&self) -> Option<String> {
        let draft = self.active()?;
        let frame = draft.drill.last()?;
        let label = frame
            .parent
            .form
            .fields
            .iter()
            .find(|field| field.id == frame.field_id)
            .map(|field| field.label)
            .unwrap_or("Item");
        Some(format!(
            "‹ {label} · {} · Ctrl+S · Esc",
            frame.parent.form.title
        ))
    }

    pub fn open(&mut self, key: impl Into<String>, cursor: Cursor, fresh: EditFormState) {
        let key = key.into();
        self.drafts.entry(key.clone()).or_insert(Draft {
            state: fresh,
            cursor,
            drill: Vec::new(),
            invalid: Vec::new(),
            text_undo: None,
            expanded: false,
        });
        self.active = Some(key);
    }

    pub fn select_item(&mut self, index: usize) {
        let Some(draft) = self.active_mut() else {
            return;
        };
        let Some(id) = focused_subform_id(&draft.state) else {
            return;
        };
        draft.state.selected_sub_item.insert(id, index);
    }

    pub fn focus_field(&mut self, id: &str) -> bool {
        let Some(draft) = self.active_mut() else {
            return false;
        };
        let Some(index) = draft
            .state
            .form
            .fields
            .iter()
            .position(|field| field.id == id)
        else {
            return false;
        };
        draft.state.focused_field = index;
        true
    }

    pub fn set_field(&mut self, id: &str, value: impl Into<String>) -> bool {
        let value = value.into();
        let Some(draft) = self.active_mut() else {
            return false;
        };
        let previous = draft.state.get(id).to_string();
        if previous == value {
            return true;
        }
        draft.text_undo = Some((id.to_string(), previous));
        draft.state.set(id, value);
        draft.invalid.retain(|existing| existing != id);
        true
    }

    pub fn type_char(&mut self, ch: char) -> bool {
        let Some(id) = self.focused_text_id() else {
            return false;
        };
        let current = self
            .active()
            .map(|draft| draft.state.get(&id).to_string())
            .unwrap_or_default();
        let next = if ch == '\n' {
            format!("{current}\n")
        } else {
            format!("{current}{ch}")
        };
        self.set_field(&id, next)
    }

    pub fn backspace_field(&mut self) -> bool {
        let Some(id) = self.focused_text_id() else {
            return false;
        };
        let current = self
            .active()
            .map(|draft| draft.state.get(&id).to_string())
            .unwrap_or_default();
        let mut next = current;
        next.pop();
        self.set_field(&id, next)
    }

    pub fn insert_markdown(&mut self, insert: MarkdownInsert) -> bool {
        let Some(id) = self.focused_text_id() else {
            return false;
        };
        let Some(draft) = self.active() else {
            return false;
        };
        let kind_is_markdown = draft
            .state
            .focused()
            .is_some_and(|field| matches!(field.kind, FieldKind::Textarea { .. }));
        if !kind_is_markdown {
            return false;
        }
        let next = apply_markdown_insert(draft.state.get(&id), insert);
        self.set_field(&id, next)
    }

    pub fn focused_is_textarea(&self) -> bool {
        self.active().is_some_and(|draft| {
            draft.state.focused().is_some_and(|field| {
                matches!(field.kind, crate::editform::FieldKind::Textarea { .. })
            })
        })
    }

    pub fn toggle_expanded(&mut self) -> bool {
        let Some(draft) = self.active_mut() else {
            return false;
        };
        let is_markdown = draft
            .state
            .focused()
            .is_some_and(|field| matches!(field.kind, FieldKind::Textarea { .. }));
        if !is_markdown {
            return false;
        }
        draft.expanded = !draft.expanded;
        true
    }

    pub fn cycle_enum(&mut self, forward: bool) {
        if let Some(draft) = self.active_mut() {
            let id = draft.state.focused().map(|field| field.id.to_string());
            let previous = id.as_ref().map(|id| draft.state.get(id).to_string());
            draft.state.cycle_enum(forward);
            if let (Some(id), Some(previous)) = (id, previous) {
                let now = draft.state.get(&id).to_string();
                if now != previous {
                    draft.text_undo = Some((id.clone(), previous));
                    draft.invalid.retain(|existing| existing != &id);
                }
            }
        }
    }

    pub fn focus_next(&mut self) {
        if let Some(draft) = self.active_mut() {
            draft.state.focus_next();
        }
    }

    pub fn focus_prev(&mut self) {
        if let Some(draft) = self.active_mut() {
            draft.state.focus_prev();
        }
    }

    pub fn add_item(&mut self) -> SubChange {
        self.sub_change(|state, id| state.add_selected_sub_item(&id))
    }

    pub fn remove_item(&mut self) -> SubChange {
        self.sub_change(|state, id| state.remove_selected_sub_item(&id))
    }

    pub fn move_item(&mut self, down: bool) -> SubChange {
        self.sub_change(|state, id| state.move_selected_sub_item(&id, down))
    }

    pub fn drag_item(&mut self, from: usize, to: usize) -> bool {
        let Some(draft) = self.active_mut() else {
            return false;
        };
        let Some(id) = focused_subform_id(&draft.state) else {
            return false;
        };
        draft.state.drag_sub_item(&id, from, to)
    }

    pub fn drill(&mut self) -> Result<(), String> {
        let Some(draft) = self.active_mut() else {
            return Err("Nothing selected.".to_string());
        };
        let Some(field_id) = focused_subform_id(&draft.state) else {
            return Err("Nothing selected.".to_string());
        };
        let selected = draft
            .state
            .selected_sub_item
            .get(&field_id)
            .copied()
            .unwrap_or(0);
        let items_len = draft
            .state
            .sub_state
            .get(&field_id)
            .map(|items| items.len())
            .unwrap_or(0);
        if selected >= items_len {
            return Err("Nothing selected.".to_string());
        }
        let original = draft.state.sub_state[&field_id][selected].clone();
        let item = original.clone();
        draft.drill.push(DrillFrame {
            parent: draft.state.clone(),
            field_id,
            item_idx: selected,
            original,
        });
        draft.state = item;
        draft.expanded = false;
        draft.text_undo = None;
        Ok(())
    }

    pub fn drill_cancel(&mut self) -> bool {
        let Some(draft) = self.active_mut() else {
            return false;
        };
        let Some(frame) = draft.drill.pop() else {
            return false;
        };
        draft.state = frame.parent;
        if let Some(items) = draft.state.sub_state.get_mut(&frame.field_id) {
            if frame.item_idx < items.len() {
                items[frame.item_idx] = frame.original;
            }
        }
        true
    }

    /// Ctrl+S while drilled: write the item back, or refuse a required field.
    pub fn save_item(&mut self) -> Commit {
        let Some(draft) = self.active_mut() else {
            return Commit::Failed("Nothing selected.".to_string());
        };
        if draft.drill.is_empty() {
            return Commit::Failed("Not editing an item.".to_string());
        }
        if let Some(message) = required_message(&draft.state) {
            draft.invalid = draft
                .state
                .empty_required_fields()
                .into_iter()
                .map(|(_, id, _)| id.to_string())
                .collect();
            return Commit::Blocked(message);
        }
        let frame = draft.drill.pop().expect("drill");
        let item = draft.state.clone();
        draft.state = frame.parent;
        if let Some(items) = draft.state.sub_state.get_mut(&frame.field_id) {
            if frame.item_idx < items.len() {
                items[frame.item_idx] = item;
            } else {
                items.push(item);
            }
        }
        draft.invalid.clear();
        draft.text_undo = None;
        Commit::ItemSaved
    }

    /// Commit a valid draft onto `site`. An invalid draft blocks the write.
    pub fn commit(&mut self, site: &mut Site) -> Commit {
        if self.active().is_some_and(|draft| !draft.drill.is_empty()) {
            return self.save_item();
        }
        let Some(key) = self.active.clone() else {
            return Commit::Applied;
        };
        let Some(draft) = self.drafts.get(&key) else {
            return Commit::Applied;
        };
        if let Some(message) = required_message(&draft.state) {
            if let Some(draft) = self.drafts.get_mut(&key) {
                let first = draft
                    .state
                    .empty_required_fields()
                    .first()
                    .map(|item| item.0);
                draft.invalid = draft
                    .state
                    .empty_required_fields()
                    .into_iter()
                    .map(|(_, id, _)| id.to_string())
                    .collect();
                if let Some(index) = first {
                    draft.state.focused_field = index;
                }
            }
            return Commit::Blocked(message);
        }
        let draft = self.drafts.get(&key).expect("draft").clone();
        let before = site.clone();
        match cursor::apply_edit_form_to_component(site, &draft.cursor, &draft.state) {
            Ok(()) => {
                self.history.push(before);
                self.drafts.remove(&key);
                if self.active.as_ref() == Some(&key) {
                    self.active = None;
                }
                Commit::Applied
            }
            Err(err) => {
                if let Some(draft) = self.drafts.get_mut(&key) {
                    draft.invalid.clear();
                }
                Commit::Failed(format!("Save failed: {err}"))
            }
        }
    }

    /// Drop the active draft. A drilled item is cancelled first.
    pub fn discard(&mut self) -> bool {
        if self.active().is_some_and(|draft| draft.expanded) {
            if let Some(draft) = self.active_mut() {
                draft.expanded = false;
            }
            return true;
        }
        if self.drill_cancel() {
            return true;
        }
        if let Some(key) = self.active.take() {
            self.drafts.remove(&key);
            return true;
        }
        false
    }

    /// Leaving a node commits a valid draft and keeps an invalid one.
    pub fn leave(&mut self, site: &mut Site) -> Commit {
        if self.active().is_none() {
            return Commit::Applied;
        }
        if self
            .active()
            .is_some_and(|draft| required_message(&draft.state).is_some())
        {
            if let Some(draft) = self.active_mut() {
                draft.invalid = draft
                    .state
                    .empty_required_fields()
                    .into_iter()
                    .map(|(_, id, _)| id.to_string())
                    .collect();
            }
            let message = self
                .active()
                .and_then(|draft| required_message(&draft.state))
                .unwrap_or_else(|| "This field is required.".to_string());
            self.active = None;
            return Commit::Blocked(message);
        }
        self.commit(site)
    }

    pub fn undo(&mut self, site: &mut Site) -> Result<String, String> {
        if let Some(draft) = self.active_mut() {
            if let Some((id, previous)) = draft.text_undo.take() {
                draft.state.set(&id, previous);
                return Ok("Undid last change.".to_string());
            }
        }
        match self.history.undo(site.clone()) {
            Some(previous) => {
                *site = previous;
                self.delete_toast = None;
                Ok("Undid last change.".to_string())
            }
            None => Err("Nothing to undo.".to_string()),
        }
    }

    pub fn redo(&mut self, site: &mut Site) -> Result<String, String> {
        match self.history.redo(site.clone()) {
            Some(next) => {
                *site = next;
                Ok("Redid last change.".to_string())
            }
            None => Err("Nothing to redo.".to_string()),
        }
    }

    pub fn insert_component(
        &mut self,
        site: &mut Site,
        page: usize,
        node: usize,
        column: usize,
        selected: usize,
        kind: ComponentKind,
        region: InsertRegion,
    ) -> Result<String, String> {
        if !kind.allowed_in(region) {
            let message = if region == InsertRegion::Site {
                "Insert is not available on Site settings.".to_string()
            } else {
                format!("{} can only be inserted on a page.", kind.label())
            };
            return Err(message);
        }
        let before = site.clone();
        let message = match kind {
            ComponentKind::Hero => {
                if region != InsertRegion::Page {
                    return Err(format!("{} can only be inserted on a page.", kind.label()));
                }
                let idx = edits::insert_page_hero(site, page, Some(node))?;
                format!("Inserted dd-hero at position {}.", idx + 1)
            }
            ComponentKind::Section => {
                let idx = edits::insert_page_section(site, page, Some(node))?;
                format!("Inserted dd-section at position {}.", idx + 1)
            }
            ComponentKind::Alert if region == InsertRegion::Header => {
                edits::place_header_alert(site)?;
                "Added dd-alert to the header slot.".to_string()
            }
            _ => {
                if region != InsertRegion::Page {
                    return Err(
                        "No header section available. Add a section first with '/'.".to_string()
                    );
                }
                let (_, message) =
                    edits::insert_page_component(site, page, node, column, selected, kind)?;
                message
            }
        };
        self.history.push(before);
        self.last_insert = Some(kind);
        Ok(message)
    }

    pub fn add_column(
        &mut self,
        site: &mut Site,
        page: usize,
        node: usize,
    ) -> Result<String, String> {
        let before = site.clone();
        let id = edits::add_page_column(site, page, node)?;
        self.history.push(before);
        Ok(format!("Added column to section. {id}"))
    }

    pub fn delete_component(
        &mut self,
        site: &mut Site,
        page: usize,
        node: usize,
        column: usize,
        component: usize,
    ) -> Result<String, String> {
        let before = site.clone();
        edits::remove_page_component(site, page, node, column, component)?;
        self.history.push(before);
        let message = "Deleted component.".to_string();
        self.delete_toast = Some(message.clone());
        Ok(message)
    }

    pub fn move_component(
        &mut self,
        site: &mut Site,
        page: usize,
        node: usize,
        column: usize,
        component: usize,
        delta: isize,
    ) -> Result<usize, String> {
        let before = site.clone();
        let dest = edits::move_page_component(site, page, node, column, component, delta)?;
        self.history.push(before);
        Ok(dest)
    }

    pub fn drag_component(
        &mut self,
        site: &mut Site,
        page: usize,
        node: usize,
        column: usize,
        from: usize,
        to: usize,
    ) -> Result<usize, String> {
        let before = site.clone();
        let dest = edits::drag_page_component(site, page, node, column, from, to)?;
        self.history.push(before);
        Ok(dest)
    }

    fn active_mut(&mut self) -> Option<&mut Draft> {
        let key = self.active.clone()?;
        self.drafts.get_mut(&key)
    }

    fn focused_text_id(&self) -> Option<String> {
        let draft = self.active()?;
        let field = draft.state.focused()?;
        match field.kind {
            FieldKind::Text { .. } | FieldKind::Url { .. } | FieldKind::Textarea { .. } => {
                Some(field.id.to_string())
            }
            _ => None,
        }
    }

    fn sub_change(
        &mut self,
        op: impl FnOnce(&mut EditFormState, String) -> SubChange,
    ) -> SubChange {
        let Some(draft) = self.active_mut() else {
            return SubChange::Ignored;
        };
        let Some(id) = focused_subform_id(&draft.state) else {
            return SubChange::Ignored;
        };
        op(&mut draft.state, id)
    }
}

fn focused_subform_id(state: &EditFormState) -> Option<String> {
    let field = state.focused()?;
    matches!(field.kind, FieldKind::SubForm { .. }).then(|| field.id.to_string())
}

fn required_message(state: &EditFormState) -> Option<String> {
    state
        .empty_required_fields()
        .first()
        .map(|(_, _, label)| format!("{label} is required."))
}

impl GuiEdit {
    /// Rename uses the TUI rule: title changes, and the slug follows unless locked.
    pub fn rename_page_at(
        &mut self,
        site: &mut Site,
        page: usize,
        title: &str,
    ) -> Result<String, String> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err("Title required.".to_string());
        }
        let current = site
            .pages
            .get(page)
            .ok_or_else(|| "Page no longer exists.".to_string())?;
        if current.head.title == trimmed {
            return Ok(format!("Renamed page: {trimmed}"));
        }
        let before = site.clone();
        let page_ref = site
            .pages
            .get_mut(page)
            .ok_or_else(|| "Page no longer exists.".to_string())?;
        page_ref.head.title = trimmed.to_string();
        if !page_ref.slug_locked {
            page_ref.slug = slug_from_title(trimmed);
        }
        let message = format!("Renamed page: {}", page_ref.head.title);
        self.history.push(before);
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cursor::{
        component_to_form_state, footer_to_form_state, hero_to_form_state, page_head_to_form_state,
        site_to_form_state,
    };
    use crate::model::{PageNode, SectionComponent};

    fn card<'a>(site: &'a Site) -> &'a crate::model::DdCard {
        let PageNode::Section(section) = &site.pages[0].nodes[1] else {
            panic!("section");
        };
        match &section.columns[0].components[0] {
            SectionComponent::Card(card) => card,
            other => panic!("card, got {other:?}"),
        }
    }

    #[test]
    fn ctrl_s_blocks_an_invalid_draft_and_keeps_it() {
        let mut site = Site::starter();
        let mut edit = GuiEdit::new();
        let state = hero_to_form_state(match &site.pages[0].nodes[0] {
            PageNode::Hero(hero) => hero,
            _ => panic!("hero"),
        });
        edit.open("hero", Cursor::PageHero { page: 0, node: 0 }, state);
        assert!(edit.set_field("parent_title", ""));
        match edit.commit(&mut site) {
            Commit::Blocked(message) => assert_eq!(message, "Title is required."),
            other => panic!("expected block, got {other:?}"),
        }
        assert_eq!(
            match &site.pages[0].nodes[0] {
                PageNode::Hero(hero) => hero.parent_title.as_str(),
                _ => "",
            },
            "Build with dd-framework"
        );
        assert!(edit.has_invalid());
        assert!(edit.draft("hero").is_some());
        edit.set_field("parent_title", "Kept");
        assert!(matches!(edit.commit(&mut site), Commit::Applied));
        assert!(!edit.has_invalid());
        assert_eq!(
            match &site.pages[0].nodes[0] {
                PageNode::Hero(hero) => hero.parent_title.as_str(),
                _ => "",
            },
            "Kept"
        );
    }

    #[test]
    fn text_undo_happens_before_the_site_stack() {
        let mut site = Site::starter();
        let original = site.name.clone();
        let mut edit = GuiEdit::new();
        edit.open(
            "head",
            Cursor::PageHead { page: 0 },
            page_head_to_form_state(&site.pages[0]),
        );
        edit.set_field("title", "Home!");
        assert_eq!(edit.undo(&mut site).unwrap(), "Undid last change.");
        assert_eq!(edit.active().unwrap().state.get("title"), "Home");
        assert_eq!(site.name, original);
        assert_eq!(edit.undo(&mut site).unwrap_err(), "Nothing to undo.");
    }

    #[test]
    fn every_field_kind_commits_through_the_shared_apply_path() {
        let mut site = Site::starter();
        let mut edit = GuiEdit::new();

        edit.open(
            "head",
            Cursor::PageHead { page: 0 },
            page_head_to_form_state(&site.pages[0]),
        );
        edit.set_field("title", "Text kind");
        assert!(matches!(edit.commit(&mut site), Commit::Applied));
        assert_eq!(site.pages[0].head.title, "Text kind");

        edit.open(
            "footer",
            Cursor::FooterRoot,
            footer_to_form_state(&site.footer),
        );
        edit.set_field("blurb", "A paragraph.");
        assert!(matches!(edit.commit(&mut site), Commit::Applied));
        assert_eq!(site.footer.blurb.as_deref(), Some("A paragraph."));

        edit.open("site", Cursor::Site, site_to_form_state(&site));
        edit.focus_field("pretty_urls");
        edit.cycle_enum(true);
        assert!(matches!(edit.commit(&mut site), Commit::Applied));
        assert!(site.pretty_urls);

        edits::insert_page_component(&mut site, 0, 1, 0, 0, ComponentKind::Card).unwrap();
        let cursor = Cursor::PageComponent {
            page: 0,
            node: 1,
            col: 0,
            comp: 0,
            items: Vec::new(),
        };
        let state = component_to_form_state(&site.pages[0].nodes[1].clone().section_component(0))
            .expect("card form");
        edit.open("card", cursor, state);
        edit.set_field("parent_width", "dd-u-1-2");
        edit.focus_field("items");
        assert_eq!(edit.add_item(), SubChange::Added);
        assert_eq!(edit.move_item(false), SubChange::MovedUp);
        assert_eq!(edit.remove_item(), SubChange::Removed);
        edit.focus_field("parent_type");
        edit.cycle_enum(true);
        assert!(matches!(edit.commit(&mut site), Commit::Applied));
        assert_eq!(card(&site).parent_width, "dd-u-1-2");

        edits::insert_page_component(&mut site, 0, 1, 0, 0, ComponentKind::Cta).unwrap();
        let cta = component_to_form_state(match &site.pages[0].nodes[1] {
            PageNode::Section(section) => &section.columns[0].components[1],
            _ => panic!("section"),
        })
        .unwrap();
        edit.open(
            "cta",
            Cursor::PageComponent {
                page: 0,
                node: 1,
                col: 0,
                comp: 1,
                items: Vec::new(),
            },
            cta,
        );
        edit.focus_field("parent_class");
        edit.cycle_enum(true);
        assert!(matches!(edit.commit(&mut site), Commit::Applied));

        edits::insert_page_component(&mut site, 0, 1, 0, 1, ComponentKind::Alternating).unwrap();
        let alt = component_to_form_state(match &site.pages[0].nodes[1] {
            PageNode::Section(section) => section.columns[0].components.last().unwrap(),
            _ => panic!("section"),
        })
        .unwrap();
        edit.open(
            "alt",
            Cursor::PageComponent {
                page: 0,
                node: 1,
                col: 0,
                comp: 2,
                items: Vec::new(),
            },
            alt,
        );
        edit.focus_field("items");
        edit.drill().expect("alternating item");
        edit.focus_field("media_kind");
        edit.set_field("media_kind", "oembed");
        edit.set_field("media_oembed_url", "https://example.com/watch");
        assert!(matches!(edit.commit(&mut site), Commit::ItemSaved));
        assert!(matches!(edit.commit(&mut site), Commit::Applied));

        edit.open(
            "hero",
            Cursor::PageHero { page: 0, node: 0 },
            hero_to_form_state(match &site.pages[0].nodes[0] {
                PageNode::Hero(hero) => hero,
                _ => panic!("hero"),
            }),
        );
        edit.set_field("media_kind", "image");
        edit.set_field("media_image_url", "assets/images/hero-2.jpg");
        edit.focus_field("parent_copy");
        assert!(edit.insert_markdown(MarkdownInsert::Bold));
        assert!(matches!(edit.commit(&mut site), Commit::Applied));
        match &site.pages[0].nodes[0] {
            PageNode::Hero(hero) => {
                assert_eq!(hero.parent_image_url, "assets/images/hero-2.jpg");
                assert!(hero.parent_copy.as_deref().unwrap_or("").contains("**"));
            }
            _ => panic!("hero"),
        }
    }

    #[test]
    fn undo_depth_twenty_on_live_inserts() {
        let mut site = Site::starter();
        let mut edit = GuiEdit::new();
        for _ in 0..25 {
            edit.insert_component(
                &mut site,
                0,
                1,
                0,
                0,
                ComponentKind::Spacer,
                InsertRegion::Page,
            )
            .unwrap();
        }
        assert_eq!(edit.undo_len(), 20);
        for _ in 0..20 {
            edit.undo(&mut site).unwrap();
        }
        assert_eq!(edit.undo(&mut site).unwrap_err(), "Nothing to undo.");
        let PageNode::Section(section) = &site.pages[0].nodes[1] else {
            panic!("section");
        };
        // 25 inserts, 20 undos: 5 spacers remain.
        assert_eq!(section.columns[0].components.len(), 5);
        edit.redo(&mut site).unwrap();
        let PageNode::Section(section) = &site.pages[0].nodes[1] else {
            panic!("section");
        };
        assert_eq!(section.columns[0].components.len(), 6);
    }

    trait SectionAt {
        fn section_component(&self, index: usize) -> SectionComponent;
    }

    impl SectionAt for PageNode {
        fn section_component(&self, index: usize) -> SectionComponent {
            match self {
                PageNode::Section(section) => section.columns[0].components[index].clone(),
                PageNode::Hero(_) => panic!("hero"),
            }
        }
    }
}
