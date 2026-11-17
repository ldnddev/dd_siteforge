//! Find in site. Same hits as the TUI (`src/tui/find.rs`): one flat list,
//! scored, at most 40, across every page.

use dd_siteforge_core::component_kind::ComponentKind;
use dd_siteforge_core::model::{DdSection, PageNode, Site};
use serde_json::Value;

use crate::commands::fuzzy_score;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindHit {
    pub score: i32,
    pub label: String,
    pub target: FindTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindTarget {
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

pub fn find_hits(site: &Site, query: &str) -> Vec<FindHit> {
    let mut hits = Vec::new();
    let q = query.trim();
    consider(&site.name, "Site", q, FindTarget::SiteRoot, &mut hits);
    consider("Header", "Header", q, FindTarget::HeaderRoot, &mut hits);
    if let Some(alert) = &site.header.alert {
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
        &site.header.sections,
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
        &site.footer.sections,
        "Footer",
        q,
        |section, column, component| FindTarget::FooterComponent {
            section,
            column,
            component,
        },
        &mut hits,
    );
    for (page_idx, page) in site.pages.iter().enumerate() {
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
                    collect_json_strings(hero, &format!("{title} / dd-hero"), q, target, &mut hits);
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
        walk_strings(&json, &mut |text| {
            if text.chars().count() < 2 {
                return;
            }
            consider(text, ctx, query, target.clone(), hits);
        });
    }
}

fn collect_section_components(
    sections: &[DdSection],
    ctx: &str,
    query: &str,
    target: impl Fn(usize, usize, usize) -> FindTarget,
    hits: &mut Vec<FindHit>,
) {
    for (section_idx, section) in sections.iter().enumerate() {
        for (column_idx, col) in section.columns.iter().enumerate() {
            for (component_idx, comp) in col.components.iter().enumerate() {
                let kind = ComponentKind::from_section_component(comp);
                let found = target(section_idx, column_idx, component_idx);
                collect_json_strings(
                    comp,
                    &format!("{ctx} / {}", kind.label()),
                    query,
                    found,
                    hits,
                );
            }
        }
    }
}

fn walk_strings(value: &Value, f: &mut impl FnMut(&str)) {
    match value {
        Value::String(text) => f(text),
        Value::Array(arr) => {
            for item in arr {
                walk_strings(item, f);
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                walk_strings(item, f);
            }
        }
        _ => {}
    }
}

fn truncate_hit(text: &str, n: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= n {
        text.to_string()
    } else {
        let mut out: String = chars[..n].iter().collect();
        out.push('…');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dd_siteforge_core::model::{Page, PageTemplate};

    #[test]
    fn find_reaches_a_later_page() {
        let mut site = Site::starter();
        let mut about = Page::from_template("About the orchard", PageTemplate::Blank);
        about.head.meta_description = Some("A second page about cider apples.".to_string());
        site.pages.push(about);
        let hits = find_hits(&site, "orchard");
        assert!(
            hits.iter().any(|hit| {
                matches!(hit.target, FindTarget::PageHead { page: 1 })
                    && hit.label.contains("About the orchard")
            }),
            "{hits:?}"
        );
        let home = find_hits(&site, "Home");
        assert!(
            home.iter()
                .any(|hit| matches!(hit.target, FindTarget::PageHead { page: 0 })),
            "the first page stays searchable: {home:?}"
        );
        let all = find_hits(&site, "");
        assert!(
            all.iter()
                .any(|hit| matches!(hit.target, FindTarget::PageHead { page: 1 })),
            "an empty query lists every page"
        );
        assert!(all.len() <= 40);
    }
}
