//! Read-only shell data: sidebar, outline, inspector, welcome rows.
//! Render and the view tests both read these strings.

use std::path::{Path, PathBuf};

use dd_siteforge_core::component_kind::ComponentKind;
use dd_siteforge_core::cursor::{
    alert_to_form_state, component_to_form_state, footer_to_form_state, header_root_to_form_state,
    hero_to_form_state, page_head_to_form_state, section_to_form_state, site_to_form_state,
};
use dd_siteforge_core::editform::{COLUMN_ITEM_FORM, EditFormState, FieldKind};
use dd_siteforge_core::model::{
    DdSection, Page, PageNode, RobotsDirective, SectionColumn, SectionComponent, Site,
};
use dd_siteforge_core::session::Session;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SideTarget {
    Site,
    Header,
    Footer,
    Page(usize),
    Folder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SideRow {
    pub target: SideTarget,
    pub label: String,
    pub depth: u32,
    pub badges: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    PageHead,
    Hero(usize),
    Section(usize),
    Column {
        node: usize,
        column: usize,
    },
    Component {
        node: usize,
        column: usize,
        component: usize,
    },
    RegionRoot,
    HeaderAlert,
    RegionSection(usize),
    RegionColumn {
        section: usize,
        column: usize,
    },
    /// Kept for a drilled column component. The column row selects the column.
    #[allow(dead_code)]
    RegionComponent {
        section: usize,
        column: usize,
        component: usize,
    },
    SiteSettings,
}

/// One column in a header or footer grid (Screens 30, 31, 40).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnCard {
    pub target: Target,
    pub label: String,
    /// The `dd-u-md-*` class when present, otherwise the base `dd-u-*` class.
    pub md_class: String,
    pub parts: u32,
    pub whole: u32,
    pub empty: bool,
    pub chips: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutlineRow {
    pub target: Target,
    pub label: String,
    pub detail: String,
    pub depth: u32,
    pub issues: usize,
    /// Header and footer sections draw these as one proportional row.
    pub columns: Vec<ColumnCard>,
}

/// Screens 33 and 34 do not draw the header or footer column row.
/// DESIGN-GAP: when the narrowest column would be under this width, the row stacks.
pub const NARROW_COLUMN_MIN: f32 = 140.0;

/// `row` keeps the drawn proportional grid. `stack` is the undesigned narrow fallback.
pub fn column_layout(columns: &[ColumnCard], outline_width: f32) -> &'static str {
    // DESIGN-GAP: Screens 33 and 34 do not say how this row behaves when narrow.
    if columns.is_empty() || outline_width <= 0.0 {
        return "row";
    }
    let whole: u32 = columns.iter().map(|column| column.parts).sum();
    if whole == 0 {
        return "row";
    }
    let narrowest = columns
        .iter()
        .map(|column| outline_width * (column.parts as f32) / (whole as f32))
        .fold(f32::MAX, f32::min);
    if narrowest < NARROW_COLUMN_MIN {
        "stack"
    } else {
        "row"
    }
}

/// Share used to size a column. Prefers `dd-u-md-*`, then a base `dd-u-N-M`.
pub fn width_share(width_class: &str) -> (u32, u32, String) {
    let tokens: Vec<&str> = width_class.split_whitespace().collect();
    if let Some(token) = tokens
        .iter()
        .copied()
        .find(|token| token.starts_with("dd-u-md-"))
    {
        if let Some((parts, whole)) = parse_ratio(token.trim_start_matches("dd-u-md-")) {
            return (parts, whole, token.to_string());
        }
    }
    if let Some(token) = tokens.iter().copied().find(|token| {
        token
            .strip_prefix("dd-u-")
            .is_some_and(|rest| rest.chars().next().is_some_and(|ch| ch.is_ascii_digit()))
    }) {
        if let Some((parts, whole)) = parse_ratio(token.trim_start_matches("dd-u-")) {
            return (parts, whole, token.to_string());
        }
    }
    (1, 1, "dd-u-1-1".to_string())
}

fn parse_ratio(text: &str) -> Option<(u32, u32)> {
    let (parts, whole) = text.split_once('-')?;
    let parts: u32 = parts.parse().ok()?;
    let whole: u32 = whole.parse().ok()?;
    (whole > 0).then_some((parts, whole))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldChrome {
    Text,
    Select,
    Toggle,
    Markdown,
    Collection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspTab {
    Content,
    Media,
    Animation,
}

impl InspTab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Content => "Content",
            Self::Media => "Media",
            Self::Animation => "Animation",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldView {
    pub label: String,
    pub value: String,
    pub required: bool,
    pub chrome: FieldChrome,
    pub error: Option<String>,
    pub meta: Option<String>,
    pub tab: InspTab,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorView {
    pub breadcrumb: String,
    pub title: String,
    pub kind: String,
    pub tabs: Vec<InspTab>,
    pub fields: Vec<FieldView>,
    pub empty: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WelcomeAction {
    Open(PathBuf),
    CreateAt(PathBuf),
    CreateHere,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WelcomeRow {
    pub label: String,
    pub action: WelcomeAction,
}

pub fn sidebar_rows(site: &Site, filter: &str) -> (Vec<SideRow>, String) {
    let pages = page_rows(site);
    let q = filter.trim();
    if q.is_empty() {
        let mut rows = vec![
            side("Site settings", SideTarget::Site, 0, vec![]),
            side("Header", SideTarget::Header, 0, vec![]),
            side("Footer", SideTarget::Footer, 0, vec![]),
        ];
        rows.extend(pages);
        let count = site.pages.len();
        return (rows, count.to_string());
    }
    let needle = q.to_ascii_lowercase();
    let mut matched = 0usize;
    let mut rows = Vec::new();
    for row in pages {
        let keep = match row.target {
            SideTarget::Page(index) => {
                let page = &site.pages[index];
                let title = page.head.title.to_ascii_lowercase();
                let slug = page.slug.to_ascii_lowercase();
                title.contains(&needle) || slug.contains(&needle)
            }
            SideTarget::Folder => false,
            _ => false,
        };
        if keep {
            matched += 1;
            rows.push(row);
        }
    }
    // Keep parent folders of a match.
    if matched > 0 {
        rows = with_folders(&page_rows(site), &rows);
    }
    let count = format!("{matched} of {}", site.pages.len());
    (rows, count)
}

fn with_folders(all: &[SideRow], matched: &[SideRow]) -> Vec<SideRow> {
    let mut out = Vec::new();
    let mut pending_folders: Vec<SideRow> = Vec::new();
    for row in all {
        match row.target {
            SideTarget::Folder => pending_folders.push(row.clone()),
            SideTarget::Page(_) => {
                if matched
                    .iter()
                    .any(|hit| hit.label == row.label && hit.depth == row.depth)
                {
                    out.append(&mut pending_folders);
                    out.push(row.clone());
                } else {
                    pending_folders.retain(|folder| folder.depth < row.depth);
                }
            }
            _ => {}
        }
    }
    out
}

fn page_rows(site: &Site) -> Vec<SideRow> {
    let mut rows = Vec::new();
    let mut folders: Vec<String> = Vec::new();
    for (index, page) in site.pages.iter().enumerate() {
        let parts: Vec<&str> = page
            .slug
            .split('/')
            .filter(|part| !part.is_empty())
            .collect();
        if parts.is_empty() {
            rows.push(page_side(page, index, 0));
            continue;
        }
        let leaf = parts.len() - 1;
        let mut depth_folders = Vec::new();
        for (depth, part) in parts.iter().enumerate().take(leaf) {
            depth_folders.push((*part).to_string());
            if folders.len() > depth && folders[depth] == *part {
                continue;
            }
            folders.truncate(depth);
            folders.push((*part).to_string());
            rows.push(side(*part, SideTarget::Folder, depth as u32, vec![]));
        }
        rows.push(page_side(page, index, leaf as u32));
    }
    rows
}

fn page_side(page: &Page, index: usize, depth: u32) -> SideRow {
    let mut badges = Vec::new();
    if page.slug == "404" || page.slug.ends_with("/404") {
        badges.push("404");
    }
    if matches!(
        page.head.robots,
        RobotsDirective::NoindexFollow | RobotsDirective::NoindexNofollow
    ) {
        badges.push("noindex");
    }
    let label = if page.head.title.trim().is_empty() {
        page.slug.clone()
    } else {
        page.head.title.clone()
    };
    side(label, SideTarget::Page(index), depth, badges)
}

fn side(
    label: impl Into<String>,
    target: SideTarget,
    depth: u32,
    badges: Vec<&'static str>,
) -> SideRow {
    SideRow {
        target,
        label: label.into(),
        depth,
        badges,
    }
}

pub fn outline_rows(site: &Site, target_region: &SideTarget, page_index: usize) -> Vec<OutlineRow> {
    match target_region {
        SideTarget::Site => vec![OutlineRow {
            target: Target::SiteSettings,
            label: "Site settings".to_string(),
            detail: site.name.clone(),
            depth: 0,
            issues: 0,
            columns: Vec::new(),
        }],
        SideTarget::Header => region_rows(site, true),
        SideTarget::Footer => region_rows(site, false),
        SideTarget::Page(index) => page_outline(site, *index),
        SideTarget::Folder => page_outline(site, page_index),
    }
}

fn page_outline(site: &Site, index: usize) -> Vec<OutlineRow> {
    let Some(page) = site.pages.get(index) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    let meta_title = page
        .head
        .meta_title
        .as_deref()
        .filter(|text| !text.is_empty())
        .unwrap_or(page.head.title.as_str());
    let description = page.head.meta_description.as_deref().unwrap_or("");
    rows.push(OutlineRow {
        target: Target::PageHead,
        label: "HEAD".to_string(),
        detail: format!(
            "meta title {}/60 · description {}/160",
            meta_title.chars().count(),
            description.chars().count()
        ),
        depth: 0,
        issues: 0,
        columns: Vec::new(),
    });
    for (node_idx, node) in page.nodes.iter().enumerate() {
        match node {
            PageNode::Hero(hero) => {
                let state = hero_to_form_state(hero);
                rows.push(OutlineRow {
                    target: Target::Hero(node_idx),
                    label: "dd-hero".to_string(),
                    detail: hero.parent_title.clone(),
                    depth: 0,
                    issues: issue_count(&state),
                    columns: Vec::new(),
                });
            }
            PageNode::Section(section) => {
                rows.push(section_row(Target::Section(node_idx), section, 0));
                for (column_idx, column) in section.columns.iter().enumerate() {
                    rows.push(column_row(
                        Target::Column {
                            node: node_idx,
                            column: column_idx,
                        },
                        column_idx,
                        column,
                        1,
                    ));
                    push_components(&mut rows, &column.components, 2, |component_idx| {
                        Target::Component {
                            node: node_idx,
                            column: column_idx,
                            component: component_idx,
                        }
                    });
                }
            }
        }
    }
    rows
}

fn region_rows(site: &Site, header: bool) -> Vec<OutlineRow> {
    let mut rows = Vec::new();
    if header {
        let cta = site
            .header
            .cta_label
            .as_deref()
            .filter(|text| !text.is_empty())
            .map(|label| format!("Header root · CTA label “{label}”"))
            .unwrap_or_else(|| "Header root".to_string());
        rows.push(OutlineRow {
            target: Target::RegionRoot,
            label: "dd-header".to_string(),
            detail: cta,
            depth: 0,
            issues: 0,
            columns: Vec::new(),
        });
        if site.header.alert.is_some() {
            rows.push(OutlineRow {
                target: Target::HeaderAlert,
                label: "dd-alert".to_string(),
                detail: String::new(),
                depth: 1,
                issues: 0,
                columns: Vec::new(),
            });
        }
        push_region_sections(&mut rows, &site.header.sections);
    } else {
        let blurb = site.footer.blurb.as_deref().unwrap_or("").trim();
        let copyright = site.footer.copyright.as_deref().unwrap_or("").trim();
        let detail = if blurb.is_empty() && copyright.is_empty() {
            "Footer root · blurb · copyright with [[YYYY]]".to_string()
        } else {
            format!("Footer root · {blurb} · {copyright}")
        };
        rows.push(OutlineRow {
            target: Target::RegionRoot,
            label: "dd-footer".to_string(),
            detail,
            depth: 0,
            issues: 0,
            columns: Vec::new(),
        });
        push_region_sections(&mut rows, &site.footer.sections);
    }
    rows
}

fn push_region_sections(rows: &mut Vec<OutlineRow>, sections: &[DdSection]) {
    for (section_idx, section) in sections.iter().enumerate() {
        let mut row = section_row(Target::RegionSection(section_idx), section, 0);
        row.columns = section
            .columns
            .iter()
            .enumerate()
            .map(|(column_idx, column)| {
                column_card(
                    Target::RegionColumn {
                        section: section_idx,
                        column: column_idx,
                    },
                    column_idx,
                    column,
                )
            })
            .collect();
        rows.push(row);
    }
}

fn column_card(target: Target, index: usize, column: &SectionColumn) -> ColumnCard {
    let (parts, whole, md_class) = width_share(&column.width_class);
    let chips = column
        .components
        .iter()
        .map(|component| {
            ComponentKind::from_section_component(component)
                .label()
                .to_string()
        })
        .collect::<Vec<_>>();
    ColumnCard {
        target,
        label: format!("col {}", index + 1),
        md_class,
        parts,
        whole,
        empty: chips.is_empty(),
        chips,
    }
}

fn section_row(target: Target, section: &DdSection, depth: u32) -> OutlineRow {
    OutlineRow {
        target,
        label: format!("DD-SECTION · {}", section.id.to_ascii_uppercase()),
        detail: section.section_title.clone().unwrap_or_default(),
        depth,
        issues: 0,
        columns: Vec::new(),
    }
}

fn column_row(target: Target, index: usize, column: &SectionColumn, depth: u32) -> OutlineRow {
    OutlineRow {
        target,
        label: format!("col {}", index + 1),
        detail: column.width_class.clone(),
        depth,
        issues: 0,
        columns: Vec::new(),
    }
}

fn push_components(
    rows: &mut Vec<OutlineRow>,
    components: &[SectionComponent],
    depth: u32,
    target_at: impl Fn(usize) -> Target,
) {
    for (component_idx, component) in components.iter().enumerate() {
        let kind = ComponentKind::from_section_component(component);
        let state = component_to_form_state(component);
        rows.push(OutlineRow {
            target: target_at(component_idx),
            label: kind.label().to_string(),
            detail: String::new(),
            depth,
            issues: state.as_ref().map(issue_count).unwrap_or(0),
            columns: Vec::new(),
        });
    }
}

pub fn inspector(
    site: &Site,
    region: &SideTarget,
    page_index: usize,
    target: &Target,
    tab: Option<InspTab>,
) -> InspectorView {
    let Some(state) = form_state(site, region, page_index, target) else {
        return InspectorView {
            breadcrumb: String::new(),
            title: "Nothing selected".to_string(),
            kind: String::new(),
            tabs: Vec::new(),
            fields: Vec::new(),
            empty: true,
        };
    };
    let mut fields = visible_fields(&state);
    if let Target::Column { .. } | Target::RegionColumn { .. } = target {
        fields.push(components_line(site, region, page_index, target));
    }
    let tabs = present_tabs(&fields);
    let active = tab
        .filter(|wanted| tabs.contains(wanted))
        .or(tabs.first().copied());
    let fields = match active {
        Some(active) => fields
            .into_iter()
            .filter(|field| field.tab == active)
            .collect(),
        None => fields,
    };
    InspectorView {
        breadcrumb: breadcrumb(site, region, page_index, target),
        title: state.form.title.to_string(),
        kind: state.form.title.to_string(),
        tabs,
        fields,
        empty: false,
    }
}

fn components_line(
    site: &Site,
    region: &SideTarget,
    page_index: usize,
    target: &Target,
) -> FieldView {
    let names = column_component_names(site, region, page_index, target);
    let value = if names.is_empty() {
        "Empty".to_string()
    } else {
        names.join("\n")
    };
    FieldView {
        label: "Components".to_string(),
        value,
        required: false,
        chrome: FieldChrome::Text,
        error: None,
        meta: None,
        tab: InspTab::Content,
    }
}

fn column_component_names(
    site: &Site,
    region: &SideTarget,
    page_index: usize,
    target: &Target,
) -> Vec<String> {
    let column = match target {
        Target::Column { node, column } => page_column(site, page_index, *node, *column),
        Target::RegionColumn { section, column } => region_column(site, region, *section, *column),
        _ => None,
    };
    column
        .map(|column| {
            column
                .components
                .iter()
                .map(|component| {
                    ComponentKind::from_section_component(component)
                        .label()
                        .to_string()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn present_tabs(fields: &[FieldView]) -> Vec<InspTab> {
    let mut tabs = Vec::new();
    for tab in [InspTab::Content, InspTab::Media, InspTab::Animation] {
        if fields.iter().any(|field| field.tab == tab) {
            tabs.push(tab);
        }
    }
    tabs
}

fn visible_fields(state: &EditFormState) -> Vec<FieldView> {
    state
        .form
        .fields
        .iter()
        .filter(|field| state.field_visible(field))
        .map(|field| {
            let tab = tab_of(field.id);
            let chrome = chrome_of(&field.kind);
            let raw = match &field.kind {
                FieldKind::SubForm { .. } => collection_value(state, field.id),
                _ => state.get(field.id).to_string(),
            };
            let empty = raw.trim().is_empty() || raw == "(no items)";
            let error = if field.required && raw.trim().is_empty() {
                Some(format!("{} is required.", field.label))
            } else {
                None
            };
            let meta = if chrome == FieldChrome::Markdown {
                Some(format!("Markdown · {} chars", raw.chars().count()))
            } else {
                None
            };
            let value = if empty && chrome != FieldChrome::Collection {
                "Empty".to_string()
            } else {
                raw
            };
            FieldView {
                label: field.label.to_string(),
                value,
                required: field.required,
                chrome,
                error,
                meta,
                tab,
            }
        })
        .collect()
}

fn collection_value(state: &EditFormState, id: &str) -> String {
    let count = state
        .sub_state
        .get(id)
        .map(|items| items.len())
        .unwrap_or(0);
    if count == 0 {
        "(no items)".to_string()
    } else {
        format!("{count} items")
    }
}

fn tab_of(id: &str) -> InspTab {
    if id == "sal" || id == "sal_duration" || id == "sal_delay" {
        InspTab::Animation
    } else if id.starts_with("media_") || id.starts_with("parent_image") {
        InspTab::Media
    } else {
        InspTab::Content
    }
}

fn chrome_of(kind: &FieldKind) -> FieldChrome {
    match kind {
        FieldKind::Textarea { .. } => FieldChrome::Markdown,
        FieldKind::SubForm { .. } => FieldChrome::Collection,
        FieldKind::Enum { options, .. } if is_toggle(options) => FieldChrome::Toggle,
        FieldKind::Enum { .. } => FieldChrome::Select,
        _ => FieldChrome::Text,
    }
}

fn is_toggle(options: &[&str]) -> bool {
    if options.len() != 2 {
        return false;
    }
    options.iter().all(|option| {
        matches!(
            option.to_ascii_lowercase().as_str(),
            "true" | "false" | "on" | "off" | "yes" | "no"
        )
    })
}

fn issue_count(state: &EditFormState) -> usize {
    state
        .form
        .fields
        .iter()
        .filter(|field| {
            state.field_visible(field) && field.required && state.get(field.id).trim().is_empty()
        })
        .count()
}

fn form_state(
    site: &Site,
    region: &SideTarget,
    page_index: usize,
    target: &Target,
) -> Option<EditFormState> {
    match target {
        Target::SiteSettings => Some(site_to_form_state(site)),
        Target::PageHead => site.pages.get(page_index).map(page_head_to_form_state),
        Target::Hero(node) => hero_at(site, page_index, *node).map(hero_to_form_state),
        Target::Section(node) => section_at(site, page_index, *node).map(section_to_form_state),
        Target::Column { node, column } => {
            let column = page_column(site, page_index, *node, *column)?;
            let mut state = EditFormState::new(&COLUMN_ITEM_FORM);
            state.set("id", column.id.clone());
            state.set("width_class", column.width_class.clone());
            Some(state)
        }
        Target::Component {
            node,
            column,
            component,
        } => {
            let column = page_column(site, page_index, *node, *column)?;
            column
                .components
                .get(*component)
                .and_then(component_to_form_state)
        }
        Target::RegionRoot => match region {
            SideTarget::Header => Some(header_root_to_form_state(&site.header)),
            SideTarget::Footer => Some(footer_to_form_state(&site.footer)),
            _ => None,
        },
        Target::HeaderAlert => site.header.alert.as_ref().map(alert_to_form_state),
        Target::RegionSection(index) => region_sections(site, region)
            .get(*index)
            .map(section_to_form_state),
        Target::RegionColumn { section, column } => {
            let column = region_column(site, region, *section, *column)?;
            let mut state = EditFormState::new(&COLUMN_ITEM_FORM);
            state.set("id", column.id.clone());
            state.set("width_class", column.width_class.clone());
            Some(state)
        }
        Target::RegionComponent {
            section,
            column,
            component,
        } => {
            let column = region_column(site, region, *section, *column)?;
            column
                .components
                .get(*component)
                .and_then(component_to_form_state)
        }
    }
}

fn hero_at(site: &Site, page: usize, node: usize) -> Option<&dd_siteforge_core::model::DdHero> {
    match site.pages.get(page)?.nodes.get(node)? {
        PageNode::Hero(hero) => Some(hero),
        _ => None,
    }
}

fn section_at(site: &Site, page: usize, node: usize) -> Option<&DdSection> {
    match site.pages.get(page)?.nodes.get(node)? {
        PageNode::Section(section) => Some(section),
        _ => None,
    }
}

fn page_column(site: &Site, page: usize, node: usize, column: usize) -> Option<&SectionColumn> {
    section_at(site, page, node)?.columns.get(column)
}

fn region_sections<'a>(site: &'a Site, region: &SideTarget) -> &'a [DdSection] {
    match region {
        SideTarget::Header => &site.header.sections,
        SideTarget::Footer => &site.footer.sections,
        _ => &[],
    }
}

fn region_column<'a>(
    site: &'a Site,
    region: &SideTarget,
    section: usize,
    column: usize,
) -> Option<&'a SectionColumn> {
    region_sections(site, region)
        .get(section)?
        .columns
        .get(column)
}

pub fn breadcrumb(site: &Site, region: &SideTarget, page_index: usize, target: &Target) -> String {
    match region {
        SideTarget::Site => "Site settings".to_string(),
        SideTarget::Header => match target {
            Target::RegionColumn { section, column } => {
                let id = site
                    .header
                    .sections
                    .get(*section)
                    .map(|section| section.id.as_str())
                    .unwrap_or("header");
                format!("Header › {id} › col {}", column + 1)
            }
            _ => "Header".to_string(),
        },
        SideTarget::Footer => "Footer".to_string(),
        SideTarget::Page(_) | SideTarget::Folder => {
            let page_at = match region {
                SideTarget::Page(index) => *index,
                _ => page_index,
            };
            let page = site.pages.get(page_at);
            let slug = page.map(|page| page.slug.as_str()).unwrap_or("page");
            match target {
                Target::PageHead => slug.to_string(),
                Target::Hero(_) => format!("{slug} › dd-hero"),
                Target::Section(node) => {
                    let id = section_at(site, page_index, *node)
                        .map(|section| section.id.as_str())
                        .unwrap_or("section");
                    format!("{slug} › {id}")
                }
                Target::Column { node, column } => {
                    let id = section_at(site, page_index, *node)
                        .map(|section| section.id.as_str())
                        .unwrap_or("section");
                    format!("{slug} › {id} › col {}", column + 1)
                }
                Target::Component {
                    node,
                    column,
                    component,
                } => {
                    let id = section_at(site, page_index, *node)
                        .map(|section| section.id.as_str())
                        .unwrap_or("section");
                    let kind = page_column(site, page_index, *node, *column)
                        .and_then(|column| column.components.get(*component))
                        .map(|component| ComponentKind::from_section_component(component).label())
                        .unwrap_or("block");
                    format!("{slug} › {id} › {kind}")
                }
                _ => slug.to_string(),
            }
        }
    }
}

pub fn title_crumb(site: &Site, region: &SideTarget, page_index: usize) -> String {
    match region {
        SideTarget::Site => "/ Site settings".to_string(),
        SideTarget::Header => "/ Header".to_string(),
        SideTarget::Footer => "/ Footer".to_string(),
        SideTarget::Page(_) | SideTarget::Folder => {
            let slug = site
                .pages
                .get(match region {
                    SideTarget::Page(index) => *index,
                    _ => page_index,
                })
                .map(|page| page.slug.as_str())
                .unwrap_or("");
            let parts: Vec<&str> = slug.split('/').filter(|part| !part.is_empty()).collect();
            if parts.is_empty() {
                String::new()
            } else {
                format!("/ {}", parts.join(" / "))
            }
        }
    }
}

pub fn block_count(site: &Site, region: &SideTarget, page_index: usize) -> usize {
    match region {
        SideTarget::Page(index) => site
            .pages
            .get(*index)
            .map(|page| page.nodes.len())
            .unwrap_or(0),
        SideTarget::Folder => site
            .pages
            .get(page_index)
            .map(|page| page.nodes.len())
            .unwrap_or(0),
        SideTarget::Header => site.header.sections.len(),
        SideTarget::Footer => site.footer.sections.len(),
        SideTarget::Site => 0,
    }
}

pub fn status_meta(site: &Site, region: &SideTarget, page_index: usize) -> String {
    match region {
        SideTarget::Page(_) | SideTarget::Folder => {
            let page = site.pages.get(match region {
                SideTarget::Page(index) => *index,
                _ => page_index,
            });
            let slug = page.map(|page| page.slug.as_str()).unwrap_or("page");
            format!("{slug} · {} blocks", block_count(site, region, page_index))
        }
        SideTarget::Header => format!("header · {} sections", site.header.sections.len()),
        SideTarget::Footer => format!("footer · {} sections", site.footer.sections.len()),
        SideTarget::Site => "site".to_string(),
    }
}

pub fn outline_caption(site: &Site, region: &SideTarget, page_index: usize) -> (String, String) {
    match region {
        SideTarget::Page(_) | SideTarget::Folder => {
            let page = site.pages.get(match region {
                SideTarget::Page(index) => *index,
                _ => page_index,
            });
            let title = page
                .map(|page| {
                    if page.head.title.trim().is_empty() {
                        page.slug.clone()
                    } else {
                        page.head.title.clone()
                    }
                })
                .unwrap_or_else(|| "Page".to_string());
            let slug = page.map(|page| page.slug.as_str()).unwrap_or("");
            (
                title,
                format!(
                    "/ {slug} · {} blocks",
                    block_count(site, region, page_index)
                ),
            )
        }
        SideTarget::Header => (
            "Header".to_string(),
            format!(
                "/ Header · {} section · {} columns",
                site.header.sections.len(),
                site.header
                    .sections
                    .first()
                    .map(|section| section.columns.len())
                    .unwrap_or(0)
            ),
        ),
        SideTarget::Footer => (
            "Footer".to_string(),
            format!(
                "/ Footer · {} section · {} columns",
                site.footer.sections.len(),
                site.footer
                    .sections
                    .first()
                    .map(|section| section.columns.len())
                    .unwrap_or(0)
            ),
        ),
        SideTarget::Site => ("Site settings".to_string(), "/ Site settings".to_string()),
    }
}

pub fn welcome_rows(session: &Session, query: &str, cwd: &Path) -> Vec<WelcomeRow> {
    let mut rows = Vec::new();
    let q = query.trim();
    if !q.is_empty() {
        let open_path = open_candidate(Path::new(q));
        let exists = open_path.is_file();
        if exists {
            rows.push(WelcomeRow {
                label: format!("Open {q}"),
                action: WelcomeAction::Open(open_path),
            });
        } else {
            rows.push(WelcomeRow {
                label: format!("Create starter at {}", create_candidate(q).display()),
                action: WelcomeAction::CreateAt(create_candidate(q)),
            });
            rows.push(WelcomeRow {
                label: format!("Open {q}"),
                action: WelcomeAction::Open(open_path),
            });
        }
    }
    let needle = q.to_ascii_lowercase();
    for recent in session.existing_recents() {
        let shown = recent.path.display().to_string();
        if needle.is_empty() || shown.to_ascii_lowercase().contains(&needle) {
            rows.push(WelcomeRow {
                label: shown,
                action: WelcomeAction::Open(recent.path),
            });
        }
    }
    let here = cwd.join("site.json");
    let query_ok = q.is_empty()
        || "site.json".contains(&needle)
        || "create starter site.json here".contains(&needle);
    if query_ok && !here.is_file() {
        rows.push(WelcomeRow {
            label: "Create starter site.json here".to_string(),
            action: WelcomeAction::CreateHere,
        });
    }
    rows
}

fn open_candidate(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join("site.json")
    } else {
        path.to_path_buf()
    }
}

fn create_candidate(query: &str) -> PathBuf {
    if query.ends_with(".json") {
        PathBuf::from(query)
    } else {
        PathBuf::from(query.trim_end_matches('/')).join("site.json")
    }
}

pub fn default_target(region: &SideTarget) -> Target {
    match region {
        SideTarget::Site => Target::SiteSettings,
        SideTarget::Header | SideTarget::Footer => Target::RegionRoot,
        SideTarget::Page(_) | SideTarget::Folder => Target::PageHead,
    }
}

pub fn load_error_message(path: &Path, err: &anyhow::Error) -> String {
    if !path.exists() {
        format!("No site JSON at {}", path.display())
    } else {
        // `load_site` already uses the TUI context strings.
        err.to_string()
    }
}
