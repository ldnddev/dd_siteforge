//! Undo, delete, copy/paste, and reorder selected rows.
use super::super::*;

/// Session clipboard for `y` (copy) and `p` (paste).
#[derive(Clone)]
pub(in crate::tui) enum Clipboard {
    Node(PageNode),
    Component(crate::model::SectionComponent),
    CollectionItem(CollectionClip),
}

#[derive(Clone)]
pub(in crate::tui) enum CollectionClip {
    Accordion(crate::model::AccordionItem),
    Alternating(crate::model::AlternatingItem),
    Card(crate::model::CardItem),
    Filmstrip(crate::model::FilmstripItem),
    Milestones(crate::model::MilestonesItem),
    Slider(crate::model::SliderItem),
    Tabs(crate::model::TabsItem),
    Timeline(crate::model::TimelineItem),
}

impl Clipboard {
    fn label(&self) -> &'static str {
        match self {
            Clipboard::Node(PageNode::Hero(_)) => "dd-hero",
            Clipboard::Node(PageNode::Section(_)) => "dd-section",
            Clipboard::Component(c) => ComponentKind::from_section_component(c).label(),
            Clipboard::CollectionItem(CollectionClip::Accordion(_)) => "accordion item",
            Clipboard::CollectionItem(CollectionClip::Alternating(_)) => "alternating item",
            Clipboard::CollectionItem(CollectionClip::Card(_)) => "card item",
            Clipboard::CollectionItem(CollectionClip::Filmstrip(_)) => "filmstrip item",
            Clipboard::CollectionItem(CollectionClip::Milestones(_)) => "milestones item",
            Clipboard::CollectionItem(CollectionClip::Slider(_)) => "slider item",
            Clipboard::CollectionItem(CollectionClip::Tabs(_)) => "tabs item",
            Clipboard::CollectionItem(CollectionClip::Timeline(_)) => "timeline item",
        }
    }
}

impl App {
    pub(in crate::tui) fn delete_selected_node(&mut self) {
        let selected = self.selected_node;
        let Some(page) = self.current_page_mut() else {
            return;
        };
        if page.nodes.is_empty() {
            self.push_toast(ToastLevel::Warning, "No node to delete.");
            return;
        }
        let idx = selected.min(page.nodes.len() - 1);
        page.nodes.remove(idx);
        if page.nodes.is_empty() {
            self.selected_node = 0;
            self.selected_column = 0;
            self.selected_component = 0;
            self.selected_nested_item = 0;
        } else {
            self.selected_node = idx.min(page.nodes.len() - 1);
            self.selected_column = 0;
            self.selected_component = 0;
            self.selected_nested_item = 0;
        }
        self.push_toast(ToastLevel::Info, format!("Deleted node {}.", idx + 1));
    }

    pub(in crate::tui) fn push_undo(&mut self) {
        self.undo_stack.push(self.site.clone());
        if self.undo_stack.len() > 20 {
            self.undo_stack.remove(0);
        }
    }

    pub(in crate::tui) fn undo_last(&mut self) {
        let Some(site) = self.undo_stack.pop() else {
            self.push_toast(ToastLevel::Warning, "Nothing to undo.");
            return;
        };
        self.site = site;
        if self.selected_page >= self.site.pages.len() {
            self.selected_page = self.site.pages.len().saturating_sub(1);
        }
        self.sync_tree_row_with_selection();
        self.push_toast(ToastLevel::Success, "Undid last change.");
    }

    pub(in crate::tui) fn request_quit(&mut self) {
        if self.dirty {
            self.modal = Some(Modal::ConfirmPrompt {
                message: "Unsaved changes. Quit anyway? y/n".to_string(),
                on_confirm: ConfirmKind::QuitUnsaved,
            });
        } else {
            self.should_quit = true;
        }
    }

    pub(in crate::tui) fn delete_selected_row(&mut self) {
        if self.warn_site_settings_unavailable() {
            return;
        }
        let Some(kind) = self.selected_tree_row_kind() else {
            self.push_toast(ToastLevel::Warning, "Nothing selected to delete.");
            return;
        };
        match kind {
            TreeRowKind::SiteRoot
            | TreeRowKind::PageHead
            | TreeRowKind::HeaderRoot
            | TreeRowKind::FooterRoot => {
                self.push_toast(ToastLevel::Warning, "Cannot delete this row.");
            }
            TreeRowKind::HeaderAlert => {
                self.push_undo();
                self.site.header.alert = None;
                self.header_alert_selected = false;
                self.push_toast(ToastLevel::Info, "Deleted header dd-alert.");
                self.sync_tree_row_with_selection();
            }
            TreeRowKind::Hero { .. } | TreeRowKind::Section { .. } => {
                self.push_undo();
                self.delete_selected_node();
                self.sync_tree_row_with_selection();
            }
            TreeRowKind::Component {
                node_idx,
                column_idx,
                component_idx,
            } => {
                self.push_undo();
                self.delete_page_component(node_idx, column_idx, component_idx);
            }
            TreeRowKind::HeaderComponent {
                section_idx,
                column_idx,
                component_idx,
            } => {
                self.push_undo();
                self.delete_header_component(section_idx, column_idx, component_idx);
            }
            TreeRowKind::FooterComponent {
                section_idx,
                column_idx,
                component_idx,
            } => {
                self.push_undo();
                self.delete_footer_component(section_idx, column_idx, component_idx);
            }
            TreeRowKind::AccordionItem { .. }
            | TreeRowKind::AlternatingItem { .. }
            | TreeRowKind::CardItem { .. }
            | TreeRowKind::FilmstripItem { .. }
            | TreeRowKind::MilestonesItem { .. }
            | TreeRowKind::SliderItem { .. }
            | TreeRowKind::TabsItem { .. }
            | TreeRowKind::TimelineItem { .. } => {
                self.push_undo();
                self.remove_selected_collection_item();
                self.sync_tree_row_with_selection();
            }
            TreeRowKind::Column { .. }
            | TreeRowKind::HeaderColumn { .. }
            | TreeRowKind::FooterColumn { .. } => {
                self.push_undo();
                self.remove_selected_column();
                self.sync_tree_row_with_selection();
            }
            TreeRowKind::HeaderSection { section_idx } => {
                if self.site.header.sections.len() <= 1 {
                    self.push_toast(ToastLevel::Warning, "Cannot delete last header section.");
                    return;
                }
                self.push_undo();
                if section_idx < self.site.header.sections.len() {
                    self.site.header.sections.remove(section_idx);
                    self.selected_header_section =
                        section_idx.min(self.site.header.sections.len().saturating_sub(1));
                    self.selected_header_column = 0;
                    self.selected_header_component = 0;
                    self.push_toast(ToastLevel::Info, "Deleted header section.");
                    self.sync_tree_row_with_selection();
                }
            }
            TreeRowKind::FooterSection { section_idx } => {
                if self.site.footer.sections.len() <= 1 {
                    self.push_toast(ToastLevel::Warning, "Cannot delete last footer section.");
                    return;
                }
                self.push_undo();
                if section_idx < self.site.footer.sections.len() {
                    self.site.footer.sections.remove(section_idx);
                    self.selected_header_section =
                        section_idx.min(self.site.footer.sections.len().saturating_sub(1));
                    self.selected_header_column = 0;
                    self.selected_header_component = 0;
                    self.push_toast(ToastLevel::Info, "Deleted footer section.");
                    self.sync_tree_row_with_selection();
                }
            }
        }
    }

    pub(in crate::tui) fn delete_page_component(
        &mut self,
        node_idx: usize,
        column_idx: usize,
        component_idx: usize,
    ) {
        let new_selected = {
            let Some(page) = self.current_page_mut() else {
                return;
            };
            let Some(PageNode::Section(section)) = page.nodes.get_mut(node_idx) else {
                self.push_toast(ToastLevel::Warning, "Selected row is not a section.");
                return;
            };
            let Some(col) = section.columns.get_mut(column_idx) else {
                return;
            };
            if component_idx >= col.components.len() {
                return;
            }
            col.components.remove(component_idx);
            component_idx.min(col.components.len().saturating_sub(1))
        };
        self.selected_node = node_idx;
        self.selected_column = column_idx;
        self.selected_component = new_selected;
        self.selected_nested_item = 0;
        self.push_toast(ToastLevel::Info, "Deleted component.");
        self.sync_tree_row_with_selection();
    }

    pub(in crate::tui) fn delete_header_component(
        &mut self,
        section_idx: usize,
        column_idx: usize,
        component_idx: usize,
    ) {
        let Some(section) = self.site.header.sections.get_mut(section_idx) else {
            return;
        };
        let Some(col) = section.columns.get_mut(column_idx) else {
            return;
        };
        if component_idx >= col.components.len() {
            return;
        }
        col.components.remove(component_idx);
        self.selected_header_section = section_idx;
        self.selected_header_column = column_idx;
        self.selected_header_component = component_idx.min(col.components.len().saturating_sub(1));
        self.push_toast(ToastLevel::Info, "Deleted header component.");
        self.sync_tree_row_with_selection();
    }

    pub(in crate::tui) fn delete_footer_component(
        &mut self,
        section_idx: usize,
        column_idx: usize,
        component_idx: usize,
    ) {
        let Some(section) = self.site.footer.sections.get_mut(section_idx) else {
            return;
        };
        let Some(col) = section.columns.get_mut(column_idx) else {
            return;
        };
        if component_idx >= col.components.len() {
            return;
        }
        col.components.remove(component_idx);
        self.selected_header_section = section_idx;
        self.selected_header_column = column_idx;
        self.selected_header_component = component_idx.min(col.components.len().saturating_sub(1));
        self.push_toast(ToastLevel::Info, "Deleted footer component.");
        self.sync_tree_row_with_selection();
    }

    pub(in crate::tui) fn copy_selected_row(&mut self) {
        if self.warn_site_settings_unavailable() {
            return;
        }
        let Some(kind) = self.selected_tree_row_kind() else {
            self.push_toast(ToastLevel::Warning, "Nothing selected to copy.");
            return;
        };
        let clip = match kind {
            TreeRowKind::HeaderAlert => self
                .site
                .header
                .alert
                .clone()
                .map(|a| Clipboard::Component(crate::model::SectionComponent::Alert(a))),
            TreeRowKind::Hero { node_idx } | TreeRowKind::Section { node_idx } => self
                .current_page()
                .nodes
                .get(node_idx)
                .cloned()
                .map(Clipboard::Node),
            TreeRowKind::Component {
                node_idx,
                column_idx,
                component_idx,
            } => match self.current_page().nodes.get(node_idx) {
                Some(PageNode::Section(section)) => section
                    .columns
                    .get(column_idx)
                    .and_then(|col| col.components.get(component_idx))
                    .cloned()
                    .map(Clipboard::Component),
                _ => None,
            },
            TreeRowKind::HeaderComponent {
                section_idx,
                column_idx,
                component_idx,
            } => self
                .site
                .header
                .sections
                .get(section_idx)
                .and_then(|s| s.columns.get(column_idx))
                .and_then(|col| col.components.get(component_idx))
                .cloned()
                .map(Clipboard::Component),
            TreeRowKind::FooterComponent {
                section_idx,
                column_idx,
                component_idx,
            } => self
                .site
                .footer
                .sections
                .get(section_idx)
                .and_then(|s| s.columns.get(column_idx))
                .and_then(|col| col.components.get(component_idx))
                .cloned()
                .map(Clipboard::Component),
            TreeRowKind::AccordionItem { .. }
            | TreeRowKind::AlternatingItem { .. }
            | TreeRowKind::CardItem { .. }
            | TreeRowKind::FilmstripItem { .. }
            | TreeRowKind::MilestonesItem { .. }
            | TreeRowKind::SliderItem { .. }
            | TreeRowKind::TabsItem { .. }
            | TreeRowKind::TimelineItem { .. } => self
                .collection_clip_from_selected()
                .map(Clipboard::CollectionItem),
            _ => {
                self.push_toast(ToastLevel::Warning, "Cannot copy this row.");
                return;
            }
        };
        match clip {
            Some(clip) => {
                let label = clip.label();
                self.clipboard = Some(clip);
                self.push_toast(ToastLevel::Success, format!("Copied {label}."));
            }
            None => self.push_toast(ToastLevel::Warning, "Could not copy this row."),
        }
    }

    pub(in crate::tui) fn paste_clipboard(&mut self) {
        if self.warn_site_settings_unavailable() {
            return;
        }
        let Some(clip) = self.clipboard.clone() else {
            self.push_toast(ToastLevel::Warning, "Nothing to paste.");
            return;
        };
        let label = clip.label();
        self.push_undo();
        let ok = match clip {
            Clipboard::Node(node) => self.paste_node(node),
            Clipboard::Component(component) => self.paste_component(component),
            Clipboard::CollectionItem(item) => self.paste_collection_item(item),
        };
        if ok {
            self.push_toast(ToastLevel::Success, format!("Pasted {label}."));
            self.sync_tree_row_with_selection();
        } else {
            self.undo_stack.pop();
        }
    }

    fn paste_node(&mut self, node: PageNode) -> bool {
        match self.selected_region {
            SelectedRegion::Site => {
                self.push_toast(ToastLevel::Warning, "Cannot paste on Site settings.");
                false
            }
            SelectedRegion::Page => {
                let page_head = self.page_head_selected;
                let after = self.selected_node;
                let Some(page) = self.current_page_mut() else {
                    return false;
                };
                let insert_at = if page_head || page.nodes.is_empty() {
                    0
                } else {
                    (after + 1).min(page.nodes.len())
                };
                page.nodes.insert(insert_at, node);
                self.selected_node = insert_at;
                self.selected_column = 0;
                self.selected_component = 0;
                self.selected_nested_item = 0;
                self.page_head_selected = false;
                true
            }
            SelectedRegion::Header | SelectedRegion::Footer => {
                let PageNode::Section(section) = node else {
                    self.push_toast(ToastLevel::Warning, "dd-hero can only be pasted on a page.");
                    return false;
                };
                let footer = self.selected_region == SelectedRegion::Footer;
                let sections = if footer {
                    &mut self.site.footer.sections
                } else {
                    &mut self.site.header.sections
                };
                let insert_at = if sections.is_empty() {
                    0
                } else {
                    (self.selected_header_section + 1).min(sections.len())
                };
                sections.insert(insert_at, section);
                self.selected_header_section = insert_at;
                self.selected_header_column = 0;
                self.selected_header_component = 0;
                true
            }
        }
    }

    fn paste_component(&mut self, component: crate::model::SectionComponent) -> bool {
        let kind = ComponentKind::from_section_component(&component);
        if matches!(kind, ComponentKind::Alert) {
            match self.selected_region {
                SelectedRegion::Header => {
                    let crate::model::SectionComponent::Alert(alert) = component else {
                        return false;
                    };
                    return self.place_header_alert(alert);
                }
                SelectedRegion::Footer => {
                    self.push_toast(
                        ToastLevel::Warning,
                        "dd-alert cannot be pasted in the footer.",
                    );
                    return false;
                }
                SelectedRegion::Site => {
                    self.push_toast(ToastLevel::Warning, "Cannot paste on Site settings.");
                    return false;
                }
                SelectedRegion::Page => {}
            }
        }
        let header_only = matches!(
            kind,
            ComponentKind::HeaderSearch | ComponentKind::HeaderMenu
        );
        match self.selected_region {
            SelectedRegion::Site => {
                self.push_toast(ToastLevel::Warning, "Cannot paste on Site settings.");
                false
            }
            SelectedRegion::Footer | SelectedRegion::Page if header_only => {
                self.push_toast(
                    ToastLevel::Warning,
                    format!("{} can only be pasted in the header.", kind.label()),
                );
                false
            }
            SelectedRegion::Header => self.insert_component_in_region(false, component),
            SelectedRegion::Footer => self.insert_component_in_region(true, component),
            SelectedRegion::Page => {
                let selected = self.selected_node;
                let selected_column = self.selected_column;
                let selected_component = self.selected_component;
                let Some(page) = self.current_page_mut() else {
                    return false;
                };
                if page.nodes.is_empty() {
                    self.push_toast(
                        ToastLevel::Warning,
                        "Select a section to paste a component.",
                    );
                    return false;
                }
                let idx = selected.min(page.nodes.len() - 1);
                match &mut page.nodes[idx] {
                    PageNode::Section(section) => {
                        let col_i = selected_column.min(section.columns.len().saturating_sub(1));
                        let components = &mut section.columns[col_i].components;
                        let insert_at = if components.is_empty() {
                            0
                        } else {
                            (selected_component + 1).min(components.len())
                        };
                        components.insert(insert_at, component);
                        self.selected_component = insert_at;
                        self.selected_nested_item = 0;
                        true
                    }
                    _ => {
                        self.push_toast(
                            ToastLevel::Warning,
                            "Select a section to paste a component.",
                        );
                        false
                    }
                }
            }
        }
    }

    fn insert_component_in_region(
        &mut self,
        footer: bool,
        component: crate::model::SectionComponent,
    ) -> bool {
        let sections = if footer {
            &mut self.site.footer.sections
        } else {
            &mut self.site.header.sections
        };
        if sections.is_empty() {
            self.push_toast(
                ToastLevel::Warning,
                if footer {
                    "No footer section available. Add a section first with '/'."
                } else {
                    "No header section available. Add a section first with '/'."
                },
            );
            return false;
        }
        let section_idx = self
            .selected_header_section
            .min(sections.len().saturating_sub(1));
        let col_idx = self
            .selected_header_column
            .min(sections[section_idx].columns.len().saturating_sub(1));
        let col = &mut sections[section_idx].columns[col_idx];
        let insert_at = if col.components.is_empty() {
            0
        } else {
            (self.selected_header_component + 1).min(col.components.len())
        };
        col.components.insert(insert_at, component);
        self.selected_header_component = insert_at;
        true
    }

    fn collection_clip_from_selected(&self) -> Option<CollectionClip> {
        let page = self.current_page();
        if page.nodes.is_empty() {
            return None;
        }
        let ni = self.selected_node.min(page.nodes.len() - 1);
        let PageNode::Section(section) = &page.nodes[ni] else {
            return None;
        };
        let col_i = self
            .selected_column
            .min(section.columns.len().saturating_sub(1));
        let ci = component_index(
            section.columns[col_i].components.len(),
            self.selected_component,
        )?;
        let item_idx = self.selected_nested_item;
        match &section.columns[col_i].components[ci] {
            crate::model::SectionComponent::Accordion(a) => a
                .items
                .get(item_idx)
                .cloned()
                .map(CollectionClip::Accordion),
            crate::model::SectionComponent::Alternating(a) => a
                .items
                .get(item_idx)
                .cloned()
                .map(CollectionClip::Alternating),
            crate::model::SectionComponent::Card(a) => {
                a.items.get(item_idx).cloned().map(CollectionClip::Card)
            }
            crate::model::SectionComponent::Filmstrip(a) => a
                .items
                .get(item_idx)
                .cloned()
                .map(CollectionClip::Filmstrip),
            crate::model::SectionComponent::Milestones(a) => a
                .items
                .get(item_idx)
                .cloned()
                .map(CollectionClip::Milestones),
            crate::model::SectionComponent::Slider(a) => {
                a.items.get(item_idx).cloned().map(CollectionClip::Slider)
            }
            crate::model::SectionComponent::Tabs(a) => {
                a.items.get(item_idx).cloned().map(CollectionClip::Tabs)
            }
            crate::model::SectionComponent::Timeline(a) => {
                a.items.get(item_idx).cloned().map(CollectionClip::Timeline)
            }
            _ => None,
        }
    }

    fn paste_collection_item(&mut self, clip: CollectionClip) -> bool {
        let ni = self.selected_node;
        let col_i = self.selected_column;
        let selected_component = self.selected_component;
        let item_idx = self.selected_nested_item;
        let Some(page) = self.current_page_mut() else {
            self.push_toast(
                ToastLevel::Warning,
                "Select a matching collection to paste.",
            );
            return false;
        };
        if page.nodes.is_empty() {
            self.push_toast(
                ToastLevel::Warning,
                "Select a matching collection to paste.",
            );
            return false;
        }
        let ni = ni.min(page.nodes.len() - 1);
        let PageNode::Section(section) = &mut page.nodes[ni] else {
            self.push_toast(
                ToastLevel::Warning,
                "Select a matching collection to paste.",
            );
            return false;
        };
        let col_i = col_i.min(section.columns.len().saturating_sub(1));
        let Some(ci) = component_index(section.columns[col_i].components.len(), selected_component)
        else {
            self.push_toast(
                ToastLevel::Warning,
                "Select a matching collection to paste.",
            );
            return false;
        };
        let inserted = match (&mut section.columns[col_i].components[ci], clip) {
            (crate::model::SectionComponent::Accordion(a), CollectionClip::Accordion(item)) => {
                let at = if a.items.is_empty() {
                    0
                } else {
                    (item_idx + 1).min(a.items.len())
                };
                a.items.insert(at, item);
                self.selected_nested_item = at;
                true
            }
            (crate::model::SectionComponent::Alternating(a), CollectionClip::Alternating(item)) => {
                let at = if a.items.is_empty() {
                    0
                } else {
                    (item_idx + 1).min(a.items.len())
                };
                a.items.insert(at, item);
                self.selected_nested_item = at;
                true
            }
            (crate::model::SectionComponent::Card(a), CollectionClip::Card(item)) => {
                let at = if a.items.is_empty() {
                    0
                } else {
                    (item_idx + 1).min(a.items.len())
                };
                a.items.insert(at, item);
                self.selected_nested_item = at;
                true
            }
            (crate::model::SectionComponent::Filmstrip(a), CollectionClip::Filmstrip(item)) => {
                let at = if a.items.is_empty() {
                    0
                } else {
                    (item_idx + 1).min(a.items.len())
                };
                a.items.insert(at, item);
                self.selected_nested_item = at;
                true
            }
            (crate::model::SectionComponent::Milestones(a), CollectionClip::Milestones(item)) => {
                let at = if a.items.is_empty() {
                    0
                } else {
                    (item_idx + 1).min(a.items.len())
                };
                a.items.insert(at, item);
                self.selected_nested_item = at;
                true
            }
            (crate::model::SectionComponent::Slider(a), CollectionClip::Slider(item)) => {
                let at = if a.items.is_empty() {
                    0
                } else {
                    (item_idx + 1).min(a.items.len())
                };
                a.items.insert(at, item);
                self.selected_nested_item = at;
                true
            }
            (crate::model::SectionComponent::Tabs(a), CollectionClip::Tabs(item)) => {
                let at = if a.items.is_empty() {
                    0
                } else {
                    (item_idx + 1).min(a.items.len())
                };
                a.items.insert(at, item);
                self.selected_nested_item = at;
                true
            }
            (crate::model::SectionComponent::Timeline(a), CollectionClip::Timeline(item)) => {
                let at = if a.items.is_empty() {
                    0
                } else {
                    (item_idx + 1).min(a.items.len())
                };
                a.items.insert(at, item);
                self.selected_nested_item = at;
                true
            }
            _ => false,
        };
        if !inserted {
            self.push_toast(
                ToastLevel::Warning,
                "Clipboard item does not match the selected collection.",
            );
        }
        inserted
    }

    pub(in crate::tui) fn move_selected_row(&mut self, delta: isize) {
        if self.warn_site_settings_unavailable() {
            return;
        }
        let Some(kind) = self.selected_tree_row_kind() else {
            return;
        };
        match kind {
            TreeRowKind::Hero { node_idx } | TreeRowKind::Section { node_idx } => {
                let dest = node_idx as isize + delta;
                let len = self.current_page().nodes.len();
                if len < 2 || node_idx >= len || dest < 0 || dest as usize >= len {
                    return;
                }
                self.push_undo();
                let page = self.current_page_mut().unwrap();
                page.nodes.swap(node_idx, dest as usize);
                self.selected_node = dest as usize;
                self.push_toast(ToastLevel::Info, "Moved node.");
                self.sync_tree_row_with_selection();
            }
            TreeRowKind::Component {
                node_idx,
                column_idx,
                component_idx,
            } => {
                let dest = component_idx as isize + delta;
                let can_move = match self.current_page().nodes.get(node_idx) {
                    Some(PageNode::Section(section)) => section
                        .columns
                        .get(column_idx)
                        .map(|col| dest >= 0 && (dest as usize) < col.components.len())
                        .unwrap_or(false),
                    _ => false,
                };
                if !can_move {
                    return;
                }
                self.push_undo();
                let page = self.current_page_mut().unwrap();
                let PageNode::Section(section) = &mut page.nodes[node_idx] else {
                    self.undo_stack.pop();
                    return;
                };
                section.columns[column_idx]
                    .components
                    .swap(component_idx, dest as usize);
                self.selected_component = dest as usize;
                self.push_toast(ToastLevel::Info, "Moved component.");
                self.sync_tree_row_with_selection();
            }
            TreeRowKind::HeaderComponent {
                section_idx,
                column_idx,
                component_idx,
            } => self.move_region_component(false, section_idx, column_idx, component_idx, delta),
            TreeRowKind::FooterComponent {
                section_idx,
                column_idx,
                component_idx,
            } => self.move_region_component(true, section_idx, column_idx, component_idx, delta),
            TreeRowKind::Column { .. }
            | TreeRowKind::HeaderColumn { .. }
            | TreeRowKind::FooterColumn { .. } => {
                self.push_undo();
                if delta > 0 {
                    self.move_selected_column_down();
                } else {
                    self.move_selected_column_up();
                }
            }
            TreeRowKind::AccordionItem { .. }
            | TreeRowKind::AlternatingItem { .. }
            | TreeRowKind::CardItem { .. }
            | TreeRowKind::FilmstripItem { .. }
            | TreeRowKind::MilestonesItem { .. }
            | TreeRowKind::SliderItem { .. }
            | TreeRowKind::TabsItem { .. }
            | TreeRowKind::TimelineItem { .. } => {
                self.push_undo();
                if self.move_selected_collection_item(delta) {
                    self.push_toast(ToastLevel::Info, "Moved item.");
                    self.sync_tree_row_with_selection();
                } else {
                    self.undo_stack.pop();
                }
            }
            _ => {}
        }
    }

    pub(in crate::tui) fn move_region_component(
        &mut self,
        footer: bool,
        section_idx: usize,
        column_idx: usize,
        component_idx: usize,
        delta: isize,
    ) {
        let dest = component_idx as isize + delta;
        let len = if footer {
            self.site
                .footer
                .sections
                .get(section_idx)
                .and_then(|s| s.columns.get(column_idx))
                .map(|c| c.components.len())
        } else {
            self.site
                .header
                .sections
                .get(section_idx)
                .and_then(|s| s.columns.get(column_idx))
                .map(|c| c.components.len())
        };
        let Some(len) = len else {
            return;
        };
        if dest < 0 || dest as usize >= len {
            return;
        }
        self.push_undo();
        let sections = if footer {
            &mut self.site.footer.sections
        } else {
            &mut self.site.header.sections
        };
        let col = &mut sections[section_idx].columns[column_idx];
        col.components.swap(component_idx, dest as usize);
        self.selected_header_component = dest as usize;
        self.push_toast(
            ToastLevel::Info,
            if footer {
                "Moved footer component."
            } else {
                "Moved header component."
            },
        );
        self.sync_tree_row_with_selection();
    }

    pub(in crate::tui) fn move_selected_collection_item(&mut self, delta: isize) -> bool {
        let ni = self.selected_node;
        let col_i = self.selected_column;
        let selected_component = self.selected_component;
        let item_idx = self.selected_nested_item;
        let Some(page) = self.current_page_mut() else {
            return false;
        };
        let ni = ni.min(page.nodes.len().saturating_sub(1));
        let PageNode::Section(section) = &mut page.nodes[ni] else {
            return false;
        };
        let col_i = col_i.min(section.columns.len().saturating_sub(1));
        let ci = match component_index(section.columns[col_i].components.len(), selected_component)
        {
            Some(v) => v,
            None => return false,
        };
        let dest = item_idx as isize + delta;
        if dest < 0 {
            return false;
        }
        let dest = dest as usize;
        let swapped = match &mut section.columns[col_i].components[ci] {
            crate::model::SectionComponent::Accordion(a) if dest < a.items.len() => {
                a.items.swap(item_idx, dest);
                true
            }
            crate::model::SectionComponent::Alternating(a) if dest < a.items.len() => {
                a.items.swap(item_idx, dest);
                true
            }
            crate::model::SectionComponent::Card(a) if dest < a.items.len() => {
                a.items.swap(item_idx, dest);
                true
            }
            crate::model::SectionComponent::Filmstrip(a) if dest < a.items.len() => {
                a.items.swap(item_idx, dest);
                true
            }
            crate::model::SectionComponent::Milestones(a) if dest < a.items.len() => {
                a.items.swap(item_idx, dest);
                true
            }
            crate::model::SectionComponent::Slider(a) if dest < a.items.len() => {
                a.items.swap(item_idx, dest);
                true
            }
            crate::model::SectionComponent::Tabs(a) if dest < a.items.len() => {
                a.items.swap(item_idx, dest);
                true
            }
            crate::model::SectionComponent::Timeline(a) if dest < a.items.len() => {
                a.items.swap(item_idx, dest);
                true
            }
            _ => false,
        };
        if swapped {
            self.selected_nested_item = dest;
        }
        swapped
    }
}
