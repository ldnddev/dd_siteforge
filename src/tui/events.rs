//! Keyboard, mouse, and Pages-panel dispatch.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pane {
    Regions,
    Pages,
    Layout,
    Details,
}

impl App {
    /// First matching pane. Overlap (shouldn't happen) is Regions, Pages, Layout, Details.
    fn pane_at(&self, x: u16, y: u16) -> Option<Pane> {
        if contains(self.regions_area, x, y) {
            return Some(Pane::Regions);
        }
        if contains(self.pages_area, x, y) {
            return Some(Pane::Pages);
        }
        if contains(self.list_area, x, y) {
            return Some(Pane::Layout);
        }
        if contains(self.details_area, x, y) {
            return Some(Pane::Details);
        }
        None
    }

    /// Try to handle a key as a Pages-panel-scoped action.
    /// Returns `true` if the key was consumed — caller should short-circuit.
    pub(super) fn try_handle_pages_panel_key(&mut self, key: &event::KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};
        match key.code {
            KeyCode::Char('A') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.begin_add_page();
                true
            }
            KeyCode::Char('X') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                if self.site.pages.len() <= 1 {
                    self.push_toast(ToastLevel::Warning, "Cannot delete last page.");
                } else {
                    let title = self.site.pages[self.selected_page].head.title.clone();
                    self.modal = Some(Modal::ConfirmPrompt {
                        message: format!("Delete \"{}\"? y/n", title),
                        on_confirm: ConfirmKind::DeletePage,
                    });
                }
                true
            }
            KeyCode::Char('u') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(page) = self.deleted_pages.pop() {
                    let msg = format!("Restored page: {}", page.head.title);
                    self.push_toast(ToastLevel::Success, msg);
                    self.site.pages.push(page);
                    self.selected_page = self.site.pages.len() - 1;
                    self.selected_node = 0;
                    self.selected_column = 0;
                    self.selected_component = 0;
                    self.selected_nested_item = 0;
                    self.reveal_selected_page();
                } else {
                    self.push_toast(ToastLevel::Warning, "No deleted pages to restore.");
                }
                true
            }
            KeyCode::Char('J') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                if self.move_selected_page_among_siblings(true) {
                    self.push_toast(ToastLevel::Success, "Moved page down.");
                }
                true
            }
            KeyCode::Char('K') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                if self.move_selected_page_among_siblings(false) {
                    self.push_toast(ToastLevel::Success, "Moved page up.");
                }
                true
            }
            KeyCode::Char(' ') => {
                self.toggle_selected_page_expanded();
                true
            }
            KeyCode::Char('h') => {
                self.collapse_selected_page();
                true
            }
            KeyCode::Char('l') => {
                self.expand_selected_page();
                true
            }
            KeyCode::Char('r')
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                let idx = self.selected_page;
                let current_title = self.site.pages[idx].head.title.clone();
                self.modal = Some(Modal::RenamePagePrompt {
                    title: current_title,
                    page_idx: idx,
                });
                self.push_toast(ToastLevel::Info, "Rename page. Edit and press Enter.");
                true
            }
            _ => false,
        }
    }

    pub(in crate::tui) fn begin_add_page(&mut self) {
        self.modal = Some(Modal::TemplatePicker { selected: 0 });
        self.push_toast(ToastLevel::Info, "New page: choose a template.");
    }

    /// Core chord table. `Some` means the key was a command or a Vim letter.
    fn dispatch_listed_key(&mut self, key: &event::KeyEvent) -> Option<bool> {
        use dd_siteforge_core::commands::{Hit, Query, Surface, lookup};
        let owned = super::keys::event_chord(key)?;
        let hit = lookup(Query {
            surface: Surface::Tui,
            mode: self.input_mode,
            vim: self.vim_keys,
            in_text: false,
            chord: owned.as_chord(),
        })?;
        match hit {
            Hit::Command { command, hint, .. } => {
                if let Some(hint) = hint {
                    self.note_alias(hint);
                }
                Some(self.run_command_key(command))
            }
            Hit::Vim(action) => Some(self.run_vim(action)),
        }
    }

    fn note_alias(&mut self, hint: &'static str) {
        if self.seen_alias_hints.insert(hint) {
            self.alias_hint = Some(dd_siteforge_core::commands::alias_hint_text(
                hint,
                self.in_tmux,
            ));
        }
    }

    pub(in crate::tui) fn run_command_key(
        &mut self,
        command: dd_siteforge_core::commands::Command,
    ) -> bool {
        use dd_siteforge_core::commands::Command;
        match command {
            Command::Export => {
                self.begin_export_flow();
                true
            }
            Command::Preview => {
                self.begin_preview_flow();
                true
            }
            Command::RefreshPreview => {
                self.refresh_preview_if_running();
                if self.preview_server.is_none() {
                    self.push_toast(ToastLevel::Info, "Preview is not running.");
                }
                false
            }
            Command::Validate | Command::ToggleIssues => {
                self.open_validation_modal();
                false
            }
            Command::Save => {
                self.begin_save_prompt();
                false
            }
            Command::Help | Command::Theme => {
                self.open_theme_ui();
                false
            }
            Command::Find => {
                self.open_find();
                false
            }
            Command::Insert => {
                self.open_component_picker();
                false
            }
            Command::AddPage => {
                self.begin_add_page();
                false
            }
            Command::BuildAssets => {
                self.start_asset_build();
                false
            }
            Command::PageHealth => {
                self.open_page_health();
                false
            }
            Command::Redo => {
                self.redo_last();
                true
            }
            Command::Quit => {
                self.request_quit();
                false
            }
            Command::NewSite | Command::Open => {
                self.open_site_picker();
                false
            }
            Command::ToggleRegions => {
                self.toggle_regions_pane();
                false
            }
            Command::ToggleDetails => {
                self.toggle_details_pane();
                false
            }
            Command::Palette => {
                self.open_palette();
                false
            }
            Command::FocusSidebar => {
                self.focus_pane(SidebarSection::Regions);
                false
            }
            Command::FocusPages => {
                self.focus_pane(SidebarSection::Pages);
                false
            }
            Command::FocusLayouts | Command::FocusOutline => {
                self.focus_pane(SidebarSection::Layouts);
                false
            }
            Command::FocusDetails => {
                self.focus_pane(SidebarSection::Details);
                false
            }
            Command::Undo => {
                self.undo_last();
                true
            }
            Command::DeleteRow => {
                self.delete_selected_row();
                true
            }
            Command::ToggleSidebar
            | Command::ToggleInspector
            | Command::InitKit
            | Command::InitTemplates => false,
        }
    }

    fn run_vim(&mut self, action: dd_siteforge_core::commands::VimAction) -> bool {
        use dd_siteforge_core::commands::VimAction;
        match action {
            VimAction::Down => {
                if self.selected_sidebar_section == SidebarSection::Details {
                    self.scroll_details_by(1);
                } else {
                    self.handle_down();
                }
                false
            }
            VimAction::Up => {
                if self.selected_sidebar_section == SidebarSection::Details {
                    self.scroll_details_by(-1);
                } else {
                    self.handle_up();
                }
                false
            }
            VimAction::First => {
                match self.selected_sidebar_section {
                    SidebarSection::Details => self.details_scroll_row = 0,
                    SidebarSection::Pages => self.jump_visible_page_to_end(false),
                    _ => self.vim_jump_to_first_row(),
                }
                false
            }
            VimAction::Last => {
                match self.selected_sidebar_section {
                    SidebarSection::Details => {
                        self.details_scroll_row = self.details_max_scroll();
                    }
                    SidebarSection::Pages => self.jump_visible_page_to_end(true),
                    _ => self.vim_jump_to_last_row(),
                }
                false
            }
            VimAction::Collapse => {
                self.vim_collapse_selected_row();
                false
            }
            VimAction::Expand => {
                self.vim_expand_selected_row();
                false
            }
            VimAction::ToggleExpand => {
                self.toggle_selected_tree_expanded();
                false
            }
            VimAction::Repeat => {
                self.repeat_last_insert();
                true
            }
            VimAction::Copy => {
                self.copy_selected_row();
                false
            }
            VimAction::Paste => {
                self.paste_clipboard();
                true
            }
            VimAction::Delete => {
                self.delete_selected_row();
                true
            }
            VimAction::Undo => {
                self.undo_last();
                true
            }
            VimAction::Rename => {
                self.begin_edit_selected_column_id();
                false
            }
            VimAction::AddItem => {
                self.add_selected_collection_item();
                true
            }
            VimAction::RemoveItem => {
                self.remove_selected_collection_item();
                true
            }
            VimAction::MoveDown => {
                self.move_selected_row(1);
                true
            }
            VimAction::MoveUp => {
                self.move_selected_row(-1);
                true
            }
            VimAction::PrevColumn => {
                self.select_prev_column();
                false
            }
            VimAction::NextColumn => {
                self.select_next_column();
                false
            }
            VimAction::AddColumn => {
                self.add_column();
                true
            }
            VimAction::RemoveColumn => {
                self.remove_selected_column();
                true
            }
        }
    }

    fn focus_pane(&mut self, pane: SidebarSection) {
        match pane {
            SidebarSection::Regions => self.regions_open = true,
            SidebarSection::Details => self.details_open = true,
            SidebarSection::Pages | SidebarSection::Layouts => {}
        }
        self.selected_sidebar_section = pane;
        if pane == SidebarSection::Pages {
            self.selected_region = SelectedRegion::Page;
            self.selected_tree_row = 0;
            self.sync_tree_row_with_selection();
        }
    }

    fn toggle_regions_pane(&mut self) {
        self.regions_open = !self.regions_open;
        if !self.regions_open && self.selected_sidebar_section == SidebarSection::Regions {
            self.selected_sidebar_section = SidebarSection::Pages;
        }
    }

    fn toggle_details_pane(&mut self) {
        self.details_open = !self.details_open;
        if !self.details_open && self.selected_sidebar_section == SidebarSection::Details {
            self.selected_sidebar_section = SidebarSection::Layouts;
        }
    }

    /// Insert a bracketed paste or a Ctrl+V clipboard block at the focused field.
    /// One call. The string is sanitized before it lands.
    fn paste_text(&mut self, raw: &str) {
        if self.overlay.is_some() {
            if editing_theme_hex(self) {
                if let Some(editor) = self.theme_editor.as_mut() {
                    editor.paste_hex(raw);
                }
            }
            return;
        }
        if self.modal.is_some() {
            let _ = self.handle_modal_paste(raw);
        }
    }

    pub(super) fn handle_event(&mut self, evt: Event) -> anyhow::Result<bool> {
        if matches!(evt, Event::FocusGained) {
            return Ok(self.refresh_followed_theme());
        }
        // Bracketed paste and Ctrl+V share one insert. Return before key
        // dispatch so the block is not also typed one character at a time.
        if let Event::Paste(text) = &evt {
            self.paste_text(text);
            return Ok(false);
        }
        if let Event::Key(k) = &evt {
            if is_clipboard_paste(k) {
                if let Some(text) = clipboard_text() {
                    self.paste_text(&text);
                }
                return Ok(false);
            }
        }
        if App::is_idle_event(&evt) {
            if let Event::Mouse(m) = &evt {
                if matches!(m.kind, MouseEventKind::Up(_)) {
                    self.scrollbar_drag = None;
                    self.form_text_drag = false;
                }
            }
            return Ok(false);
        }

        // A confirm stays above the sheet, so Esc answers the prompt.
        if matches!(self.modal, Some(Modal::ConfirmPrompt { .. })) {
            if let Some(modal_result) = self.handle_modal_event(evt.clone()) {
                let mutated = matches!(modal_result, ModalResult::CloseSuccess);
                return Ok(mutated);
            }
        }

        // Overlay sits above the other modals. F1 and F2 close it.
        // Tab and brackets change the page. Esc on Themes stays in the editor.
        if self.overlay.is_some() {
            match evt {
                Event::Key(k) => {
                    if matches!(k.code, KeyCode::F(1) | KeyCode::F(2)) {
                        self.close_theme(true);
                        return Ok(false);
                    }
                    if k.code == KeyCode::PageDown {
                        self.scroll_overlay(10);
                        self.settings_follow = false;
                    } else if k.code == KeyCode::PageUp {
                        self.scroll_overlay(-10);
                        self.settings_follow = false;
                    } else if !settings_section_key(self, &k) {
                        if self.settings_nav == super::settings::HELP {
                            if k.code == KeyCode::Esc {
                                self.close_theme(true);
                            } else {
                                let max = self.overlay_scroll_max;
                                if let Some(Overlay::Theme { scroll }) = &mut self.overlay {
                                    match k.code {
                                        KeyCode::Down | KeyCode::Char('j') => {
                                            *scroll = scroll.saturating_add(1).min(max);
                                        }
                                        KeyCode::Up | KeyCode::Char('k') => {
                                            *scroll = scroll.saturating_sub(1);
                                        }
                                        KeyCode::Home | KeyCode::Char('g') => *scroll = 0,
                                        KeyCode::End | KeyCode::Char('G') => *scroll = max,
                                        _ => {}
                                    }
                                }
                            }
                        } else if self.settings_nav == 0 {
                            if save_target_key(self, &k) {
                                toggle_save_target(self);
                            } else if let Some(ek) = map_siteforge_editor_key(&k) {
                                let shift = k.modifiers.contains(KeyModifiers::SHIFT);
                                if self.theme_editor.is_some() {
                                    self.dispatch_theme_editor(ek, shift);
                                    self.settings_follow = true;
                                } else {
                                    self.dispatch_theme_picker(ek);
                                }
                            }
                        } else {
                            match k.code {
                                KeyCode::Esc => self.close_theme(true),
                                KeyCode::Down | KeyCode::Char('j') => {
                                    super::settings::move_row(self, 1);
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    super::settings::move_row(self, -1);
                                }
                                KeyCode::Home | KeyCode::Char('g') => {
                                    super::settings::move_row(self, isize::MIN);
                                }
                                KeyCode::End | KeyCode::Char('G') => {
                                    super::settings::move_row(self, isize::MAX);
                                }
                                KeyCode::Enter => super::settings::activate(self),
                                _ => {}
                            }
                        }
                    }
                }
                Event::Mouse(m) => match m.kind {
                    MouseEventKind::ScrollUp => {
                        self.scroll_overlay(-3);
                        self.settings_follow = false;
                    }
                    MouseEventKind::ScrollDown => {
                        self.scroll_overlay(3);
                        self.settings_follow = false;
                    }
                    MouseEventKind::Down(MouseButton::Left)
                        if !settings_nav_click(self, m.column, m.row) =>
                    {
                        if place_hex_click(self, m.column, m.row) {
                            self.settings_follow = false;
                            return Ok(false);
                        }
                        let track = if self.settings_nav == super::settings::HELP {
                            self.help_scrollbar_track
                        } else {
                            self.theme_scrollbar_track
                        };
                        if track.rect.height > 0 && contains(track.rect, m.column, m.row) {
                            let max = self.overlay_scroll_max;
                            self.set_overlay_scroll((track.offset_at(m.row) as u16).min(max));
                            self.scrollbar_drag =
                                Some(if self.settings_nav == super::settings::HELP {
                                    ScrollbarDrag::Help
                                } else {
                                    ScrollbarDrag::Theme
                                });
                            self.settings_follow = false;
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        let track = match self.scrollbar_drag {
                            Some(ScrollbarDrag::Help) => self.help_scrollbar_track,
                            Some(ScrollbarDrag::Theme) => self.theme_scrollbar_track,
                            _ => ScrollbarTrack::default(),
                        };
                        if track.rect.height > 0 {
                            let max = self.overlay_scroll_max;
                            self.set_overlay_scroll((track.offset_at(m.row) as u16).min(max));
                        }
                    }
                    MouseEventKind::Up(_) => self.scrollbar_drag = None,
                    _ => {}
                },
                _ => {}
            }
            return Ok(false);
        }

        if let Some(modal_result) = self.handle_modal_event(evt.clone()) {
            let mutated = matches!(modal_result, ModalResult::CloseSuccess);
            return Ok(mutated);
        }

        if self.awaiting_site {
            if let Event::Key(k) = &evt {
                if k.code == KeyCode::Char('q') && k.modifiers.contains(KeyModifiers::CONTROL) {
                    self.should_quit = true;
                }
            }
            return Ok(false);
        }

        let mutated = match evt {
            Event::Key(k) => {
                self.alias_hint = None;
                if self.selected_sidebar_section == SidebarSection::Pages
                    && self.vim_keys
                    && self.try_handle_pages_panel_key(&k)
                {
                    self.sync_tree_row_with_selection();
                    matches!(
                        k.code,
                        KeyCode::Char('u') | KeyCode::Char('J') | KeyCode::Char('K')
                    )
                } else if let Some(changed) = self.dispatch_listed_key(&k) {
                    changed
                } else {
                    match k.code {
                        KeyCode::Up => {
                            self.handle_up();
                            false
                        }
                        KeyCode::Down => {
                            self.handle_down();
                            false
                        }
                        KeyCode::PageUp => {
                            self.page_focused_pane(-5);
                            false
                        }
                        KeyCode::PageDown => {
                            self.page_focused_pane(5);
                            false
                        }
                        KeyCode::Enter => {
                            self.handle_enter_on_selected_row();
                            false
                        }
                        KeyCode::Tab => {
                            self.select_next_page();
                            false
                        }
                        KeyCode::BackTab => {
                            self.select_prev_page();
                            false
                        }
                        KeyCode::Char('f')
                            if !k.modifiers.contains(KeyModifiers::CONTROL)
                                && !k.modifiers.contains(KeyModifiers::ALT) =>
                        {
                            self.begin_edit_selected_column_width_class();
                            false
                        }
                        _ => false,
                    }
                }
            }
            Event::Mouse(m) => {
                match m.kind {
                    MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                        let up = matches!(m.kind, MouseEventKind::ScrollUp);
                        match self.pane_at(m.column, m.row) {
                            Some(Pane::Regions) => {
                                self.cycle_selected_region(if up { -1 } else { 1 });
                            }
                            Some(Pane::Pages) => {
                                self.select_visible_page_by(if up { -1 } else { 1 }, true);
                            }
                            Some(Pane::Layout) => {
                                if up {
                                    self.select_prev();
                                } else {
                                    self.select_next();
                                }
                            }
                            Some(Pane::Details) => {
                                self.scroll_details_by(if up { -3 } else { 3 });
                            }
                            None => {}
                        }
                    }
                    MouseEventKind::Down(MouseButton::Left) => {
                        let col = m.column;
                        let row = m.row;
                        // Hit-test scrollbars before pane click-to-select so a click on
                        // the bar jumps scroll instead of selecting a grain / tree row.
                        if contains(self.details_scrollbar_track.rect, col, row) {
                            self.details_scroll_row = self.details_scrollbar_track.offset_at(row);
                            self.scrollbar_drag = Some(ScrollbarDrag::Details);
                        } else if contains(self.layout_scrollbar_track.rect, col, row) {
                            self.jump_layout_to_scrollbar_y(row);
                            self.scrollbar_drag = Some(ScrollbarDrag::Layout);
                        } else {
                            self.scrollbar_drag = None;
                            let now = std::time::Instant::now();
                            let is_double = if let Some((last_col, last_row, last_time)) =
                                self.last_mouse_click
                            {
                                last_col == col
                                    && last_row == row
                                    && now.duration_since(last_time).as_millis()
                                        < DOUBLE_CLICK_THRESHOLD_MS
                            } else {
                                false
                            };
                            self.last_mouse_click = Some((col, row, now));
                            if is_double {
                                self.handle_double_click(col, row);
                            } else {
                                self.handle_click(col, row);
                            }
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        if self.scrollbar_drag == Some(ScrollbarDrag::Details) {
                            self.details_scroll_row = self.details_scrollbar_track.offset_at(m.row);
                        } else if self.scrollbar_drag == Some(ScrollbarDrag::Layout) {
                            self.jump_layout_to_scrollbar_y(m.row);
                        }
                    }
                    MouseEventKind::Up(_) => {
                        self.scrollbar_drag = None;
                    }
                    _ => {}
                }
                false
            }
            _ => false,
        };
        self.sync_tree_row_with_selection();
        Ok(mutated)
    }

    pub(super) fn handle_click(&mut self, x: u16, y: u16) {
        // Layout tree (list_area)
        if contains(self.list_area, x, y) {
            let tree_rows = self.build_tree_rows();
            if tree_rows.is_empty() {
                return;
            }
            let body_top = self.list_area.y.saturating_add(1);
            let body_bottom = self
                .list_area
                .y
                .saturating_add(self.list_area.height.saturating_sub(1));
            if y < body_top || y >= body_bottom {
                return;
            }
            if contains(self.layout_scrollbar_track.rect, x, y) {
                return;
            }
            let idx = (y - body_top) as usize + self.layout_list_state.offset();
            if idx < tree_rows.len() {
                self.selected_tree_row = idx;
                self.apply_tree_row_selection(tree_rows[idx]);
                self.selected_sidebar_section = SidebarSection::Layouts;
            }
            return;
        }

        // Pages list
        if contains(self.pages_area, x, y) {
            let body_top = self.pages_area.y.saturating_add(1);
            if y >= body_top {
                let rel = (y - body_top) as usize + self.pages_list_state.offset();
                let rows = self.build_pages_panel_rows();
                if let Some(row) = rows.get(rel) {
                    self.selected_region = SelectedRegion::Page;
                    self.selected_sidebar_section = SidebarSection::Pages;
                    self.focus_page(row.page_idx);
                }
            }
            return;
        }

        // Regions list
        if contains(self.regions_area, x, y) {
            let body_top = self.regions_area.y.saturating_add(1);
            if y >= body_top {
                let rel = (y - body_top) as usize;
                match rel {
                    0 => {
                        self.selected_region = SelectedRegion::Site;
                        self.selected_tree_row = 0;
                        self.selected_sidebar_section = SidebarSection::Regions;
                        self.sync_tree_row_with_selection();
                    }
                    1 => {
                        self.selected_region = SelectedRegion::Header;
                        self.selected_header_section = 0;
                        self.selected_header_column = 0;
                        self.selected_header_component = 0;
                        self.selected_sidebar_section = SidebarSection::Regions;
                        self.sync_tree_row_with_selection();
                    }
                    2 => {
                        self.selected_region = SelectedRegion::Footer;
                        self.selected_sidebar_section = SidebarSection::Regions;
                        self.sync_tree_row_with_selection();
                    }
                    _ => {}
                }
            }
            return;
        }

        // Details panel — click focuses (except the scrollbar) and still hit-tests.
        if contains(self.details_area, x, y) {
            if contains(self.details_scrollbar_track.rect, x, y) {
                return;
            }
            self.selected_sidebar_section = SidebarSection::Details;
            let area = self.details_area;
            let content_top = area.y.saturating_add(1);
            let content_left = area.x.saturating_add(1);
            let scrollbar_x = area.x.saturating_add(area.width.saturating_sub(2));
            let content_bottom = area.y.saturating_add(area.height.saturating_sub(1));
            if y < content_top || y >= content_bottom || x < content_left || x >= scrollbar_x {
                return;
            }
            let rel = (y - content_top) as usize;
            let text_line = rel + self.details_scroll_row;
            let char_x = (x as usize).saturating_sub(content_left as usize);
            self.select_item_from_details_click(text_line, char_x);
            return;
        }
    }

    pub(super) fn handle_double_click(&mut self, x: u16, y: u16) {
        self.handle_click(x, y);
        // Special case for double-click on an item (not the title bar) in the Pages list:
        // Switch to that page (already done in handle_click), then pin the Layout tree selection
        // to its [HEAD] row (by setting the flag + resync), and activate the Layouts sidebar section.
        // The handle_enter below will then see a PageHead row and open the unified page-head editor.
        let pages_body_top = self.pages_area.y.saturating_add(1);
        if contains(self.pages_area, x, y) && y >= pages_body_top {
            self.selected_sidebar_section = SidebarSection::Layouts;
            self.page_head_selected = true;
            self.sync_tree_row_with_selection();
        }
        self.handle_enter_on_selected_row();
    }

    /// PageUp/PageDown follow the focused pane. Layouts jump the tree
    /// selection; Pages jump visible page-tree rows (clamped, no wrap);
    /// Details and Regions scroll the blueprint.
    fn page_focused_pane(&mut self, delta: isize) {
        let steps = delta.unsigned_abs();
        match self.selected_sidebar_section {
            SidebarSection::Layouts => {
                for _ in 0..steps {
                    if delta > 0 {
                        self.select_next();
                    } else {
                        self.select_prev();
                    }
                }
            }
            SidebarSection::Pages => self.select_visible_page_by(delta, false),
            SidebarSection::Regions | SidebarSection::Details => self.scroll_details_by(delta),
        }
    }

    /// Jump `selected_tree_row` to the first row of the window under `y` on
    /// the Layout scrollbar (proportional, same mapping as Details).
    fn jump_layout_to_scrollbar_y(&mut self, y: u16) {
        let offset = self.layout_scrollbar_track.offset_at(y);
        let rows = self.build_tree_rows();
        if rows.is_empty() {
            return;
        }
        let idx = offset.min(rows.len() - 1);
        self.selected_tree_row = idx;
        self.apply_tree_row_selection(rows[idx]);
        *self.layout_list_state.offset_mut() = offset.min(rows.len().saturating_sub(1));
        self.layout_list_state.select(Some(idx));
        self.selected_sidebar_section = SidebarSection::Layouts;
    }

    /// Re-read `omarchy: follow` when the desktop theme file changes.
    /// Leaves the startup tagline alone.
    pub(in crate::tui) fn refresh_followed_theme(&mut self) -> bool {
        if matches!(self.overlay, Some(Overlay::Theme { .. })) {
            return false;
        }
        if !matches!(
            self.theme.omarchy,
            Some(ldnddev_theme::OmarchyChoice::Follow)
        ) {
            return false;
        }
        let stamp = ldnddev_theme::follow_stamp(&self.omarchy_paths);
        if stamp == self.follow_stamp {
            return false;
        }
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let (theme, source, warning) = super::theme::AppTheme::load_resolved(
            &cwd,
            ldnddev_theme::default_config_home().as_deref(),
            &self.omarchy_paths,
        );
        let header_quotes = self.theme.header_quotes.clone();
        self.follow_stamp = stamp;
        self.theme = theme;
        self.theme.header_quotes = header_quotes;
        self.theme_source = source;
        if let Some(msg) = warning {
            self.theme_status = Some(msg.clone());
            self.push_toast(ToastLevel::Warning, msg);
        }
        true
    }

    pub(in crate::tui) fn save_gui_window(&self) {
        if cfg!(test) {
            return;
        }
        let path = dd_siteforge_core::scaffold::config_scaffold_dir().join("gui.json");
        let mut value = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        if let Some(object) = value.as_object_mut() {
            object.insert(
                "density".to_string(),
                serde_json::json!(if self.density_compact {
                    "compact"
                } else {
                    "comfortable"
                }),
            );
            object.insert("vim_keys".to_string(), serde_json::json!(self.vim_keys));
            object
                .entry("reduced_motion".to_string())
                .or_insert(serde_json::json!(false));
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(&value) {
            let _ = std::fs::write(path, text);
        }
    }

    pub(in crate::tui) fn open_theme_ui(&mut self) {
        if matches!(self.overlay, Some(Overlay::Theme { .. })) {
            self.close_theme(true);
            return;
        }
        self.settings_nav = 0;
        self.settings_row = 0;
        self.settings_follow = true;
        let palette = super::theme::palette_from_theme(&self.theme);
        let picker = ldnddev_theme::ThemePicker::new(
            palette.clone(),
            super::theme::extra_theme_fields(),
            self.omarchy_paths.clone(),
        );
        if picker.lists_omarchy() {
            self.theme_picker = Some(picker);
            self.theme_editor = None;
        } else {
            self.theme_picker = None;
            self.theme_editor = Some(ldnddev_theme::ThemeEditor::new(
                palette,
                super::theme::extra_theme_fields(),
            ));
        }
        self.overlay = Some(Overlay::Theme { scroll: 0 });
    }

    fn close_theme(&mut self, revert: bool) {
        if revert {
            if let Some(picker) = self.theme_picker.as_ref() {
                let palette = picker.revert_palette();
                super::theme::apply_palette(&mut self.theme, &palette);
            } else if let Some(mut editor) = self.theme_editor.take() {
                editor.revert();
                super::theme::apply_palette(&mut self.theme, &editor.palette);
            }
        }
        self.theme_picker = None;
        self.theme_editor = None;
        self.overlay = None;
    }

    fn scroll_overlay(&mut self, delta: i32) {
        let max = self.overlay_scroll_max;
        if let Some(Overlay::Theme { scroll }) = &mut self.overlay {
            let next = *scroll as i32 + delta;
            *scroll = next.clamp(0, i32::from(max)) as u16;
        }
    }

    fn set_overlay_scroll(&mut self, value: u16) {
        if let Some(Overlay::Theme { scroll }) = &mut self.overlay {
            *scroll = value;
        }
    }

    fn dispatch_theme_picker(&mut self, ek: ldnddev_theme::EditorKey) {
        let outcome = {
            let Some(picker) = self.theme_picker.as_mut() else {
                return;
            };
            picker.handle(ek)
        };
        match outcome {
            ldnddev_theme::PickerOutcome::Applied => {
                if let Some(picker) = &self.theme_picker {
                    super::theme::apply_palette(&mut self.theme, &picker.palette);
                }
            }
            ldnddev_theme::PickerOutcome::EditColors => {
                let Some(picker) = self.theme_picker.as_ref() else {
                    return;
                };
                let mut editor = ldnddev_theme::ThemeEditor::new(
                    picker.palette.clone(),
                    super::theme::extra_theme_fields(),
                );
                editor.save_target = picker.save_target;
                self.theme_editor = Some(editor);
            }
            ldnddev_theme::PickerOutcome::RequestSave => {
                let saved = self
                    .theme_picker
                    .as_ref()
                    .map(|picker| (picker.palette.clone(), picker.save_target));
                if let Some((palette, target)) = saved {
                    self.save_theme_palette(&palette, target);
                }
            }
            ldnddev_theme::PickerOutcome::Error(msg) => {
                self.push_toast(ToastLevel::Error, msg);
            }
            ldnddev_theme::PickerOutcome::Closed { .. } => self.close_theme(true),
            ldnddev_theme::PickerOutcome::None => {
                if theme_motion(ek) {
                    self.preview_theme_row();
                    self.settings_follow = true;
                }
            }
        }
    }

    fn preview_theme_row(&mut self) {
        let outcome = {
            let Some(picker) = self.theme_picker.as_mut() else {
                return;
            };
            picker.preview_selected()
        };
        if let ldnddev_theme::PickerOutcome::Applied = outcome {
            if let Some(picker) = &self.theme_picker {
                super::theme::apply_palette(&mut self.theme, &picker.palette);
            }
        } else if let ldnddev_theme::PickerOutcome::Error(msg) = outcome {
            self.push_toast(ToastLevel::Error, msg);
        }
    }

    fn save_theme_palette(
        &mut self,
        palette: &ldnddev_theme::Palette,
        target: ldnddev_theme::ThemeSaveTarget,
    ) {
        let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let fields = self
            .theme_editor
            .as_ref()
            .map(|editor| editor.fields.clone())
            .unwrap_or_else(|| {
                let mut fields = ldnddev_theme::COLOR_FIELDS.to_vec();
                for extra in super::theme::extra_theme_fields() {
                    if !fields.iter().any(|field| field.key == extra.key) {
                        fields.push(*extra);
                    }
                }
                fields
            });
        match ldnddev_theme::save_theme(
            palette,
            &root,
            "dd_siteforge_theme.yml",
            target,
            ldnddev_theme::default_config_home().as_deref(),
            &fields,
        ) {
            Ok(path) => {
                super::theme::apply_palette(&mut self.theme, palette);
                self.theme_source = target.label().to_string();
                self.follow_stamp = ldnddev_theme::follow_stamp(&self.omarchy_paths);
                self.theme_editor = None;
                self.theme_picker = None;
                self.overlay = None;
                self.push_toast(
                    ToastLevel::Success,
                    format!("Theme saved ({})", path.display()),
                );
            }
            Err(err) => {
                self.push_toast(ToastLevel::Error, format!("Theme save failed: {err}"));
            }
        }
    }

    fn dispatch_theme_editor(&mut self, ek: ldnddev_theme::EditorKey, shift: bool) {
        let outcome = {
            let Some(editor) = self.theme_editor.as_mut() else {
                return;
            };
            editor.handle(ek, shift)
        };
        match outcome {
            ldnddev_theme::EditorOutcome::PaletteChanged => {
                if let Some(editor) = &self.theme_editor {
                    if let Some(picker) = self.theme_picker.as_mut() {
                        picker.palette = editor.palette.clone();
                        picker.save_target = editor.save_target;
                    }
                    super::theme::apply_palette(&mut self.theme, &editor.palette);
                }
            }
            ldnddev_theme::EditorOutcome::RequestSave => {
                if let Some(editor) = self.theme_editor.as_ref() {
                    let palette = editor.palette.clone();
                    let target = editor.save_target;
                    self.save_theme_palette(&palette, target);
                }
            }
            ldnddev_theme::EditorOutcome::Closed { .. } => {
                if self.theme_picker.is_some() {
                    if let Some(editor) = self.theme_editor.take() {
                        if let Some(picker) = self.theme_picker.as_mut() {
                            picker.palette = editor.palette.clone();
                            picker.save_target = editor.save_target;
                        }
                        super::theme::apply_palette(&mut self.theme, &editor.palette);
                    }
                } else {
                    self.close_theme(true);
                }
            }
            ldnddev_theme::EditorOutcome::HexError(msg) => {
                self.push_toast(ToastLevel::Error, msg);
            }
            ldnddev_theme::EditorOutcome::None => {}
        }
    }
}

fn is_clipboard_paste(k: &event::KeyEvent) -> bool {
    k.kind == KeyEventKind::Press
        && matches!(k.code, KeyCode::Char('v') | KeyCode::Char('V'))
        && k.modifiers.contains(KeyModifiers::CONTROL)
        && !k.modifiers.contains(KeyModifiers::ALT)
        && !k.modifiers.contains(KeyModifiers::SHIFT)
}

fn clipboard_text() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    let text = clipboard.get_text().ok()?;
    if text.is_empty() { None } else { Some(text) }
}

fn theme_motion(key: ldnddev_theme::EditorKey) -> bool {
    matches!(
        key,
        ldnddev_theme::EditorKey::Down
            | ldnddev_theme::EditorKey::Up
            | ldnddev_theme::EditorKey::Char('j')
            | ldnddev_theme::EditorKey::Char('k')
            | ldnddev_theme::EditorKey::Char('g')
            | ldnddev_theme::EditorKey::Char('G')
    )
}

fn editing_theme_hex(app: &App) -> bool {
    app.theme_editor
        .as_ref()
        .is_some_and(|editor| editor.editing_hex)
}

/// Tab, Shift-Tab, `[`, and `]` change the settings page.
/// A hex field keeps the brackets. Tab still moves the section.
fn settings_section_key(app: &mut App, k: &crossterm::event::KeyEvent) -> bool {
    let Some(dir) = settings_section_dir(app, k) else {
        return false;
    };
    let next =
        (app.settings_nav as isize + dir).clamp(0, crate::tui::settings::LAST as isize) as u8;
    if next != app.settings_nav {
        app.settings_nav = next;
        app.settings_row = 0;
        app.settings_follow = true;
        if let Some(Overlay::Theme { scroll }) = &mut app.overlay {
            *scroll = 0;
        }
    }
    true
}

fn settings_section_dir(app: &App, k: &crossterm::event::KeyEvent) -> Option<isize> {
    if k.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(k.code, KeyCode::Tab | KeyCode::BackTab)
    {
        return None;
    }
    let tab = matches!(k.code, KeyCode::Tab | KeyCode::BackTab);
    if !tab && editing_theme_hex(app) {
        return None;
    }
    match k.code {
        KeyCode::BackTab => Some(-1),
        KeyCode::Tab => {
            if k.modifiers.contains(KeyModifiers::SHIFT) {
                Some(-1)
            } else {
                Some(1)
            }
        }
        KeyCode::Char(']') => Some(1),
        KeyCode::Char('[') => Some(-1),
        _ => None,
    }
}

fn save_target_key(app: &App, k: &crossterm::event::KeyEvent) -> bool {
    if editing_theme_hex(app) || k.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }
    matches!(k.code, KeyCode::Char('t') | KeyCode::Char('T'))
}

fn toggle_save_target(app: &mut App) {
    let from_editor = if let Some(editor) = app.theme_editor.as_mut() {
        editor.save_target = editor.save_target.toggled();
        Some(editor.save_target)
    } else {
        None
    };
    if let Some(target) = from_editor {
        if let Some(picker) = app.theme_picker.as_mut() {
            picker.save_target = target;
        }
        return;
    }
    if let Some(picker) = app.theme_picker.as_mut() {
        picker.save_target = picker.save_target.toggled();
    }
}

fn place_hex_click(app: &mut App, col: u16, row: u16) -> bool {
    let line = app.hex_line;
    if line.area.width == 0 || !contains(line.area, col, row) {
        return false;
    }
    let Some(editor) = app.theme_editor.as_mut() else {
        return false;
    };
    let local = usize::from(col.saturating_sub(line.area.x).saturating_sub(line.prefix));
    editor.place_hex_cursor(local);
    true
}

fn settings_nav_click(app: &mut App, col: u16, row: u16) -> bool {
    let area = app.settings_nav_area;
    if area.width == 0 || area.height == 0 {
        return false;
    }
    let inside = col >= area.x
        && row >= area.y
        && col < area.x.saturating_add(area.width)
        && row < area.y.saturating_add(area.height);
    if !inside {
        return false;
    }
    let rel = usize::from(row.saturating_sub(area.y));
    if rel >= crate::tui::settings::NAV.len() {
        return true;
    }
    app.settings_nav = rel as u8;
    app.settings_row = 0;
    app.settings_follow = true;
    if let Some(Overlay::Theme { scroll }) = &mut app.overlay {
        *scroll = 0;
    }
    true
}

fn map_siteforge_editor_key(k: &crossterm::event::KeyEvent) -> Option<ldnddev_theme::EditorKey> {
    Some(match k.code {
        KeyCode::Up => ldnddev_theme::EditorKey::Up,
        KeyCode::Down => ldnddev_theme::EditorKey::Down,
        KeyCode::Left => ldnddev_theme::EditorKey::Left,
        KeyCode::Right => ldnddev_theme::EditorKey::Right,
        KeyCode::Tab => ldnddev_theme::EditorKey::Tab,
        KeyCode::Enter => ldnddev_theme::EditorKey::Enter,
        KeyCode::Esc => ldnddev_theme::EditorKey::Esc,
        KeyCode::Backspace => ldnddev_theme::EditorKey::Backspace,
        KeyCode::Delete => ldnddev_theme::EditorKey::Delete,
        KeyCode::Home => ldnddev_theme::EditorKey::Home,
        KeyCode::End => ldnddev_theme::EditorKey::End,
        KeyCode::Char(c) => ldnddev_theme::EditorKey::Char(c),
        _ => return None,
    })
}
