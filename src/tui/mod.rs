use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

pub(super) use crate::model::{PageNode, SectionColumn, Site};
pub(super) use crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
pub(super) use ratatui::layout::{Constraint, Direction, Layout, Rect};
pub(super) use ratatui::style::{Color, Modifier, Style};
pub(super) use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
};
pub(super) const AUTOSAVE_DEBOUNCE: std::time::Duration = std::time::Duration::from_secs(2);
pub(super) const TOAST_TTL: std::time::Duration = std::time::Duration::from_secs(5);
pub(super) const TEXTAREA_MAX_DISPLAY_ROWS: u16 = 35;
pub(super) const DOUBLE_CLICK_THRESHOLD_MS: u128 = 420;

/// One-level text undo for the open FormEdit field.
pub(super) struct FormTextUndo {
    pub(super) field_id: String,
    pub(super) value: String,
    pub(super) cursor_pos: usize,
    pub(super) selection_anchor: Option<usize>,
}

mod component_kind;
pub mod cursor;
mod details;
mod draw;
pub mod editform;
mod events;
mod find;
mod form_textarea;
mod grunt;
mod help;
mod modals;
mod open_site;
mod pages;
mod palette;
mod scrollbar;
#[cfg(test)]
mod tests;
mod theme;
mod tree;
mod util;

use component_kind::*;
use details::*;
use form_textarea::*;
use help::*;
use modals::*;
use scrollbar::*;
use theme::*;
use tree::*;
use util::*;

#[allow(dead_code)]
pub fn run_tui(site: Site, path: Option<PathBuf>) -> anyhow::Result<()> {
    let (theme, theme_source, load_warning) = AppTheme::load();
    let app = App::new(site, path, theme, theme_source, load_warning.clone());
    launch_app(app, load_warning)
}

pub fn run_tui_restored(
    site: Site,
    path: PathBuf,
    page: usize,
    tree_row: usize,
    region: &str,
) -> anyhow::Result<()> {
    let (theme, theme_source, load_warning) = AppTheme::load();
    let mut app = App::new(site, Some(path), theme, theme_source, load_warning.clone());
    app.restore_selection(page, tree_row, open_site::region_from_name(region));
    launch_app(app, load_warning)
}

pub fn run_tui_open_picker() -> anyhow::Result<()> {
    let (theme, theme_source, load_warning) = AppTheme::load();
    let mut app = App::new(
        Site::starter(),
        None,
        theme,
        theme_source,
        load_warning.clone(),
    );
    app.awaiting_site = true;
    app.open_site_picker();
    launch_app(app, load_warning)
}

fn launch_app(mut app: App, load_warning: Option<String>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    if let Some(msg) = load_warning {
        app.push_toast(ToastLevel::Warning, msg);
    }
    let run_res = app.run(&mut terminal);
    app.persist_session();

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableBracketedPaste,
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    run_res
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum SidebarSection {
    Regions,
    Pages,
    Layouts,
    Details,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectedRegion {
    Page,
    Site,
    Header,
    Footer,
}

/// F1 Help / F2 Theme. Lives beside `modal`, never inside it, so Esc
/// cannot drop ImagePicker or a paused FormEdit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Overlay {
    Help { scroll: u16 },
    Theme { scroll: u16 },
}

pub(super) struct App {
    site: Site,
    theme: AppTheme,
    theme_source: String,
    header_copy: String,
    selected_page: usize,
    selected_node: usize,
    selected_tree_row: usize,
    selected_column: usize,
    selected_component: usize,
    selected_nested_item: usize,
    selected_sidebar_section: SidebarSection,
    selected_region: SelectedRegion,
    selected_header_section: usize,
    selected_header_column: usize,
    selected_header_component: usize,
    /// True when the `[HEAD]` row is the active tree selection. Needed
    /// because page-head has no `selected_*` index of its own; without this
    /// flag, `sync_tree_row_with_selection` would always fall back to the
    /// first Hero/Section row and make `[HEAD]` unreachable via j/k.
    page_head_selected: bool,
    /// True when the header `dd-alert` slot row is selected.
    header_alert_selected: bool,
    /// Session trash — deleted pages pushed here for `u` undo.
    /// Not persisted. Capped at 20 entries (oldest drops off).
    deleted_pages: Vec<crate::model::Page>,
    /// Site snapshots taken before structural tree edits. `u` in Layout pops.
    /// Capped at 20.
    undo_stack: Vec<crate::model::Site>,
    /// Snapshots undone with `u`; `Ctrl+R` pops. Cleared on a new `push_undo`.
    redo_stack: Vec<crate::model::Site>,
    /// Last successful insert kind, repeated with `.`.
    last_insert_kind: Option<ComponentKind>,
    /// Background `lando grunt build` / `npx grunt build`.
    build_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    /// True until the OpenSite picker loads a JSON file. Blocks layout editing.
    awaiting_site: bool,
    /// Copied tree grain for `y` / `p`. Session-only, not persisted.
    clipboard: Option<Clipboard>,
    /// Index of a page inserted by the add-page flow and not yet confirmed
    /// with Ctrl+S on the HEAD form. Esc discards it.
    creating_page_idx: Option<usize>,
    /// Ephemeral bottom-right notifications; expire ~5s after `shown_at`.
    toasts: Vec<Toast>,
    /// True when in-memory site differs from `last_saved_json`.
    dirty: bool,
    /// Instant of the first mutation since `last_saved_json` was synced.
    /// `None` while clean.
    dirty_since: Option<std::time::Instant>,
    /// JSON snapshot of the site at the most recent successful disk write.
    /// Used both for dirty detection and for skipping no-op autosaves.
    last_saved_json: String,
    list_area: Rect,
    details_area: Rect,
    details_scroll_row: usize,
    /// Draw-time hit segments for the Details pane, one row per painted line.
    /// Each segment is `(x0, x1, col, comp)` in content-cell coordinates.
    details_hits: Vec<Vec<(usize, usize, usize, usize)>>,
    details_scrollbar_track: ScrollbarTrack,
    layout_scrollbar_track: ScrollbarTrack,
    help_scrollbar_track: ScrollbarTrack,
    theme_scrollbar_track: ScrollbarTrack,
    form_scrollbar_track: std::cell::RefCell<ScrollbarTrack>,
    scrollbar_drag: Option<ScrollbarDrag>,
    regions_area: Rect,
    pages_area: Rect,
    pages_list_state: ListState,
    layout_list_state: ListState,
    last_mouse_click: Option<(u16, u16, std::time::Instant)>,
    path: Option<PathBuf>,
    preview_server: Option<crate::serve::StaticServer>,
    should_quit: bool,
    modal: Option<Modal>,
    component_kind: ComponentKind,
    overlay: Option<Overlay>,
    theme_editor: Option<ldnddev_theme::ThemeEditor>,
    /// Maximum legal overlay scroll, recomputed every render from the
    /// current overlay area + line count so key/wheel handlers can clamp.
    overlay_scroll_max: u16,
    theme_status: Option<String>,
    /// Per-frame cache of (field_idx, input_area_rect) for whichever
    /// multi-field modal is currently rendered. Click-to-focus lookups
    /// search this cache; render writes it. Empty when no eligible modal
    /// is open.
    modal_field_areas: std::cell::RefCell<Vec<(usize, Rect)>>,
    /// FormEdit modal that was paused when the image picker opened on top
    /// of it. Restored when the picker closes (Esc or after a commit).
    paused_form_edit_modal: Option<Modal>,
    /// When true, FormEdit paints the focused textarea full-size. Esc
    /// returns to the compact form; Ctrl+S still saves the component.
    form_textarea_expanded: bool,
    /// Last text-field mutation in the open FormEdit, for Ctrl+Z.
    form_text_undo: Option<FormTextUndo>,
    /// True while the pointer is dragging a text selection inside FormEdit.
    form_text_drag: bool,
    /// Draw-time hit targets for `[Expand]` on textarea field labels.
    form_expand_hits: std::cell::RefCell<Vec<(usize, Rect)>>,
    /// Draw-time hit targets for Browse on file-picker URL fields.
    form_browse_hits: std::cell::RefCell<Vec<(usize, Rect)>>,
    expanded_sections: HashSet<(usize, usize)>,
    expanded_accordion_items: HashSet<(usize, usize, usize, usize)>,
    expanded_alternating_items: HashSet<(usize, usize, usize, usize)>,
    expanded_card_items: HashSet<(usize, usize, usize, usize)>,
    expanded_filmstrip_items: HashSet<(usize, usize, usize, usize)>,
    expanded_milestones_items: HashSet<(usize, usize, usize, usize)>,
    expanded_slider_items: HashSet<(usize, usize, usize, usize)>,
    expanded_tabs_items: HashSet<(usize, usize, usize, usize)>,
    expanded_timeline_items: HashSet<(usize, usize, usize, usize)>,
    expanded_data_table_rows: HashSet<(usize, usize, usize, usize)>,
    header_column_expanded: bool,
    /// Page ids whose children are hidden in `[2] Pages`. Default expanded.
    collapsed_page_ids: HashSet<String>,
}

impl App {
    pub(super) fn new(
        mut site: Site,
        path: Option<PathBuf>,
        theme: AppTheme,
        theme_source: String,
        theme_status: Option<String>,
    ) -> Self {
        for page in &mut site.pages {
            ensure_page_section_ids(page);
        }
        let last_saved_json = serde_json::to_string(&site).unwrap_or_default();

        let header_copy = choose_header_copy(&theme.header_quotes);

        let mut app = Self {
            site,
            theme,
            theme_source,
            header_copy,
            selected_page: 0,
            selected_node: 0,
            selected_tree_row: 0,
            selected_column: 0,
            selected_component: 0,
            selected_nested_item: 0,
            selected_sidebar_section: SidebarSection::Layouts,
            selected_region: SelectedRegion::Page,
            selected_header_section: 0,
            selected_header_column: 0,
            selected_header_component: 0,
            page_head_selected: false,
            header_alert_selected: false,
            deleted_pages: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_insert_kind: None,
            build_rx: None,
            awaiting_site: false,
            clipboard: None,
            creating_page_idx: None,
            toasts: Vec::new(),
            list_area: Rect::default(),
            details_area: Rect::default(),
            details_scroll_row: 0,
            details_hits: Vec::new(),
            details_scrollbar_track: ScrollbarTrack::default(),
            layout_scrollbar_track: ScrollbarTrack::default(),
            help_scrollbar_track: ScrollbarTrack::default(),
            theme_scrollbar_track: ScrollbarTrack::default(),
            form_scrollbar_track: std::cell::RefCell::new(ScrollbarTrack::default()),
            scrollbar_drag: None,
            regions_area: Rect::default(),
            pages_area: Rect::default(),
            pages_list_state: ListState::default(),
            layout_list_state: ListState::default(),
            last_mouse_click: None,
            path,
            preview_server: None,
            should_quit: false,
            modal: None,
            component_kind: ComponentKind::Banner,
            overlay: None,
            theme_editor: None,
            overlay_scroll_max: 0,
            theme_status,
            modal_field_areas: std::cell::RefCell::new(Vec::new()),
            paused_form_edit_modal: None,
            form_textarea_expanded: false,
            form_text_undo: None,
            form_text_drag: false,
            form_expand_hits: std::cell::RefCell::new(Vec::new()),
            form_browse_hits: std::cell::RefCell::new(Vec::new()),
            expanded_sections: HashSet::new(),
            expanded_accordion_items: HashSet::new(),
            expanded_alternating_items: HashSet::new(),
            expanded_card_items: HashSet::new(),
            expanded_filmstrip_items: HashSet::new(),
            expanded_milestones_items: HashSet::new(),
            expanded_slider_items: HashSet::new(),
            expanded_tabs_items: HashSet::new(),
            expanded_timeline_items: HashSet::new(),
            expanded_data_table_rows: HashSet::new(),
            header_column_expanded: true,
            collapsed_page_ids: HashSet::new(),
            dirty: false,
            dirty_since: None,
            last_saved_json,
        };

        if let Some(p) = app.path.as_ref() {
            let backup = backup_path_for(p);
            if backup.exists() && p.exists() {
                if let (Ok(main), Ok(bak)) =
                    (std::fs::read_to_string(p), std::fs::read_to_string(&backup))
                {
                    if main != bak {
                        let mtime = std::fs::metadata(&backup).and_then(|m| m.modified()).ok();
                        let when = mtime
                            .and_then(chrono_like_format)
                            .unwrap_or_else(|| "unknown".into());
                        app.push_toast(
                            ToastLevel::Info,
                            format!("Loaded state differs from last manual save ({}).", when),
                        );
                    }
                }
            }
        }

        app
    }

    pub(super) fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> anyhow::Result<()>
    where
        B::Error: Send + Sync + 'static,
    {
        terminal.draw(|f| self.draw(f))?;
        while !self.should_quit {
            let mut redraw = self.poll_asset_build();
            match self.next_idle_timeout() {
                Some(timeout) => {
                    if event::poll(timeout)? {
                        redraw |= self.drain_pending_events()?;
                    }
                }
                None => {
                    redraw |= self.drain_pending_events()?;
                }
            }
            redraw |= self.prune_toasts();
            redraw |= self.tick_autosave(std::time::Instant::now());
            if redraw {
                terminal.draw(|f| self.draw(f))?;
            }
        }

        Ok(())
    }

    /// Events that never change pixels: skip the following frame.
    /// Mouse Up still clears `scrollbar_drag` in `handle_event`; the last Drag already painted.
    pub(super) fn is_idle_event(evt: &Event) -> bool {
        match evt {
            Event::Key(k) => k.kind == KeyEventKind::Release,
            Event::FocusGained | Event::FocusLost => true,
            Event::Mouse(m) => matches!(m.kind, MouseEventKind::Moved | MouseEventKind::Up(_)),
            _ => false,
        }
    }

    /// Soonest deadline for autosave or toast expiry. `None` means block on the next event.
    pub(super) fn next_idle_timeout(&self) -> Option<Duration> {
        let now = std::time::Instant::now();
        let mut wait: Option<Duration> = None;
        let consider = |wait: &mut Option<Duration>, d: Duration| {
            *wait = Some(wait.map(|w| w.min(d)).unwrap_or(d));
        };

        if self.dirty && self.path.is_some() {
            match self.dirty_since {
                Some(since) => {
                    let elapsed = now.saturating_duration_since(since);
                    if elapsed >= AUTOSAVE_DEBOUNCE {
                        consider(&mut wait, Duration::ZERO);
                    } else {
                        consider(&mut wait, AUTOSAVE_DEBOUNCE - elapsed);
                    }
                }
                None => consider(&mut wait, AUTOSAVE_DEBOUNCE),
            }
        }

        for toast in &self.toasts {
            let elapsed = now.saturating_duration_since(toast.shown_at);
            if elapsed >= TOAST_TTL {
                consider(&mut wait, Duration::ZERO);
            } else {
                consider(&mut wait, TOAST_TTL - elapsed);
            }
        }
        if self.build_rx.is_some() {
            consider(&mut wait, Duration::from_millis(200));
        }
        wait
    }

    fn drain_pending_events(&mut self) -> anyhow::Result<bool> {
        let mut check_dirty = false;
        let mut redraw = false;
        loop {
            let evt = event::read()?;
            let idle = Self::is_idle_event(&evt);
            if self.handle_event(evt)? {
                check_dirty = true;
                redraw = true;
            } else if !idle {
                redraw = true;
            }
            if !event::poll(Duration::ZERO)? {
                break;
            }
        }
        if check_dirty {
            self.mark_dirty_if_changed();
        }
        Ok(redraw)
    }

    pub(super) fn begin_save_prompt(&mut self) {
        if let Some(path) = self.path.clone() {
            match self.commit_save_with_backup(&path) {
                Ok(()) => {
                    self.push_toast(ToastLevel::Success, format!("Saved {}", path.display()));
                }
                Err(e) => {
                    self.push_toast(ToastLevel::Error, format!("Failed to save: {}", e));
                }
            }
            return;
        }
        self.modal = Some(Modal::SavePrompt {
            path: "site.json".to_string(),
        });
    }

    pub(super) fn current_page(&self) -> &crate::model::Page {
        &self.site.pages[self.selected_page]
    }

    pub(super) fn current_page_mut(&mut self) -> Option<&mut crate::model::Page> {
        self.site.pages.get_mut(self.selected_page)
    }

    pub(super) fn selected_index_for_page(
        page: &crate::model::Page,
        selected_node: usize,
    ) -> Option<usize> {
        if page.nodes.is_empty() {
            None
        } else {
            Some(selected_node.min(page.nodes.len() - 1))
        }
    }

    /// Recompute the JSON snapshot of `self.site` and set `dirty` if it
    /// differs from `last_saved_json`. Idempotent: re-calling on an already
    /// dirty app does NOT advance `dirty_since`, preserving the original
    /// debounce anchor.
    pub(super) fn mark_dirty_if_changed(&mut self) {
        let current = match serde_json::to_string(&self.site) {
            Ok(s) => s,
            Err(_) => return,
        };
        if current != self.last_saved_json {
            if !self.dirty {
                self.dirty_since = Some(std::time::Instant::now());
            }
            self.dirty = true;
        }
    }

    /// Write `self.site` to `path` AND to `<path>.backup`. Both writes share
    /// a single serialization so the two files are guaranteed byte-identical.
    /// Refreshes the saved snapshot and clears the dirty flag on success.
    pub(super) fn commit_save_with_backup(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        crate::storage::save_site(path, &self.site)?;
        let backup = backup_path_for(path);
        std::fs::copy(path, &backup)?;
        self.last_saved_json = serde_json::to_string(&self.site).unwrap_or_default();
        self.dirty = false;
        self.dirty_since = None;
        self.path = Some(path.to_path_buf());
        Ok(())
    }

    /// If the site is dirty, has a path, and the debounce window has elapsed,
    /// write `self.site` to the active path and refresh the saved snapshot.
    /// Errors are surfaced as an error toast and leave `dirty` set so the
    /// next tick can retry.
    pub(super) fn tick_autosave(&mut self, now: std::time::Instant) -> bool {
        if !self.dirty {
            return false;
        }
        let Some(since) = self.dirty_since else {
            // Defensive: dirty without a timestamp shouldn't happen; treat as
            // freshly dirty.
            self.dirty_since = Some(now);
            return false;
        };
        if now.duration_since(since) < AUTOSAVE_DEBOUNCE {
            return false;
        }
        let Some(path) = self.path.clone() else {
            return false;
        };
        match crate::storage::save_site(&path, &self.site) {
            Ok(()) => {
                self.last_saved_json = serde_json::to_string(&self.site).unwrap_or_default();
                self.dirty = false;
                self.dirty_since = None;
                true
            }
            Err(e) => {
                let msg = format!("Autosave failed: {}", e);
                self.push_toast(ToastLevel::Error, msg);
                true
            }
        }
    }
}
