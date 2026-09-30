//! Per-page authoring health: SEO fields, alt text, headings, internal links.
use crate::model::{Media, Page, PageNode, SectionComponent, Site};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthItem {
    pub ok: bool,
    pub message: String,
}

pub fn page_health(site: &Site, page_idx: usize) -> Vec<HealthItem> {
    let Some(page) = site.pages.get(page_idx) else {
        return vec![HealthItem {
            ok: false,
            message: "No page selected.".into(),
        }];
    };
    let mut items = Vec::new();
    check_head(page, &mut items);
    let slugs: Vec<String> = site.pages.iter().map(|p| p.slug.clone()).collect();
    if let Ok(value) = serde_json::to_value(page) {
        walk_value(&value, &slugs, &mut items);
    }
    check_headings(page, &mut items);
    if items.iter().all(|i| i.ok) && items.len() < 3 {
        items.push(HealthItem {
            ok: true,
            message: "No issues on this page.".into(),
        });
    }
    items
}

fn check_head(page: &Page, items: &mut Vec<HealthItem>) {
    let meta_title = page
        .head
        .meta_title
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if meta_title.is_none() {
        items.push(HealthItem {
            ok: false,
            message: "Meta title is empty (HTML title falls back to the page label).".into(),
        });
    } else {
        items.push(HealthItem {
            ok: true,
            message: format!("Meta title: {}", meta_title.unwrap()),
        });
    }
    let desc = page
        .head
        .meta_description
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if desc.is_none() {
        items.push(HealthItem {
            ok: false,
            message: "Meta description is empty.".into(),
        });
    } else {
        items.push(HealthItem {
            ok: true,
            message: "Meta description is set.".into(),
        });
    }
}

fn check_headings(page: &Page, items: &mut Vec<HealthItem>) {
    let mut outline: Vec<(u8, String)> = Vec::new();
    for node in &page.nodes {
        if let PageNode::Hero(hero) = node {
            let title = hero.parent_title.trim();
            if !title.is_empty() {
                outline.push((1, title.to_string()));
            }
            if let Some(copy) = hero.parent_copy.as_deref() {
                collect_markdown_headings(copy, &mut outline);
            }
        }
        if let PageNode::Section(section) = node {
            for col in &section.columns {
                for comp in &col.components {
                    collect_component_headings(comp, &mut outline);
                }
            }
        }
    }
    if !outline.iter().any(|(l, _)| *l == 1) {
        items.push(HealthItem {
            ok: false,
            message: "No H1 (hero title or markdown # heading).".into(),
        });
    }
    let mut last = 0u8;
    for (level, text) in &outline {
        if last > 0 && *level > last + 1 {
            items.push(HealthItem {
                ok: false,
                message: format!("Heading jumps to H{level} ({text})"),
            });
            break;
        }
        last = *level;
    }
    if outline.len() > 1 {
        let summary = outline
            .iter()
            .take(8)
            .map(|(l, t)| format!("H{l} {t}"))
            .collect::<Vec<_>>()
            .join(" · ");
        items.push(HealthItem {
            ok: true,
            message: format!("Outline: {summary}"),
        });
    }
}

fn collect_markdown_headings(copy: &str, outline: &mut Vec<(u8, String)>) {
    for line in copy.lines() {
        let t = line.trim();
        let n = t.chars().take_while(|c| *c == '#').count();
        if (1..=6).contains(&n) {
            let rest = t.get(n..).unwrap_or("").trim_start();
            if rest.starts_with(' ') || t.chars().nth(n) == Some(' ') {
                let title = rest.trim().to_string();
                if !title.is_empty() {
                    outline.push((n as u8, title));
                }
            }
        }
    }
}

fn collect_component_headings(comp: &SectionComponent, outline: &mut Vec<(u8, String)>) {
    if let Ok(value) = serde_json::to_value(comp) {
        walk_strings(&value, &mut |s| collect_markdown_headings(s, outline));
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

fn walk_value(value: &Value, slugs: &[String], items: &mut Vec<HealthItem>) {
    match value {
        Value::Array(arr) => {
            for v in arr {
                walk_value(v, slugs, items);
            }
        }
        Value::Object(map) => {
            let kind = map.get("kind").and_then(Value::as_str).unwrap_or("");
            if kind == "image" {
                let url = map.get("url").and_then(Value::as_str).unwrap_or("").trim();
                let alt = map.get("alt").and_then(Value::as_str).unwrap_or("").trim();
                if !url.is_empty() && alt.is_empty() {
                    items.push(HealthItem {
                        ok: false,
                        message: format!("Image missing alt text ({url})"),
                    });
                }
            }
            pair_alt(map, "parent_image_url", "parent_image_alt", items);
            pair_alt(map, "child_image_url", "child_image_alt", items);
            if let Some(url) = map.get("url").and_then(Value::as_str) {
                check_internal_link(url, slugs, items);
            }
            if let Some(url) = map.get("cta_url").and_then(Value::as_str) {
                check_internal_link(url, slugs, items);
            }
            if let Some(url) = map.get("parent_link_url").and_then(Value::as_str) {
                check_internal_link(url, slugs, items);
            }
            if let Some(url) = map.get("child_link_url").and_then(Value::as_str) {
                check_internal_link(url, slugs, items);
            }
            for v in map.values() {
                walk_value(v, slugs, items);
            }
        }
        _ => {}
    }
}

fn pair_alt(
    map: &serde_json::Map<String, Value>,
    url_key: &str,
    alt_key: &str,
    items: &mut Vec<HealthItem>,
) {
    let url = map
        .get(url_key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let alt = map
        .get(alt_key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if !url.is_empty() && alt.is_empty() {
        items.push(HealthItem {
            ok: false,
            message: format!("Image missing alt text ({url})"),
        });
    }
}

fn check_internal_link(url: &str, slugs: &[String], items: &mut Vec<HealthItem>) {
    let url = url.trim();
    if !url.starts_with('/') || url.starts_with("//") {
        return;
    }
    let path = url
        .split('#')
        .next()
        .unwrap_or(url)
        .split('?')
        .next()
        .unwrap_or(url);
    let mut slug = path.trim_start_matches('/').trim_end_matches('/');
    slug = slug.strip_suffix(".html").unwrap_or(slug);
    slug = slug.trim_end_matches('/');
    if slug == "index" {
        slug = "";
    } else if let Some(stripped) = slug.strip_suffix("/index") {
        slug = stripped;
    }
    if slug.is_empty() {
        return;
    }
    if !slugs.iter().any(|s| s == slug) {
        items.push(HealthItem {
            ok: false,
            message: format!("Internal link /{slug} does not match a page slug."),
        });
    }
}

#[allow(dead_code)]
pub fn media_missing_alt(media: &Media) -> bool {
    matches!(media, Media::Image { url, alt } if !url.trim().is_empty() && alt.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Site;

    #[test]
    fn starter_home_reports_meta_gaps() {
        let site = Site::starter();
        let items = page_health(&site, 0);
        assert!(
            items
                .iter()
                .any(|i| !i.ok && i.message.contains("Meta title")),
            "{items:?}"
        );
    }

    #[test]
    fn broken_internal_link_is_flagged() {
        let mut site = Site::starter();
        site.pages[0].head.meta_title = Some("Home".into());
        site.pages[0].head.meta_description = Some("Hello".into());
        if let PageNode::Hero(hero) = &mut site.pages[0].nodes[0] {
            hero.links[0].url = "/missing-page".into();
        }
        let items = page_health(&site, 0);
        assert!(
            items.iter().any(|i| i.message.contains("missing-page")),
            "{items:?}"
        );
    }
}
