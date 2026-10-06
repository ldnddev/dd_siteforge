//! Modal event router plus save/template/rename/confirm/validation.
use super::super::*;

impl App {
    pub(in crate::tui) fn handle_validation_errors_event(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<ModalResult> {
        use crossterm::event::KeyCode;
        let (errors_len, scroll) = match &self.modal {
            Some(Modal::ValidationErrors {
                errors,
                scroll_offset,
            }) => (errors.len(), *scroll_offset),
            _ => return Some(ModalResult::CloseCancel),
        };
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                self.modal = None;
                Some(ModalResult::CloseSuccess)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(Modal::ValidationErrors { scroll_offset, .. }) = self.modal.as_mut() {
                    *scroll_offset = scroll_offset.saturating_sub(1);
                }
                Some(ModalResult::Continue)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(Modal::ValidationErrors { scroll_offset, .. }) = self.modal.as_mut() {
                    if scroll + 1 < errors_len.max(1) {
                        *scroll_offset += 1;
                    }
                }
                Some(ModalResult::Continue)
            }
            KeyCode::PageUp => {
                if let Some(Modal::ValidationErrors { scroll_offset, .. }) = self.modal.as_mut() {
                    *scroll_offset = scroll_offset.saturating_sub(5);
                }
                Some(ModalResult::Continue)
            }
            KeyCode::PageDown => {
                if let Some(Modal::ValidationErrors { scroll_offset, .. }) = self.modal.as_mut() {
                    *scroll_offset = (scroll + 5).min(errors_len.saturating_sub(1));
                }
                Some(ModalResult::Continue)
            }
            _ => Some(ModalResult::Continue),
        }
    }

    fn handle_modal_paste(&mut self, text: &str) -> ModalResult {
        match &self.modal {
            Some(Modal::FormEdit { .. }) => {
                self.paste_into_form(text);
                ModalResult::Continue
            }
            Some(Modal::SavePrompt { path }) => {
                let mut path = path.clone();
                path.push_str(&sanitize_paste(text, false));
                self.modal = Some(Modal::SavePrompt { path });
                ModalResult::Continue
            }
            Some(Modal::RenamePagePrompt { title, page_idx }) => {
                let mut title = title.clone();
                let page_idx = *page_idx;
                title.push_str(&sanitize_paste(text, false));
                self.modal = Some(Modal::RenamePagePrompt { title, page_idx });
                ModalResult::Continue
            }
            Some(Modal::ExportPathPrompt { path }) => {
                let mut path = path.clone();
                path.push_str(&sanitize_paste(text, false));
                self.modal = Some(Modal::ExportPathPrompt { path });
                ModalResult::Continue
            }
            Some(Modal::PreviewPathPrompt { path }) => {
                let mut path = path.clone();
                path.push_str(&sanitize_paste(text, false));
                self.modal = Some(Modal::PreviewPathPrompt { path });
                ModalResult::Continue
            }
            Some(Modal::ComponentPicker { query, selected }) => {
                let mut query = query.clone();
                let selected = *selected;
                query.push_str(&sanitize_paste(text, false));
                self.modal = Some(Modal::ComponentPicker { query, selected });
                ModalResult::Continue
            }
            Some(Modal::OpenSite { query, selected }) => {
                let mut query = query.clone();
                let selected = *selected;
                query.push_str(&sanitize_paste(text, false));
                self.modal = Some(Modal::OpenSite { query, selected });
                ModalResult::Continue
            }
            Some(Modal::Find { query, selected }) => {
                let mut query = query.clone();
                let selected = *selected;
                query.push_str(&sanitize_paste(text, false));
                self.modal = Some(Modal::Find { query, selected });
                ModalResult::Continue
            }
            Some(Modal::Palette { query, selected }) => {
                let mut query = query.clone();
                let selected = *selected;
                query.push_str(&sanitize_paste(text, false));
                self.modal = Some(Modal::Palette { query, selected });
                ModalResult::Continue
            }
            Some(Modal::ImagePicker { .. }) => {
                if let Some(Modal::ImagePicker { state }) = self.modal.as_mut() {
                    state.filter.push_str(&sanitize_paste(text, false));
                    state.selected = 0;
                }
                ModalResult::Continue
            }
            Some(Modal::PagePicker { .. }) => {
                if let Some(Modal::PagePicker { state }) = self.modal.as_mut() {
                    state.filter.push_str(&sanitize_paste(text, false));
                    state.selected = 0;
                }
                ModalResult::Continue
            }
            _ => ModalResult::Continue,
        }
    }

    pub(in crate::tui) fn handle_modal_event(&mut self, evt: Event) -> Option<ModalResult> {
        let _ = self.modal.as_ref()?;

        if let Event::Paste(text) = &evt {
            return Some(self.handle_modal_paste(text));
        }

        if let Event::Key(key) = &evt {
            if key.code == KeyCode::F(1) {
                self.overlay = Some(Overlay::Help { scroll: 0 });
                return Some(ModalResult::Continue);
            }
            if key.code == KeyCode::F(2) {
                self.overlay = Some(Overlay::Theme { scroll: 0 });
                return Some(ModalResult::Continue);
            }
            let key = *key;
            return match self.modal.as_ref()? {
                Modal::ComponentPicker { .. } => self.handle_component_picker_event_unified(key),
                Modal::SavePrompt { .. } => self.handle_save_prompt_event_unified(key),
                Modal::FormEdit { .. } => self.handle_form_edit_event(key),
                Modal::TemplatePicker { .. } => self.handle_template_picker_event(key),
                Modal::ExportPathPrompt { .. } => self.handle_export_path_prompt_event(key),
                Modal::PreviewPathPrompt { .. } => self.handle_preview_path_prompt_event(key),
                Modal::RenamePagePrompt { .. } => self.handle_rename_page_prompt_event(key),
                Modal::ConfirmPrompt { .. } => self.handle_confirm_prompt_event(key),
                Modal::ValidationErrors { .. } => self.handle_validation_errors_event(key),
                Modal::ImagePicker { .. } => self.handle_image_picker_event(key),
                Modal::PagePicker { .. } => self.handle_page_picker_event(key),
                Modal::OpenSite { .. } => self.handle_open_site_event(key),
                Modal::Find { .. } => self.handle_find_event(key),
                Modal::Palette { .. } => self.handle_palette_event(key),
                Modal::PageHealth { .. } => self.handle_page_health_event(key),
            };
        }

        if let Event::Mouse(m) = &evt {
            let kind = m.kind;
            let (col, row) = (m.column, m.row);

            if matches!(kind, MouseEventKind::Up(_)) {
                self.scrollbar_drag = None;
                self.form_text_drag = false;
                return Some(ModalResult::Continue);
            }

            if matches!(kind, MouseEventKind::Drag(MouseButton::Left))
                && self.scrollbar_drag == Some(ScrollbarDrag::FormEdit)
            {
                let sb = *self.form_scrollbar_track.borrow();
                if let Some(Modal::FormEdit { scroll_offset, .. }) = self.modal.as_mut() {
                    *scroll_offset = sb.offset_at(row).min(u16::MAX as usize) as u16;
                }
                return Some(ModalResult::Continue);
            }

            if matches!(kind, MouseEventKind::Drag(MouseButton::Left)) && self.form_text_drag {
                if let Some((_, pos)) = self.form_text_hit(col, row, true) {
                    if let Some(Modal::FormEdit { cursor_pos, .. }) = self.modal.as_mut() {
                        *cursor_pos = pos;
                    }
                }
                return Some(ModalResult::Continue);
            }

            // FormEdit scrollbar before field click-to-focus so a track click
            // jumps instead of focusing the adjacent input.
            if matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
                let shift = m.modifiers.contains(KeyModifiers::SHIFT);
                let expand_hit = self
                    .form_expand_hits
                    .borrow()
                    .iter()
                    .find(|(_, r)| contains(*r, col, row))
                    .map(|(idx, _)| *idx);
                if let Some(idx) = expand_hit {
                    if let Some(Modal::FormEdit {
                        state,
                        cursor_pos,
                        selection_anchor,
                        ..
                    }) = self.modal.as_mut()
                    {
                        if state.focused_field != idx {
                            state.focused_field = idx;
                            let field_id = state.form.fields.get(idx).map(|f| f.id);
                            if let Some(field_id) = field_id {
                                *cursor_pos = text_end(state.get(field_id));
                            }
                            *selection_anchor = None;
                        }
                    }
                    self.form_textarea_expanded = true;
                    return Some(ModalResult::Continue);
                }
                let browse_hit = self
                    .form_browse_hits
                    .borrow()
                    .iter()
                    .find(|(_, r)| contains(*r, col, row))
                    .map(|(idx, _)| *idx);
                if let Some(idx) = browse_hit {
                    if let Some(Modal::FormEdit {
                        state,
                        cursor_pos,
                        selection_anchor,
                        ..
                    }) = self.modal.as_mut()
                    {
                        state.focused_field = idx;
                        let field_id = state.form.fields.get(idx).map(|f| f.id);
                        if let Some(field_id) = field_id {
                            *cursor_pos = text_end(state.get(field_id));
                        }
                        *selection_anchor = None;
                    }
                    self.try_open_form_url_picker();
                    return Some(ModalResult::Continue);
                }
                let text_click = self.form_text_hit(col, row, false);
                if let Some((idx, pos)) = text_click {
                    let now = std::time::Instant::now();
                    let is_double = if let Some((last_col, last_row, last_time)) =
                        self.last_mouse_click
                    {
                        last_col == col
                            && last_row == row
                            && now.duration_since(last_time).as_millis() < DOUBLE_CLICK_THRESHOLD_MS
                    } else {
                        false
                    };
                    self.last_mouse_click = Some((col, row, now));
                    if let Some(Modal::FormEdit {
                        state,
                        cursor_pos,
                        selection_anchor,
                        ..
                    }) = self.modal.as_mut()
                    {
                        if state.focused_field != idx {
                            *selection_anchor = None;
                        }
                        state.focused_field = idx;
                        if is_double {
                            let value = state.get(state.form.fields[idx].id);
                            let (from, to) = word_bounds_at(value, pos);
                            *selection_anchor = Some(from);
                            *cursor_pos = to;
                        } else if shift {
                            begin_shift_selection(selection_anchor, *cursor_pos);
                            *cursor_pos = pos;
                        } else {
                            *cursor_pos = pos;
                            *selection_anchor = Some(pos);
                        }
                    }
                    self.form_text_drag = true;
                    return Some(ModalResult::Continue);
                }
                if self.form_textarea_expanded {
                    return Some(ModalResult::Continue);
                }
                let sb = *self.form_scrollbar_track.borrow();
                if contains(sb.rect, col, row) {
                    if let Some(Modal::FormEdit { scroll_offset, .. }) = self.modal.as_mut() {
                        *scroll_offset = sb.offset_at(row).min(u16::MAX as usize) as u16;
                    }
                    self.scrollbar_drag = Some(ScrollbarDrag::FormEdit);
                    return Some(ModalResult::Continue);
                }
                let hit = self
                    .modal_field_areas
                    .borrow()
                    .iter()
                    .find(|(_, r)| {
                        col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
                    })
                    .map(|(idx, _)| *idx);
                if let Some(idx) = hit {
                    if let Some(modal) = self.modal.as_mut() {
                        match modal {
                            Modal::FormEdit {
                                state,
                                selection_anchor,
                                ..
                            } => {
                                if state.focused_field != idx {
                                    *selection_anchor = None;
                                }
                                state.focused_field = idx;
                            }
                            _ => {}
                        }
                    }
                    return Some(ModalResult::Continue);
                }
            }
            match kind {
                MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                    let delta: i32 = if matches!(kind, MouseEventKind::ScrollUp) {
                        -3
                    } else {
                        3
                    };
                    if self.form_textarea_expanded {
                        let wrap_width = self.focused_textarea_wrap_width();
                        if let Some(Modal::FormEdit {
                            state, cursor_pos, ..
                        }) = self.modal.as_mut()
                        {
                            let field_id = match state.form.fields.get(state.focused_field) {
                                Some(f)
                                    if matches!(f.kind, editform::FieldKind::Textarea { .. }) =>
                                {
                                    Some(f.id)
                                }
                                _ => None,
                            };
                            if let Some(field_id) = field_id {
                                *cursor_pos = textarea_move_cursor_vertical(
                                    state.get(field_id),
                                    *cursor_pos,
                                    delta as isize,
                                    wrap_width,
                                );
                            }
                        }
                        return Some(ModalResult::Continue);
                    }
                    if let Some(modal) = self.modal.as_mut() {
                        match modal {
                            Modal::ValidationErrors {
                                errors,
                                scroll_offset,
                            } => {
                                let max = errors.len().saturating_sub(1);
                                let next = (*scroll_offset as i32 + delta).max(0) as usize;
                                *scroll_offset = next.min(max);
                            }
                            Modal::FormEdit { scroll_offset, .. } => {
                                let next = (*scroll_offset as i32 + delta).max(0) as u16;
                                *scroll_offset = next;
                            }
                            _ => {}
                        }
                    }
                    return Some(ModalResult::Continue);
                }
                _ => {}
            }
        }

        Some(ModalResult::Continue)
    }

    pub(in crate::tui) fn handle_save_prompt_event_unified(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<ModalResult> {
        use crossterm::event::KeyCode;

        let path = if let Some(Modal::SavePrompt { path }) = self.modal.take() {
            path
        } else {
            return Some(ModalResult::CloseCancel);
        };

        match key.code {
            KeyCode::Esc => {
                self.push_toast(ToastLevel::Info, "Save cancelled.");
                Some(ModalResult::CloseCancel)
            }
            KeyCode::Enter => {
                let raw = path.trim();
                if raw.is_empty() {
                    self.push_toast(ToastLevel::Warning, "Save path cannot be empty.");
                    self.modal = Some(Modal::SavePrompt { path });
                    Some(ModalResult::Continue)
                } else {
                    let path_buf = std::path::PathBuf::from(raw);
                    if let Err(e) = self.commit_save_with_backup(&path_buf) {
                        self.push_toast(ToastLevel::Error, format!("Failed to save: {}", e));
                        self.modal = Some(Modal::SavePrompt { path });
                        Some(ModalResult::Continue)
                    } else {
                        let msg = format!("Saved {}", path_buf.display());
                        self.push_toast(ToastLevel::Success, msg);
                        Some(ModalResult::CloseSuccess)
                    }
                }
            }
            KeyCode::Backspace => {
                let mut new_path = path;
                new_path.pop();
                self.modal = Some(Modal::SavePrompt { path: new_path });
                Some(ModalResult::Continue)
            }
            KeyCode::Char(c) => {
                let mut new_path = path;
                new_path.push(c);
                self.modal = Some(Modal::SavePrompt { path: new_path });
                Some(ModalResult::Continue)
            }
            _ => {
                self.modal = Some(Modal::SavePrompt { path });
                Some(ModalResult::Continue)
            }
        }
    }

    pub(in crate::tui) fn handle_template_picker_event(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<ModalResult> {
        use crossterm::event::KeyCode;
        let Some(Modal::TemplatePicker { selected }) = self.modal.as_mut() else {
            return Some(ModalResult::CloseCancel);
        };
        match key.code {
            KeyCode::Esc => {
                self.modal = None;
                self.push_toast(ToastLevel::Info, "Add page cancelled.");
                Some(ModalResult::CloseCancel)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if *selected > 0 {
                    *selected -= 1;
                }
                Some(ModalResult::Continue)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if *selected < 3 {
                    *selected += 1;
                }
                Some(ModalResult::Continue)
            }
            KeyCode::Enter => {
                let picked = *selected;
                let mut new_page = match picked {
                    1 => {
                        crate::model::Page::from_template("", crate::model::PageTemplate::HeroOnly)
                    }
                    2 => crate::model::Page::from_template(
                        "",
                        crate::model::PageTemplate::HeroPlusSection,
                    ),
                    3 if !self.site.pages.is_empty() => {
                        let src_idx = self
                            .selected_page
                            .min(self.site.pages.len().saturating_sub(1));
                        crate::model::Page::duplicate_from(&self.site.pages[src_idx])
                    }
                    _ => crate::model::Page::from_template("", crate::model::PageTemplate::Blank),
                };
                Self::dedup_page_id_slug(&self.site.pages, None, &mut new_page);
                self.site.pages.push(new_page);
                self.selected_page = self.site.pages.len() - 1;
                self.selected_node = 0;
                self.selected_column = 0;
                self.selected_component = 0;
                self.selected_nested_item = 0;
                self.page_head_selected = true;
                self.reveal_selected_page();
                self.sync_tree_row_with_selection();
                self.open_new_page_head_form();
                Some(ModalResult::Continue)
            }
            _ => Some(ModalResult::Continue),
        }
    }

    fn open_new_page_head_form(&mut self) {
        let page_idx = self.selected_page;
        let Some(page) = self.site.pages.get(page_idx) else {
            self.modal = None;
            return;
        };
        let state = cursor::page_head_to_form_state(page);
        let focused = state
            .focused_field
            .min(state.form.fields.len().saturating_sub(1));
        let cursor_pos = text_end(state.get(state.form.fields[focused].id));
        self.creating_page_idx = Some(page_idx);
        self.modal = Some(Modal::FormEdit {
            state,
            cursor: cursor::Cursor::PageHead { page: page_idx },
            cursor_pos,
            selection_anchor: None,
            drill_stack: Vec::new(),
            scroll_offset: 0,
        });
        self.push_toast(
            ToastLevel::Info,
            "New page — fill in title and HEAD, Ctrl+S to add.",
        );
    }

    fn dedup_page_id_slug(
        pages: &[crate::model::Page],
        skip_idx: Option<usize>,
        page: &mut crate::model::Page,
    ) {
        let taken = |id: &str| {
            pages
                .iter()
                .enumerate()
                .any(|(i, p)| skip_idx != Some(i) && p.id == id)
        };
        if !taken(&page.id) {
            return;
        }
        let base_id = page.id.clone();
        let base_slug = page.slug.clone();
        for n in 2.. {
            let candidate_id = format!("{}-{}", base_id, n);
            if !taken(&candidate_id) {
                page.id = candidate_id;
                page.slug = format!("{}-{}", base_slug, n);
                return;
            }
        }
    }

    pub(in crate::tui) fn finalize_creating_page_identity(&mut self, page_idx: usize) {
        let Some(page) = self.site.pages.get_mut(page_idx) else {
            return;
        };
        page.id = format!("page-{}", page.slug);
        let mut page = self.site.pages[page_idx].clone();
        Self::dedup_page_id_slug(&self.site.pages, Some(page_idx), &mut page);
        self.site.pages[page_idx] = page;
    }

    pub(in crate::tui) fn discard_creating_page(&mut self) {
        let Some(idx) = self.creating_page_idx.take() else {
            return;
        };
        if idx < self.site.pages.len() {
            self.site.pages.remove(idx);
            self.selected_page = idx
                .saturating_sub(1)
                .min(self.site.pages.len().saturating_sub(1));
            self.selected_node = 0;
            self.selected_column = 0;
            self.selected_component = 0;
            self.selected_nested_item = 0;
            self.page_head_selected = false;
            self.sync_tree_row_with_selection();
        }
        self.push_toast(ToastLevel::Info, "Add page cancelled.");
    }

    pub(in crate::tui) fn handle_rename_page_prompt_event(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<ModalResult> {
        use crossterm::event::KeyCode;
        let (title, page_idx) = match &self.modal {
            Some(Modal::RenamePagePrompt { title, page_idx }) => (title.clone(), *page_idx),
            _ => return Some(ModalResult::CloseCancel),
        };
        match key.code {
            KeyCode::Esc => {
                self.modal = None;
                self.push_toast(ToastLevel::Info, "Rename cancelled.");
                Some(ModalResult::CloseCancel)
            }
            KeyCode::Enter => self.commit_rename_page(title, page_idx),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.commit_rename_page(title, page_idx)
            }
            KeyCode::Backspace => {
                let mut new_title = title;
                new_title.pop();
                self.modal = Some(Modal::RenamePagePrompt {
                    title: new_title,
                    page_idx,
                });
                Some(ModalResult::Continue)
            }
            KeyCode::Char(c) => {
                let mut new_title = title;
                new_title.push(c);
                self.modal = Some(Modal::RenamePagePrompt {
                    title: new_title,
                    page_idx,
                });
                Some(ModalResult::Continue)
            }
            _ => Some(ModalResult::Continue),
        }
    }

    pub(in crate::tui) fn commit_rename_page(
        &mut self,
        title: String,
        page_idx: usize,
    ) -> Option<ModalResult> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            self.push_toast(ToastLevel::Warning, "Title required.");
            self.modal = Some(Modal::RenamePagePrompt { title, page_idx });
            return Some(ModalResult::Continue);
        }
        if let Some(page) = self.site.pages.get_mut(page_idx) {
            page.head.title = trimmed.to_string();
            if !page.slug_locked {
                page.slug = crate::model::slug_from_title(trimmed);
            }
            let msg = format!("Renamed page: {}", page.head.title);
            self.push_toast(ToastLevel::Success, msg);
        } else {
            self.push_toast(ToastLevel::Warning, "Page no longer exists.");
        }
        self.modal = None;
        Some(ModalResult::CloseSuccess)
    }

    pub(in crate::tui) fn handle_confirm_prompt_event(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<ModalResult> {
        use crossterm::event::KeyCode;
        let kind = match &self.modal {
            Some(Modal::ConfirmPrompt { on_confirm, .. }) => on_confirm.clone(),
            _ => return Some(ModalResult::CloseCancel),
        };
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                match kind {
                    ConfirmKind::DeletePage => self.commit_delete_page(),
                    ConfirmKind::QuitUnsaved => self.should_quit = true,
                }
                self.modal = None;
                Some(ModalResult::CloseSuccess)
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.modal = None;
                self.push_toast(ToastLevel::Info, "Cancelled.");
                Some(ModalResult::CloseCancel)
            }
            _ => Some(ModalResult::Continue),
        }
    }

    pub(in crate::tui) fn commit_delete_page(&mut self) {
        if self.site.pages.len() <= 1 {
            self.push_toast(ToastLevel::Warning, "Cannot delete last page.");
            return;
        }
        let idx = self.selected_page.min(self.site.pages.len() - 1);
        let removed = self.site.pages.remove(idx);
        let msg = format!("Deleted page: {}", removed.head.title);
        self.push_toast(ToastLevel::Success, msg);
        self.deleted_pages.push(removed);
        // Cap trash at 20 (oldest dropped).
        if self.deleted_pages.len() > 20 {
            self.deleted_pages.remove(0);
        }
        self.selected_page = idx.min(self.site.pages.len() - 1);
        self.selected_node = 0;
        self.selected_column = 0;
        self.selected_component = 0;
        self.selected_nested_item = 0;
    }

    /// Run `validate_site` on the current site. Open `Modal::ValidationErrors`
    /// if any errors; otherwise set a green status and leave no modal open.
    pub(in crate::tui) fn open_validation_modal(&mut self) {
        let root = self
            .path
            .as_ref()
            .and_then(|p| p.parent().map(std::path::Path::to_path_buf));
        let errors = crate::validate::validate_site_with_root(&self.site, root.as_deref());
        if errors.is_empty() {
            self.push_toast(ToastLevel::Success, "No validation errors.");
        } else {
            self.modal = Some(Modal::ValidationErrors {
                errors,
                scroll_offset: 0,
            });
        }
    }
}
