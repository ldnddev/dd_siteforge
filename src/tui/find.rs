//! Find in site (`?` / `Ctrl+F`).
use super::*;
use serde_json::Value;

#[derive(Debug, Clone)]
pub(in crate::tui) struct FindHit {
    pub score: i32,
    pub label: String,
    pub target: FindTarget,
}

#[derive(Debug, Clone)]
pub(in crate::tui) enum FindTarget {
    SiteRoot,
    HeaderRoot,
    HeaderAlert,
    HeaderComponent {
        section: usize,
        column: usize,
        component: usize,
    },
    FooterRoot,
    FooterComponent {
        section: usize,
        column: usize,
        component: usize,
    },
    PageHead {
        page: usize,
    },
    Hero {
        page: usize,
        node: usize,
    },
    Section {
        page: usize,
        node: usize,
    },
    Component {
        page: usize,
        node: usize,
        column: usize,
        component: usize,
    },
}

impl App {
    pub(in crate::tui) fn open_find(&mut self) {
        self.modal = Some(Modal::Find {
            query: String::new(),
            selected: 0,
        });
    }

    pub(in crate::tui) fn find_hits(&self, query: &str) -> Vec<FindHit> {
        let mut hits = Vec::new();
        let q = query.trim();
        consider(&self.site.name, "Site", q, FindTarget::SiteRoot, &mut hits);
        consider("Header", "Header", q, FindTarget::HeaderRoot, &mut hits);
        if let Some(alert) = &self.site.header.alert {
            if let Some(title) = alert.parent_title.as_deref() {
                consider(
                    title,
                    "Header / dd-alert",
                    q,
                    FindTarget::HeaderAlert,
                    &mut hits,
                );
            }
            consider(
                &alert.parent_copy,
                "Header / dd-alert",
                q,
                FindTarget::HeaderAlert,
                &mut hits,
            );
        }
        collect_section_components(
            &self.site.header.sections,
            "Header",
            q,
            |section, column, component| FindTarget::HeaderComponent {
                section,
                column,
                component,
            },
            &mut hits,
        );
        consider("Footer", "Footer", q, FindTarget::FooterRoot, &mut hits);
        collect_section_components(
            &self.site.footer.sections,
            "Footer",
            q,
            |section, column, component| FindTarget::FooterComponent {
                section,
                column,
                component,
            },
            &mut hits,
        );
        for (page_idx, page) in self.site.pages.iter().enumerate() {
            let title = page.head.title.as_str();
            consider(
                title,
                &format!("{title} / [HEAD]"),
                q,
                FindTarget::PageHead { page: page_idx },
                &mut hits,
            );
            consider(
                &page.slug,
                &format!("{title} / slug"),
                q,
                FindTarget::PageHead { page: page_idx },
                &mut hits,
            );
            if let Some(meta) = page.head.meta_title.as_deref() {
                consider(
                    meta,
                    &format!("{title} / meta title"),
                    q,
                    FindTarget::PageHead { page: page_idx },
                    &mut hits,
                );
            }
            if let Some(desc) = page.head.meta_description.as_deref() {
                consider(
                    desc,
                    &format!("{title} / meta description"),
                    q,
                    FindTarget::PageHead { page: page_idx },
                    &mut hits,
                );
            }
            for (node_idx, node) in page.nodes.iter().enumerate() {
                match node {
                    PageNode::Hero(hero) => {
                        let target = FindTarget::Hero {
                            page: page_idx,
                            node: node_idx,
                        };
                        collect_json_strings(
                            hero,
                            &format!("{title} / dd-hero"),
                            q,
                            target,
                            &mut hits,
                        );
                    }
                    PageNode::Section(section) => {
                        let target = FindTarget::Section {
                            page: page_idx,
                            node: node_idx,
                        };
                        if let Some(st) = section.section_title.as_deref() {
                            consider(
                                st,
                                &format!("{title} / dd-section"),
                                q,
                                target.clone(),
                                &mut hits,
                            );
                        }
                        consider(
                            &section.id,
                            &format!("{title} / dd-section id"),
                            q,
                            target,
                            &mut hits,
                        );
                        collect_section_components(
                            std::slice::from_ref(section),
                            &format!("{title} / dd-section"),
                            q,
                            |_, column, component| FindTarget::Component {
                                page: page_idx,
                                node: node_idx,
                                column,
                                component,
                            },
                            &mut hits,
                        );
                    }
                }
            }
        }
        hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.label.cmp(&b.label)));
        hits.dedup_by(|a, b| a.label == b.label && a.score == b.score);
        hits.truncate(40);
        hits
    }

    pub(in crate::tui) fn jump_to_find(&mut self, target: FindTarget) {
        self.header_alert_selected = false;
        self.page_head_selected = false;
        match target {
            FindTarget::SiteRoot => {
                self.selected_region = SelectedRegion::Site;
                self.selected_sidebar_section = SidebarSection::Regions;
            }
            FindTarget::HeaderRoot => {
                self.selected_region = SelectedRegion::Header;
                self.selected_sidebar_section = SidebarSection::Layouts;
                self.selected_header_section = 0;
                self.selected_header_column = 0;
                self.selected_header_component = 0;
            }
            FindTarget::HeaderAlert => {
                self.selected_region = SelectedRegion::Header;
                self.selected_sidebar_section = SidebarSection::Layouts;
                self.header_alert_selected = true;
            }
            FindTarget::HeaderComponent {
                section,
                column,
                component,
            } => {
                self.selected_region = SelectedRegion::Header;
                self.selected_sidebar_section = SidebarSection::Layouts;
                self.selected_header_section = section;
                self.selected_header_column = column;
                self.selected_header_component = component;
                self.header_column_expanded = true;
            }
            FindTarget::FooterRoot => {
                self.selected_region = SelectedRegion::Footer;
                self.selected_sidebar_section = SidebarSection::Layouts;
            }
            FindTarget::FooterComponent {
                section,
                column,
                component,
            } => {
                self.selected_region = SelectedRegion::Footer;
                self.selected_sidebar_section = SidebarSection::Layouts;
                self.selected_header_section = section;
                self.selected_header_column = column;
                self.selected_header_component = component;
            }
            FindTarget::PageHead { page } => {
                self.selected_region = SelectedRegion::Page;
                self.selected_sidebar_section = SidebarSection::Layouts;
                self.selected_page = page.min(self.site.pages.len().saturating_sub(1));
                self.page_head_selected = true;
            }
            FindTarget::Hero { page, node } => {
                self.select_page_node(page, node);
            }
            FindTarget::Section { page, node } => {
                self.select_page_node(page, node);
                self.set_section_expanded(node, true);
            }
            FindTarget::Component {
                page,
                node,
                column,
                component,
            } => {
                self.select_page_node(page, node);
                self.selected_column = column;
                self.selected_component = component;
                self.set_section_expanded(node, true);
            }
        }
        self.sync_tree_row_with_selection();
    }

    fn select_page_node(&mut self, page: usize, node: usize) {
        self.selected_region = SelectedRegion::Page;
        self.selected_sidebar_section = SidebarSection::Layouts;
        self.selected_page = page.min(self.site.pages.len().saturating_sub(1));
        self.selected_node = node;
        self.selected_column = 0;
        self.selected_component = 0;
        self.page_head_selected = false;
    }

    pub(in crate::tui) fn handle_find_event(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<ModalResult> {
        let (query, selected) = if let Some(Modal::Find { query, selected }) = self.modal.take() {
            (query, selected)
        } else {
            return Some(ModalResult::CloseCancel);
        };
        let hits = self.find_hits(&query);
        match key.code {
            KeyCode::Esc => {
                self.push_toast(ToastLevel::Info, "Find cancelled.");
                Some(ModalResult::CloseCancel)
            }
            KeyCode::Up => {
                self.modal = Some(Modal::Find {
                    query,
                    selected: selected.saturating_sub(1),
                });
                Some(ModalResult::Continue)
            }
            KeyCode::Down => {
                let max = hits.len().saturating_sub(1);
                self.modal = Some(Modal::Find {
                    query,
                    selected: if hits.is_empty() {
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
                self.modal = Some(Modal::Find { query, selected: 0 });
                Some(ModalResult::Continue)
            }
            KeyCode::Enter => {
                if let Some(hit) = hits.get(selected) {
                    let label = hit.label.clone();
                    let target = hit.target.clone();
                    self.jump_to_find(target);
                    self.push_toast(ToastLevel::Info, format!("Jumped to {label}"));
                    Some(ModalResult::CloseSuccess)
                } else {
                    self.push_toast(ToastLevel::Warning, "No matches.");
                    self.modal = Some(Modal::Find { query, selected });
                    Some(ModalResult::Continue)
                }
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let mut query = query;
                query.push(c);
                self.modal = Some(Modal::Find { query, selected: 0 });
                Some(ModalResult::Continue)
            }
            _ => {
                self.modal = Some(Modal::Find { query, selected });
                Some(ModalResult::Continue)
            }
        }
    }
}

fn consider(hay: &str, ctx: &str, query: &str, target: FindTarget, hits: &mut Vec<FindHit>) {
    let hay = hay.trim();
    if hay.is_empty() {
        return;
    }
    if query.is_empty() {
        hits.push(FindHit {
            score: 0,
            label: format!("{ctx}: {}", truncate_hit(hay, 60)),
            target,
        });
        return;
    }
    if let Some(score) = fuzzy_score(query, hay) {
        hits.push(FindHit {
            score,
            label: format!("{ctx}: {}", truncate_hit(hay, 60)),
            target,
        });
    }
}

fn collect_json_strings<T: serde::Serialize>(
    value: &T,
    ctx: &str,
    query: &str,
    target: FindTarget,
    hits: &mut Vec<FindHit>,
) {
    if query.trim().is_empty() {
        return;
    }
    if let Ok(json) = serde_json::to_value(value) {
        walk_strings(&json, &mut |s| {
            if s.chars().count() < 2 {
                return;
            }
            consider(s, ctx, query, target.clone(), hits);
        });
    }
}

fn collect_section_components(
    sections: &[crate::model::DdSection],
    ctx: &str,
    query: &str,
    target: impl Fn(usize, usize, usize) -> FindTarget,
    hits: &mut Vec<FindHit>,
) {
    for (section_idx, section) in sections.iter().enumerate() {
        for (column_idx, col) in section.columns.iter().enumerate() {
            for (component_idx, comp) in col.components.iter().enumerate() {
                let kind = ComponentKind::from_section_component(comp);
                let t = target(section_idx, column_idx, component_idx);
                collect_json_strings(comp, &format!("{ctx} / {}", kind.label()), query, t, hits);
            }
        }
    }
}

fn walk_strings(value: &Value, f: &mut impl FnMut(&str)) {
    match value {
        Value::String(s) => f(s),
        Value::Array(arr) => {
            for v in arr {
                walk_strings(v, f);
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                walk_strings(v, f);
            }
        }
        _ => {}
    }
}

fn truncate_hit(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= n {
        s.to_string()
    } else {
        let mut out: String = chars[..n].iter().collect();
        out.push('…');
        out
    }
}
