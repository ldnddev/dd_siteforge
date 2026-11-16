//! Phase 2 window: Welcome, new site, load states, and the read-only shell.

use std::path::{Path, PathBuf};
use std::time::Instant;

use dd_siteforge_core::component_kind::{ComponentKind, InsertRegion};
use dd_siteforge_core::controls::MarkdownInsert;
use dd_siteforge_core::cursor::Cursor;
use dd_siteforge_core::editform::SubChange;
use dd_siteforge_core::gui_edit::{Commit, GuiEdit};
use dd_siteforge_core::model::{Page, PageTemplate, Site};
use dd_siteforge_core::session;
use dd_siteforge_core::storage::{self, checkpoint_path};
use dd_siteforge_core::validate::validate_site_with_root;
use gpui::{
    App, ClickEvent, Context, FocusHandle, Focusable, IntoElement, KeyDownEvent, ParentElement,
    Render, StatefulInteractiveElement, Styled, Window, div, prelude::*, px,
};
use gpui_component::button::Button;

use crate::document::{Document, SavePhase};
use crate::keys::{self, show_inspector_tooltip, show_sidebar_tooltip};
use crate::new_site::{self, CreatedSite};
use crate::paint::{self, token};
use crate::present::{
    FieldChrome, InspTab, InspectorView, OutlineRow, SideRow, SideTarget, Target, WelcomeAction,
    WelcomeRow, column_layout, default_target, form_state, inspector, inspector_draft,
    load_error_message, outline_caption, outline_rows, sidebar_rows, status_meta, title_crumb,
    welcome_rows,
};
use crate::preview;
use crate::settings::GuiSettings;
use crate::theme_load::LiveTheme;

const TEMPLATES: [&str; 4] = ["Blank", "Hero only", "Hero + Section", "Duplicate current"];

pub struct ShellOptions {
    pub path: Option<PathBuf>,
    pub session_path: PathBuf,
    pub cwd: PathBuf,
    /// Open `./site.json` when no path argument was given.
    pub open_cwd: bool,
    pub tick: bool,
    pub quit_runtime: bool,
    /// Test override for the starter-kit overlay directory.
    pub overlay: Option<PathBuf>,
}

impl ShellOptions {
    pub fn production(path: Option<PathBuf>) -> Self {
        Self {
            path,
            session_path: session::session_file(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            open_cwd: true,
            tick: true,
            quit_runtime: true,
            overlay: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Zone {
    Sidebar,
    Outline,
    Inspector,
}

struct ShrinkOffer {
    source: PathBuf,
    field_id: String,
    long_edge: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Welcome,
    Loading,
    Error,
    Editor,
}

struct LoadFailure {
    path: PathBuf,
    message: String,
    folder: String,
}

struct NewSiteForm {
    open: bool,
    folder: String,
    project: String,
    error: Option<String>,
    field: u8,
}

pub struct Shell {
    focus: FocusHandle,
    theme: LiveTheme,
    settings: GuiSettings,
    paths: ldnddev_theme::OmarchyPaths,
    session_path: PathBuf,
    cwd: PathBuf,
    overlay: Option<PathBuf>,
    quit_runtime: bool,
    screen: Screen,
    document: Option<Document>,
    load_error: Option<LoadFailure>,
    loading_label: String,
    region: SideTarget,
    page_index: usize,
    target: Target,
    tab: Option<InspTab>,
    filter: String,
    welcome_query: String,
    welcome_index: usize,
    sidebar_index: usize,
    outline_index: usize,
    sidebar_open: bool,
    inspector_open: bool,
    pin_sidebar: bool,
    pin_inspector: bool,
    new_site: NewSiteForm,
    template_open: bool,
    template_index: usize,
    switcher: bool,
    switcher_index: usize,
    quit_dialog: bool,
    restore_open: bool,
    backup_note: Option<String>,
    toasts: Vec<String>,
    preview_note: String,
    revealed: Option<PathBuf>,
    pub quit: bool,
    /// Set when Ctrl+R is handled. Redo itself arrives with phase 3's stack.
    pub redo_handled: bool,
    /// Last window width, so key toggles use the same width as the title-bar buttons.
    last_width: f32,
    edit: GuiEdit,
    zone: Zone,
    rename: Option<String>,
    insert_open: bool,
    insert_query: String,
    insert_index: usize,
    shrink: Option<ShrinkOffer>,
    drag_arm: Option<Target>,
    last_theme_check: Instant,
    awaiting_choose: bool,
    awaiting_save_as: bool,
}

impl Shell {
    pub fn open(opts: ShellOptions, cx: &mut Context<Self>) -> Self {
        let settings = crate::settings::load();
        let paths = ldnddev_theme::OmarchyPaths::system();
        let theme = LiveTheme::load(&paths, settings.reduced_motion);
        paint::apply_component_theme(&theme, cx);
        let mut shell = Self {
            focus: cx.focus_handle(),
            theme,
            settings,
            paths,
            session_path: opts.session_path,
            cwd: opts.cwd.clone(),
            overlay: opts.overlay,
            quit_runtime: opts.quit_runtime,
            screen: Screen::Welcome,
            document: None,
            load_error: None,
            loading_label: String::new(),
            region: SideTarget::Page(0),
            page_index: 0,
            target: Target::PageHead,
            tab: None,
            filter: String::new(),
            welcome_query: String::new(),
            welcome_index: 0,
            sidebar_index: 3,
            outline_index: 0,
            sidebar_open: true,
            inspector_open: true,
            pin_sidebar: false,
            pin_inspector: false,
            new_site: NewSiteForm {
                open: false,
                folder: String::new(),
                project: String::new(),
                error: None,
                field: 0,
            },
            template_open: false,
            template_index: 0,
            switcher: false,
            switcher_index: 0,
            quit_dialog: false,
            restore_open: false,
            backup_note: None,
            toasts: Vec::new(),
            preview_note: String::new(),
            revealed: None,
            quit: false,
            redo_handled: false,
            last_width: 1280.0,
            edit: GuiEdit::new(),
            zone: Zone::Outline,
            rename: None,
            insert_open: false,
            insert_query: String::new(),
            insert_index: 0,
            shrink: None,
            drag_arm: None,
            last_theme_check: Instant::now(),
            awaiting_choose: false,
            awaiting_save_as: false,
        };
        if let Some(path) = opts.path {
            shell.open_site(&path);
        } else if opts.open_cwd {
            let here = opts.cwd.join("site.json");
            if here.is_file() {
                shell.open_site(&here);
            }
        }
        if opts.tick {
            shell.spawn_tick(cx);
        }
        shell
    }

    fn spawn_tick(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(250))
                    .await;
                let alive = this
                    .update(cx, |shell, cx| {
                        shell.tick();
                        cx.notify();
                    })
                    .is_ok();
                if !alive {
                    break;
                }
            }
        })
        .detach();
    }

    fn tick(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_theme_check) >= std::time::Duration::from_secs(2) {
            self.last_theme_check = now;
            if self.theme.refresh(&self.paths) {
                // Theme swap is applied on the next render via `apply` in the caller.
            }
        }
        if let Some(doc) = self.document.as_mut() {
            doc.autosave(now);
        }
        self.take_notice();
    }

    pub fn visible_text(&self) -> String {
        self.lines().join("\n")
    }

    pub fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(match self.screen {
            Screen::Welcome => "screen:welcome".to_string(),
            Screen::Loading => "screen:loading".to_string(),
            Screen::Error => "screen:error".to_string(),
            Screen::Editor => "screen:editor".to_string(),
        });
        lines.push(format!("title:{}", self.title_name()));
        if self.screen == Screen::Editor {
            if let Some(doc) = &self.document {
                lines.push(format!(
                    "crumb:{}",
                    title_crumb(&doc.site, &self.region, self.page_index)
                ));
            }
        }
        match self.screen {
            Screen::Welcome => {
                lines.push("Open site".to_string());
                lines.push("dd_siteforge".to_string());
                for row in self.welcome_items() {
                    lines.push(row.label);
                }
                lines.push("Open…".to_string());
                lines.push("New site…".to_string());
                lines.push("↑↓ select · Enter open · Esc quit".to_string());
            }
            Screen::Loading => {
                lines.push(format!("Loading {}", self.loading_label));
                lines.push("Reading site.json…".to_string());
            }
            Screen::Error => {
                if let Some(err) = &self.load_error {
                    lines.push(format!("Couldn't load {}", err.folder));
                    lines.push(err.message.clone());
                    lines.push(format!(
                        "{}/site.json · the file was not changed",
                        err.folder
                    ));
                }
                lines.push("Retry".to_string());
                lines.push("Choose file…".to_string());
                lines.push("Reveal in folder".to_string());
                if self.backup_available() {
                    lines.push("Open backup".to_string());
                }
                lines.push("Couldn't load site.json".to_string());
            }
            Screen::Editor => {
                if let Some(doc) = &self.document {
                    let (rows, count) = sidebar_rows(&doc.site, &self.filter);
                    lines.push(format!("pages:{count}"));
                    if doc.site.pages.is_empty() && self.filter.is_empty() {
                        lines.push("No pages".to_string());
                        lines.push("Use + to add the first page.".to_string());
                        lines.push("No pages yet".to_string());
                        lines.push(
                            "This site.json has no pages. Add one to start editing.".to_string(),
                        );
                    }
                    if !self.filter.is_empty()
                        && !rows
                            .iter()
                            .any(|row| matches!(row.target, SideTarget::Page(_)))
                    {
                        lines.push(format!("No pages match “{}”", self.filter));
                        lines.push(
                            "Search looks at page titles and slugs. Esc clears it.".to_string(),
                        );
                    }
                    for row in &rows {
                        let mut label = row.label.clone();
                        for badge in &row.badges {
                            label.push(' ');
                            label.push_str(badge);
                        }
                        lines.push(label);
                    }
                    let outline = outline_rows(&doc.site, &self.region, self.page_index);
                    let (heading, caption) =
                        outline_caption(&doc.site, &self.region, self.page_index);
                    lines.push(heading);
                    lines.push(caption);
                    if outline.is_empty() && matches!(self.region, SideTarget::Page(_)) {
                        lines.push("No nodes on this page.".to_string());
                    }
                    for row in &outline {
                        lines.push(row.label.clone());
                        if !row.detail.is_empty() {
                            lines.push(row.detail.clone());
                        }
                        let issues = row.issues + usize::from(self.draft_issue(&row.target));
                        if issues > 0 {
                            lines.push(format!(
                                "{} issue{}",
                                issues,
                                if issues == 1 { "" } else { "s" }
                            ));
                            lines.push("Issue".to_string());
                        }
                        if !row.columns.is_empty() {
                            let shares = row
                                .columns
                                .iter()
                                .map(|column| format!("{}/{}", column.parts, column.whole))
                                .collect::<Vec<_>>()
                                .join(" ");
                            lines.push(format!("column-row {shares}"));
                            for column in &row.columns {
                                let state = if column.target == self.target {
                                    "selected"
                                } else if column.empty {
                                    "empty"
                                } else {
                                    "filled"
                                };
                                lines.push(format!("{} {} {state}", column.label, column.md_class));
                                if column.empty {
                                    lines.push("Empty · / to add".to_string());
                                }
                                lines.extend(column.chips.iter().cloned());
                            }
                        }
                    }
                    let view = self.inspector_view();
                    if view.empty {
                        lines.push("Nothing selected".to_string());
                        lines.push(
                            "Select a block in the Outline, or press Enter on a row, to edit its fields here."
                                .to_string(),
                        );
                    } else {
                        lines.push(view.breadcrumb);
                        lines.push(view.title);
                        if view.tabs.len() > 1 {
                            for tab in &view.tabs {
                                lines.push(tab.label().to_string());
                            }
                        }
                        for field in &view.fields {
                            let star = if field.required { " *" } else { "" };
                            lines.push(format!("{}{star}", field.label));
                            lines.push(field.value.clone());
                            if let Some(meta) = &field.meta {
                                lines.push(meta.clone());
                            }
                            if let Some(error) = &field.error {
                                lines.push(error.clone());
                            }
                            if field.toolbar {
                                lines.push("H2".to_string());
                                lines.push("B".to_string());
                                lines.push("I".to_string());
                                lines.push("Link".to_string());
                                lines.push("List".to_string());
                            }
                        }
                        if let Some(back) = &view.back {
                            lines.push(back.clone());
                        }
                    }
                    lines.push(status_meta(&doc.site, &self.region, self.page_index));
                    lines.push(format!("Issues {}", self.issue_count()));
                }
            }
        }
        if self.screen == Screen::Editor || self.screen == Screen::Welcome {
            lines.push("Preview closed · Ctrl+P".to_string());
        }
        if let Some(doc) = &self.document {
            if self.screen == Screen::Editor {
                lines.push(doc.indicator_text());
            }
        }
        if let Some(note) = &self.backup_note {
            lines.push(note.clone());
            lines.push("Restore checkpoint…".to_string());
            lines.push("Keep current".to_string());
        }
        if let Some(err) = self
            .document
            .as_ref()
            .and_then(|doc| doc.save_error.clone())
        {
            lines.push("Autosave failed".to_string());
            lines.push(err);
            lines.push("Save as…".to_string());
            lines.push("Retry".to_string());
        }
        if self.document.as_ref().is_some_and(|doc| doc.conflict) {
            lines.push("Changed on disk. Not overwritten. Reload, or keep yours.".to_string());
            lines.push("Reload".to_string());
            lines.push("Keep mine".to_string());
        }
        if self.new_site.open {
            lines.push("New site".to_string());
            lines.push("Ctrl Shift N".to_string());
            lines.push(format!("Folder {}", self.new_site.folder));
            lines.push(format!("Slug: {}", self.slug_preview()));
            lines.push(
                "Creates site.json and the starter kit (Gruntfile.js, package.json, .lando.yml, .ddev/config.yaml, source/, templates)."
                    .to_string(),
            );
            lines.push(
                "The site name starts as “My Site”. Change it in Site settings › Name.".to_string(),
            );
            lines.push("Cancel".to_string());
            lines.push("Create".to_string());
            if let Some(err) = &self.new_site.error {
                lines.push(err.clone());
                if err.contains("already exists") {
                    lines.push("Open it".to_string());
                }
            }
        }
        if self.template_open {
            lines.push("New page — choose template".to_string());
            for name in TEMPLATES {
                lines.push(name.to_string());
            }
        }
        if self.switcher {
            lines.push("Recent sites".to_string());
            for recent in self.recents() {
                lines.push(recent.display().to_string());
            }
            lines.push("Open…".to_string());
            lines.push("New site…".to_string());
        }
        if self.quit_dialog {
            lines.push("Unsaved changes. Quit anyway?".to_string());
            lines.push(self.quit_body());
            lines.push("Cancel".to_string());
            lines.push("Quit anyway".to_string());
        }
        if self.restore_open {
            lines.push("Restore from checkpoint?".to_string());
            lines.push(self.restore_body());
            lines.push("Cancel".to_string());
            lines.push("Restore".to_string());
        }
        if self.sidebar_collapsed_for(1280.0) {
            // Width-specific tooltips are added by `dock_notes` from render.
        }
        lines.extend(self.dock_notes(self.last_width));
        if self.insert_open {
            lines.push("Insert block…".to_string());
            lines
                .push("Type to filter | Up/Down: select | Enter: insert | Esc: cancel".to_string());
            for kind in self.filtered_kinds() {
                lines.push(dd_siteforge_core::edits::insert_group(kind).to_string());
                lines.push(kind.label().to_string());
            }
            let after = self
                .edit
                .last_insert
                .map(|kind| kind.label())
                .unwrap_or("the selection");
            lines.push(format!("Enter inserts after {after} · 24 kinds"));
        }
        if let Some(title) = &self.rename {
            lines.push("Rename page. Edit and press Enter.".to_string());
            lines.push(title.clone());
        }
        if let Some(offer) = &self.shrink {
            lines.push(format!(
                "This image is {} px on the long edge.",
                offer.long_edge
            ));
            lines.push("Copy as-is".to_string());
            lines.push("Shrink to 2560".to_string());
        }
        if self.edit.delete_toast.is_some() {
            lines.push("Undo".to_string());
            lines.push("Ctrl Z".to_string());
        }
        lines.extend(self.toasts.iter().cloned());
        if !self.preview_note.is_empty() {
            lines.push(self.preview_note.clone());
        }
        lines
    }

    pub fn docks_visible(&self) -> (bool, bool) {
        self.docks(self.last_width)
    }

    pub fn dock_notes(&self, width: f32) -> Vec<String> {
        let mut notes = Vec::new();
        let (sidebar, inspector) = self.docks(width);
        if !inspector {
            notes.push(show_inspector_tooltip());
        }
        if !sidebar {
            notes.push(show_sidebar_tooltip());
        }
        notes
    }

    fn docks(&self, width: f32) -> (bool, bool) {
        let sidebar = if width <= 720.0 {
            self.pin_sidebar
        } else {
            self.sidebar_open || self.pin_sidebar
        };
        let inspector = if width < 1000.0 {
            self.pin_inspector
        } else {
            self.inspector_open || self.pin_inspector
        };
        (sidebar, inspector)
    }

    fn sidebar_collapsed_for(&self, _width: f32) -> bool {
        false
    }

    pub fn handle_key(&mut self, key: &str) {
        let key = key.to_ascii_lowercase();
        if key == "ctrl-b" {
            // Build CSS/JS. Not a dock toggle.
            return;
        }
        // Same actions as the title-bar buttons (J4).
        if key == keys::TOGGLE_SIDEBAR_KEY {
            self.toggle_sidebar(self.last_width);
            return;
        }
        if key == keys::TOGGLE_INSPECTOR_KEY && self.screen == Screen::Editor {
            self.toggle_inspector(self.last_width);
            return;
        }
        if self.quit_dialog {
            self.key_quit_dialog(&key);
            return;
        }
        if self.restore_open {
            self.key_restore(&key);
            return;
        }
        if self.template_open {
            self.key_template(&key);
            return;
        }
        if self.new_site.open {
            self.key_new_site(&key);
            return;
        }
        if self.switcher {
            self.key_switcher(&key);
            return;
        }
        if self.screen == Screen::Editor && self.edit_key(&key) {
            return;
        }
        match key.as_str() {
            keys::QUIT_KEY => self.request_quit(),
            keys::NEW_SITE_KEY => self.open_new_site(None),
            keys::ADD_PAGE_KEY => self.open_template(),
            keys::THEME_KEY => self.open_theme(),
            keys::REDO_KEY => self.redo(),
            keys::OPEN_KEY => self.request_choose_file(),
            "ctrl-p" => self.open_preview(),
            "ctrl-s" => self.checkpoint(),
            "ctrl-1" => self.zone_sidebar(),
            "ctrl-2" => self.zone_outline(),
            "escape" => self.escape(),
            "enter" => self.activate(),
            "up" | "k" => self.move_sel(-1),
            "down" | "j" => self.move_sel(1),
            "backspace" => self.backspace(),
            other if self.screen == Screen::Welcome && is_text(other) => {
                self.welcome_query.push_str(other);
                self.welcome_index = 0;
            }
            other if self.screen == Screen::Editor && is_text(other) => {
                self.filter.push_str(other);
                self.sidebar_index = 0;
            }
            _ => {}
        }
    }

    fn key_quit_dialog(&mut self, key: &str) {
        match key {
            "escape" => self.quit_dialog = false,
            "enter" => {}
            _ => {}
        }
    }

    fn key_restore(&mut self, key: &str) {
        if key == "escape" {
            self.restore_open = false;
        }
    }

    fn key_template(&mut self, key: &str) {
        match key {
            "escape" => {
                self.template_open = false;
                self.toast("Add page cancelled.");
            }
            "up" | "k" if self.template_index > 0 => self.template_index -= 1,
            "down" | "j" if self.template_index + 1 < TEMPLATES.len() => self.template_index += 1,
            "enter" => self.commit_template(),
            _ => {}
        }
    }

    fn key_new_site(&mut self, key: &str) {
        match key {
            "escape" => self.new_site.open = false,
            "enter" => {
                let _ = self.submit_new_site();
            }
            "backspace" => self.backspace_new_site(),
            "tab" => self.new_site.field = 1 - self.new_site.field.min(1),
            other if is_text(other) => self.type_new_site(other),
            _ => {}
        }
    }

    fn key_switcher(&mut self, key: &str) {
        let count = self.recents().len();
        match key {
            "escape" => self.switcher = false,
            "up" | "k" if self.switcher_index > 0 => self.switcher_index -= 1,
            "down" | "j" if self.switcher_index + 1 < count => self.switcher_index += 1,
            "enter" => {
                let recents = self.recents();
                if let Some(path) = recents.get(self.switcher_index).cloned() {
                    self.switcher = false;
                    self.open_site(&path);
                }
            }
            keys::NEW_SITE_KEY => {
                self.switcher = false;
                self.open_new_site(None);
            }
            _ => {}
        }
    }

    fn escape(&mut self) {
        if self.screen == Screen::Welcome {
            if self.welcome_query.is_empty() {
                self.quit_now();
            } else {
                self.welcome_query.clear();
            }
            return;
        }
        if !self.filter.is_empty() {
            self.filter.clear();
        }
    }

    fn activate(&mut self) {
        match self.screen {
            Screen::Welcome => self.activate_welcome(),
            Screen::Editor => self.activate_outline(),
            Screen::Error => self.retry_load(),
            Screen::Loading => {}
        }
    }

    fn activate_welcome(&mut self) {
        let rows = self.welcome_items();
        let Some(row) = rows.get(self.welcome_index) else {
            self.toast("Type a path or pick a recent site.");
            return;
        };
        match &row.action {
            WelcomeAction::Open(path) => {
                let path = path.clone();
                self.open_site(&path);
            }
            WelcomeAction::CreateAt(path) => {
                let folder = path.parent().unwrap_or(path).display().to_string();
                self.open_new_site(Some(folder));
            }
            WelcomeAction::CreateHere => {
                self.open_new_site(Some(self.cwd.display().to_string()));
            }
        }
    }

    fn activate_outline(&mut self) {
        let Some(doc) = &self.document else { return };
        let rows = outline_rows(&doc.site, &self.region, self.page_index);
        if let Some(row) = rows.get(self.outline_index) {
            self.target = row.target.clone();
            self.tab = None;
        }
    }

    fn move_sel(&mut self, delta: i32) {
        if self.screen == Screen::Welcome {
            let len = self.welcome_items().len() as i32;
            if len == 0 {
                return;
            }
            self.welcome_index = (self.welcome_index as i32 + delta).clamp(0, len - 1) as usize;
            return;
        }
        if self.screen != Screen::Editor {
            return;
        }
        let Some(doc) = &self.document else { return };
        let len = outline_rows(&doc.site, &self.region, self.page_index).len() as i32;
        if len == 0 {
            return;
        }
        self.outline_index = (self.outline_index as i32 + delta).clamp(0, len - 1) as usize;
        self.activate_outline();
    }

    fn backspace(&mut self) {
        if self.screen == Screen::Welcome {
            self.welcome_query.pop();
        } else if self.screen == Screen::Editor {
            self.filter.pop();
        }
    }

    pub fn open_site(&mut self, path: &Path) {
        let path = resolve_site_path(path);
        let label = folder_name(&path);
        self.loading_label = label.clone();
        // Loads are synchronous, same as the TUI. The loading screen is for a
        // slow read; tests call `show_loading` to render that state.
        match Document::load(path.clone()) {
            Ok(mut doc) => {
                ensure_section_ids(&mut doc.site);
                let (page, _row, region) =
                    session::load_from(&self.session_path).selection_for(&doc.path);
                self.page_index = page.min(doc.site.pages.len().saturating_sub(1));
                self.region = match region.as_str() {
                    "header" => SideTarget::Header,
                    "footer" => SideTarget::Footer,
                    "site" => SideTarget::Site,
                    _ => SideTarget::Page(self.page_index),
                };
                self.target = default_target(&self.region);
                self.outline_index = 0;
                self.tab = None;
                self.backup_note = backup_note(&doc.path);
                new_site::remember(
                    &self.session_path,
                    &doc.path,
                    self.page_index,
                    region_name(&self.region),
                );
                self.toast(format!("Opened {}", doc.path.display()));
                self.document = Some(doc);
                self.load_error = None;
                self.screen = Screen::Editor;
                self.new_site.open = false;
            }
            Err(err) => {
                self.load_error = Some(LoadFailure {
                    path: path.clone(),
                    message: load_error_message(&path, &err),
                    folder: label,
                });
                self.document = None;
                self.screen = Screen::Error;
            }
        }
    }

    pub fn show_loading(&mut self, label: &str) {
        self.loading_label = label.to_string();
        self.screen = Screen::Loading;
    }

    pub fn show_phase(&mut self, phase: SavePhase) {
        if let Some(doc) = self.document.as_mut() {
            doc.show_phase(phase);
        }
    }

    pub fn retry_load(&mut self) {
        if let Some(err) = &self.load_error {
            let path = err.path.clone();
            self.open_site(&path);
        }
    }

    pub fn reveal_in_folder(&mut self) {
        let path = self
            .load_error
            .as_ref()
            .map(|err| err.path.clone())
            .or_else(|| self.document.as_ref().map(|doc| doc.path.clone()));
        let Some(path) = path else { return };
        let folder = path.parent().unwrap_or(path.as_path()).to_path_buf();
        self.revealed = Some(folder.clone());
        // A headless test has no portal. Don't inherit the test's stdout, or
        // xdg-open keeps the cargo pipe open after the process hangs.
        if std::env::var_os("WAYLAND_DISPLAY").is_some() || std::env::var_os("DISPLAY").is_some() {
            let _ = std::process::Command::new("xdg-open")
                .arg(&folder)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
    }

    pub fn request_choose_file(&mut self) {
        // DESIGN-GAP: the file chooser is the XDG portal, which is not drawn
        // as a sheet. The button is the Screen 29 / Welcome action. Tests
        // finish the choice with `choose_file`.
        self.awaiting_choose = true;
    }

    pub fn choose_file(&mut self, path: PathBuf) {
        self.awaiting_choose = false;
        self.open_site(&path);
    }

    pub fn open_new_site(&mut self, folder: Option<String>) {
        self.new_site.open = true;
        self.new_site.error = None;
        self.new_site.field = 0;
        if let Some(folder) = folder {
            self.new_site.folder = folder;
        }
        self.refresh_new_site_error();
    }

    pub fn submit_new_site(&mut self) -> Result<CreatedSite, String> {
        self.refresh_new_site_error();
        if self.new_site.folder.trim().is_empty() {
            let err = "Choose a folder.".to_string();
            self.new_site.error = Some(err.clone());
            return Err(err);
        }
        if self
            .new_site
            .error
            .as_ref()
            .is_some_and(|err| err.contains("already exists"))
        {
            return Err(self.new_site.error.clone().unwrap_or_default());
        }
        let folder = PathBuf::from(self.new_site.folder.trim());
        match new_site::create_new_site(
            &folder,
            &self.new_site.project,
            &self.session_path,
            self.overlay.as_deref(),
        ) {
            Ok(created) => {
                if !created.warnings.is_empty() {
                    self.toast(format!(
                        "Site created, but the starter kit wasn't fully written: {}",
                        created.warnings.join("; ")
                    ));
                }
                for note in &created.notes {
                    self.toast(note.clone());
                }
                let path = created.path.clone();
                self.new_site.open = false;
                self.open_site(&path);
                Ok(created)
            }
            Err(err) => {
                self.new_site.error = Some(err.clone());
                Err(err)
            }
        }
    }

    fn refresh_new_site_error(&mut self) {
        let folder = PathBuf::from(self.new_site.folder.trim());
        if self.new_site.folder.trim().is_empty() {
            self.new_site.error = None;
            return;
        }
        let site_json = if folder.ends_with("site.json") {
            folder
        } else {
            folder.join("site.json")
        };
        if site_json.exists() {
            self.new_site.error = Some("A site.json already exists here.".to_string());
        } else {
            self.new_site.error = None;
        }
    }

    fn slug_preview(&self) -> String {
        let folder = PathBuf::from(if self.new_site.folder.trim().is_empty() {
            self.cwd.display().to_string()
        } else {
            self.new_site.folder.trim().to_string()
        });
        new_site::project_slug(&folder, &self.new_site.project)
    }

    fn type_new_site(&mut self, text: &str) {
        if self.new_site.field == 0 {
            self.new_site.folder.push_str(text);
        } else {
            self.new_site.project.push_str(text);
        }
        self.refresh_new_site_error();
    }

    fn backspace_new_site(&mut self) {
        if self.new_site.field == 0 {
            self.new_site.folder.pop();
        } else {
            self.new_site.project.pop();
        }
        self.refresh_new_site_error();
    }

    pub fn open_existing_from_sheet(&mut self) {
        let folder = PathBuf::from(self.new_site.folder.trim());
        let path = if folder.ends_with("site.json") {
            folder
        } else {
            folder.join("site.json")
        };
        self.new_site.open = false;
        self.open_site(&path);
    }

    fn open_template(&mut self) {
        if self.document.is_none() {
            return;
        }
        self.template_open = true;
        self.template_index = 0;
        self.toast("New page: choose a template.");
    }

    fn commit_template(&mut self) {
        let which = self.template_index;
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        let mut page = match which {
            1 => Page::from_template("", PageTemplate::HeroOnly),
            2 => Page::from_template("", PageTemplate::HeroPlusSection),
            3 => doc
                .site
                .pages
                .get(self.page_index)
                .map(Page::duplicate_from)
                .unwrap_or_else(|| Page::from_template("", PageTemplate::Blank)),
            _ => Page::from_template("", PageTemplate::Blank),
        };
        dedup_page(&doc.site.pages, &mut page);
        doc.site.pages.push(page);
        let index = doc.site.pages.len() - 1;
        doc.mark_dirty();
        self.page_index = index;
        self.region = SideTarget::Page(index);
        self.target = Target::PageHead;
        self.outline_index = 0;
        self.template_open = false;
        self.toast("New page — fill in title and HEAD, Ctrl+S to add.");
    }

    fn open_theme(&mut self) {
        // DESIGN-GAP: F2 opens Settings › Appearance (spec 06 §3), a phase 5
        // board. The closest drawn phase-2 component is a Toast, so the key
        // names the theme there and does not invent a settings layout.
        let _ = keys::THEME_KEY;
        self.toast(format!("Theme · {}", self.theme.label));
    }

    fn redo(&mut self) {
        self.redo_edit();
    }

    fn checkpoint(&mut self) {
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        if self.edit.active_key().is_some() {
            match self.edit.commit(&mut doc.site) {
                Commit::Blocked(message) | Commit::Failed(message) => {
                    self.toast(message);
                    return;
                }
                Commit::ItemSaved => {
                    self.toast("Item saved — editing parent.");
                    return;
                }
                Commit::Applied => doc.mark_dirty(),
            }
        }
        doc.checkpoint();
        self.take_notice();
    }

    pub fn revealed(&self) -> Option<PathBuf> {
        self.revealed.clone()
    }

    pub fn new_site_folder(&mut self, folder: &str) {
        self.new_site.folder = folder.to_string();
        self.new_site.field = 0;
        self.refresh_new_site_error();
    }

    pub fn new_site_project(&mut self, name: &str) {
        self.new_site.project = name.to_string();
        self.new_site.field = 1;
    }

    pub fn select_sidebar_label(&mut self, label: &str) {
        let Some(doc) = &self.document else { return };
        let (rows, _) = sidebar_rows(&doc.site, &self.filter);
        if let Some(index) = rows.iter().position(|row| row.label == label) {
            self.select_sidebar(index);
        }
    }

    pub fn select_outline_label(&mut self, label: &str) {
        let Some(doc) = &self.document else { return };
        let rows = outline_rows(&doc.site, &self.region, self.page_index);
        if let Some(index) = rows.iter().position(|row| row.label == label) {
            self.select_outline(index);
        }
    }

    /// Select a header or footer column in the proportional row.
    pub fn select_column(&mut self, label: &str) {
        let Some(doc) = &self.document else { return };
        let rows = outline_rows(&doc.site, &self.region, self.page_index);
        let Some(column) = rows
            .iter()
            .flat_map(|row| row.columns.iter())
            .find(|column| column.label == label)
        else {
            return;
        };
        self.target = column.target.clone();
        self.tab = None;
        self.remember_selection();
    }

    /// `row` or `stack` for the current region's column grids at a window width.
    pub fn column_row_mode(&self, window_width: f32) -> &'static str {
        let Some(doc) = &self.document else {
            return "row";
        };
        let width = self.outline_content_width(window_width);
        let rows = outline_rows(&doc.site, &self.region, self.page_index);
        rows.iter()
            .find(|row| !row.columns.is_empty())
            .map(|row| crate::present::column_layout(&row.columns, width))
            .unwrap_or("row")
    }

    fn outline_content_width(&self, window_width: f32) -> f32 {
        let (sidebar, inspector) = self.docks(window_width);
        let mut width = window_width;
        if sidebar {
            width -= 280.0;
        }
        if inspector {
            width -= 360.0;
        }
        (width - 24.0).max(0.0)
    }

    pub fn reload_from_disk(&mut self) {
        if let Some(doc) = self.document.as_mut() {
            if let Err(err) = doc.reload() {
                self.toast(format!("Failed to reload: {err}"));
            }
        }
        self.take_notice();
    }

    pub fn keep_mine(&mut self) {
        if let Some(doc) = self.document.as_mut() {
            doc.keep_mine();
        }
        self.take_notice();
    }

    pub fn retry_save(&mut self) {
        if let Some(doc) = self.document.as_mut() {
            doc.retry_save();
        }
        self.take_notice();
    }

    pub fn save_copy_to(&mut self, path: PathBuf) -> Result<(), String> {
        let site = self
            .document
            .as_ref()
            .map(|doc| doc.site.clone())
            .ok_or_else(|| "No site".to_string())?;
        storage::save_site(&path, &site).map_err(|err| err.to_string())?;
        let _ = site;
        self.awaiting_save_as = false;
        self.open_site(&path);
        if let Some(doc) = self.document.as_mut() {
            doc.save_error = None;
            doc.checkpoint();
        }
        self.take_notice();
        Ok(())
    }

    pub fn request_save_as(&mut self) {
        self.awaiting_save_as = true;
    }

    fn request_quit(&mut self) {
        if self.screen != Screen::Editor {
            self.quit_now();
            return;
        }
        let Some(doc) = self.document.as_mut() else {
            self.quit_now();
            return;
        };
        if self.edit.has_invalid() || doc.phase() == SavePhase::Failed {
            self.quit_dialog = true;
            return;
        }
        if doc.dirty {
            doc.dirty_since = Some(Instant::now() - crate::document::AUTOSAVE_AFTER);
            doc.autosave(Instant::now());
        }
        if doc.phase() == SavePhase::Failed || doc.dirty {
            self.quit_dialog = true;
        } else {
            self.quit_now();
        }
    }

    pub fn confirm_quit(&mut self) {
        self.quit_dialog = false;
        self.quit_now();
    }

    pub fn cancel_quit(&mut self) {
        self.quit_dialog = false;
    }

    fn quit_now(&mut self) {
        self.quit = true;
        if let Some(doc) = &self.document {
            let region = region_name(&self.region);
            new_site::remember(&self.session_path, &doc.path, self.page_index, region);
        }
    }

    fn quit_body(&self) -> String {
        let when = self
            .document
            .as_ref()
            .map(|doc| crate::document::clock_label(doc.saved_at))
            .unwrap_or_else(|| "the last save".to_string());
        format!(
            "Autosave failed, so your edits since {when} are only in this window. Quitting now loses them."
        )
    }

    pub fn open_restore(&mut self) {
        if self.backup_available() {
            self.restore_open = true;
        }
    }

    pub fn confirm_restore(&mut self) {
        self.restore_open = false;
        let failed = self.load_error.as_ref().map(|err| err.path.clone());
        if let Some(path) = failed {
            if let Ok(doc) = Document::load(path) {
                self.document = Some(doc);
            }
        }
        if let Some(doc) = self.document.as_mut() {
            if let Err(err) = doc.restore_backup() {
                self.toast(err.to_string());
                return;
            }
        }
        self.take_notice();
        if let Some(doc) = &self.document {
            let path = doc.path.clone();
            self.open_site(&path);
            self.toast(format!(
                "Restored {path} from checkpoint.",
                path = path.display()
            ));
        }
    }

    fn restore_body(&self) -> String {
        let when = self
            .backup_mtime()
            .map(crate::document::backup_stamp)
            .unwrap_or_else(|| "an earlier checkpoint".to_string());
        format!(
            "This replaces site.json with site.json.backup (saved {when}). The current site.json is overwritten."
        )
    }

    fn backup_available(&self) -> bool {
        let path = self
            .document
            .as_ref()
            .map(|doc| doc.path.clone())
            .or_else(|| self.load_error.as_ref().map(|err| err.path.clone()));
        path.is_some_and(|path| checkpoint_path(&path).is_file())
    }

    fn backup_mtime(&self) -> Option<std::time::SystemTime> {
        let path = self
            .document
            .as_ref()
            .map(|doc| doc.path.clone())
            .or_else(|| self.load_error.as_ref().map(|err| err.path.clone()))?;
        std::fs::metadata(checkpoint_path(&path))
            .and_then(|meta| meta.modified())
            .ok()
    }

    pub fn dismiss_backup(&mut self) {
        self.backup_note = None;
    }

    fn open_preview(&mut self) {
        let Some(doc) = self.document.as_ref() else {
            self.preview_note = "Open a site.json to preview.".into();
            return;
        };
        let Some(dir) = doc.site.export_dir.as_deref() else {
            self.preview_note = "Preview opens in the browser once an export folder is set.".into();
            return;
        };
        let root = doc.path.parent().unwrap_or(Path::new(".")).join(dir);
        match dd_siteforge_core::serve::StaticServer::start(root) {
            Ok(server) => {
                let url = server.url_for("index", doc.site.pretty_urls);
                let url_for_note = url.clone();
                std::mem::forget(server);
                if let Err(err) = preview::open_in_browser(&url) {
                    self.preview_note = format!("Couldn't open the browser: {err}");
                } else {
                    self.preview_note = format!("Opened {url_for_note} in the browser");
                }
            }
            Err(err) => self.preview_note = format!("Couldn't start preview: {err}"),
        }
    }

    pub fn select_sidebar(&mut self, index: usize) {
        let Some(doc) = &self.document else { return };
        let (rows, _) = sidebar_rows(&doc.site, &self.filter);
        let Some(row) = rows.get(index) else { return };
        if matches!(row.target, SideTarget::Folder) {
            return;
        }
        self.zone = Zone::Sidebar;
        self.sidebar_index = index;
        self.region = row.target.clone();
        if let SideTarget::Page(page) = row.target {
            self.page_index = page;
        }
        self.target = default_target(&self.region);
        self.outline_index = 0;
        self.tab = None;
        self.remember_selection();
    }

    pub fn select_outline(&mut self, index: usize) {
        self.leave_draft();
        self.zone = Zone::Outline;
        self.outline_index = index;
        self.activate_outline();
        self.remember_selection();
    }

    pub fn select_tab(&mut self, tab: InspTab) {
        self.tab = Some(tab);
    }

    pub fn toggle_sidebar(&mut self, width: f32) {
        if width <= 720.0 {
            self.pin_sidebar = !self.pin_sidebar;
        } else {
            self.sidebar_open = !self.sidebar_open;
            self.pin_sidebar = false;
        }
    }

    pub fn toggle_inspector(&mut self, width: f32) {
        if width < 1000.0 {
            self.pin_inspector = !self.pin_inspector;
        } else {
            self.inspector_open = !self.inspector_open;
            self.pin_inspector = false;
        }
    }

    pub fn toggle_switcher(&mut self) {
        self.switcher = !self.switcher;
        self.switcher_index = 0;
    }

    fn remember_selection(&self) {
        if let Some(doc) = &self.document {
            new_site::remember(
                &self.session_path,
                &doc.path,
                self.page_index,
                region_name(&self.region),
            );
        }
    }

    fn zone_sidebar(&mut self) {}
    fn zone_outline(&mut self) {}

    fn welcome_items(&self) -> Vec<WelcomeRow> {
        welcome_rows(
            &session::load_from(&self.session_path),
            &self.welcome_query,
            &self.cwd,
        )
    }

    fn recents(&self) -> Vec<PathBuf> {
        session::load_from(&self.session_path)
            .existing_recents()
            .into_iter()
            .map(|recent| recent.path)
            .collect()
    }

    fn draft_issue(&self, target: &Target) -> bool {
        let Some(cursor) = current_cursor(&self.region, self.page_index, target) else {
            return false;
        };
        self.edit
            .draft(&format!("{cursor:?}"))
            .is_some_and(|draft| !draft.invalid.is_empty())
    }

    fn outline_with_draft_issues(&self) -> Option<Vec<crate::present::OutlineRow>> {
        let doc = self.document.as_ref()?;
        let mut rows = outline_rows(&doc.site, &self.region, self.page_index);
        for row in &mut rows {
            if self.draft_issue(&row.target) {
                row.issues += 1;
            }
        }
        Some(rows)
    }

    fn inspector_view(&self) -> InspectorView {
        let Some(doc) = &self.document else {
            return InspectorView {
                breadcrumb: String::new(),
                title: "Nothing selected".to_string(),
                kind: String::new(),
                tabs: Vec::new(),
                fields: Vec::new(),
                empty: true,
                back: None,
            };
        };
        let crumb =
            crate::present::breadcrumb(&doc.site, &self.region, self.page_index, &self.target);
        if self.zone == Zone::Inspector {
            if let Some(draft) = self.edit.active() {
                return inspector_draft(&draft.state, crumb, self.tab, self.edit.back_label());
            }
        }
        inspector(
            &doc.site,
            &self.region,
            self.page_index,
            &self.target,
            self.tab,
        )
    }

    fn issue_count(&self) -> usize {
        let Some(doc) = &self.document else { return 0 };
        validate_site_with_root(&doc.site, doc.path.parent()).len()
    }

    fn title_name(&self) -> String {
        match self.screen {
            Screen::Welcome => "No site".to_string(),
            Screen::Loading => self.loading_label.clone(),
            Screen::Error => self
                .load_error
                .as_ref()
                .map(|err| err.folder.clone())
                .unwrap_or_else(|| "No site".to_string()),
            Screen::Editor => self
                .document
                .as_ref()
                .map(|doc| doc.site.name.clone())
                .unwrap_or_else(|| "No site".to_string()),
        }
    }

    fn toast(&mut self, text: impl Into<String>) {
        self.toasts.push(text.into());
        if self.toasts.len() > 5 {
            self.toasts.remove(0);
        }
    }

    fn take_notice(&mut self) {
        let notice = self.document.as_mut().and_then(|doc| doc.notice.take());
        if let Some(notice) = notice {
            if !notice.starts_with("Autosave failed") && notice != "Saved" {
                self.toast(notice);
            }
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        self.handle_key(&chord(event));
        if self.awaiting_choose {
            self.portal_choose(cx);
        }
        if self.awaiting_save_as {
            self.portal_save_as(cx);
        }
    }

    fn portal_choose(&mut self, cx: &mut Context<Self>) {
        self.awaiting_choose = false;
        if !self.quit_runtime {
            return;
        }
        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: true,
            directories: true,
            multiple: false,
            prompt: Some("Open site.json".into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            this.update(cx, |shell, cx| {
                shell.choose_file(path);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn portal_save_as(&mut self, cx: &mut Context<Self>) {
        self.awaiting_save_as = false;
        if !self.quit_runtime {
            return;
        }
        let directory = self
            .document
            .as_ref()
            .and_then(|doc| doc.path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| self.cwd.clone());
        let receiver = cx.prompt_for_new_path(&directory, Some("site.json"));
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(path))) = receiver.await else {
                return;
            };
            this.update(cx, |shell, cx| {
                let _ = shell.save_copy_to(path);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn edit_key(&mut self, key: &str) -> bool {
        if self.shrink.is_some() {
            match key {
                "escape" => self.shrink = None,
                "enter" | "s" => self.finish_shrink(false),
                "2" => self.finish_shrink(true),
                _ => {}
            }
            return true;
        }
        if self.rename.is_some() {
            match key {
                "escape" => self.rename = None,
                "enter" => self.commit_rename(),
                "backspace" => {
                    if let Some(text) = self.rename.as_mut() {
                        text.pop();
                    }
                }
                other if is_text(other) => {
                    if let Some(text) = self.rename.as_mut() {
                        text.push_str(other);
                    }
                }
                _ => {}
            }
            return true;
        }
        if self.insert_open {
            self.insert_key(key);
            return true;
        }
        match key {
            "ctrl-z" => {
                self.undo_edit();
                true
            }
            "ctrl-shift-z" => {
                self.redo_edit();
                true
            }
            keys::REDO_KEY => {
                self.redo_edit();
                true
            }
            "ctrl-s" => {
                self.checkpoint();
                true
            }
            "ctrl-e" if self.zone == Zone::Inspector => {
                let _ = self.edit.toggle_expanded();
                true
            }
            "ctrl-i" | "/" if self.zone != Zone::Inspector => {
                self.open_insert();
                true
            }
            "r" if self.zone != Zone::Inspector => {
                self.begin_rename();
                true
            }
            "delete" if self.zone == Zone::Outline => {
                self.delete_selection();
                true
            }
            "backspace" if self.zone == Zone::Outline => {
                self.delete_selection();
                true
            }
            "shift-j" if self.zone == Zone::Outline => {
                self.move_selection(1);
                true
            }
            "shift-k" if self.zone == Zone::Outline => {
                self.move_selection(-1);
                true
            }
            "enter" if self.zone == Zone::Outline => {
                self.enter_inspector();
                true
            }
            "escape" if self.zone == Zone::Inspector => {
                self.escape_inspector();
                true
            }
            "ctrl-1" => {
                self.zone = Zone::Sidebar;
                true
            }
            "ctrl-2" => {
                self.zone = Zone::Outline;
                true
            }
            _ if self.zone == Zone::Inspector => self.inspector_key(key),
            _ => false,
        }
    }

    fn inspector_key(&mut self, key: &str) -> bool {
        use dd_siteforge_core::editform::FieldKind;
        match key {
            "tab" | "down" | "j" => {
                self.edit.focus_next();
                true
            }
            "shift-tab" | "up" | "k" => {
                self.edit.focus_prev();
                true
            }
            "left" => {
                self.edit.cycle_enum(false);
                true
            }
            "right" => {
                self.edit.cycle_enum(true);
                true
            }
            "shift-a" => {
                let change = self.edit.add_item();
                self.toast_sub(change);
                true
            }
            "shift-x" => {
                let change = self.edit.remove_item();
                self.toast_sub(change);
                true
            }
            "shift-j" => {
                let change = self.edit.move_item(true);
                self.toast_sub(change);
                true
            }
            "shift-k" => {
                let change = self.edit.move_item(false);
                self.toast_sub(change);
                true
            }
            "enter" => {
                let kind = self
                    .edit
                    .active()
                    .and_then(|draft| draft.state.focused())
                    .map(|field| std::mem::discriminant(&field.kind));
                let subform = self.edit.active().and_then(|draft| {
                    draft
                        .state
                        .focused()
                        .map(|field| matches!(field.kind, FieldKind::SubForm { .. }))
                });
                let textarea = self.edit.active().and_then(|draft| {
                    draft
                        .state
                        .focused()
                        .map(|field| matches!(field.kind, FieldKind::Textarea { .. }))
                });
                let _ = kind;
                if subform == Some(true) {
                    if self.edit.drill().is_ok() {
                        self.toast("Editing item. Ctrl+S returns to parent.");
                    }
                } else if textarea == Some(true) {
                    let _ = self.edit.type_char('\n');
                } else {
                    self.edit.focus_next();
                }
                true
            }
            "backspace" => {
                let _ = self.edit.backspace_field();
                true
            }
            "ctrl-o" => {
                self.browse_focused_image();
                true
            }
            other if is_text(other) => {
                if let Some(ch) = other.chars().next() {
                    let _ = self.edit.type_char(ch);
                }
                true
            }
            _ => false,
        }
    }

    fn toast_sub(&mut self, change: SubChange) {
        let message = match change {
            SubChange::Added => "Item added.",
            SubChange::Removed => "Item removed.",
            SubChange::MovedDown => "Moved item down.",
            SubChange::MovedUp => "Moved item up.",
            SubChange::AtMax(max) => {
                self.toast(format!("Maximum {max} item(s)."));
                return;
            }
            SubChange::AtMin(min) => {
                self.toast(format!("Must keep at least {min} item(s)."));
                return;
            }
            SubChange::Ignored => return,
        };
        self.toast(message);
    }

    fn leave_draft(&mut self) {
        if self.edit.active_key().is_none() {
            return;
        }
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        match self.edit.leave(&mut doc.site) {
            Commit::Applied => doc.mark_dirty(),
            Commit::Blocked(message) => self.toast(message),
            Commit::Failed(message) => self.toast(message),
            Commit::ItemSaved => self.toast("Item saved — editing parent."),
        }
    }

    fn enter_inspector(&mut self) {
        self.activate_outline();
        self.zone = Zone::Inspector;
        self.open_current_draft();
    }

    pub fn focus_inspector(&mut self) {
        self.enter_inspector();
    }

    fn open_current_draft(&mut self) {
        let Some(doc) = &self.document else {
            return;
        };
        let Some(cursor) = current_cursor(&self.region, self.page_index, &self.target) else {
            return;
        };
        let key = match self.target {
            Target::Column { node, column } => {
                format!("column:{}:{node}:{column}", self.page_index)
            }
            Target::RegionColumn { section, column } => {
                format!("region-column:{section}:{column}")
            }
            _ => format!("{cursor:?}"),
        };
        if self.edit.activate(&key) {
            return;
        }
        let source = match self.target {
            Target::Column { node, .. } => Target::Section(node),
            Target::RegionColumn { section, .. } => Target::RegionSection(section),
            _ => self.target.clone(),
        };
        let Some(state) = form_state(&doc.site, &self.region, self.page_index, &source) else {
            return;
        };
        self.edit.open(key, cursor, state);
        if let Target::Column { column, .. } | Target::RegionColumn { column, .. } = self.target {
            self.edit.focus_field("columns");
            self.edit.select_item(column);
            let _ = self.edit.drill();
        }
    }

    fn escape_inspector(&mut self) {
        let expanded = self.edit.active().is_some_and(|draft| draft.expanded);
        let drilled = self.edit.back_label().is_some();
        if !self.edit.discard() {
            self.zone = Zone::Outline;
            return;
        }
        if expanded {
            return;
        }
        if drilled {
            self.toast("Item edit cancelled.");
            return;
        }
        self.zone = Zone::Outline;
    }

    fn undo_edit(&mut self) {
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        match self.edit.undo(&mut doc.site) {
            Ok(message) => {
                doc.mark_dirty();
                self.toast(message);
            }
            Err(message) => self.toast(message),
        }
    }

    fn redo_edit(&mut self) {
        self.redo_handled = true;
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        match self.edit.redo(&mut doc.site) {
            Ok(message) => {
                doc.mark_dirty();
                self.toast(message);
            }
            Err(message) => self.toast(message),
        }
    }

    fn open_insert(&mut self) {
        self.insert_open = true;
        self.insert_query.clear();
        self.insert_index = 0;
    }

    fn insert_key(&mut self, key: &str) {
        let kinds = self.filtered_kinds();
        match key {
            "escape" => {
                self.insert_open = false;
                self.toast("Component picker cancelled.");
            }
            "up" | "k" if self.insert_index > 0 => self.insert_index -= 1,
            "down" | "j" if self.insert_index + 1 < kinds.len() => self.insert_index += 1,
            "enter" => self.commit_insert(&kinds),
            "backspace" => {
                self.insert_query.pop();
                self.insert_index = 0;
            }
            other if is_text(other) => {
                self.insert_query.push_str(other);
                self.insert_index = 0;
            }
            _ => {}
        }
    }

    fn filtered_kinds(&self) -> Vec<ComponentKind> {
        let region = match self.region {
            SideTarget::Header => InsertRegion::Header,
            SideTarget::Footer => InsertRegion::Footer,
            SideTarget::Site => InsertRegion::Site,
            _ => InsertRegion::Page,
        };
        let query = self.insert_query.to_ascii_lowercase();
        ComponentKind::all()
            .iter()
            .copied()
            .filter(|kind| kind.allowed_in(region))
            .filter(|kind| query.is_empty() || kind.label().contains(&query))
            .collect()
    }

    fn commit_insert(&mut self, kinds: &[ComponentKind]) {
        let Some(kind) = kinds.get(self.insert_index).copied() else {
            self.toast("No component selected.");
            self.insert_open = false;
            return;
        };
        let (page, node, column, component) = self.insert_place();
        let region = match self.region {
            SideTarget::Header => InsertRegion::Header,
            SideTarget::Footer => InsertRegion::Footer,
            SideTarget::Site => InsertRegion::Site,
            _ => InsertRegion::Page,
        };
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        match self
            .edit
            .insert_component(&mut doc.site, page, node, column, component, kind, region)
        {
            Ok(message) => {
                doc.mark_dirty();
                self.toast(message);
                self.insert_open = false;
            }
            Err(message) => {
                self.toast(message);
                self.insert_open = false;
            }
        }
    }

    fn insert_place(&self) -> (usize, usize, usize, usize) {
        let page = self.page_index;
        match self.target {
            Target::Component {
                node,
                column,
                component,
            } => (page, node, column, component),
            Target::Section(node) | Target::Hero(node) => (page, node, 0, 0),
            Target::Column { node, column } => (page, node, column, 0),
            _ => (page, 0, 0, 0),
        }
    }

    fn delete_selection(&mut self) {
        let Target::Component {
            node,
            column,
            component,
        } = self.target
        else {
            self.toast("Nothing selected to delete.");
            return;
        };
        let page = self.page_index;
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        match self
            .edit
            .delete_component(&mut doc.site, page, node, column, component)
        {
            Ok(message) => {
                doc.mark_dirty();
                self.toast(message);
            }
            Err(message) => self.toast(message),
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let Target::Component {
            node,
            column,
            component,
        } = self.target
        else {
            return;
        };
        let page = self.page_index;
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        match self
            .edit
            .move_component(&mut doc.site, page, node, column, component, delta)
        {
            Ok(dest) => {
                doc.mark_dirty();
                self.target = Target::Component {
                    node,
                    column,
                    component: dest,
                };
                self.toast("Moved component.");
            }
            Err(_) => {}
        }
    }

    fn finish_row_drag(&mut self, target: Target) {
        let Some(from) = self.drag_arm.take() else {
            return;
        };
        let (
            Target::Component {
                node,
                column,
                component,
            },
            Target::Component {
                node: to_node,
                column: to_column,
                component: to,
            },
        ) = (from, target)
        else {
            return;
        };
        if node != to_node || column != to_column || component == to {
            return;
        }
        self.target = Target::Component {
            node,
            column,
            component,
        };
        self.drag_component_to(to);
    }

    pub fn drag_component_to(&mut self, to: usize) {
        let Target::Component {
            node,
            column,
            component,
        } = self.target
        else {
            return;
        };
        let page = self.page_index;
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        if let Ok(dest) = self
            .edit
            .drag_component(&mut doc.site, page, node, column, component, to)
        {
            doc.mark_dirty();
            self.target = Target::Component {
                node,
                column,
                component: dest,
            };
        }
    }

    pub fn drag_collection_to(&mut self, from: usize, to: usize) {
        if self.edit.drag_item(from, to) {
            self.toast("Moved item.");
        }
    }

    fn begin_rename(&mut self) {
        if !matches!(self.region, SideTarget::Page(_)) {
            return;
        }
        let title = self
            .document
            .as_ref()
            .and_then(|doc| doc.site.pages.get(self.page_index))
            .map(|page| page.head.title.clone())
            .unwrap_or_default();
        self.rename = Some(title);
        self.toast("Rename page. Edit and press Enter.");
    }

    fn commit_rename(&mut self) {
        let Some(title) = self.rename.clone() else {
            return;
        };
        let page = self.page_index;
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        match self.edit.rename_page_at(&mut doc.site, page, &title) {
            Ok(message) => {
                doc.mark_dirty();
                self.rename = None;
                self.toast(message);
            }
            Err(message) => self.toast(message),
        }
    }

    pub fn rename_page_to(&mut self, title: &str) {
        self.rename = Some(title.to_string());
        self.commit_rename();
    }

    fn browse_focused_image(&mut self) {
        let Some(draft) = self.edit.active() else {
            return;
        };
        let Some(field) = draft.state.focused() else {
            return;
        };
        if !field.id.contains("image") && !field.id.contains("poster") && !field.id.contains("mp4")
        {
            return;
        }
        let id = field.id.to_string();
        self.toast(format!("Browse… {id}"));
    }

    /// Portal and drag-and-drop both land here. A large raster asks before shrinking.
    pub fn import_dropped(&mut self, path: PathBuf, downscale: Option<bool>) {
        let Some(field_id) = self.focused_import_field() else {
            self.toast("Pick an image field first.");
            return;
        };
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(err) => {
                self.toast(format!("Source folder not found: {err}"));
                return;
            }
        };
        if downscale.is_none() {
            if let Some(edge) = crate::images::long_edge(&bytes) {
                if edge > crate::images::LONG_EDGE {
                    self.shrink = Some(ShrinkOffer {
                        source: path,
                        field_id,
                        long_edge: edge,
                    });
                    return;
                }
            }
        }
        self.write_import(&path, field_id, downscale.unwrap_or(false));
    }

    fn finish_shrink(&mut self, downscale: bool) {
        let Some(offer) = self.shrink.take() else {
            return;
        };
        self.write_import(&offer.source, offer.field_id, downscale);
    }

    fn write_import(&mut self, path: &Path, field_id: String, downscale: bool) {
        let Some(doc) = &self.document else {
            return;
        };
        let dir = crate::images::images_dir(&doc.path);
        match crate::images::import_path(path, &dir, downscale) {
            Ok(imported) => {
                if self.edit.set_field(&field_id, imported.stored.clone()) {
                    self.toast(format!("Picked: {}", imported.stored));
                }
            }
            Err(err) => self.toast(format!("Save failed: {err}")),
        }
    }

    fn focused_import_field(&self) -> Option<String> {
        let field = self.edit.active()?.state.focused()?;
        (field.id.contains("image") || field.id.contains("poster") || field.id.contains("mp4"))
            .then(|| field.id.to_string())
    }

    pub fn set_field_value(&mut self, id: &str, value: &str) -> bool {
        self.edit.set_field(id, value)
    }

    pub fn focus_field_id(&mut self, id: &str) -> bool {
        self.edit.focus_field(id)
    }

    pub fn insert_markdown(&mut self, which: MarkdownInsert) -> bool {
        self.edit.insert_markdown(which)
    }

    pub fn undo_depth(&self) -> usize {
        self.edit.undo_len()
    }

    fn portal_folder(&mut self, cx: &mut Context<Self>) {
        if !self.quit_runtime {
            return;
        }
        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Site folder".into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            this.update(cx, |shell, cx| {
                shell.new_site_folder(&path.display().to_string());
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

impl Focusable for Shell {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.quit && self.quit_runtime {
            cx.quit();
        }
        let width = f32::from(window.viewport_size().width);
        self.last_width = width;
        let theme = &self.theme.gui;
        let base = token(theme, "surface.base");
        let panel = token(theme, "surface.panel");
        let text = token(theme, "text.primary");
        let secondary = token(theme, "text.secondary");
        let placeholder = token(theme, "text.placeholder");
        let accent = token(theme, "accent.fill");
        let error = token(theme, "status.error-text");
        let error_fill = token(theme, "status.error-subtle");
        let info_fill = token(theme, "status.info-subtle");
        let info = token(theme, "status.info");
        let border = token(theme, "border.subtle");
        let border_card = token(theme, "border.card");
        let border_active = token(theme, "border.active");
        let raised = token(theme, "surface.raised");
        let input = token(theme, "surface.input");
        let selected = token(theme, "selection.bg");
        let family = gpui::SharedString::from(self.theme.fonts.ui.clone());
        let (sidebar_on, inspector_on) = self.docks(width);
        let outline_width = self.outline_content_width(width);
        let selected_target = self.target.clone();
        let save = self
            .document
            .as_ref()
            .filter(|_| self.screen == Screen::Editor)
            .map(|doc| doc.indicator_text())
            .unwrap_or_default();
        let title = self.title_name();
        let outline = self.outline_with_draft_issues();
        let sidebar = self
            .document
            .as_ref()
            .map(|doc| sidebar_rows(&doc.site, &self.filter));
        let view = if self.screen == Screen::Editor {
            Some(self.inspector_view())
        } else {
            None
        };
        let welcome = if self.screen == Screen::Welcome {
            self.welcome_items()
        } else {
            Vec::new()
        };
        let issues = if self.screen == Screen::Editor {
            self.issue_count()
        } else {
            0
        };
        let meta = self
            .document
            .as_ref()
            .map(|doc| status_meta(&doc.site, &self.region, self.page_index));
        let caption = self
            .document
            .as_ref()
            .map(|doc| outline_caption(&doc.site, &self.region, self.page_index));
        let crumb = self
            .document
            .as_ref()
            .map(|doc| title_crumb(&doc.site, &self.region, self.page_index));
        let load = self
            .load_error
            .as_ref()
            .map(|err| (err.folder.clone(), err.message.clone()));
        let backup = self.backup_note.clone();
        let conflict = self.document.as_ref().is_some_and(|doc| doc.conflict);
        let save_error = self
            .document
            .as_ref()
            .and_then(|doc| doc.save_error.clone());
        let new_site_open = self.new_site.open;
        let folder_value = self.new_site.folder.clone();
        let slug = if new_site_open {
            self.slug_preview()
        } else {
            String::new()
        };
        let new_site_error = self.new_site.error.clone();
        let create_enabled =
            new_site_open && !folder_value.trim().is_empty() && new_site_error.is_none();
        let template_open = self.template_open;
        let template_index = self.template_index;
        let switcher = self.switcher;
        let recents = if switcher { self.recents() } else { Vec::new() };
        let quit_dialog = self.quit_dialog;
        let restore_open = self.restore_open;
        let quit_body = if quit_dialog {
            self.quit_body()
        } else {
            String::new()
        };
        let restore_body = if restore_open {
            self.restore_body()
        } else {
            String::new()
        };
        let toasts = self.toasts.clone();
        let preview_note = self.preview_note.clone();
        let mode_label = self.theme.mode_label();
        let theme_warning = self.theme.warning.clone();
        let filter = self.filter.clone();
        let welcome_index = self.welcome_index;
        let sidebar_index = self.sidebar_index;
        let outline_index = self.outline_index;
        let active_tab = view.as_ref().and_then(|view| {
            self.tab
                .filter(|tab| view.tabs.contains(tab))
                .or(view.tabs.first().copied())
        });
        let show_backup_button = self.screen == Screen::Error && self.backup_available();
        let bar = if self.settings.density == crate::settings::Density::Compact {
            34.0
        } else {
            40.0
        };
        let status_h = if self.settings.density == crate::settings::Density::Compact {
            22.0
        } else {
            24.0
        };

        div()
            .id("shell")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|shell, event: &KeyDownEvent, _, cx| {
                shell.on_key_down(event, cx);
                cx.notify();
            }))
            .on_drop(cx.listener(|shell, paths: &gpui::ExternalPaths, _, cx| {
                if let Some(path) = paths.paths().first().cloned() {
                    shell.import_dropped(path, None);
                }
                cx.notify();
            }))
            .size_full()
            .flex()
            .flex_col()
            .bg(base)
            .text_color(text)
            .font_family(family)
            .text_size(px(13.0))
            .child(title_bar(
                cx,
                &title,
                crumb.unwrap_or_default(),
                self.screen == Screen::Editor,
                sidebar_on,
                inspector_on,
                bar,
                secondary,
                placeholder,
                border,
                selected,
            ))
            .when(save_error.is_some(), |row| {
                let err = save_error.clone().unwrap_or_default();
                row.child(banner(cx, &err, error, error_fill, secondary, border))
            })
            .when(conflict, |row| {
                row.child(conflict_banner(cx, panel, secondary))
            })
            .when(backup.is_some(), |row| {
                row.child(info_banner(
                    cx,
                    backup.clone().unwrap_or_default(),
                    info,
                    info_fill,
                    secondary,
                ))
            })
            .child(
                div()
                    .flex()
                    .flex_1()
                    .overflow_hidden()
                    .when(self.screen == Screen::Welcome, |body| {
                        body.child(welcome_card(
                            cx,
                            &welcome,
                            welcome_index,
                            panel,
                            border,
                            secondary,
                            placeholder,
                        ))
                    })
                    .when(self.screen == Screen::Loading, |body| {
                        body.child(centered(
                            vec![
                                format!("Loading {}", self.loading_label),
                                "Reading site.json…".to_string(),
                            ],
                            secondary,
                        ))
                    })
                    .when(self.screen == Screen::Error, |body| {
                        let (folder, message) = load.clone().unwrap_or_default();
                        body.child(load_error_view(
                            cx,
                            &folder,
                            &message,
                            show_backup_button,
                            error,
                            secondary,
                        ))
                    })
                    .when(self.screen == Screen::Editor, |body| {
                        body.child(editor_body(
                            cx,
                            sidebar.clone(),
                            outline.clone(),
                            view.clone(),
                            caption.clone(),
                            filter,
                            sidebar_index,
                            outline_index,
                            outline_width,
                            &selected_target,
                            sidebar_on,
                            inspector_on,
                            active_tab,
                            panel,
                            base,
                            raised,
                            input,
                            selected,
                            secondary,
                            placeholder,
                            accent,
                            error,
                            border,
                            border_card,
                            border_active,
                            text,
                        ))
                    }),
            )
            .child(status_bar(
                self.screen,
                &save,
                meta.as_deref().unwrap_or(""),
                issues,
                secondary,
                placeholder,
                error,
                border,
                status_h,
                cx,
            ))
            .when(new_site_open, |row| {
                row.child(new_site_sheet(
                    cx,
                    &folder_value,
                    &slug,
                    new_site_error.as_deref(),
                    create_enabled,
                    panel,
                    border,
                    secondary,
                    text,
                ))
            })
            .when(template_open, |row| {
                row.child(template_sheet(
                    cx,
                    template_index,
                    panel,
                    border,
                    selected,
                    secondary,
                ))
            })
            .when(switcher, |row| {
                row.child(switcher_menu(cx, &recents, panel, border, secondary))
            })
            .when(quit_dialog, |row| {
                row.child(confirm_dialog(
                    cx,
                    "Unsaved changes. Quit anyway?",
                    &quit_body,
                    "Quit anyway",
                    true,
                    panel,
                    border,
                    secondary,
                ))
            })
            .when(restore_open, |row| {
                row.child(confirm_dialog(
                    cx,
                    "Restore from checkpoint?",
                    &restore_body,
                    "Restore",
                    true,
                    panel,
                    border,
                    secondary,
                ))
            })
            .children(toasts.into_iter().map(|toast| {
                div()
                    .px(px(12.0))
                    .py(px(8.0))
                    .bg(panel)
                    .border_1()
                    .border_color(border)
                    .child(toast)
            }))
            .when(!preview_note.is_empty(), |row| {
                row.child(div().px(px(12.0)).child(preview_note.clone()))
            })
            .child(div().px(px(12.0)).text_color(placeholder).child(mode_label))
            .when(theme_warning.is_some(), |row| {
                row.child(
                    div()
                        .px(px(12.0))
                        .text_color(secondary)
                        .child(theme_warning.clone().unwrap_or_default()),
                )
            })
    }
}

fn title_bar(
    cx: &mut Context<Shell>,
    title: &str,
    crumb: String,
    editor: bool,
    sidebar_on: bool,
    inspector_on: bool,
    height: f32,
    secondary: gpui::Hsla,
    placeholder: gpui::Hsla,
    border: gpui::Hsla,
    selected: gpui::Hsla,
) -> gpui::Div {
    div()
        .h(px(height))
        .px(px(8.0))
        .flex()
        .items_center()
        .gap(px(6.0))
        .border_b_1()
        .border_color(border)
        .child(
            Button::new("toggle-sidebar")
                .label(if sidebar_on {
                    "Sidebar".to_string()
                } else {
                    show_sidebar_tooltip()
                })
                .on_click(cx.listener(|shell, _: &ClickEvent, window, cx| {
                    let width = f32::from(window.viewport_size().width);
                    shell.toggle_sidebar(width);
                    cx.notify();
                })),
        )
        .child(
            div()
                .id("site-switcher")
                .px(px(4.0))
                .when(editor, |row| row.bg(selected))
                .child(title.to_string())
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    if shell.screen == Screen::Editor {
                        shell.toggle_switcher();
                        cx.notify();
                    }
                })),
        )
        .when(editor && !crumb.is_empty(), |row| {
            row.child(div().text_color(secondary).child(crumb))
        })
        .when(!editor, |row| {
            row.child(div().text_color(placeholder).child(""))
        })
        .child(div().flex_1())
        .when(editor, |row| {
            row.child(
                Button::new("toggle-inspector")
                    .label(if inspector_on {
                        "Inspector".to_string()
                    } else {
                        show_inspector_tooltip()
                    })
                    .on_click(cx.listener(|shell, _: &ClickEvent, window, cx| {
                        let width = f32::from(window.viewport_size().width);
                        shell.toggle_inspector(width);
                        cx.notify();
                    })),
            )
        })
}

fn banner(
    cx: &mut Context<Shell>,
    err: &str,
    error: gpui::Hsla,
    fill: gpui::Hsla,
    secondary: gpui::Hsla,
    border: gpui::Hsla,
) -> gpui::Div {
    div()
        .px(px(16.0))
        .py(px(8.0))
        .flex()
        .items_center()
        .gap(px(12.0))
        .bg(fill)
        .border_l_2()
        .border_color(error)
        .border_b_1()
        .child(div().text_color(error).child("Autosave failed"))
        .child(div().text_color(secondary).child(err.to_string()))
        .child(
            Button::new("save-as")
                .label("Save as…")
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    shell.request_save_as();
                    shell.portal_save_as(cx);
                    cx.notify();
                })),
        )
        .child(
            Button::new("retry-save")
                .label("Retry")
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    shell.retry_save();
                    cx.notify();
                })),
        )
        .border_color(border)
}

fn conflict_banner(cx: &mut Context<Shell>, panel: gpui::Hsla, secondary: gpui::Hsla) -> gpui::Div {
    div()
        .px(px(16.0))
        .py(px(8.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .bg(panel)
        .child(
            div()
                .text_color(secondary)
                .child("Changed on disk. Not overwritten. Reload, or keep yours."),
        )
        .child(
            Button::new("reload-disk")
                .label("Reload")
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    shell.reload_from_disk();
                    cx.notify();
                })),
        )
        .child(
            Button::new("keep-mine")
                .label("Keep mine")
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    shell.keep_mine();
                    cx.notify();
                })),
        )
}

fn info_banner(
    cx: &mut Context<Shell>,
    text: String,
    info: gpui::Hsla,
    fill: gpui::Hsla,
    secondary: gpui::Hsla,
) -> gpui::Div {
    div()
        .px(px(16.0))
        .py(px(8.0))
        .flex()
        .gap(px(12.0))
        .bg(fill)
        .border_l_2()
        .border_color(info)
        .child(div().text_color(secondary).child(text))
        .child(
            Button::new("restore-checkpoint")
                .label("Restore checkpoint…")
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    shell.open_restore();
                    cx.notify();
                })),
        )
        .child(
            Button::new("keep-current")
                .label("Keep current")
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    shell.dismiss_backup();
                    cx.notify();
                })),
        )
}

fn welcome_card(
    cx: &mut Context<Shell>,
    rows: &[WelcomeRow],
    selected: usize,
    panel: gpui::Hsla,
    border: gpui::Hsla,
    secondary: gpui::Hsla,
    placeholder: gpui::Hsla,
) -> gpui::Div {
    div().flex().flex_1().items_center().justify_center().child(
        div()
            .w(px(680.0))
            .p(px(8.0))
            .bg(panel)
            .border_1()
            .border_color(border)
            .rounded(px(8.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(div().text_size(px(16.0)).child("Open site"))
            .child(div().text_color(secondary).child("dd_siteforge"))
            .child(div().text_color(placeholder).child("Type a path"))
            .children(rows.iter().enumerate().map(|(index, row)| {
                let label = row.label.clone();
                div()
                    .id(("welcome", index))
                    .px(px(8.0))
                    .py(px(4.0))
                    .child(if index == selected {
                        format!("> {label}")
                    } else {
                        label
                    })
                    .on_click(cx.listener(move |shell, _: &ClickEvent, _, cx| {
                        shell.welcome_index = index;
                        shell.activate_welcome();
                        cx.notify();
                    }))
            }))
            .child(
                div()
                    .text_color(placeholder)
                    .child("↑↓ select · Enter open · Esc quit"),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .child(
                        Button::new("open-file")
                            .label("Open…")
                            .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                                shell.request_choose_file();
                                shell.portal_choose(cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("new-site")
                            .label("New site…")
                            .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                                shell.open_new_site(None);
                                cx.notify();
                            })),
                    ),
            ),
    )
}

fn load_error_view(
    cx: &mut Context<Shell>,
    folder: &str,
    message: &str,
    backup: bool,
    error: gpui::Hsla,
    secondary: gpui::Hsla,
) -> gpui::Div {
    div().flex().flex_1().items_center().justify_center().child(
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.0))
            .child(
                div()
                    .text_size(px(16.0))
                    .child(format!("Couldn't load {folder}")),
            )
            .child(div().text_color(error).child(message.to_string()))
            .child(
                div()
                    .text_color(secondary)
                    .child(format!("{folder}/site.json · the file was not changed")),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .child(
                        Button::new("retry-load")
                            .label("Retry")
                            .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                                shell.retry_load();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("choose-file")
                            .label("Choose file…")
                            .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                                shell.request_choose_file();
                                shell.portal_choose(cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("reveal")
                            .label("Reveal in folder")
                            .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                                shell.reveal_in_folder();
                                if let Some(path) = shell.revealed.clone() {
                                    cx.reveal_path(&path);
                                }
                                cx.notify();
                            })),
                    )
                    .when(backup, |row| {
                        row.child(Button::new("open-backup").label("Open backup").on_click(
                            cx.listener(|shell, _: &ClickEvent, _, cx| {
                                shell.open_restore();
                                cx.notify();
                            }),
                        ))
                    }),
            ),
    )
}

fn editor_body(
    cx: &mut Context<Shell>,
    sidebar: Option<(Vec<SideRow>, String)>,
    outline: Option<Vec<OutlineRow>>,
    view: Option<InspectorView>,
    caption: Option<(String, String)>,
    filter: String,
    sidebar_index: usize,
    outline_index: usize,
    outline_width: f32,
    selected_target: &Target,
    sidebar_on: bool,
    inspector_on: bool,
    active_tab: Option<InspTab>,
    panel: gpui::Hsla,
    base: gpui::Hsla,
    raised: gpui::Hsla,
    input: gpui::Hsla,
    selected: gpui::Hsla,
    secondary: gpui::Hsla,
    placeholder: gpui::Hsla,
    accent: gpui::Hsla,
    error: gpui::Hsla,
    border: gpui::Hsla,
    border_card: gpui::Hsla,
    border_active: gpui::Hsla,
    text: gpui::Hsla,
) -> gpui::Div {
    let rows = sidebar.clone().map(|(rows, _)| rows).unwrap_or_default();
    let count = sidebar.clone().map(|(_, count)| count).unwrap_or_default();
    let outline = outline.unwrap_or_default();
    let (heading, cap) = caption.unwrap_or_default();
    div()
        .flex()
        .flex_1()
        .overflow_hidden()
        .when(sidebar_on, |body| {
            body.child(sidebar_view(
                cx,
                &rows,
                &count,
                &filter,
                sidebar_index,
                panel,
                selected,
                secondary,
                border,
                accent,
            ))
        })
        .child(outline_view(
            cx,
            &heading,
            &cap,
            &outline,
            outline_index,
            outline_width,
            selected_target,
            base,
            raised,
            panel,
            selected,
            secondary,
            placeholder,
            border,
            border_card,
            border_active,
            accent,
            error,
        ))
        .when(inspector_on, |body| {
            body.child(inspector_view(
                cx,
                view.as_ref(),
                active_tab,
                panel,
                input,
                secondary,
                placeholder,
                error,
                border,
                text,
            ))
        })
}

fn sidebar_view(
    cx: &mut Context<Shell>,
    rows: &[SideRow],
    count: &str,
    filter: &str,
    selected: usize,
    panel: gpui::Hsla,
    selected_bg: gpui::Hsla,
    secondary: gpui::Hsla,
    border: gpui::Hsla,
    accent: gpui::Hsla,
) -> gpui::Div {
    div()
        .w(px(280.0))
        .h_full()
        .bg(panel)
        .border_r_1()
        .border_color(border)
        .flex()
        .flex_col()
        .child(div().p(px(12.0)).child(if filter.is_empty() {
            "Filter pages…".to_string()
        } else {
            filter.to_string()
        }))
        .child(
            div()
                .px(px(12.0))
                .text_color(secondary)
                .child(format!("PAGES {count}")),
        )
        .child(
            div()
                .id("sidebar-rows")
                .flex_1()
                .overflow_y_scroll()
                .children(rows.iter().enumerate().map(|(index, row)| {
                    let mark = if index == selected { "> " } else { "" };
                    let badges = row.badges.join(" ");
                    div()
                        .id(("side", index))
                        .h(px(28.0))
                        .pl(px(16.0 * row.depth as f32 + 8.0))
                        .when(index == selected, |row| {
                            row.bg(selected_bg).border_l_2().border_color(accent)
                        })
                        .child(format!("{mark}{} {badges}", row.label))
                        .on_click(cx.listener(move |shell, event: &ClickEvent, _, cx| {
                            shell.select_sidebar(index);
                            if event.click_count() >= 2 {
                                shell.begin_rename();
                            }
                            cx.notify();
                        }))
                })),
        )
}

fn outline_view(
    cx: &mut Context<Shell>,
    heading: &str,
    caption: &str,
    rows: &[OutlineRow],
    selected: usize,
    outline_width: f32,
    selected_target: &Target,
    base: gpui::Hsla,
    raised: gpui::Hsla,
    panel: gpui::Hsla,
    selected_bg: gpui::Hsla,
    secondary: gpui::Hsla,
    placeholder: gpui::Hsla,
    border: gpui::Hsla,
    border_card: gpui::Hsla,
    border_active: gpui::Hsla,
    accent: gpui::Hsla,
    error: gpui::Hsla,
) -> gpui::Div {
    div()
        .flex_1()
        .h_full()
        .bg(base)
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(
            div()
                .px(px(16.0))
                .py(px(12.0))
                .border_b_1()
                .border_color(border)
                .child(heading.to_string())
                .child(div().text_color(secondary).child(caption.to_string())),
        )
        .child(
            div()
                .flex_1()
                .id("outline-rows")
                .overflow_y_scroll()
                .p(px(12.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(rows.iter().enumerate().map(|(index, row)| {
                    let grid = column_grid(
                        cx,
                        row,
                        outline_width,
                        selected_target,
                        panel,
                        raised,
                        secondary,
                        placeholder,
                        border,
                        border_card,
                        border_active,
                    );
                    let drag_target = row.target.clone();
                    let drag_target_up = drag_target.clone();
                    div()
                        .id(("outline", index))
                        .when(row.columns.is_empty(), |card| {
                            card.p(px(12.0))
                                .bg(if index == selected {
                                    selected_bg
                                } else {
                                    raised
                                })
                                .border_1()
                                .border_color(if index == selected { accent } else { border })
                                .ml(px(16.0 * row.depth as f32))
                        })
                        .child(row.label.clone())
                        .when(!row.detail.is_empty(), |card| {
                            card.child(div().text_color(secondary).child(row.detail.clone()))
                        })
                        .when(row.issues > 0, |card| {
                            card.child(div().text_color(error).child(format!(
                                "{} issue{}",
                                row.issues,
                                if row.issues == 1 { "" } else { "s" }
                            )))
                        })
                        .when(grid.is_some(), |card| card.child(grid.unwrap()))
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(move |shell, _, _, _| {
                                shell.drag_arm = Some(drag_target.clone());
                            }),
                        )
                        .on_mouse_up(
                            gpui::MouseButton::Left,
                            cx.listener(move |shell, _, _, cx| {
                                shell.finish_row_drag(drag_target_up.clone());
                                cx.notify();
                            }),
                        )
                        .on_click(cx.listener(move |shell, _: &ClickEvent, _, cx| {
                            shell.select_outline(index);
                            cx.notify();
                        }))
                }))
                .when(rows.is_empty(), |col| {
                    col.child(div().text_color(secondary).child("No nodes on this page."))
                }),
        )
}

fn column_grid(
    cx: &mut Context<Shell>,
    row: &OutlineRow,
    outline_width: f32,
    selected_target: &Target,
    panel: gpui::Hsla,
    raised: gpui::Hsla,
    secondary: gpui::Hsla,
    placeholder: gpui::Hsla,
    border: gpui::Hsla,
    border_card: gpui::Hsla,
    border_active: gpui::Hsla,
) -> Option<gpui::Div> {
    if row.columns.is_empty() {
        return None;
    }
    // DESIGN-GAP: Screens 33 and 34 do not draw this row. Under 140px the
    // proportional columns stack so the chips stay readable.
    let stack = column_layout(&row.columns, outline_width) == "stack";
    let columns = row.columns.clone();
    Some(
        div()
            .mt(px(8.0))
            .w_full()
            .gap(px(8.0))
            .when(!stack, |grid| grid.flex().flex_row())
            .when(stack, |grid| grid.flex().flex_col())
            .children(columns.into_iter().enumerate().map(|(index, column)| {
                let selected = column.target == *selected_target;
                let empty = column.empty;
                let label = column.label.clone();
                let md_class = column.md_class.clone();
                let chips = column.chips.clone();
                div()
                    .id(("column", index))
                    .when(!stack, |card| {
                        card.flex_grow(column.parts as f32).flex_basis(px(0.0))
                    })
                    .when(stack, |card| card.w_full())
                    .p(px(8.0))
                    .bg(panel)
                    .rounded(px(6.0))
                    .border_1()
                    .when(empty && !selected, |card| {
                        card.border_dashed().border_color(border)
                    })
                    .when(!empty && !selected, |card| card.border_color(border_card))
                    .when(selected, |card| card.border_2().border_color(border_active))
                    .child(div().text_color(secondary).child(label))
                    .child(div().text_color(secondary).child(md_class))
                    .when(empty, |card| {
                        card.child(div().text_color(placeholder).child("Empty · / to add"))
                    })
                    .children(chips.into_iter().map(|chip| {
                        div()
                            .mt(px(6.0))
                            .p(px(8.0))
                            .bg(raised)
                            .border_1()
                            .border_color(border_card)
                            .rounded(px(4.0))
                            .child(chip)
                    }))
                    .on_click(cx.listener(move |shell, _: &ClickEvent, _, cx| {
                        shell.select_column(&column.label);
                        cx.notify();
                    }))
            })),
    )
}

fn inspector_view(
    cx: &mut Context<Shell>,
    view: Option<&InspectorView>,
    active: Option<InspTab>,
    panel: gpui::Hsla,
    input: gpui::Hsla,
    secondary: gpui::Hsla,
    placeholder: gpui::Hsla,
    error: gpui::Hsla,
    border: gpui::Hsla,
    _text: gpui::Hsla,
) -> gpui::Stateful<gpui::Div> {
    let Some(view) = view else {
        return div().id("inspector-empty").w(px(360.0)).bg(panel);
    };
    if view.empty {
        return div()
            .id("inspector-none")
            .w(px(360.0))
            .h_full()
            .bg(panel)
            .border_l_1()
            .border_color(border)
            .p(px(16.0))
            .child("Nothing selected")
            .child(div().text_color(secondary).child(
                "Select a block in the Outline, or press Enter on a row, to edit its fields here.",
            ));
    }
    div()
        .id("inspector-fields")
        .w(px(360.0))
        .h_full()
        .bg(panel)
        .border_l_1()
        .border_color(border)
        .overflow_y_scroll()
        .p(px(16.0))
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(div().text_color(secondary).child(view.breadcrumb.clone()))
        .child(view.title.clone())
        .when(view.tabs.len() > 1, |col| {
            col.child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .children(view.tabs.iter().enumerate().map(|(index, tab)| {
                        let label = tab.label();
                        let chosen = *tab;
                        let on = active == Some(chosen);
                        div()
                            .id(("tab", index))
                            .px(px(8.0))
                            .child(label)
                            .when(on, |row| row.border_b_1().border_color(border))
                            .on_click(cx.listener(move |shell, _: &ClickEvent, _, cx| {
                                shell.select_tab(chosen);
                                cx.notify();
                            }))
                    })),
            )
        })
        .children(
            view.fields
                .iter()
                .map(|field| field_view(field, input, secondary, placeholder, error, border)),
        )
}

fn field_view(
    field: &crate::present::FieldView,
    input: gpui::Hsla,
    secondary: gpui::Hsla,
    placeholder: gpui::Hsla,
    error: gpui::Hsla,
    border: gpui::Hsla,
) -> gpui::Div {
    let value_color = if field.value == "Empty" {
        placeholder
    } else {
        secondary
    };
    if field.chrome == FieldChrome::Toggle {
        return div()
            .flex()
            .items_center()
            .gap(px(12.0))
            .child(label_line(&field.label, field.required, secondary, error))
            .child(div().child(field.value.clone()));
    }
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(label_line(&field.label, field.required, secondary, error))
        .child(
            div()
                .w_full()
                .p(px(8.0))
                .bg(input)
                .border_1()
                .border_color(border)
                .rounded(px(4.0))
                .text_color(value_color)
                .child(field.value.clone()),
        )
        .when(field.toolbar, |col| {
            // S7: insert buttons only. The preview eye is not drawn.
            col.child(
                div()
                    .flex()
                    .gap(px(4.0))
                    .child("H2")
                    .child("B")
                    .child("I")
                    .child("Link")
                    .child("List"),
            )
        })
        .when(field.meta.is_some(), |col| {
            col.child(
                div()
                    .text_color(secondary)
                    .child(field.meta.clone().unwrap_or_default()),
            )
        })
        .when(field.error.is_some(), |col| {
            col.child(
                div()
                    .text_color(error)
                    .child(field.error.clone().unwrap_or_default()),
            )
        })
}

fn label_line(label: &str, required: bool, secondary: gpui::Hsla, error: gpui::Hsla) -> gpui::Div {
    div()
        .flex()
        .child(div().text_color(secondary).child(label.to_string()))
        .when(required, |row| {
            row.child(div().text_color(error).child(" *"))
        })
}

fn status_bar(
    screen: Screen,
    save: &str,
    meta: &str,
    issues: usize,
    secondary: gpui::Hsla,
    placeholder: gpui::Hsla,
    error: gpui::Hsla,
    border: gpui::Hsla,
    height: f32,
    cx: &mut Context<Shell>,
) -> gpui::Div {
    let preview = match screen {
        Screen::Error => "Couldn't load site.json",
        Screen::Loading => "Loading…",
        _ => "Preview closed · Ctrl+P",
    };
    let preview_color = if screen == Screen::Error {
        error
    } else {
        placeholder
    };
    div()
        .h(px(height))
        .px(px(12.0))
        .flex()
        .items_center()
        .gap(px(12.0))
        .border_t_1()
        .border_color(border)
        .child(
            div()
                .id("preview-status")
                .text_color(preview_color)
                .child(preview)
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    shell.open_preview();
                    cx.notify();
                })),
        )
        .child(div().flex_1())
        .when(screen == Screen::Editor, |row| {
            row.child(
                div()
                    .text_color(secondary)
                    .child(format!("Issues {issues}")),
            )
            .child(div().text_color(secondary).child(meta.to_string()))
        })
        .when(screen == Screen::Error, |row| {
            row.child(div().text_color(secondary).child("Issues 1"))
        })
        .when(!save.is_empty(), |row| {
            row.child(
                div()
                    .id("save-indicator")
                    .text_color(secondary)
                    .child(save.to_string()),
            )
        })
}

fn new_site_sheet(
    cx: &mut Context<Shell>,
    folder: &str,
    slug: &str,
    error: Option<&str>,
    create_enabled: bool,
    panel: gpui::Hsla,
    border: gpui::Hsla,
    secondary: gpui::Hsla,
    _text: gpui::Hsla,
) -> gpui::Div {
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("new-site")
                .w(px(520.0))
                .p(px(16.0))
                .bg(panel)
                .border_1()
                .border_color(border)
                .rounded(px(6.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(div().child("New site"))
                .child(div().text_color(secondary).child("Ctrl Shift N"))
                .child(div().child(format!("Folder {folder}")))
                .child(
                    Button::new("browse-folder")
                        .label("Browse…")
                        .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                            shell.portal_folder(cx);
                            cx.notify();
                        })),
                )
                .child(div().child("Project name"))
                .child(div().text_color(secondary).child(format!("Slug: {slug}")))
                .child(div().text_color(secondary).child(
                    "Creates site.json and the starter kit (Gruntfile.js, package.json, .lando.yml, .ddev/config.yaml, source/, templates).",
                ))
                .child(div().text_color(secondary).child(
                    "The site name starts as “My Site”. Change it in Site settings › Name.",
                ))
                .when(error.is_some(), |col| col.child(div().child(error.unwrap_or("").to_string())))
                .when(error.is_some_and(|err| err.contains("already exists")), |col| {
                    col.child(
                        Button::new("open-existing")
                            .label("Open it")
                            .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                                shell.open_existing_from_sheet();
                                cx.notify();
                            })),
                    )
                })
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(
                            Button::new("cancel-new")
                                .label("Cancel")
                                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                                    shell.new_site.open = false;
                                    cx.notify();
                                })),
                        )
                        .when(create_enabled, |row| {
                            row.child(
                                Button::new("create-site")
                                    .label("Create")
                                    .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                                        let _ = shell.submit_new_site();
                                        cx.notify();
                                    })),
                            )
                        }),
                ),
        )
}

fn template_sheet(
    cx: &mut Context<Shell>,
    selected: usize,
    panel: gpui::Hsla,
    border: gpui::Hsla,
    selected_bg: gpui::Hsla,
    _secondary: gpui::Hsla,
) -> gpui::Div {
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(420.0))
                .p(px(12.0))
                .bg(panel)
                .border_1()
                .border_color(border)
                .child("New page — choose template")
                .children(TEMPLATES.iter().enumerate().map(|(index, name)| {
                    div()
                        .id(("template", index))
                        .px(px(8.0))
                        .py(px(4.0))
                        .when(index == selected, |row| row.bg(selected_bg))
                        .child(*name)
                        .on_click(cx.listener(move |shell, _: &ClickEvent, _, cx| {
                            shell.template_index = index;
                            shell.commit_template();
                            cx.notify();
                        }))
                })),
        )
}

fn switcher_menu(
    cx: &mut Context<Shell>,
    recents: &[PathBuf],
    panel: gpui::Hsla,
    border: gpui::Hsla,
    secondary: gpui::Hsla,
) -> gpui::Div {
    div()
        .absolute()
        .top(px(44.0))
        .left(px(48.0))
        .w(px(320.0))
        .p(px(4.0))
        .bg(panel)
        .border_1()
        .border_color(border)
        .rounded(px(6.0))
        .child(div().text_color(secondary).child("Recent sites"))
        .children(recents.iter().enumerate().map(|(index, path)| {
            let label = path.display().to_string();
            div()
                .id(("recent", index))
                .px(px(8.0))
                .py(px(4.0))
                .child(label)
                .on_click(cx.listener(move |shell, _: &ClickEvent, _, cx| {
                    shell.switcher = false;
                    let path = shell.recents().get(index).cloned();
                    if let Some(path) = path {
                        shell.open_site(&path);
                    }
                    cx.notify();
                }))
        }))
        .child(
            Button::new("switcher-open")
                .label("Open…")
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    shell.switcher = false;
                    shell.request_choose_file();
                    cx.notify();
                })),
        )
        .child(
            Button::new("switcher-new")
                .label("New site…")
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    shell.switcher = false;
                    shell.open_new_site(None);
                    cx.notify();
                })),
        )
}

fn confirm_dialog(
    cx: &mut Context<Shell>,
    title: &str,
    body: &str,
    confirm: &str,
    danger: bool,
    panel: gpui::Hsla,
    border: gpui::Hsla,
    secondary: gpui::Hsla,
) -> gpui::Div {
    let confirm_label = confirm.to_string();
    let is_quit = danger && confirm == "Quit anyway";
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(480.0))
                .p(px(16.0))
                .bg(panel)
                .border_1()
                .border_color(border)
                .rounded(px(8.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(title.to_string())
                .child(div().text_color(secondary).child(body.to_string()))
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(
                            Button::new("dialog-cancel")
                                .label("Cancel")
                                .on_click(cx.listener(move |shell, _: &ClickEvent, _, cx| {
                                    if is_quit {
                                        shell.cancel_quit();
                                    } else {
                                        shell.restore_open = false;
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(Button::new("dialog-confirm").label(confirm_label).on_click(
                            cx.listener(move |shell, _: &ClickEvent, _, cx| {
                                if is_quit {
                                    shell.confirm_quit();
                                } else {
                                    shell.confirm_restore();
                                }
                                cx.notify();
                            }),
                        )),
                ),
        )
}

fn centered(lines: Vec<String>, secondary: gpui::Hsla) -> gpui::Div {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .flex_col()
        .gap(px(8.0))
        .children(lines.into_iter().enumerate().map(|(index, line)| {
            div()
                .when(index > 0, |row| row.text_color(secondary))
                .child(line)
        }))
}

fn chord(event: &KeyDownEvent) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mods = event.keystroke.modifiers;
    if mods.control || mods.platform {
        parts.push("ctrl".to_string());
    }
    if mods.shift {
        parts.push("shift".to_string());
    }
    if mods.alt {
        parts.push("alt".to_string());
    }
    parts.push(event.keystroke.key.to_ascii_lowercase());
    parts.join("-")
}

fn is_text(key: &str) -> bool {
    key.chars().count() == 1 && !key.chars().any(|ch| ch.is_control())
}

fn region_name(region: &SideTarget) -> &'static str {
    match region {
        SideTarget::Header => "header",
        SideTarget::Footer => "footer",
        SideTarget::Site => "site",
        _ => "page",
    }
}

fn resolve_site_path(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join("site.json")
    } else {
        path.to_path_buf()
    }
}

fn folder_name(path: &Path) -> String {
    path.parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string())
}

fn backup_note(path: &Path) -> Option<String> {
    let backup = checkpoint_path(path);
    let current = std::fs::read(path).ok()?;
    let previous = std::fs::read(&backup).ok()?;
    if current == previous {
        return None;
    }
    let when = std::fs::metadata(&backup)
        .and_then(|meta| meta.modified())
        .map(crate::document::backup_stamp)
        .unwrap_or_else(|_| "an earlier checkpoint".to_string());
    Some(format!(
        "Loaded state differs from last manual save ({when})."
    ))
}

fn ensure_section_ids(site: &mut Site) {
    dd_siteforge_core::edits::ensure_section_ids(site);
}

fn current_cursor(region: &SideTarget, page: usize, target: &Target) -> Option<Cursor> {
    match target {
        Target::SiteSettings => Some(Cursor::Site),
        Target::PageHead => Some(Cursor::PageHead { page }),
        Target::Hero(node) => Some(Cursor::PageHero { page, node: *node }),
        Target::Section(node) | Target::Column { node, .. } => {
            Some(Cursor::PageSection { page, node: *node })
        }
        Target::Component {
            node,
            column,
            component,
        } => Some(Cursor::PageComponent {
            page,
            node: *node,
            col: *column,
            comp: *component,
            items: Vec::new(),
        }),
        Target::RegionRoot => match region {
            SideTarget::Header => Some(Cursor::HeaderRoot),
            SideTarget::Footer => Some(Cursor::FooterRoot),
            _ => None,
        },
        Target::HeaderAlert => Some(Cursor::HeaderAlert),
        Target::RegionSection(index) => match region {
            SideTarget::Header => Some(Cursor::HeaderSection { sec: *index }),
            SideTarget::Footer => Some(Cursor::FooterSection { sec: *index }),
            _ => None,
        },
        Target::RegionColumn { section, .. } => match region {
            SideTarget::Header => Some(Cursor::HeaderSection { sec: *section }),
            SideTarget::Footer => Some(Cursor::FooterSection { sec: *section }),
            _ => None,
        },
        Target::RegionComponent {
            section,
            column,
            component,
        } => match region {
            SideTarget::Header => Some(Cursor::HeaderComponent {
                sec: *section,
                col: *column,
                comp: *component,
                items: Vec::new(),
            }),
            SideTarget::Footer => Some(Cursor::FooterComponent {
                sec: *section,
                col: *column,
                comp: *component,
                items: Vec::new(),
            }),
            _ => None,
        },
    }
}

fn dedup_page(pages: &[Page], page: &mut Page) {
    let taken = |id: &str| pages.iter().any(|existing| existing.id == id);
    if !taken(&page.id) {
        return;
    }
    let base_id = page.id.clone();
    let base_slug = page.slug.clone();
    for n in 2..10_000 {
        let candidate = format!("{base_id}-{n}");
        if !taken(&candidate) {
            page.id = candidate;
            page.slug = format!("{base_slug}-{n}");
            return;
        }
    }
}
