//! Flattened per-page search documents written to `search-index.json` on export.
use std::fs;
use std::path::Path;

use anyhow::Context;
use serde::Serialize;

use crate::model::{
    DdAccordion, DdAlert, DdAlternating, DdBanner, DdBlockquote, DdCard, DdCta, DdDataTable,
    DdFilmstrip, DdHero, DdImage, DdMilestones, DdModal, DdNavigation, DdRichText, DdSection,
    DdSlider, DdTabs, DdTimeline, Media, NavigationItem, Page, PageNode, RobotsDirective,
    SectionComponent, Site, page_href, page_public_path,
};

const INDEX_VERSION: u32 = 1;
const BODY_MAX_CHARS: usize = 12_000;
const DESCRIPTION_MAX_CHARS: usize = 160;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SearchIndex {
    pub v: u32,
    pub pages: Vec<SearchPage>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SearchPage {
    pub url: String,
    pub path: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub headings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

pub fn write_search_index(site: &Site, output_dir: &Path) -> anyhow::Result<()> {
    let index = build_search_index(site);
    let json = serde_json::to_string_pretty(&index).context("failed to serialize search-index")?;
    fs::write(output_dir.join("search-index.json"), json + "\n")
        .context("failed to write search-index.json")
}

pub fn build_search_index(site: &Site) -> SearchIndex {
    let pages = site
        .pages
        .iter()
        .filter(|page| is_indexed(page))
        .map(|page| index_page(page, site.pretty_urls))
        .collect();
    SearchIndex {
        v: INDEX_VERSION,
        pages,
    }
}

fn is_indexed(page: &Page) -> bool {
    !matches!(
        page.head.robots,
        RobotsDirective::NoindexFollow | RobotsDirective::NoindexNofollow
    )
}

fn index_page(page: &Page, pretty: bool) -> SearchPage {
    let mut parts: Vec<String> = Vec::new();
    let mut headings: Vec<String> = Vec::new();
    push_text(&mut parts, &page.head.title);
    for node in &page.nodes {
        match node {
            PageNode::Hero(hero) => collect_hero(hero, &mut parts, &mut headings),
            PageNode::Section(section) => collect_section(section, &mut parts, &mut headings),
        }
    }
    let body = truncate_chars(&collapse_ws(&parts.join(" ")), BODY_MAX_CHARS);
    let description = page_description(&page.head, &body);
    let image = page
        .head
        .og_image
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    SearchPage {
        url: page_href(&page.slug, pretty),
        path: page_public_path(&page.slug, pretty),
        title: page.head.title.trim().to_string(),
        description,
        body,
        headings,
        image,
    }
}

fn page_description(head: &crate::model::DdHead, body: &str) -> String {
    first_nonempty(&[
        head.meta_description.as_deref(),
        head.og_description.as_deref(),
    ])
    .unwrap_or_else(|| truncate_chars(body, DESCRIPTION_MAX_CHARS))
}

fn first_nonempty(candidates: &[Option<&str>]) -> Option<String> {
    candidates
        .iter()
        .filter_map(|c| c.map(str::trim).filter(|s| !s.is_empty()))
        .next()
        .map(str::to_string)
}

fn collect_hero(hero: &DdHero, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    push_heading(headings, parts, &hero.parent_title);
    push_text(parts, &hero.parent_subtitle);
    if let Some(copy) = hero.parent_copy.as_deref() {
        push_markdown(parts, copy);
    }
    for link in hero.resolved_links() {
        push_text(parts, &link.label);
    }
    push_media(parts, &hero.resolved_media());
}

fn collect_section(section: &DdSection, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    if let Some(title) = section.section_title.as_deref() {
        push_heading(headings, parts, title);
    }
    for column in &section.columns {
        for component in &column.components {
            collect_component(component, parts, headings);
        }
    }
}

fn collect_component(
    component: &SectionComponent,
    parts: &mut Vec<String>,
    headings: &mut Vec<String>,
) {
    match component {
        SectionComponent::Alternating(v) => collect_alternating(v, parts, headings),
        SectionComponent::Card(v) => collect_card(v, parts, headings),
        SectionComponent::Cta(v) => collect_cta(v, parts, headings),
        SectionComponent::Filmstrip(v) => collect_filmstrip(v, parts, headings),
        SectionComponent::Milestones(v) => collect_milestones(v, parts, headings),
        SectionComponent::Slider(v) => collect_slider(v, parts, headings),
        SectionComponent::Modal(v) => collect_modal(v, parts, headings),
        SectionComponent::Banner(v) => collect_banner(v, parts),
        SectionComponent::Accordion(v) => collect_accordion(v, parts, headings),
        SectionComponent::Blockquote(v) => collect_blockquote(v, parts),
        SectionComponent::Alert(v) => collect_alert(v, parts, headings),
        SectionComponent::Image(v) => collect_image(v, parts),
        SectionComponent::RichText(v) => collect_rich_text(v, parts),
        SectionComponent::Navigation(v) => collect_navigation(v, parts),
        SectionComponent::Tabs(v) => collect_tabs(v, parts, headings),
        SectionComponent::Timeline(v) => collect_timeline(v, parts, headings),
        SectionComponent::DataTable(v) => collect_data_table(v, parts, headings),
        SectionComponent::HeaderSearch(_)
        | SectionComponent::HeaderMenu(_)
        | SectionComponent::Spacer(_)
        | SectionComponent::SearchResults(_) => {}
        SectionComponent::Headline(v) => {
            push_heading(headings, parts, &v.text);
        }
    }
}

fn collect_alternating(v: &DdAlternating, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    for item in &v.items {
        push_heading(headings, parts, &item.child_title);
        push_text(parts, &item.child_subtitle);
        push_markdown(parts, &item.child_copy);
        for link in &item.links {
            push_text(parts, &link.label);
        }
        push_media(parts, &item.resolved_media());
    }
}

fn collect_card(v: &DdCard, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    for item in &v.items {
        push_heading(headings, parts, &item.child_title);
        push_text(parts, &item.child_subtitle);
        push_markdown(parts, &item.child_copy);
        if let Some(label) = item.child_link_label.as_deref() {
            push_text(parts, label);
        }
        push_text(parts, &item.child_image_alt);
    }
}

fn collect_cta(v: &DdCta, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    push_heading(headings, parts, &v.parent_title);
    push_text(parts, &v.parent_subtitle);
    push_markdown(parts, &v.parent_copy);
    for link in v.resolved_links() {
        push_text(parts, &link.label);
    }
    push_text(parts, &v.parent_image_alt);
}

fn collect_filmstrip(v: &DdFilmstrip, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    for item in &v.items {
        push_heading(headings, parts, &item.child_title);
        push_text(parts, &item.child_image_alt);
    }
}

fn collect_milestones(v: &DdMilestones, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    for item in &v.items {
        push_heading(headings, parts, &item.child_title);
        push_text(parts, &item.child_percentage);
        push_text(parts, &item.child_subtitle);
        push_markdown(parts, &item.child_copy);
        if let Some(label) = item.child_link_label.as_deref() {
            push_text(parts, label);
        }
    }
}

fn collect_slider(v: &DdSlider, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    push_heading(headings, parts, &v.parent_title);
    for item in &v.items {
        push_heading(headings, parts, &item.child_title);
        push_markdown(parts, &item.child_copy);
        for link in item.resolved_links() {
            push_text(parts, &link.label);
        }
        push_media(parts, &item.resolved_media());
    }
}

fn collect_modal(v: &DdModal, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    push_heading(headings, parts, &v.parent_title);
    push_markdown(parts, &v.parent_copy);
}

fn collect_banner(v: &DdBanner, parts: &mut Vec<String>) {
    push_media(parts, &v.resolved_media());
}

fn collect_accordion(v: &DdAccordion, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    for item in &v.items {
        push_heading(headings, parts, &item.child_title);
        push_markdown(parts, &item.child_copy);
    }
}

fn collect_blockquote(v: &DdBlockquote, parts: &mut Vec<String>) {
    push_text(parts, &v.parent_name);
    push_text(parts, &v.parent_role);
    push_markdown(parts, &v.parent_copy);
    push_text(parts, &v.parent_image_alt);
}

fn collect_alert(v: &DdAlert, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    if let Some(title) = v.parent_title.as_deref() {
        push_heading(headings, parts, title);
    }
    push_markdown(parts, &v.parent_copy);
}

fn collect_image(v: &DdImage, parts: &mut Vec<String>) {
    push_text(parts, &v.parent_image_alt);
}

fn collect_rich_text(v: &DdRichText, parts: &mut Vec<String>) {
    push_markdown(parts, &v.parent_copy);
}

fn collect_navigation(v: &DdNavigation, parts: &mut Vec<String>) {
    collect_nav_items(&v.items, parts);
}

fn collect_nav_items(items: &[NavigationItem], parts: &mut Vec<String>) {
    for item in items {
        push_text(parts, &item.child_link_label);
        collect_nav_items(&item.items, parts);
    }
}

fn collect_tabs(v: &DdTabs, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    for item in &v.items {
        push_heading(headings, parts, &item.child_title);
        push_markdown(parts, &item.child_copy);
    }
}

fn collect_timeline(v: &DdTimeline, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    for item in &v.items {
        push_text(parts, &item.child_year);
        push_heading(headings, parts, &item.child_title);
        push_markdown(parts, &item.child_copy);
        if let Some(alt) = item.child_image_alt.as_deref() {
            push_text(parts, alt);
        }
    }
}

fn collect_data_table(v: &DdDataTable, parts: &mut Vec<String>, headings: &mut Vec<String>) {
    push_heading(headings, parts, &v.caption);
    for col in &v.columns {
        push_text(parts, &col.label);
    }
    for row in &v.rows {
        for cell in &row.cells {
            push_text(parts, &cell.text);
        }
    }
    if let Some(msg) = v.empty_message.as_deref() {
        push_text(parts, msg);
    }
}

fn push_heading(headings: &mut Vec<String>, parts: &mut Vec<String>, raw: &str) {
    let t = collapse_ws(raw);
    if t.is_empty() {
        return;
    }
    if !headings.iter().any(|h| h.eq_ignore_ascii_case(&t)) {
        headings.push(t.clone());
    }
    parts.push(t);
}

fn push_text(parts: &mut Vec<String>, raw: &str) {
    let t = collapse_ws(raw);
    if !t.is_empty() {
        parts.push(t);
    }
}

fn push_markdown(parts: &mut Vec<String>, raw: &str) {
    push_text(parts, &markdown_to_plain(raw));
}

fn push_media(parts: &mut Vec<String>, media: &Media) {
    match media {
        Media::None | Media::Oembed { .. } => {}
        Media::Image { alt, .. } => push_text(parts, alt),
        Media::LocalVideo { name, .. } => push_text(parts, name),
    }
}

fn markdown_to_plain(input: &str) -> String {
    crate::markdown::to_plain(input)
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut t: String = s.chars().take(max).collect();
    if let Some(i) = t.rfind(' ') {
        t.truncate(i);
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        DdHead, DdHeadline, DdRichText, HeadingLevel, PageNode, PageTemplate, RobotsDirective,
        SalAnimation, SchemaType, SectionComponent, Site,
    };

    #[test]
    fn starter_index_has_hero_and_section_copy() {
        let site = Site::starter();
        let index = build_search_index(&site);
        assert_eq!(index.v, 1);
        assert_eq!(index.pages.len(), 1);
        let page = &index.pages[0];
        assert_eq!(page.url, "index.html");
        assert_eq!(page.path, "/");
        assert_eq!(page.title, "Home");
        assert_eq!(page.description, "Starter page");
        assert!(
            page.headings
                .contains(&"Build with dd-framework".to_string())
        );
        assert!(page.headings.contains(&"Ready to publish?".to_string()));
        assert!(
            page.body
                .contains("Compose pages with typed component schemas.")
        );
        assert!(page.body.contains("Get Started"));
        assert!(page.image.is_none());
    }

    #[test]
    fn headline_text_is_indexed_as_a_heading() {
        let mut site = Site::starter();
        let PageNode::Section(section) = &mut site.pages[0].nodes[1] else {
            panic!("starter node 1 expected to be a section");
        };
        section.columns[0]
            .components
            .push(SectionComponent::Headline(DdHeadline {
                text: "Five seconds".to_string(),
                heading_level: HeadingLevel::H2,
                custom_css: None,
                sal: SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
            }));
        let index = build_search_index(&site);
        let page = &index.pages[0];
        assert!(page.headings.iter().any(|h| h == "Five seconds"));
        assert!(page.body.contains("Five seconds"));
    }

    #[test]
    fn pretty_urls_use_directory_href_and_path() {
        let mut site = Site::starter();
        site.pretty_urls = true;
        site.pages[0].slug = "services".to_string();
        site.pages[0].head.title = "Services".to_string();
        let page = &build_search_index(&site).pages[0];
        assert_eq!(page.url, "services/index.html");
        assert_eq!(page.path, "/services/");
    }

    #[test]
    fn noindex_pages_are_omitted() {
        let mut site = Site::starter();
        site.pages
            .push(Page::from_template("Hidden", PageTemplate::Blank));
        site.pages[1].head.robots = RobotsDirective::NoindexNofollow;
        let index = build_search_index(&site);
        assert_eq!(index.pages.len(), 1);
        assert_eq!(index.pages[0].title, "Home");
    }

    #[test]
    fn markdown_is_stripped_and_fills_description() {
        let mut site = Site::starter();
        site.pages[0].head.meta_description = None;
        site.pages[0].head.og_description = None;
        site.pages[0].nodes = vec![PageNode::Section(crate::model::DdSection {
            id: "section-1".to_string(),
            section_title: None,
            section_class: None,
            item_box_class: None,
            bg: None,
            padding: None,
            custom_css: None,
            aria_label: None,
            sal: SalAnimation::NoAnimation,
            sal_duration: None,
            sal_delay: None,
            columns: vec![crate::model::SectionColumn {
                id: "section-1-column-1".to_string(),
                width_class: "dd-u-1-1".to_string(),
                components: vec![SectionComponent::RichText(DdRichText {
                    parent_class: None,
                    sal: SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_copy: "**Hello** and [there](https://ex.com).".to_string(),
                })],
            }],
        })];
        let page = &build_search_index(&site).pages[0];
        assert_eq!(page.body, "Home Hello and there.");
        assert_eq!(page.description, "Home Hello and there.");
        assert!(!page.body.contains("**"));
        assert!(!page.body.contains("https://ex.com"));
    }

    #[test]
    fn description_falls_back_to_og_then_body() {
        let mut head = DdHead {
            title: "T".into(),
            meta_title: None,
            meta_description: None,
            canonical_url: None,
            robots: RobotsDirective::IndexFollow,
            schema_type: SchemaType::WebPage,
            og_title: None,
            og_description: Some("  OG copy  ".into()),
            og_image: Some("assets/images/og.jpg".into()),
        };
        assert_eq!(page_description(&head, "body text"), "OG copy");
        head.og_description = None;
        assert_eq!(page_description(&head, "body text here"), "body text here");
    }

    #[test]
    fn markdown_to_plain_keeps_code_and_lists() {
        let plain = markdown_to_plain("- one\n- two\n\n`code`");
        assert!(plain.contains("one"));
        assert!(plain.contains("two"));
        assert!(plain.contains("code"));
    }

    #[test]
    fn header_and_footer_copy_are_omitted() {
        let mut site = Site::starter();
        site.header.banner = Some("Header-only banner copy".into());
        site.footer.blurb = Some("Footer-only blurb copy".into());
        let page = &build_search_index(&site).pages[0];
        assert!(!page.body.contains("Header-only banner copy"));
        assert!(!page.body.contains("Footer-only blurb copy"));
    }

    #[test]
    fn og_image_is_copied_when_present() {
        let mut site = Site::starter();
        site.pages[0].head.og_image = Some("assets/images/og.jpg".into());
        let page = &build_search_index(&site).pages[0];
        assert_eq!(page.image.as_deref(), Some("assets/images/og.jpg"));
    }
}
