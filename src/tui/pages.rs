//! Pages-panel tree: nest by slug path, expand/collapse, sibling reorder.
use super::*;
use std::collections::HashSet;

/// One visible row in `[2] Pages`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PageTreeRow {
    pub(super) page_idx: usize,
    pub(super) depth: usize,
    pub(super) has_children: bool,
    pub(super) expanded: bool,
    /// 1-based pre-order index among all pages (hidden descendants still count).
    pub(super) tree_ord: usize,
}

/// Longest existing slug prefix of `pages[idx]`, if any.
/// `blog/entry` nests under `blog`; `blog/2024/entry` nests under `blog/2024`
/// when that page exists, else under `blog`.
pub(super) fn page_parent_idx(pages: &[crate::model::Page], idx: usize) -> Option<usize> {
    let slug = pages.get(idx)?.slug.as_str();
    if slug == "index" {
        return None;
    }
    let mut prefix = slug;
    while let Some(slash) = prefix.rfind('/') {
        prefix = &prefix[..slash];
        if prefix.is_empty() {
            break;
        }
        if let Some((parent, _)) = pages
            .iter()
            .enumerate()
            .find(|(i, p)| *i != idx && p.slug == prefix)
        {
            return Some(parent);
        }
    }
    None
}

fn page_forest(pages: &[crate::model::Page]) -> (Vec<usize>, Vec<Vec<usize>>) {
    let n = pages.len();
    let mut children = vec![Vec::new(); n];
    let mut roots = Vec::new();
    for i in 0..n {
        match page_parent_idx(pages, i) {
            Some(parent) => children[parent].push(i),
            None => roots.push(i),
        }
    }
    (roots, children)
}

fn assign_tree_ords(roots: &[usize], children: &[Vec<usize>]) -> Vec<usize> {
    let mut ords = vec![0; children.len()];
    let mut next = 1;
    fn walk(idx: usize, children: &[Vec<usize>], ords: &mut [usize], next: &mut usize) {
        ords[idx] = *next;
        *next += 1;
        for &child in &children[idx] {
            walk(child, children, ords, next);
        }
    }
    for &root in roots {
        walk(root, children, &mut ords, &mut next);
    }
    ords
}

impl App {
    pub(super) fn set_page_expanded(&mut self, page_idx: usize, expanded: bool) {
        let Some(id) = self.site.pages.get(page_idx).map(|p| p.id.clone()) else {
            return;
        };
        if expanded {
            self.collapsed_page_ids.remove(&id);
        } else {
            self.collapsed_page_ids.insert(id);
            if self.page_is_descendant_of(self.selected_page, page_idx) {
                self.focus_page(page_idx);
            }
        }
    }

    fn page_is_descendant_of(&self, page_idx: usize, ancestor_idx: usize) -> bool {
        if page_idx == ancestor_idx {
            return false;
        }
        let mut cur = page_idx;
        while let Some(parent) = page_parent_idx(&self.site.pages, cur) {
            if parent == ancestor_idx {
                return true;
            }
            if parent == cur {
                break;
            }
            cur = parent;
        }
        false
    }

    /// Expand every ancestor so `selected_page` is a visible row.
    pub(super) fn reveal_selected_page(&mut self) {
        if self.site.pages.is_empty() {
            return;
        }
        let idx = self.selected_page.min(self.site.pages.len() - 1);
        let mut cur = idx;
        let mut ancestors = Vec::new();
        while let Some(parent) = page_parent_idx(&self.site.pages, cur) {
            ancestors.push(parent);
            if parent == cur {
                break;
            }
            cur = parent;
        }
        for parent in ancestors {
            self.set_page_expanded(parent, true);
        }
    }

    pub(super) fn build_pages_panel_rows(&self) -> Vec<PageTreeRow> {
        let pages = &self.site.pages;
        if pages.is_empty() {
            return Vec::new();
        }
        let (roots, children) = page_forest(pages);
        let ords = assign_tree_ords(&roots, &children);
        let mut rows = Vec::with_capacity(pages.len());
        fn walk(
            idx: usize,
            depth: usize,
            pages: &[crate::model::Page],
            children: &[Vec<usize>],
            ords: &[usize],
            collapsed: &HashSet<String>,
            rows: &mut Vec<PageTreeRow>,
        ) {
            let has_children = !children[idx].is_empty();
            let expanded = has_children && !collapsed.contains(&pages[idx].id);
            rows.push(PageTreeRow {
                page_idx: idx,
                depth,
                has_children,
                expanded,
                tree_ord: ords[idx],
            });
            if expanded {
                for &child in &children[idx] {
                    walk(child, depth + 1, pages, children, ords, collapsed, rows);
                }
            }
        }
        for &root in &roots {
            walk(
                root,
                0,
                pages,
                &children,
                &ords,
                &self.collapsed_page_ids,
                &mut rows,
            );
        }
        rows
    }

    pub(super) fn page_tree_ordinal(&self, page_idx: usize) -> usize {
        if self.site.pages.is_empty() {
            return 0;
        }
        let (roots, children) = page_forest(&self.site.pages);
        let ords = assign_tree_ords(&roots, &children);
        ords.get(page_idx).copied().unwrap_or(page_idx + 1)
    }

    pub(super) fn page_tree_label(&self, row: &PageTreeRow) -> String {
        let page = &self.site.pages[row.page_idx];
        let title = page.head.title.trim();
        let label_body = if title.is_empty() {
            page.slug.rsplit('/').next().unwrap_or(page.slug.as_str())
        } else {
            title
        };
        let indent = "  ".repeat(row.depth);
        let num = format!("{:02}", row.tree_ord);
        if row.has_children {
            let marker = if row.expanded { "[-]" } else { "[+]" };
            format!("{indent}{num} {marker} {label_body}")
        } else {
            format!("{indent}{num} {label_body}")
        }
    }

    pub(super) fn focus_page(&mut self, page_idx: usize) {
        if self.site.pages.is_empty() {
            return;
        }
        let idx = page_idx.min(self.site.pages.len() - 1);
        self.selected_page = idx;
        self.selected_node = 0;
        self.selected_tree_row = 0;
        self.selected_column = 0;
        self.selected_component = 0;
        self.selected_nested_item = 0;
        self.details_scroll_row = 0;
        self.page_head_selected = false;
        self.reveal_selected_page();
        self.sync_tree_row_with_selection();
    }

    /// `delta` steps along visible rows. `wrap` matches j/k and wheel.
    pub(super) fn select_visible_page_by(&mut self, delta: isize, wrap: bool) {
        let vis: Vec<usize> = self
            .build_pages_panel_rows()
            .iter()
            .map(|r| r.page_idx)
            .collect();
        if vis.is_empty() {
            return;
        }
        let cur = vis
            .iter()
            .position(|&i| i == self.selected_page)
            .unwrap_or(0);
        let last = vis.len() as isize - 1;
        let next = if wrap {
            let n = vis.len() as isize;
            ((cur as isize + delta).rem_euclid(n)) as usize
        } else {
            (cur as isize + delta).clamp(0, last) as usize
        };
        if vis[next] != self.selected_page {
            self.focus_page(vis[next]);
        }
    }

    pub(super) fn jump_visible_page_to_end(&mut self, last: bool) {
        let vis: Vec<usize> = self
            .build_pages_panel_rows()
            .iter()
            .map(|r| r.page_idx)
            .collect();
        let idx = if last {
            vis.last().copied()
        } else {
            vis.first().copied()
        };
        let Some(idx) = idx else {
            return;
        };
        if idx != self.selected_page {
            self.focus_page(idx);
        }
    }

    pub(super) fn toggle_selected_page_expanded(&mut self) {
        let rows = self.build_pages_panel_rows();
        let Some(row) = rows.iter().find(|r| r.page_idx == self.selected_page) else {
            return;
        };
        if !row.has_children {
            return;
        }
        self.set_page_expanded(row.page_idx, !row.expanded);
    }

    pub(super) fn collapse_selected_page(&mut self) {
        let rows = self.build_pages_panel_rows();
        let Some(row) = rows.iter().find(|r| r.page_idx == self.selected_page) else {
            return;
        };
        if row.has_children && row.expanded {
            self.set_page_expanded(row.page_idx, false);
        }
    }

    pub(super) fn expand_selected_page(&mut self) {
        let rows = self.build_pages_panel_rows();
        let Some(row) = rows.iter().find(|r| r.page_idx == self.selected_page) else {
            return;
        };
        if row.has_children && !row.expanded {
            self.set_page_expanded(row.page_idx, true);
        }
    }

    /// Swap the selected page with the next/prev sibling in tree order.
    pub(super) fn move_selected_page_among_siblings(&mut self, down: bool) -> bool {
        if self.site.pages.len() < 2 {
            return false;
        }
        let idx = self.selected_page.min(self.site.pages.len() - 1);
        let (roots, children) = page_forest(&self.site.pages);
        let sibs: &[usize] = match page_parent_idx(&self.site.pages, idx) {
            Some(parent) => children.get(parent).map(Vec::as_slice).unwrap_or(&[]),
            None => roots.as_slice(),
        };
        let Some(pos) = sibs.iter().position(|&i| i == idx) else {
            return false;
        };
        let other_pos = if down { pos + 1 } else { pos.wrapping_sub(1) };
        let Some(&other) = sibs.get(other_pos) else {
            return false;
        };
        self.site.pages.swap(idx, other);
        self.selected_page = other;
        true
    }
}
