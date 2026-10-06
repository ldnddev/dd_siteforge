//! Recents / path picker shown when launched with no site.json.
use super::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
enum OpenSiteAction {
    Open(PathBuf),
    Create(PathBuf),
}

impl App {
    pub(in crate::tui) fn open_site_picker(&mut self) {
        self.modal = Some(Modal::OpenSite {
            query: String::new(),
            selected: 0,
        });
    }

    pub(in crate::tui) fn restore_selection(
        &mut self,
        page: usize,
        tree_row: usize,
        region: SelectedRegion,
    ) {
        if !self.site.pages.is_empty() {
            self.selected_page = page.min(self.site.pages.len() - 1);
            self.reveal_selected_page();
        }
        self.selected_region = region;
        self.selected_sidebar_section = match region {
            SelectedRegion::Page => SidebarSection::Layouts,
            SelectedRegion::Header | SelectedRegion::Footer | SelectedRegion::Site => {
                SidebarSection::Regions
            }
        };
        if region == SelectedRegion::Page && !self.site.pages.is_empty() {
            let n = self.site.pages[self.selected_page].nodes.len();
            for i in 0..n {
                if matches!(
                    self.site.pages[self.selected_page].nodes.get(i),
                    Some(PageNode::Section(_))
                ) {
                    self.set_section_expanded(i, true);
                }
            }
        }
        if region == SelectedRegion::Header {
            self.header_column_expanded = true;
        }
        let rows = self.build_tree_rows();
        if rows.is_empty() {
            self.selected_tree_row = 0;
            return;
        }
        self.selected_tree_row = tree_row.min(rows.len() - 1);
        self.apply_tree_row_selection(rows[self.selected_tree_row]);
    }

    pub(in crate::tui) fn persist_session(&self) {
        let Some(path) = &self.path else {
            return;
        };
        let mut session = crate::session::load();
        session.record(
            path,
            self.selected_page,
            self.selected_tree_row,
            region_name(self.selected_region),
        );
        let _ = crate::session::save(&session);
    }

    pub(in crate::tui) fn load_site_into_app(&mut self, path: PathBuf) -> Result<(), String> {
        let site = crate::storage::load_site(&path).map_err(|e| e.to_string())?;
        self.replace_site(path, site);
        Ok(())
    }

    fn replace_site(&mut self, path: PathBuf, mut site: crate::model::Site) {
        for page in &mut site.pages {
            ensure_page_section_ids(page);
        }
        self.site = site;
        self.path = Some(path.clone());
        self.last_saved_json = serde_json::to_string(&self.site).unwrap_or_default();
        self.dirty = false;
        self.dirty_since = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.clipboard = None;
        self.awaiting_site = false;
        self.modal = None;
        self.preview_server = None;
        self.deleted_pages.clear();
        let session = crate::session::load();
        let (page, tree_row, region) = session.selection_for(&path);
        self.restore_selection(page, tree_row, region_from_name(&region));
        let mut session = session;
        session.record(
            &path,
            self.selected_page,
            self.selected_tree_row,
            region_name(self.selected_region),
        );
        let _ = crate::session::save(&session);
        self.push_toast(ToastLevel::Success, format!("Opened {}", path.display()));
    }

    fn create_and_open_site(&mut self, path: PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
        let site = crate::model::Site::starter();
        crate::storage::save_site(&path, &site).map_err(|e| e.to_string())?;
        let root = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let slug = crate::scaffold::slugify_project_name(&crate::scaffold::dir_hint(&root));
        let overlay = crate::scaffold::overlay_if_present();
        let _ = crate::scaffold::seed_scaffold(
            &root,
            crate::scaffold::SeedOpts {
                force: false,
                project_name: Some(&slug),
                overlay: overlay.as_deref(),
            },
        );
        let _ = crate::templates::seed_templates(&root, false, None);
        self.replace_site(path, site);
        Ok(())
    }

    pub(in crate::tui) fn handle_open_site_event(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<ModalResult> {
        let (query, selected) = if let Some(Modal::OpenSite { query, selected }) = self.modal.take()
        {
            (query, selected)
        } else {
            return Some(ModalResult::CloseCancel);
        };
        let items = open_site_choices(&query);
        match key.code {
            KeyCode::Esc => {
                self.should_quit = true;
                Some(ModalResult::CloseCancel)
            }
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                Some(ModalResult::CloseCancel)
            }
            KeyCode::Up => {
                self.modal = Some(Modal::OpenSite {
                    query,
                    selected: selected.saturating_sub(1),
                });
                Some(ModalResult::Continue)
            }
            KeyCode::Down => {
                let max = items.len().saturating_sub(1);
                self.modal = Some(Modal::OpenSite {
                    query,
                    selected: if items.is_empty() {
                        0
                    } else {
                        (selected + 1).min(max)
                    },
                });
                Some(ModalResult::Continue)
            }
            KeyCode::Backspace => {
                let mut query = query;
                query.pop();
                self.modal = Some(Modal::OpenSite { query, selected: 0 });
                Some(ModalResult::Continue)
            }
            KeyCode::Enter => {
                let Some((_, action)) = items.get(selected) else {
                    self.push_toast(ToastLevel::Warning, "Type a path or pick a recent site.");
                    self.modal = Some(Modal::OpenSite { query, selected });
                    return Some(ModalResult::Continue);
                };
                let result = match action {
                    OpenSiteAction::Open(path) => self.open_existing_or_dir(path),
                    OpenSiteAction::Create(path) => self.create_and_open_site(path.clone()),
                };
                match result {
                    Ok(()) => Some(ModalResult::CloseSuccess),
                    Err(e) => {
                        self.push_toast(ToastLevel::Error, e);
                        self.modal = Some(Modal::OpenSite { query, selected });
                        Some(ModalResult::Continue)
                    }
                }
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let mut query = query;
                query.push(c);
                self.modal = Some(Modal::OpenSite { query, selected: 0 });
                Some(ModalResult::Continue)
            }
            _ => {
                self.modal = Some(Modal::OpenSite { query, selected });
                Some(ModalResult::Continue)
            }
        }
    }

    fn open_existing_or_dir(&mut self, path: &Path) -> Result<(), String> {
        let resolved = if path.is_dir() {
            path.join("site.json")
        } else {
            path.to_path_buf()
        };
        if !resolved.is_file() {
            return Err(format!("No site JSON at {}", resolved.display()));
        }
        self.load_site_into_app(resolved)
    }
}

pub(in crate::tui) fn region_name(region: SelectedRegion) -> &'static str {
    match region {
        SelectedRegion::Page => "page",
        SelectedRegion::Header => "header",
        SelectedRegion::Footer => "footer",
        SelectedRegion::Site => "site",
    }
}

pub(in crate::tui) fn region_from_name(name: &str) -> SelectedRegion {
    match name {
        "header" => SelectedRegion::Header,
        "footer" => SelectedRegion::Footer,
        "site" => SelectedRegion::Site,
        _ => SelectedRegion::Page,
    }
}

fn open_site_choices(query: &str) -> Vec<(String, OpenSiteAction)> {
    let mut items = Vec::new();
    let q = query.trim();
    if !q.is_empty() {
        let path = PathBuf::from(q);
        if path.is_file() || (path.is_dir() && path.join("site.json").is_file()) {
            items.push((format!("Open {}", q), OpenSiteAction::Open(path.clone())));
        } else {
            let create_path = if path.extension().and_then(|e| e.to_str()) == Some("json") {
                path.clone()
            } else {
                path.join("site.json")
            };
            items.push((
                format!("Create starter at {}", create_path.display()),
                OpenSiteAction::Create(create_path),
            ));
            items.push((format!("Open {}", q), OpenSiteAction::Open(path)));
        }
    }
    let recents = crate::session::load().existing_recents();
    for recent in recents {
        let display = recent.path.display().to_string();
        if q.is_empty() || fuzzy_score(q, &display).is_some() {
            items.push((display, OpenSiteAction::Open(recent.path)));
        }
    }
    if !Path::new("site.json").exists()
        && (q.is_empty()
            || fuzzy_score(q, "site.json").is_some()
            || fuzzy_score(q, "create").is_some())
    {
        items.push((
            "Create starter site.json here".into(),
            OpenSiteAction::Create(PathBuf::from("site.json")),
        ));
    }
    items
}

impl App {
    pub(in crate::tui) fn open_site_item_labels(&self, query: &str) -> Vec<String> {
        open_site_choices(query)
            .into_iter()
            .map(|(label, _)| label)
            .collect()
    }

    pub(in crate::tui) fn find_item_labels(&self, query: &str) -> Vec<String> {
        self.find_hits(query).into_iter().map(|h| h.label).collect()
    }

    pub(in crate::tui) fn palette_item_labels(&self, query: &str) -> Vec<String> {
        self.palette_items(query)
            .into_iter()
            .map(|c| c.title().to_string())
            .collect()
    }
}
