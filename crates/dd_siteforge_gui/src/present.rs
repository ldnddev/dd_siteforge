//! Read-only shell data: sidebar, outline, inspector, welcome rows.
//! Render and the view tests both read these strings.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use dd_siteforge_core::component_kind::ComponentKind;
use dd_siteforge_core::controls::{self, FieldControl};
use dd_siteforge_core::cursor::{
    alert_to_form_state, component_to_form_state, footer_to_form_state, header_root_to_form_state,
    hero_to_form_state, page_head_to_form_state, section_to_form_state, site_to_form_state,
};
use dd_siteforge_core::editform::{COLUMN_ITEM_FORM, EditFormState, FieldKind};
use dd_siteforge_core::model::{
    CardItem, DdHero, DdSection, Media, Page, PageNode, RobotsDirective, SectionColumn,
    SectionComponent, Site, page_parent_index,
};
use dd_siteforge_core::session::Session;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SideTarget {
    Site,
    Header,
    Footer,
    Page(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SideRow {
    pub target: SideTarget,
    pub label: String,
    pub depth: u32,
    pub badges: Vec<&'static str>,
    /// A page that has child pages. Site, header, and footer rows are never expandable.
    pub expandable: bool,
    /// Child pages are drawn under this row.
    pub expanded: bool,
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
    /// One `dd-card` entry. The TUI lists these under the card; selecting one edits that item.
    CardItem {
        node: usize,
        column: usize,
        component: usize,
        item: usize,
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
    /// A page section whose columns are hidden in the blueprint.
    pub collapsed: bool,
    pub issues: usize,
    /// Header and footer sections draw these as one proportional row.
    pub columns: Vec<ColumnCard>,
    /// Identifying content for the blueprint. Empty fields stay blank.
    pub face: BlueprintFace,
    /// Card entries share this `parts/whole` so the blueprint can pack a row.
    pub card_span: Option<(u32, u32)>,
}

/// Text and image the blueprint draws for a hero or a card entry.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BlueprintFace {
    pub image: String,
    pub title: String,
    pub subtitle: String,
    pub copy: String,
    pub links: Vec<String>,
}

/// One column inside a page-section frame, plus the components that sit in it.
pub struct SectionColumnBlock {
    pub header: usize,
    pub body: Vec<OutlineBlock>,
}

/// One painted outline block. A page section wraps its columns. Card entries pack into rows.
pub enum OutlineBlock {
    Row(usize),
    Cards(Vec<Vec<usize>>),
    Section {
        header: usize,
        columns: Vec<SectionColumnBlock>,
    },
}

fn bare_row(
    target: Target,
    label: String,
    detail: String,
    depth: u32,
    issues: usize,
) -> OutlineRow {
    OutlineRow {
        target,
        label,
        detail,
        depth,
        collapsed: false,
        issues,
        columns: Vec::new(),
        face: BlueprintFace::default(),
        card_span: None,
    }
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
    Image,
    Page,
    Width,
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
    pub id: String,
    pub label: String,
    pub value: String,
    pub required: bool,
    pub chrome: FieldChrome,
    pub error: Option<String>,
    pub meta: Option<String>,
    pub tab: InspTab,
    /// Markdown fields draw H2 / B / I / Link / List and no preview eye.
    pub toolbar: bool,
    /// `value` is a stand-in ("Empty", a file prompt) rather than stored text.
    pub placeholder: bool,
    /// Enum choices for a dropdown. Empty for every other control, including toggles.
    pub options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorView {
    pub breadcrumb: String,
    pub title: String,
    pub kind: String,
    pub tabs: Vec<InspTab>,
    pub fields: Vec<FieldView>,
    pub empty: bool,
    /// Item back bar. Shown while a collection item is drilled open.
    pub back: Option<String>,
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

pub fn sidebar_rows(
    site: &Site,
    filter: &str,
    expanded: &HashSet<String>,
) -> (Vec<SideRow>, String) {
    let (pages, matched) = page_rows(site, expanded, filter);
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
    let count = format!("{matched} of {}", site.pages.len());
    (pages, count)
}

fn page_forest(site: &Site) -> (Vec<usize>, Vec<Vec<usize>>) {
    let n = site.pages.len();
    let mut children = vec![Vec::new(); n];
    let mut roots = Vec::new();
    for index in 0..n {
        match page_parent_index(&site.pages, index) {
            Some(parent) => children[parent].push(index),
            None => roots.push(index),
        }
    }
    (roots, children)
}

/// Visible page rows. An empty filter hides children of collapsed pages.
/// A filter shows matches and their ancestor pages, and draws those ancestors open.
fn page_rows(site: &Site, expanded: &HashSet<String>, filter: &str) -> (Vec<SideRow>, usize) {
    let (roots, children) = page_forest(site);
    let needle = filter.trim().to_ascii_lowercase();
    let (keep, matched) = if needle.is_empty() {
        (None, site.pages.len())
    } else {
        let mut keep = vec![false; site.pages.len()];
        let mut matched = 0usize;
        for (index, page) in site.pages.iter().enumerate() {
            let title = page.head.title.to_ascii_lowercase();
            let slug = page.slug.to_ascii_lowercase();
            if title.contains(&needle) || slug.contains(&needle) {
                matched += 1;
                keep[index] = true;
                let mut cur = index;
                while let Some(parent) = page_parent_index(&site.pages, cur) {
                    keep[parent] = true;
                    if parent == cur {
                        break;
                    }
                    cur = parent;
                }
            }
        }
        (Some(keep), matched)
    };
    let mut rows = Vec::new();
    fn walk(
        idx: usize,
        depth: u32,
        site: &Site,
        children: &[Vec<usize>],
        expanded: &HashSet<String>,
        keep: Option<&[bool]>,
        rows: &mut Vec<SideRow>,
    ) {
        if keep.is_some_and(|keep| !keep[idx]) {
            return;
        }
        let has_children = !children[idx].is_empty();
        let open = if let Some(keep) = keep {
            children[idx].iter().any(|&child| keep[child])
        } else {
            has_children && expanded.contains(&site.pages[idx].id)
        };
        rows.push(page_side(
            &site.pages[idx],
            idx,
            depth,
            has_children,
            open && has_children,
        ));
        if open {
            for &child in &children[idx] {
                walk(child, depth + 1, site, children, expanded, keep, rows);
            }
        }
    }
    let keep_slice = keep.as_deref();
    for &root in &roots {
        walk(root, 0, site, &children, expanded, keep_slice, &mut rows);
    }
    (rows, matched)
}

fn page_side(page: &Page, index: usize, depth: u32, expandable: bool, expanded: bool) -> SideRow {
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
    let mut row = side(label, SideTarget::Page(index), depth, badges);
    row.expandable = expandable;
    row.expanded = expanded;
    row
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
        expandable: false,
        expanded: false,
    }
}

pub fn outline_rows(
    site: &Site,
    target_region: &SideTarget,
    _page_index: usize,
) -> Vec<OutlineRow> {
    match target_region {
        SideTarget::Site => vec![bare_row(
            Target::SiteSettings,
            "Site settings".to_string(),
            site.name.clone(),
            0,
            0,
        )],
        SideTarget::Header => region_rows(site, true),
        SideTarget::Footer => region_rows(site, false),
        SideTarget::Page(index) => page_outline(site, *index),
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
    rows.push(bare_row(
        Target::PageHead,
        "HEAD".to_string(),
        format!(
            "meta title {}/60 · description {}/160",
            meta_title.chars().count(),
            description.chars().count()
        ),
        0,
        0,
    ));
    for (node_idx, node) in page.nodes.iter().enumerate() {
        match node {
            PageNode::Hero(hero) => {
                let state = hero_to_form_state(hero);
                let mut row = bare_row(
                    Target::Hero(node_idx),
                    "dd-hero".to_string(),
                    hero.parent_title.clone(),
                    0,
                    issue_count(&state),
                );
                row.face = hero_face(hero);
                rows.push(row);
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
        rows.push(bare_row(
            Target::RegionRoot,
            "dd-header".to_string(),
            cta,
            0,
            0,
        ));
        if site.header.alert.is_some() {
            rows.push(bare_row(
                Target::HeaderAlert,
                "dd-alert".to_string(),
                String::new(),
                1,
                0,
            ));
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
        rows.push(bare_row(
            Target::RegionRoot,
            "dd-footer".to_string(),
            detail,
            0,
            0,
        ));
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
    bare_row(
        target,
        format!("DD-SECTION · {}", section.id.to_ascii_uppercase()),
        section.section_title.clone().unwrap_or_default(),
        depth,
        0,
    )
}

fn column_row(target: Target, index: usize, column: &SectionColumn, depth: u32) -> OutlineRow {
    bare_row(
        target,
        format!("col {}", index + 1),
        column.width_class.clone(),
        depth,
        0,
    )
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
        let target = target_at(component_idx);
        let mut row = bare_row(
            target.clone(),
            kind.label().to_string(),
            String::new(),
            depth,
            state.as_ref().map(issue_count).unwrap_or(0),
        );
        if !matches!(component, SectionComponent::Card(_)) {
            if let Some(state) = &state {
                row.face = face_from_state(state);
            }
        }
        rows.push(row);
        let SectionComponent::Card(card) = component else {
            continue;
        };
        let Target::Component {
            node,
            column,
            component: comp,
        } = target
        else {
            continue;
        };
        let span = largest_dd_u_ratio(&card.parent_width);
        for (item_idx, item) in card.items.iter().enumerate() {
            let mut row = bare_row(
                Target::CardItem {
                    node,
                    column,
                    component: comp,
                    item: item_idx,
                },
                format!("- {}: {}", item_idx + 1, short_title(&item.child_title)),
                String::new(),
                depth + 1,
                state
                    .as_ref()
                    .and_then(|parent| parent.sub_state.get("items"))
                    .and_then(|items| items.get(item_idx))
                    .map(issue_count)
                    .unwrap_or(0),
            );
            row.face = card_item_face(item);
            row.card_span = Some(span);
            rows.push(row);
        }
    }
}

/// Outline label for a card entry. Matches the TUI tree row, without the indent.
fn short_title(value: &str) -> String {
    let trimmed = value.trim();
    let text = if trimmed.is_empty() {
        "(untitled)"
    } else {
        trimmed
    };
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= 40 {
        return text.to_string();
    }
    let mut out: String = chars.into_iter().take(37).collect();
    out.push_str("...");
    out
}

fn one_line(value: &str) -> String {
    let text = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= 160 {
        return text;
    }
    let mut out: String = chars.into_iter().take(157).collect();
    out.push_str("...");
    out
}

fn hero_face(hero: &DdHero) -> BlueprintFace {
    let image = match hero.resolved_media() {
        Media::Image { url, .. } => url.trim().to_string(),
        _ => String::new(),
    };
    BlueprintFace {
        image,
        title: hero.parent_title.trim().to_string(),
        subtitle: hero.parent_subtitle.trim().to_string(),
        copy: one_line(hero.parent_copy.as_deref().unwrap_or("")),
        links: hero
            .resolved_links()
            .into_iter()
            .filter_map(|link| {
                let label = link.label.trim();
                if label.is_empty() {
                    None
                } else {
                    Some(label.to_string())
                }
            })
            .collect(),
    }
}

fn card_item_face(item: &CardItem) -> BlueprintFace {
    let label = item.child_link_label.as_deref().unwrap_or("").trim();
    let url = item.child_link_url.as_deref().unwrap_or("").trim();
    let link = if !label.is_empty() {
        Some(label.to_string())
    } else if !url.is_empty() {
        Some(url.to_string())
    } else {
        None
    };
    BlueprintFace {
        image: item.child_image_url.trim().to_string(),
        title: item.child_title.trim().to_string(),
        subtitle: item.child_subtitle.trim().to_string(),
        copy: one_line(&item.child_copy),
        links: link.into_iter().collect(),
    }
}

fn face_from_state(state: &EditFormState) -> BlueprintFace {
    let mut face = BlueprintFace::default();
    for field in state.form.fields {
        if !state.field_visible(field) {
            continue;
        }
        let value = one_line(state.get(field.id));
        if value.is_empty() {
            continue;
        }
        match field.id {
            "parent_title" | "child_title" | "section_title" | "text" => face.title = value,
            "parent_subtitle" | "child_subtitle" => face.subtitle = value,
            "parent_copy" | "child_copy" => face.copy = value,
            "parent_image_url" | "media_image_url" | "child_image_url" if face.image.is_empty() => {
                face.image = value;
            }
            "child_link_label" | "cta_label" => face.links.push(value),
            "links" => {
                let Some(items) = state.sub_state.get(field.id) else {
                    continue;
                };
                for item in items {
                    let label = one_line(item.get("label"));
                    if !label.is_empty() {
                        face.links.push(label);
                    }
                }
            }
            _ => {}
        }
    }
    face
}

/// Largest authored `dd-u-*` breakpoint, matching the TUI card blueprint.
fn largest_dd_u_ratio(width_class: &str) -> (u32, u32) {
    let tokens: Vec<&str> = width_class.split_whitespace().collect();
    for prefix in ["dd-u-xxl-", "dd-u-xl-", "dd-u-lg-", "dd-u-md-", "dd-u-sm-"] {
        for token in &tokens {
            if let Some(rest) = token.strip_prefix(prefix) {
                if let Some(ratio) = parse_ratio(rest) {
                    return ratio;
                }
            }
        }
    }
    for token in &tokens {
        if let Some(rest) = token.strip_prefix("dd-u-") {
            if let Some(ratio) = parse_ratio(rest) {
                return ratio;
            }
        }
    }
    (1, 1)
}

fn card_parent(target: &Target) -> Option<(usize, usize, usize)> {
    match target {
        Target::CardItem {
            node,
            column,
            component,
            ..
        } => Some((*node, *column, *component)),
        _ => None,
    }
}

/// Group outline indexes. A page section owns every following row until the next
/// top-level block, and its columns own the components under them. Card entries
/// pack into rows the way the TUI packs `dd-u-*` fractions (three across at `8/24`).
pub fn outline_blocks(rows: &[OutlineRow]) -> Vec<OutlineBlock> {
    let mut blocks = Vec::new();
    let mut index = 0;
    while index < rows.len() {
        if matches!(rows[index].target, Target::Section(_)) {
            let header = index;
            index += 1;
            let body_end = (index..rows.len())
                .find(|cursor| rows[*cursor].depth == 0)
                .unwrap_or(rows.len());
            let mut columns = Vec::new();
            let mut cursor = index;
            while cursor < body_end {
                if matches!(rows[cursor].target, Target::Column { .. }) {
                    let col_header = cursor;
                    cursor += 1;
                    let start = cursor;
                    while cursor < body_end && !matches!(rows[cursor].target, Target::Column { .. })
                    {
                        cursor += 1;
                    }
                    columns.push(SectionColumnBlock {
                        header: col_header,
                        body: pack_inner(rows, start..cursor),
                    });
                } else {
                    cursor += 1;
                }
            }
            blocks.push(OutlineBlock::Section { header, columns });
            index = body_end;
        } else if rows[index].card_span.is_some() {
            let parent = card_parent(&rows[index].target);
            let start = index;
            index += 1;
            while index < rows.len()
                && rows[index].card_span.is_some()
                && card_parent(&rows[index].target) == parent
            {
                index += 1;
            }
            blocks.push(OutlineBlock::Cards(pack_card_rows(rows, start..index)));
        } else {
            blocks.push(OutlineBlock::Row(index));
            index += 1;
        }
    }
    blocks
}

fn pack_inner(rows: &[OutlineRow], range: std::ops::Range<usize>) -> Vec<OutlineBlock> {
    let mut blocks = Vec::new();
    let mut index = range.start;
    while index < range.end {
        if rows[index].card_span.is_none() {
            blocks.push(OutlineBlock::Row(index));
            index += 1;
            continue;
        }
        let parent = card_parent(&rows[index].target);
        let mut end = index + 1;
        while end < range.end
            && rows[end].card_span.is_some()
            && card_parent(&rows[end].target) == parent
        {
            end += 1;
        }
        blocks.push(OutlineBlock::Cards(pack_card_rows(rows, index..end)));
        index = end;
    }
    blocks
}

fn pack_card_rows(rows: &[OutlineRow], range: std::ops::Range<usize>) -> Vec<Vec<usize>> {
    let mut visual = Vec::new();
    let mut current = Vec::new();
    let mut used_num = 0u64;
    let mut used_den = 1u64;
    for index in range {
        let (parts, whole) = rows[index].card_span.unwrap_or((1, 1));
        let parts = u64::from(parts.max(1));
        let whole = u64::from(whole.max(1));
        if current.is_empty() {
            current.push(index);
            used_num = parts;
            used_den = whole;
            continue;
        }
        let sum_num = used_num * whole + parts * used_den;
        let sum_den = used_den * whole;
        if sum_num * 64 > sum_den * 65 {
            visual.push(std::mem::take(&mut current));
            current.push(index);
            used_num = parts;
            used_den = whole;
        } else {
            current.push(index);
            used_num = sum_num;
            used_den = sum_den;
        }
    }
    if !current.is_empty() {
        visual.push(current);
    }
    visual
}

/// `assets/images/…` resolved under `source/images`, when that file exists.
pub fn blueprint_image_file(site_json: &Path, url: &str) -> Option<PathBuf> {
    let rel = dd_siteforge_core::og_image::source_image_rel(url)?;
    let path = site_json.parent()?.join("source").join("images").join(rel);
    path.is_file().then_some(path)
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
            back: None,
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
        back: None,
    }
}

/// Inspector for a live draft, including a drilled item's back bar.
pub fn inspector_draft(
    state: &EditFormState,
    breadcrumb: String,
    tab: Option<InspTab>,
    back: Option<String>,
) -> InspectorView {
    let mut fields = visible_fields(state);
    let tabs = present_tabs(&fields);
    let active = tab
        .filter(|wanted| tabs.contains(wanted))
        .or(tabs.first().copied());
    if let Some(active) = active {
        fields.retain(|field| field.tab == active);
    }
    let title = if back.is_some() {
        format!("Item · {}", state.form.title)
    } else {
        state.form.title.to_string()
    };
    InspectorView {
        breadcrumb,
        kind: state.form.title.to_string(),
        title,
        tabs,
        fields,
        empty: false,
        back,
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
        id: "components".to_string(),
        label: "Components".to_string(),
        value,
        required: false,
        chrome: FieldChrome::Text,
        error: None,
        meta: None,
        tab: InspTab::Content,
        toolbar: false,
        placeholder: false,
        options: Vec::new(),
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
            let chrome = chrome_of(field);
            let raw = match &field.kind {
                FieldKind::SubForm { .. } => collection_value(state, field),
                _ => state.get(field.id).to_string(),
            };
            let empty =
                raw.trim().is_empty() || raw == "(no items)" || raw.starts_with("(no items");
            let error = if field.required && state.field_value_empty(field) {
                Some(format!("{} is required.", field.label))
            } else {
                None
            };
            let meta = match chrome {
                FieldChrome::Markdown => Some(format!("Markdown · {} chars", raw.chars().count())),
                FieldChrome::Image => Some(if field.id.contains("mp4") {
                    "MP4".to_string()
                } else {
                    "JPG, PNG, WebP, GIF, SVG".to_string()
                }),
                FieldChrome::Collection => {
                    Some("A add · X remove · Shift+J/K move · Enter edit".to_string())
                }
                _ => None,
            };
            let placeholder = match chrome {
                FieldChrome::Collection => false,
                FieldChrome::Image | FieldChrome::Page => empty,
                _ => empty,
            };
            let options = match (&field.kind, chrome) {
                (FieldKind::Enum { options, .. }, FieldChrome::Select) => {
                    options.iter().map(|option| (*option).to_string()).collect()
                }
                _ => Vec::new(),
            };
            let value = if chrome == FieldChrome::Image && empty {
                "Drop an image, or browse".to_string()
            } else if chrome == FieldChrome::Page && empty {
                "Choose a page…".to_string()
            } else if empty && chrome != FieldChrome::Collection {
                "Empty".to_string()
            } else if chrome == FieldChrome::Image {
                file_name(&raw)
            } else {
                raw
            };
            FieldView {
                id: field.id.to_string(),
                label: field.label.to_string(),
                value,
                required: field.required,
                chrome,
                error,
                meta,
                tab,
                toolbar: chrome == FieldChrome::Markdown,
                placeholder,
                options,
            }
        })
        .collect()
}

fn collection_value(
    state: &EditFormState,
    field: &dd_siteforge_core::editform::FormField,
) -> String {
    let summary_id = match &field.kind {
        FieldKind::SubForm {
            summary_field_id, ..
        } => *summary_field_id,
        _ => "",
    };
    let items = state
        .sub_state
        .get(field.id)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if items.is_empty() {
        return "(no items; press A to add)".to_string();
    }
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let summary = item.get(summary_id).trim();
            let summary = if summary.is_empty() {
                "(untitled)"
            } else {
                summary
            };
            format!("{}. {summary}", index + 1)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn file_name(stored: &str) -> String {
    stored
        .rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(stored)
        .to_string()
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

fn chrome_of(field: &dd_siteforge_core::editform::FormField) -> FieldChrome {
    match controls::control_for(field) {
        FieldControl::Markdown => FieldChrome::Markdown,
        FieldControl::Collection(_) => FieldChrome::Collection,
        FieldControl::Toggle => FieldChrome::Toggle,
        FieldControl::Image { .. } => FieldChrome::Image,
        FieldControl::PagePicker => FieldChrome::Page,
        FieldControl::Width => FieldChrome::Width,
        FieldControl::Select | FieldControl::Position | FieldControl::Media => FieldChrome::Select,
        FieldControl::Text | FieldControl::None => FieldChrome::Text,
    }
}

fn issue_count(state: &EditFormState) -> usize {
    state
        .form
        .fields
        .iter()
        .filter(|field| {
            state.field_visible(field) && field.required && state.field_value_empty(field)
        })
        .count()
}

pub fn form_state(
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
        Target::CardItem {
            node,
            column,
            component,
            item,
        } => {
            let column = page_column(site, page_index, *node, *column)?;
            component_to_form_state(column.components.get(*component)?)?
                .sub_state
                .get("items")?
                .get(*item)
                .cloned()
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
        SideTarget::Page(index) => {
            let page = site.pages.get(*index);
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
                Target::CardItem {
                    node,
                    column,
                    component,
                    item,
                } => {
                    let id = section_at(site, page_index, *node)
                        .map(|section| section.id.as_str())
                        .unwrap_or("section");
                    let title = page_column(site, page_index, *node, *column)
                        .and_then(|column| column.components.get(*component))
                        .and_then(|component| match component {
                            SectionComponent::Card(card) => card.items.get(*item),
                            _ => None,
                        })
                        .map(|entry| short_title(&entry.child_title))
                        .unwrap_or_else(|| "(untitled)".to_string());
                    format!("{slug} › {id} › dd-card › {}: {title}", item + 1)
                }
                _ => slug.to_string(),
            }
        }
    }
}

pub fn title_crumb(site: &Site, region: &SideTarget, _page_index: usize) -> String {
    match region {
        SideTarget::Site => "/ Site settings".to_string(),
        SideTarget::Header => "/ Header".to_string(),
        SideTarget::Footer => "/ Footer".to_string(),
        SideTarget::Page(index) => {
            let slug = site
                .pages
                .get(*index)
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

pub fn block_count(site: &Site, region: &SideTarget, _page_index: usize) -> usize {
    match region {
        SideTarget::Page(index) => site
            .pages
            .get(*index)
            .map(|page| page.nodes.len())
            .unwrap_or(0),
        SideTarget::Header => site.header.sections.len(),
        SideTarget::Footer => site.footer.sections.len(),
        SideTarget::Site => 0,
    }
}

pub fn status_meta(site: &Site, region: &SideTarget, page_index: usize) -> String {
    match region {
        SideTarget::Page(index) => {
            let page = site.pages.get(*index);
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
        SideTarget::Page(index) => {
            let page = site.pages.get(*index);
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
        SideTarget::Page(_) => Target::PageHead,
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
