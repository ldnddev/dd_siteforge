//! Site mutations shared by the GUI editor and the TUI.
//!
//! The bytes written to `site.json` come from these functions plus
//! `apply_edit_form_to_component`. Toasts and selection stay in the frontend.

use crate::component_kind::ComponentKind;
use crate::model::{
    CtaTarget, DdHero, DdSection, HeroCopyPosition, HeroImageClass, Media, PageNode, SalAnimation,
    SectionClass, SectionColumn, SectionItemBoxClass, Site, next_column_id,
};

/// Give a section its first column when the list is empty.
/// Fill blank or duplicate section ids. The TUI does this when an `App` opens.
pub fn ensure_section_ids(site: &mut Site) {
    for page in &mut site.pages {
        let mut used = std::collections::HashSet::new();
        let mut next_idx = 1usize;
        for node in &mut page.nodes {
            let PageNode::Section(section) = node else {
                continue;
            };
            let current = section.id.trim().to_string();
            if !current.is_empty() && used.insert(current) {
                continue;
            }
            loop {
                let candidate = format!("section-{next_idx}");
                next_idx += 1;
                if used.insert(candidate.clone()) {
                    section.id = candidate;
                    break;
                }
            }
        }
    }
}

pub fn normalize_section_columns(section: &mut DdSection) {
    if section.columns.is_empty() {
        let id = next_column_id(&section.id, std::iter::empty::<&str>());
        section.columns.push(SectionColumn {
            id,
            width_class: "dd-u-1-1".to_string(),
            components: Vec::new(),
        });
    }
}

pub fn next_section_id(page_nodes: &[PageNode]) -> String {
    let mut used = std::collections::HashSet::new();
    for node in page_nodes {
        if let PageNode::Section(section) = node {
            if !section.id.trim().is_empty() {
                used.insert(section.id.clone());
            }
        }
    }
    let mut idx = 1usize;
    loop {
        let candidate = format!("section-{idx}");
        if !used.contains(&candidate) {
            return candidate;
        }
        idx += 1;
    }
}

/// Append a `dd-u-1-1` column and return its id.
pub fn push_column(section: &mut DdSection) -> String {
    normalize_section_columns(section);
    let id = next_column_id(
        &section.id,
        section.columns.iter().map(|column| column.id.as_str()),
    );
    section.columns.push(SectionColumn {
        id: id.clone(),
        width_class: "dd-u-1-1".to_string(),
        components: Vec::new(),
    });
    id
}

pub fn add_page_column(site: &mut Site, page: usize, node: usize) -> Result<String, String> {
    let page = site
        .pages
        .get_mut(page)
        .ok_or_else(|| "No selected section.".to_string())?;
    let node = page
        .nodes
        .get_mut(node)
        .ok_or_else(|| "No selected section.".to_string())?;
    match node {
        PageNode::Section(section) => Ok(push_column(section)),
        PageNode::Hero(_) => Err("Selected node is not a section.".to_string()),
    }
}

/// Insert `kind` into a section column, after `selected` when the column is not empty.
pub fn insert_section_component(
    section: &mut DdSection,
    column: usize,
    selected: usize,
    kind: ComponentKind,
) -> Result<usize, String> {
    if matches!(kind, ComponentKind::Hero | ComponentKind::Section) {
        return Err("dd-hero and dd-section are top-level insert types.".to_string());
    }
    normalize_section_columns(section);
    if section.columns.is_empty() {
        return Err("No selected section.".to_string());
    }
    let col_i = column.min(section.columns.len() - 1);
    let components = &mut section.columns[col_i].components;
    let insert_at = if components.is_empty() {
        0
    } else {
        (selected + 1).min(components.len())
    };
    components.insert(insert_at, kind.default_component());
    Ok(insert_at)
}

pub fn insert_page_component(
    site: &mut Site,
    page: usize,
    node: usize,
    column: usize,
    selected: usize,
    kind: ComponentKind,
) -> Result<(usize, String), String> {
    let page = site
        .pages
        .get_mut(page)
        .ok_or_else(|| "No selected section.".to_string())?;
    if page.nodes.is_empty() {
        return Err("No selected section.".to_string());
    }
    let idx = node.min(page.nodes.len() - 1);
    match &mut page.nodes[idx] {
        PageNode::Section(section) => {
            let at = insert_section_component(section, column, selected, kind)?;
            let id = section
                .columns
                .get(column.min(section.columns.len() - 1))
                .map(|column| column.id.clone())
                .unwrap_or_default();
            Ok((
                at,
                format!("Added {} to selected section column '{id}'.", kind.label()),
            ))
        }
        PageNode::Hero(_) => Err("Selected node is not a section.".to_string()),
    }
}

pub fn remove_section_component(section: &mut DdSection, column: usize, component: usize) -> bool {
    let Some(col) = section.columns.get_mut(column) else {
        return false;
    };
    if component >= col.components.len() {
        return false;
    }
    col.components.remove(component);
    true
}

pub fn remove_page_component(
    site: &mut Site,
    page: usize,
    node: usize,
    column: usize,
    component: usize,
) -> Result<(), String> {
    let page = site
        .pages
        .get_mut(page)
        .ok_or_else(|| "Nothing selected to delete.".to_string())?;
    let Some(PageNode::Section(section)) = page.nodes.get_mut(node) else {
        return Err("Selected row is not a section.".to_string());
    };
    if remove_section_component(section, column, component) {
        Ok(())
    } else {
        Err("Nothing selected to delete.".to_string())
    }
}

/// Swap a component with the neighbor `delta` steps away (-1 or 1).
pub fn move_page_component(
    site: &mut Site,
    page: usize,
    node: usize,
    column: usize,
    component: usize,
    delta: isize,
) -> Result<usize, String> {
    let dest = component as isize + delta;
    let page = site
        .pages
        .get_mut(page)
        .ok_or_else(|| "Nothing selected.".to_string())?;
    let Some(PageNode::Section(section)) = page.nodes.get_mut(node) else {
        return Err("Selected row is not a section.".to_string());
    };
    let Some(col) = section.columns.get_mut(column) else {
        return Err("Nothing selected.".to_string());
    };
    if dest < 0 || dest as usize >= col.components.len() {
        return Err("Nothing selected.".to_string());
    }
    col.components.swap(component, dest as usize);
    Ok(dest as usize)
}

/// Move a component to an absolute index. Drag-and-drop uses this.
pub fn drag_page_component(
    site: &mut Site,
    page: usize,
    node: usize,
    column: usize,
    from: usize,
    to: usize,
) -> Result<usize, String> {
    if from == to {
        return Ok(to);
    }
    let page = site
        .pages
        .get_mut(page)
        .ok_or_else(|| "Nothing selected.".to_string())?;
    let Some(PageNode::Section(section)) = page.nodes.get_mut(node) else {
        return Err("Selected row is not a section.".to_string());
    };
    let Some(col) = section.columns.get_mut(column) else {
        return Err("Nothing selected.".to_string());
    };
    if from >= col.components.len() || to >= col.components.len() {
        return Err("Nothing selected.".to_string());
    }
    let component = col.components.remove(from);
    col.components.insert(to, component);
    Ok(to)
}

pub fn insert_page_hero(
    site: &mut Site,
    page: usize,
    after_node: Option<usize>,
) -> Result<usize, String> {
    let page = site
        .pages
        .get_mut(page)
        .ok_or_else(|| "No page.".to_string())?;
    let hero = DdHero {
        parent_image_url: "/assets/images/hero-new.jpg".to_string(),
        parent_class: Some(HeroImageClass::FullFull),
        sal: Some(SalAnimation::NoAnimation),
        sal_duration: None,
        sal_delay: None,
        parent_custom_css: None,
        parent_title: "New Hero".to_string(),
        parent_subtitle: "Add subtitle".to_string(),
        parent_copy: None,
        links: Vec::new(),
        link_1_label: None,
        link_1_url: None,
        link_1_target: Some(CtaTarget::SelfTarget),
        link_2_label: None,
        link_2_url: None,
        link_2_target: Some(CtaTarget::SelfTarget),
        parent_image_alt: Some("Hero image".to_string()),
        parent_image_mobile: None,
        parent_image_tablet: None,
        parent_image_desktop: None,
        parent_image_class: Some(HeroImageClass::FullFull),
        media: Media::None,
        overlay: None,
        copy_position: Some(HeroCopyPosition::Left),
        id: None,
        aria_label: Some("Introduction".to_string()),
    };
    let idx = after_node
        .map(|value| value + 1)
        .unwrap_or(0)
        .min(page.nodes.len());
    page.nodes.insert(idx, PageNode::Hero(hero));
    Ok(idx)
}

pub fn insert_page_section(
    site: &mut Site,
    page: usize,
    after_node: Option<usize>,
) -> Result<usize, String> {
    let page = site
        .pages
        .get_mut(page)
        .ok_or_else(|| "No page.".to_string())?;
    let next_id = next_section_id(&page.nodes);
    let column_id = next_column_id(&next_id, std::iter::empty::<&str>());
    let section = DdSection {
        id: next_id,
        section_title: None,
        section_class: Some(SectionClass::FullContained),
        item_box_class: Some(SectionItemBoxClass::LBox),
        bg: None,
        padding: None,
        custom_css: None,
        aria_label: None,
        sal: SalAnimation::NoAnimation,
        sal_duration: None,
        sal_delay: None,
        columns: vec![SectionColumn {
            id: column_id,
            width_class: "dd-u-1-1".to_string(),
            components: Vec::new(),
        }],
    };
    let idx = after_node
        .map(|value| value + 1)
        .unwrap_or(0)
        .min(page.nodes.len());
    page.nodes.insert(idx, PageNode::Section(section));
    Ok(idx)
}

pub fn place_header_alert(site: &mut Site) -> Result<(), String> {
    if site.header.alert.is_some() {
        return Err("Header already has a dd-alert. Delete it first.".to_string());
    }
    let alert = match ComponentKind::Alert.default_component() {
        crate::model::SectionComponent::Alert(alert) => alert,
        _ => return Err("Alert default is missing.".to_string()),
    };
    site.header.alert = Some(alert);
    Ok(())
}

/// DESIGN-GAP: spec 04 names LAYOUT / CONTENT / INTERACTIVE but does not
/// list which of the 24 kinds belongs to which header. This split follows
/// the kind's job until the insert-popover board is confirmed.
pub fn insert_group(kind: ComponentKind) -> &'static str {
    match kind {
        ComponentKind::Hero | ComponentKind::Section | ComponentKind::Spacer => "LAYOUT",
        ComponentKind::Accordion
        | ComponentKind::Tabs
        | ComponentKind::Slider
        | ComponentKind::Modal
        | ComponentKind::Navigation
        | ComponentKind::HeaderSearch
        | ComponentKind::HeaderMenu
        | ComponentKind::SearchResults
        | ComponentKind::DataTable => "INTERACTIVE",
        _ => "CONTENT",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_card_then_column_then_move_matches_the_tui_indexes() {
        let mut site = Site::starter();
        let (at, message) =
            insert_page_component(&mut site, 0, 1, 0, 0, ComponentKind::Card).unwrap();
        assert_eq!(at, 0);
        assert!(message.contains("dd-card"));
        assert!(message.contains("section-1-column-1"));
        let (at, _) =
            insert_page_component(&mut site, 0, 1, 0, 0, ComponentKind::Headline).unwrap();
        assert_eq!(at, 1);
        let dest = move_page_component(&mut site, 0, 1, 0, 1, -1).unwrap();
        assert_eq!(dest, 0);
        let id = add_page_column(&mut site, 0, 1).unwrap();
        assert_eq!(id, "section-1-column-2");
        remove_page_component(&mut site, 0, 1, 0, 0).unwrap();
        let PageNode::Section(section) = &site.pages[0].nodes[1] else {
            panic!("section");
        };
        assert_eq!(section.columns.len(), 2);
        assert_eq!(section.columns[0].components.len(), 1);
    }
}
