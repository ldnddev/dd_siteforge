use std::fs;
use std::path::Path;

use anyhow::Context;
use serde_json::{Value, json};

use crate::model::{
    ButtonStyle, DATA_TABLE_MAX_COLUMNS, DataTableAlign, DataTableBadge, DataTableCellType,
    DdAccordion, DdAlert, DdAlternating, DdBanner, DdBlockquote, DdCard, DdCta, DdDataTable,
    DdFilmstrip, DdFooter, DdHead, DdHeader, DdHeadline, DdHero, DdLink, DdMilestones, DdModal,
    DdSearchResults, DdSection, DdSlider, DdSpacer, DdTabs, DdTimeline, Media, OembedProvider,
    Page, PageNode, SectionComponent, Site, html_id_from_text, parse_oembed_url, uniquify_id,
};
use crate::templates::Renderer;

pub fn render_site_to_dir(
    site: &Site,
    output_dir: &Path,
    site_root: Option<&Path>,
) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir).context("failed to create export directory")?;
    let r = Renderer::load(site_root)?;
    let header_html = render_header(&r, &site.header)?;
    let footer_html = render_footer(&r, &site.footer, &site.name)?;
    for page in &site.pages {
        let html = render_page_html_with_chrome(&r, page, &header_html, &footer_html, site)?;
        let pretty = page.slug != "404" && site.pretty_urls;
        let file_name = crate::model::page_file_name(&page.slug, pretty);
        let out_path = output_dir.join(&file_name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create page directory '{}'", parent.display())
            })?;
        }
        fs::write(&out_path, html)
            .with_context(|| format!("failed to write page output '{}'", out_path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
fn render_page_html(page: &Page) -> anyhow::Result<String> {
    let site = Site::starter();
    let r = Renderer::bundled_only()?;
    render_page_html_with_chrome(&r, page, "", "", &site)
}

pub fn render_page_html_with_chrome(
    r: &Renderer,
    page: &Page,
    header_html: &str,
    footer_html: &str,
    site: &Site,
) -> anyhow::Result<String> {
    let mut used_ids = reserved_html_ids(site, page);
    let mut content = String::new();
    for node in &page.nodes {
        match node {
            PageNode::Hero(hero) => content.push_str(&render_hero(r, hero)?),
            PageNode::Section(section) => {
                content.push_str(&render_section(r, section, &mut used_ids)?)
            }
        }
        content.push('\n');
    }

    let head_html = render_head(r, &page.head, site, page)?;
    let lang = if site.lang.trim().is_empty() {
        "en"
    } else {
        site.lang.trim()
    };
    let body_slug = page.slug.replace('/', "-");
    let html = r.render(
        "_page",
        &json!({
            "lang": lang,
            "head_html": head_html,
            "header_html": header_html,
            "footer_html": footer_html,
            "content": content,
            "body_class": format!("page page-{body_slug}"),
            "body_gtm": gtm_body_html(site.body_gtm_tag.as_deref()),
        }),
    )?;
    let prefix =
        crate::model::url_prefix_for_page(&page.slug, site.pretty_urls, site.base_url.as_deref());
    Ok(prefix_relative_urls(&html, &prefix))
}

fn render_head(r: &Renderer, head: &DdHead, site: &Site, page: &Page) -> anyhow::Result<String> {
    let robots = robots_token(head.robots);
    let schema_type = schema_type_token(head.schema_type);
    let mut schema = serde_json::Map::new();
    schema.insert(
        "@context".to_string(),
        Value::String("https://schema.org".to_string()),
    );
    schema.insert("@type".to_string(), Value::String(schema_type.to_string()));
    let html_title = head.html_title().to_string();
    schema.insert("name".to_string(), Value::String(html_title.clone()));
    if let Some(d) = head
        .meta_description
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        schema.insert("description".to_string(), Value::String(d.to_string()));
    }
    let canonical = crate::model::resolve_canonical_url(
        site.base_url.as_deref(),
        head.canonical_url.as_deref(),
        &page.slug,
        site.pretty_urls,
    );
    if let Some(u) = canonical.as_deref() {
        schema.insert("url".to_string(), Value::String(u.to_string()));
    }
    let og_image = head
        .og_image
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| resolve_head_asset_url(v, site, page));
    if let Some(i) = og_image.as_deref() {
        schema.insert("image".to_string(), Value::String(i.to_string()));
    }
    let schema_json =
        serde_json::to_string_pretty(&Value::Object(schema)).unwrap_or_else(|_| "{}".to_string());

    // Empty OG title uses meta title, then the page title (`html_title`).
    // Empty OG description uses meta description.
    let og_title = head
        .og_title
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .or_else(|| Some(html_title.clone()));
    let og_description = head
        .og_description
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .or_else(|| {
            head.meta_description
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
        });
    let og_url = canonical.clone();
    let twitter_card = og_image.as_ref().map(|_| "summary_large_image".to_string());

    let data = json!({
        "title": html_title,
        "meta_description": head.meta_description,
        "canonical_url": canonical,
        "robots": robots,
        "og_title": og_title,
        "og_description": og_description,
        "og_image": og_image,
        "og_url": og_url,
        "twitter_card": twitter_card,
        "schema_json": schema_json,
        "header_gtm": gtm_header_html(site.header_gtm_tag.as_deref()),
    });
    r.render("_head", &data)
}

pub(crate) fn render_header(r: &Renderer, header: &DdHeader) -> anyhow::Result<String> {
    let custom = header
        .custom_css
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| format!(" {}", v))
        .unwrap_or_default();
    let alert_html = if let Some(alert) = &header.alert {
        render_alert(r, alert)?
    } else {
        String::new()
    };
    let mut used_ids = std::collections::HashSet::new();
    for section in &header.sections {
        reserve_section_ids(section, &mut used_ids);
    }
    let mut sections_html = String::new();
    for section in &header.sections {
        sections_html.push_str(&render_section(r, section, &mut used_ids)?);
        sections_html.push('\n');
    }
    let cta_url = header
        .cta_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let cta_label = header
        .cta_label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Contact");
    let banner = header
        .banner
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    r.render(
        "dd-header",
        &json!({
            "custom": custom,
            "banner": banner,
            "alert_html": alert_html,
            "sections_html": sections_html,
            "cta_url": cta_url,
            "cta_label": cta_label,
        }),
    )
}

pub(crate) fn render_footer(
    r: &Renderer,
    footer: &DdFooter,
    site_name: &str,
) -> anyhow::Result<String> {
    let custom = footer
        .custom_css
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| format!(" {}", v))
        .unwrap_or_default();
    let mut used_ids = std::collections::HashSet::new();
    for section in &footer.sections {
        reserve_section_ids(section, &mut used_ids);
    }
    let mut sections_html = String::new();
    for section in &footer.sections {
        sections_html.push_str(&render_section(r, section, &mut used_ids)?);
        sections_html.push('\n');
    }
    let blurb = footer
        .blurb
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| html_escape_attr(s).replace('\n', "<br>"));
    let copyright = footer
        .copyright
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            let name = site_name.trim();
            if name.is_empty() {
                format!("© {}", copyright_year())
            } else {
                format!("© {} {}", copyright_year(), name)
            }
        });
    let mut socials = Vec::new();
    for (url, label, icon) in [
        (footer.social_linkedin.as_deref(), "LinkedIn", "fa-linkedin"),
        (footer.social_x.as_deref(), "X", "fa-x-twitter"),
        (footer.social_github.as_deref(), "GitHub", "fa-github"),
    ] {
        if let Some(url) = url.map(str::trim).filter(|s| !s.is_empty()) {
            socials.push(json!({
                "url": url,
                "label": label,
                "icon": icon,
            }));
        }
    }
    r.render(
        "dd-footer",
        &json!({
            "custom": custom,
            "blurb": blurb,
            "sections_html": sections_html,
            "has_socials": !socials.is_empty(),
            "socials": socials,
            "copyright": copyright,
        }),
    )
}

fn copyright_year() -> i32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| (d.as_secs() / 86400) as i64)
        .unwrap_or(0);
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    (yoe as i64 + era * 400) as i32
}

fn gtm_header_html(snippet: Option<&str>) -> Option<String> {
    let id = crate::model::extract_gtm_id(snippet.unwrap_or(""))?;
    Some(format!(
        r#"<script>(function(w,d,s,l,i){{w[l]=w[l]||[];w[l].push({{'gtm.start':new Date().getTime(),event:'gtm.js'}});var f=d.getElementsByTagName(s)[0],j=d.createElement(s),dl=l!='dataLayer'?'&l='+l:'';j.async=true;j.src='https://www.googletagmanager.com/gtm.js?id='+i+dl;f.parentNode.insertBefore(j,f);}})(window,document,'script','dataLayer','{id}');</script>"#
    ))
}

fn gtm_body_html(snippet: Option<&str>) -> Option<String> {
    let id = crate::model::extract_gtm_id(snippet.unwrap_or(""))?;
    Some(format!(
        r#"<noscript><iframe src="https://www.googletagmanager.com/ns.html?id={id}" height="0" width="0" style="display:none;visibility:hidden" title="Google Tag Manager"></iframe></noscript>"#
    ))
}

fn robots_token(r: crate::model::RobotsDirective) -> &'static str {
    match r {
        crate::model::RobotsDirective::IndexFollow => "index, follow",
        crate::model::RobotsDirective::NoindexFollow => "noindex, follow",
        crate::model::RobotsDirective::IndexNofollow => "index, nofollow",
        crate::model::RobotsDirective::NoindexNofollow => "noindex, nofollow",
    }
}

fn schema_type_token(s: crate::model::SchemaType) -> &'static str {
    match s {
        crate::model::SchemaType::WebPage => "WebPage",
        crate::model::SchemaType::Article => "Article",
        crate::model::SchemaType::AboutPage => "AboutPage",
        crate::model::SchemaType::ContactPage => "ContactPage",
        crate::model::SchemaType::CollectionPage => "CollectionPage",
        crate::model::SchemaType::Organization => "Organization",
        crate::model::SchemaType::LocalBusiness => "LocalBusiness",
        crate::model::SchemaType::Product => "Product",
        crate::model::SchemaType::Service => "Service",
    }
}

fn render_hero(r: &Renderer, hero: &DdHero) -> anyhow::Result<String> {
    r.render("dd-hero", &hero_to_json(hero))
}

fn render_section(
    r: &Renderer,
    section: &DdSection,
    used_ids: &mut std::collections::HashSet<String>,
) -> anyhow::Result<String> {
    let mut columns_html = String::new();
    let item_box_class = section
        .item_box_class
        .as_ref()
        .and_then(|v| serde_json::to_value(v).ok())
        .map(|v| stringify_json(&v))
        .unwrap_or_else(|| "l-box".to_string());
    for column in &section.columns {
        let mut inner = String::new();
        for component in &column.components {
            let html = match component {
                SectionComponent::Alternating(v) => render_alternating(r, v)?,
                SectionComponent::Card(v) => render_card(r, v)?,
                SectionComponent::Cta(v) => render_cta(r, v)?,
                SectionComponent::Filmstrip(v) => render_filmstrip(r, v)?,
                SectionComponent::Milestones(v) => render_milestones(r, v)?,
                SectionComponent::Slider(v) => render_slider(r, v)?,
                SectionComponent::Modal(v) => render_modal(r, v)?,
                SectionComponent::Banner(v) => render_banner(r, v)?,
                SectionComponent::Accordion(v) => render_accordion(r, v)?,
                SectionComponent::Blockquote(v) => render_blockquote(r, v)?,
                SectionComponent::Alert(v) => render_alert(r, v)?,
                SectionComponent::Image(v) => render_image(r, v)?,
                SectionComponent::RichText(v) => render_rich_text(r, v)?,
                SectionComponent::Navigation(v) => render_navigation(r, v)?,
                SectionComponent::HeaderSearch(v) => render_header_search(r, v)?,
                SectionComponent::HeaderMenu(v) => render_header_menu(r, v)?,
                SectionComponent::Spacer(v) => render_spacer(r, v)?,
                SectionComponent::Tabs(v) => render_tabs(r, v)?,
                SectionComponent::Timeline(v) => render_timeline(r, v)?,
                SectionComponent::DataTable(v) => render_data_table(r, v)?,
                SectionComponent::SearchResults(v) => render_search_results(r, v)?,
                SectionComponent::Headline(v) => render_headline(r, v, used_ids)?,
            };
            inner.push_str(&html);
            inner.push('\n');
        }
        columns_html.push_str(&r.render(
            "dd-section-column",
            &json!({
                "id": column.id,
                "width_class": column.width_class,
                "item_box_class": item_box_class,
                "inner": inner,
            }),
        )?);
        columns_html.push('\n');
    }

    let aria_label = section
        .section_title
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            section
                .aria_label
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Content section".to_string());
    let bg = section
        .bg
        .and_then(|v| serde_json::to_value(v).ok())
        .map(|v| stringify_json(&v));
    let padding = section
        .padding
        .and_then(|v| serde_json::to_value(v).ok())
        .map(|v| stringify_json(&v));
    r.render(
        "dd-section",
        &json!({
            "id": section.id,
            "section_class": section
                .section_class
                .as_ref()
                .and_then(|v| serde_json::to_value(v).ok())
                .map(|v| stringify_json(&v))
                .unwrap_or_else(|| "-full-contained".to_string()),
            "bg": bg,
            "padding": padding,
            "custom_css": section
                .custom_css
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty()),
            "aria_label": aria_label,
            "sal_attr": sal_html_attrs(section.sal, section.sal_duration, section.sal_delay),
            "section_title": section.section_title,
            "content": columns_html
        }),
    )
}

fn render_alternating(r: &Renderer, alternating: &DdAlternating) -> anyhow::Result<String> {
    let mut v = serde_json::to_value(alternating)?;
    if let Some(obj) = v.as_object_mut() {
        obj.insert(
            "parent_type".to_string(),
            Value::String(
                serde_json::to_value(alternating.parent_type)
                    .map(|raw| stringify_json(&raw))
                    .unwrap_or_else(|_| "-default".to_string()),
            ),
        );
        obj.insert("sal".to_string(), Value::String(sal_token(alternating.sal)));
        if let Some(items) = obj.get_mut("items").and_then(|v| v.as_array_mut()) {
            attach_sal_stagger(
                items,
                alternating.sal,
                alternating.sal_duration,
                alternating.sal_delay,
            );
            inject_item_copy_html(items);
            for (i, item) in items.iter_mut().enumerate() {
                let resolved = alternating
                    .items
                    .get(i)
                    .map(|it| it.links.clone())
                    .unwrap_or_default();
                let (has_links, links) = links_payload(&resolved);
                if let Some(obj) = item.as_object_mut() {
                    obj.insert("has_links".to_string(), json!(has_links));
                    obj.insert("links".to_string(), json!(links));
                    obj.insert(
                        "media".to_string(),
                        media_to_json(&alternating.items[i].resolved_media()),
                    );
                }
            }
        }
    }
    r.render("dd-alternating", &v)
}

fn render_card(r: &Renderer, card: &DdCard) -> anyhow::Result<String> {
    let mut items = Vec::new();
    for item in &card.items {
        let link_url = item
            .child_link_url
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        let link_label = item
            .child_link_label
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        let has_link = link_url.is_some() && link_label.is_some();
        let link_target = item
            .child_link_target
            .as_ref()
            .and_then(|v| serde_json::to_value(v).ok())
            .map(|v| stringify_json(&v))
            .unwrap_or_else(|| "_self".to_string());
        items.push(json!({
            "child_image_url": item.child_image_url,
            "child_image_alt": item.child_image_alt,
            "child_title": item.child_title,
            "child_subtitle": item.child_subtitle,
            "child_copy": item.child_copy,
            "child_copy_html": markdown_to_html(&item.child_copy),
            "child_link_url": link_url.unwrap_or_default(),
            "child_link_target": link_target,
            "child_link_label": link_label.unwrap_or_default(),
            "child_link_style": button_style_token(item.child_link_style),
            "has_link": has_link
        }));
    }
    attach_sal_stagger(&mut items, card.sal, card.sal_duration, card.sal_delay);
    let data = json!({
        "parent_type": serde_json::to_value(card.parent_type).map(|raw| stringify_json(&raw)).unwrap_or_else(|_| "-default".to_string()),
        "parent_width": card.parent_width,
        "items": items
    });
    r.render("dd-card", &data)
}

fn render_banner(r: &Renderer, banner: &DdBanner) -> anyhow::Result<String> {
    let mut v = serde_json::to_value(banner)?;
    if let Some(obj) = v.as_object_mut() {
        obj.insert(
            "parent_class".to_string(),
            Value::String(
                serde_json::to_value(banner.parent_class)
                    .map(|raw| stringify_json(&raw))
                    .unwrap_or_else(|_| "-bg-center-center".to_string()),
            ),
        );
        obj.insert(
            "sal_attr".to_string(),
            json!(sal_html_attrs(
                banner.sal,
                banner.sal_duration,
                banner.sal_delay
            )),
        );
        obj.insert("media".to_string(), media_to_json(&banner.resolved_media()));
    }
    r.render("dd-banner", &v)
}

fn render_cta(r: &Renderer, cta: &DdCta) -> anyhow::Result<String> {
    let resolved = cta.resolved_links();
    let (has_links, links) = links_payload(&resolved);
    let data = json!({
        "parent_class": serde_json::to_value(cta.parent_class).map(|raw| stringify_json(&raw)).unwrap_or_else(|_| "-top-left".to_string()),
        "parent_image_url": cta.parent_image_url,
        "parent_image_alt": cta.parent_image_alt,
        "sal_attr": sal_html_attrs(cta.sal, cta.sal_duration, cta.sal_delay),
        "parent_title": cta.parent_title,
        "parent_subtitle": cta.parent_subtitle,
        "parent_copy": cta.parent_copy,
        "parent_copy_html": markdown_to_html(&cta.parent_copy),
        "has_links": has_links,
        "links": links
    });
    r.render("dd-cta", &data)
}

fn render_filmstrip(r: &Renderer, filmstrip: &DdFilmstrip) -> anyhow::Result<String> {
    let data = json!({
        "parent_type": serde_json::to_value(filmstrip.parent_type).map(|raw| stringify_json(&raw)).unwrap_or_else(|_| "-default".to_string()),
        "sal_attr": sal_html_attrs(filmstrip.sal, filmstrip.sal_duration, filmstrip.sal_delay),
        "items": filmstrip.items
    });
    r.render("dd-filmstrip", &data)
}

fn render_milestones(r: &Renderer, milestones: &DdMilestones) -> anyhow::Result<String> {
    let mut items = Vec::new();
    for item in &milestones.items {
        let link_url = item
            .child_link_url
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        let link_label = item
            .child_link_label
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        let has_link = link_url.is_some() && link_label.is_some();
        let link_target = item
            .child_link_target
            .as_ref()
            .and_then(|v| serde_json::to_value(v).ok())
            .map(|v| stringify_json(&v))
            .unwrap_or_else(|| "_self".to_string());
        items.push(json!({
            "child_percentage": item.child_percentage,
            "child_title": item.child_title,
            "child_subtitle": item.child_subtitle,
            "child_copy": item.child_copy,
            "child_copy_html": markdown_to_html(&item.child_copy),
            "child_link_url": link_url.unwrap_or_default(),
            "child_link_target": link_target,
            "child_link_label": link_label.unwrap_or_default(),
            "child_link_style": button_style_token(item.child_link_style),
            "has_link": has_link
        }));
    }
    attach_sal_stagger(
        &mut items,
        milestones.sal,
        milestones.sal_duration,
        milestones.sal_delay,
    );
    let data = json!({
        "parent_width": milestones.parent_width,
        "items": items
    });
    r.render("dd-milestones", &data)
}

fn render_modal(r: &Renderer, modal: &DdModal) -> anyhow::Result<String> {
    let data = json!({
        "parent_title": modal.parent_title,
        "parent_copy": modal.parent_copy,
        "parent_copy_html": markdown_to_html(&modal.parent_copy),
        "parent_modal_id": html_id_safe_from_title(&modal.parent_title, "modal")
    });
    r.render("dd-modal", &data)
}

fn render_slider(r: &Renderer, slider: &DdSlider) -> anyhow::Result<String> {
    let fallback_uid = stable_uid_from_title(&slider.parent_title);
    let parent_uid = html_id_safe_from_title(&slider.parent_title, &fallback_uid);
    let mut items = Vec::new();
    for item in &slider.items {
        let resolved = item.resolved_links();
        let (has_links, links) = links_payload(&resolved);
        items.push(json!({
            "child_title": item.child_title,
            "child_copy": item.child_copy,
            "child_copy_html": markdown_to_html(&item.child_copy),
            "media": media_to_json(&item.resolved_media()),
            "has_links": has_links,
            "links": links
        }));
    }
    let data = json!({
        "parent_title": slider.parent_title,
        "has_parent_title": !slider.parent_title.trim().is_empty(),
        "parent_uid": parent_uid,
        "items": items
    });
    r.render("dd-slider", &data)
}

fn render_accordion(r: &Renderer, accordion: &DdAccordion) -> anyhow::Result<String> {
    let mut v = serde_json::to_value(accordion)?;
    let faq_schema = serde_json::to_string(&json!({
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "mainEntity": accordion.items.iter().map(|item| {
            json!({
                "@type": "Question",
                "name": item.child_title,
                "acceptedAnswer": {
                    "@type": "Answer",
                    "text": item.child_copy
                }
            })
        }).collect::<Vec<_>>()
    }))?;
    if let Some(obj) = v.as_object_mut() {
        obj.insert(
            "parent_type".to_string(),
            Value::String(
                serde_json::to_value(accordion.parent_type)
                    .map(|v| stringify_json(&v))
                    .unwrap_or_else(|_| "-default".to_string()),
            ),
        );
        obj.insert(
            "parent_class".to_string(),
            Value::String(
                serde_json::to_value(accordion.parent_class)
                    .map(|v| stringify_json(&v))
                    .unwrap_or_else(|_| "-primary".to_string()),
            ),
        );
        obj.insert(
            "sal_attr".to_string(),
            json!(sal_html_attrs(
                accordion.sal,
                accordion.sal_duration,
                accordion.sal_delay
            )),
        );
        obj.insert(
            "has_faq_schema".to_string(),
            Value::Bool(matches!(
                accordion.parent_type,
                crate::model::AccordionType::Faq
            )),
        );
        obj.insert("faq_schema_json".to_string(), Value::String(faq_schema));
        if let Some(items) = obj.get_mut("items").and_then(|v| v.as_array_mut()) {
            inject_item_copy_html(items);
        }
    }
    r.render("dd-accordion", &v)
}

fn render_blockquote(r: &Renderer, blockquote: &DdBlockquote) -> anyhow::Result<String> {
    let blockquote_schema_json = serde_json::to_string(&json!({
      "@context": "https://schema.org/",
      "@type": "Quotation",
      "creator": {
        "@type": "Person",
        "name": format!(
            "{}, {}",
            blockquote.parent_name, blockquote.parent_role
        )
      },
      "text": blockquote.parent_copy
    }))?;
    let mut v = serde_json::to_value(blockquote)?;
    if let Some(obj) = v.as_object_mut() {
        obj.insert(
            "sal_attr".to_string(),
            json!(sal_html_attrs(
                blockquote.sal,
                blockquote.sal_duration,
                blockquote.sal_delay
            )),
        );
        obj.insert(
            "blockquote_schema_json".to_string(),
            Value::String(blockquote_schema_json),
        );
        obj.insert(
            "parent_copy_html".to_string(),
            Value::String(markdown_to_html(&blockquote.parent_copy)),
        );
    }
    r.render("dd-blockquote", &v)
}

fn render_spacer(r: &Renderer, spacer: &DdSpacer) -> anyhow::Result<String> {
    let size = serde_json::to_value(spacer.size)
        .ok()
        .map(|v| stringify_json(&v))
        .unwrap_or_else(|| "-md".to_string());
    r.render(
        "dd-spacer",
        &json!({
            "size": size,
            "divider": spacer.divider,
        }),
    )
}

fn render_tabs(r: &Renderer, tabs: &DdTabs) -> anyhow::Result<String> {
    let parent_id = {
        let id = tabs.parent_id.trim();
        if id.is_empty() {
            stable_uid_from_title(
                tabs.items
                    .first()
                    .map(|i| i.child_title.as_str())
                    .unwrap_or("tabs"),
            )
        } else {
            id.to_string()
        }
    };
    let parent_class = serde_json::to_value(tabs.parent_class)
        .ok()
        .map(|v| stringify_json(&v))
        .unwrap_or_else(|| "-horizontal".to_string());
    let aria_label = tabs
        .aria_label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Content tabs");
    let items: Vec<Value> = tabs
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let n = i + 1;
            json!({
                "child_title": item.child_title,
                "child_copy_html": markdown_to_html(&item.child_copy),
                "tab_id": format!("{parent_id}-tab-{n}"),
                "panel_id": format!("{parent_id}-panel-{n}"),
                "active": i == 0,
                "aria_selected": if i == 0 { "true" } else { "false" },
                "tabindex": if i == 0 { "0" } else { "-1" },
            })
        })
        .collect();
    r.render(
        "dd-tabs",
        &json!({
            "parent_id": parent_id,
            "parent_class": parent_class,
            "aria_label": aria_label,
            "sal_attr": sal_html_attrs(tabs.sal, tabs.sal_duration, tabs.sal_delay),
            "items": items,
        }),
    )
}

fn render_timeline(r: &Renderer, timeline: &DdTimeline) -> anyhow::Result<String> {
    let aria_label = timeline
        .aria_label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Timeline");
    let mut items: Vec<Value> = timeline
        .items
        .iter()
        .map(|item| {
            let level = match item.heading_level {
                2..=6 => item.heading_level,
                _ => 3,
            };
            let title = html_escape_attr(&item.child_title);
            let image_url = item
                .child_image_url
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(public_url);
            json!({
                "child_year": item.child_year,
                "child_datetime": item
                    .child_datetime
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty()),
                "title_html": format!("<h{level}>{title}</h{level}>"),
                "child_copy_html": markdown_to_html(&item.child_copy),
                "child_image_url": image_url,
                "child_image_alt": item.child_image_alt.clone().unwrap_or_default(),
            })
        })
        .collect();
    attach_sal_stagger(
        &mut items,
        timeline.sal,
        timeline.sal_duration,
        timeline.sal_delay,
    );
    r.render(
        "dd-timeline",
        &json!({
            "aria_label": aria_label,
            "items": items,
        }),
    )
}

fn render_data_table(r: &Renderer, table: &DdDataTable) -> anyhow::Result<String> {
    let caption = table.caption.trim();
    let scroll_label = table
        .scroll_label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{caption}, scrollable"));
    let empty_message = table
        .empty_message
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("No data to display.");
    let col_n = table.columns.len().min(DATA_TABLE_MAX_COLUMNS);
    let mut used_keys = std::collections::HashSet::new();
    let columns: Vec<Value> = table
        .columns
        .iter()
        .take(col_n)
        .enumerate()
        .map(|(i, col)| {
            json!({
                "label": col.label,
                "align": data_table_align_str(col.align),
                "sortable": col.sortable,
                "sort_key": data_table_sort_key(&col.label, i, &mut used_keys),
            })
        })
        .collect();
    let rows: Vec<Value> = table
        .rows
        .iter()
        .map(|row| {
            let mut cells = Vec::new();
            for i in 0..col_n {
                let cell = row.cells.get(i);
                let kind = cell.map(|c| c.kind).unwrap_or_default();
                let text = cell.map(|c| c.text.as_str()).unwrap_or("");
                let badge = cell.map(|c| c.badge).unwrap_or_default();
                let align = table
                    .columns
                    .get(i)
                    .map(|c| data_table_align_str(c.align))
                    .unwrap_or("start");
                let is_badge = kind == DataTableCellType::Badge;
                cells.push(json!({
                    "is_header": i == 0,
                    "is_badge": is_badge,
                    "text": text,
                    "badge": if is_badge {
                        data_table_badge_str(badge)
                    } else {
                        ""
                    },
                    "align": align,
                }));
            }
            json!({ "cells": cells })
        })
        .collect();
    r.render(
        "dd-data-table",
        &json!({
            "dense": table.dense,
            "scroll_label": scroll_label,
            "caption": caption,
            "columns": columns,
            "rows": rows,
            "is_empty": table.rows.is_empty(),
            "colspan": col_n.max(1),
            "empty_message": empty_message,
        }),
    )
}

fn data_table_align_str(align: DataTableAlign) -> &'static str {
    match align {
        DataTableAlign::Start => "start",
        DataTableAlign::Center => "center",
        DataTableAlign::End => "end",
    }
}

fn data_table_badge_str(badge: DataTableBadge) -> &'static str {
    match badge {
        DataTableBadge::Critical => "-critical",
        DataTableBadge::Warning => "-warning",
        DataTableBadge::Info => "-info",
        DataTableBadge::Pass => "-pass",
    }
}

fn data_table_sort_key(
    label: &str,
    index: usize,
    used: &mut std::collections::HashSet<String>,
) -> String {
    let mut key = crate::model::slug_from_title(label);
    if used.contains(&key) || key == "untitled" {
        key = format!("col-{index}");
        let mut n = 2u32;
        while used.contains(&key) {
            key = format!("col-{index}-{n}");
            n += 1;
        }
    }
    used.insert(key.clone());
    key
}

fn render_alert(r: &Renderer, alert: &DdAlert) -> anyhow::Result<String> {
    let data = json!({
        "parent_type": serde_json::to_value(alert.parent_type).map(|raw| stringify_json(&raw)).unwrap_or_else(|_| "-default".to_string()),
        "parent_class": serde_json::to_value(alert.parent_class).map(|raw| stringify_json(&raw)).unwrap_or_else(|_| "-default".to_string()),
        "sal_attr": sal_html_attrs(alert.sal, alert.sal_duration, alert.sal_delay),
        "parent_title": alert.parent_title.as_deref().unwrap_or(""),
        "has_title": alert.parent_title.as_ref().map(|t| !t.trim().is_empty()).unwrap_or(false),
        "parent_copy": alert.parent_copy,
        "parent_copy_html": markdown_to_html(&alert.parent_copy)
    });
    r.render("dd-alert", &data)
}

fn render_image(r: &Renderer, image: &crate::model::DdImage) -> anyhow::Result<String> {
    let sal_attr = sal_html_attrs(image.sal, image.sal_duration, image.sal_delay);
    let has_link = image
        .parent_link_url
        .as_deref()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let link_target = image
        .parent_link_target
        .as_ref()
        .and_then(|v| serde_json::to_value(v).ok())
        .map(|v| stringify_json(&v))
        .unwrap_or_else(|| "_self".to_string());
    let dark = image
        .parent_image_url_dark
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let data = json!({
        "sal_attr": sal_attr,
        "has_link": has_link,
        "parent_image_url": image.parent_image_url,
        "parent_image_url_dark": dark,
        "parent_image_alt": image.parent_image_alt,
        "parent_link_url": image.parent_link_url.clone().unwrap_or_default(),
        "parent_link_target": link_target,
    });
    r.render("dd-image", &data)
}

fn render_rich_text(r: &Renderer, rt: &crate::model::DdRichText) -> anyhow::Result<String> {
    let parent_class = rt
        .parent_class
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let parent_copy_html = markdown_to_html(&rt.parent_copy);
    let data = json!({
        "parent_class": parent_class,
        "sal_attr": sal_html_attrs(rt.sal, rt.sal_duration, rt.sal_delay),
        "parent_copy_html": parent_copy_html,
    });
    r.render("dd-rich_text", &data)
}

fn render_navigation(r: &Renderer, nav: &crate::model::DdNavigation) -> anyhow::Result<String> {
    let parent_class = navigation_class_token(nav.parent_class);
    let aria_label = match nav.parent_type {
        crate::model::NavigationType::HeaderNav => "header navigation",
        crate::model::NavigationType::FooterNav => "footer navigation",
    };
    r.render(
        "dd-navigation",
        &json!({
            "parent_class": parent_class,
            "sal_attr": sal_html_attrs(nav.sal, nav.sal_duration, nav.sal_delay),
            "aria_label": aria_label,
            "items": nav_items_to_json(&nav.items),
        }),
    )
}

fn nav_items_to_json(items: &[crate::model::NavigationItem]) -> Vec<Value> {
    items.iter().map(nav_item_to_json).collect()
}

fn nav_item_to_json(item: &crate::model::NavigationItem) -> Value {
    let nested = nav_items_to_json(&item.items);
    json!({
        "is_link": matches!(item.child_kind, crate::model::NavigationKind::Link),
        "is_button": matches!(item.child_kind, crate::model::NavigationKind::Button),
        "child_link_label": item.child_link_label,
        "child_link_url": item.child_link_url.as_deref().unwrap_or(""),
        "child_link_target": item
            .child_link_target
            .map(link_target_token)
            .unwrap_or("_self"),
        "child_link_css": item.child_link_css.as_deref().unwrap_or(""),
        "has_children": !nested.is_empty(),
        "items": nested,
    })
}

fn render_header_search(
    r: &Renderer,
    search: &crate::model::DdHeaderSearch,
) -> anyhow::Result<String> {
    let data = json!({
        "sal_attr": sal_html_attrs(search.sal, search.sal_duration, search.sal_delay),
    });
    r.render("dd-header-search", &data)
}

fn render_header_menu(r: &Renderer, menu: &crate::model::DdHeaderMenu) -> anyhow::Result<String> {
    let data = json!({
        "sal_attr": sal_html_attrs(menu.sal, menu.sal_duration, menu.sal_delay),
    });
    r.render("dd-header-menu", &data)
}

fn render_headline(
    r: &Renderer,
    headline: &DdHeadline,
    used_ids: &mut std::collections::HashSet<String>,
) -> anyhow::Result<String> {
    let id = uniquify_id(&html_id_from_text(&headline.text), used_ids);
    used_ids.insert(id.clone());
    let custom_css = headline
        .custom_css
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    r.render(
        "dd-headline",
        &json!({
            "id": id,
            "heading_tag": headline.heading_level.tag(),
            "text": headline.text,
            "custom_css": custom_css,
            "sal_attr": sal_html_attrs(headline.sal, headline.sal_duration, headline.sal_delay),
        }),
    )
}

/// Ids already emitted on the page (and in header/footer chrome) so a headline
/// anchor does not reuse a section, column, hero, tab, or modal id.
fn reserved_html_ids(site: &Site, page: &Page) -> std::collections::HashSet<String> {
    let mut used = std::collections::HashSet::new();
    for section in site
        .header
        .sections
        .iter()
        .chain(site.footer.sections.iter())
    {
        reserve_section_ids(section, &mut used);
    }
    for node in &page.nodes {
        match node {
            PageNode::Hero(hero) => {
                if let Some(id) = hero.id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                    used.insert(id.to_string());
                }
            }
            PageNode::Section(section) => reserve_section_ids(section, &mut used),
        }
    }
    used
}

fn reserve_section_ids(section: &DdSection, used: &mut std::collections::HashSet<String>) {
    let id = section.id.trim();
    if !id.is_empty() {
        used.insert(id.to_string());
    }
    for column in &section.columns {
        let id = column.id.trim();
        if !id.is_empty() {
            used.insert(id.to_string());
        }
        for component in &column.components {
            reserve_component_ids(component, used);
        }
    }
}

fn reserve_component_ids(
    component: &SectionComponent,
    used: &mut std::collections::HashSet<String>,
) {
    match component {
        SectionComponent::Tabs(tabs) => {
            let parent_id = {
                let id = tabs.parent_id.trim();
                if id.is_empty() {
                    stable_uid_from_title(
                        tabs.items
                            .first()
                            .map(|item| item.child_title.as_str())
                            .unwrap_or("tabs"),
                    )
                } else {
                    id.to_string()
                }
            };
            for (i, _) in tabs.items.iter().enumerate() {
                let n = i + 1;
                used.insert(format!("{parent_id}-tab-{n}"));
                used.insert(format!("{parent_id}-panel-{n}"));
            }
        }
        SectionComponent::Modal(modal) => {
            used.insert(html_id_safe_from_title(&modal.parent_title, "modal"));
        }
        _ => {}
    }
}

fn render_search_results(r: &Renderer, search: &DdSearchResults) -> anyhow::Result<String> {
    let data = json!({
        "sal_attr": sal_html_attrs(search.sal, search.sal_duration, search.sal_delay),
    });
    r.render("dd-search-results", &data)
}

fn sal_token(sal: crate::model::SalAnimation) -> String {
    serde_json::to_value(sal)
        .map(|v| stringify_json(&v))
        .unwrap_or_else(|_| "fade".to_string())
}

/// Attribute string for templates (` data-sal="fade"` …). Empty when style is
/// `no-animation` so the element is omitted from SAL's `[data-sal]` observer.
fn sal_html_attrs(
    sal: crate::model::SalAnimation,
    duration: Option<u16>,
    delay: Option<u16>,
) -> String {
    if !sal.is_animated() {
        return String::new();
    }
    let mut out = format!(" data-sal=\"{}\"", sal_token(sal));
    if let Some(ms) = duration.filter(|&ms| ms != 400 && crate::model::is_valid_sal_duration(ms)) {
        out.push_str(&format!(" data-sal-duration=\"{ms}\""));
    }
    if let Some(ms) = delay.filter(|&ms| ms > 0 && crate::model::is_valid_sal_delay(ms)) {
        out.push_str(&format!(" data-sal-delay=\"{ms}\""));
    }
    out
}

fn attach_sal_stagger(
    items: &mut [Value],
    sal: crate::model::SalAnimation,
    duration: Option<u16>,
    author_delay: Option<u16>,
) {
    let base = u32::from(author_delay.unwrap_or(0));
    for (i, item) in items.iter_mut().enumerate() {
        let Some(obj) = item.as_object_mut() else {
            continue;
        };
        let delay = (base + i as u32 * 100).min(1000) as u16;
        let attr = sal_html_attrs(sal, duration, (delay > 0).then_some(delay));
        obj.insert("sal_attr".to_string(), json!(attr));
    }
}

fn link_target_token(target: crate::model::CardLinkTarget) -> &'static str {
    match target {
        crate::model::CardLinkTarget::SelfTarget => "_self",
        crate::model::CardLinkTarget::Blank => "_blank",
    }
}

fn button_style_token(style: ButtonStyle) -> &'static str {
    match style {
        ButtonStyle::Primary => "-primary",
        ButtonStyle::Secondary => "-secondary",
        ButtonStyle::Tertiary => "-tertiary",
        ButtonStyle::Ghost => "-ghost",
    }
}

fn dd_link_json(link: &DdLink) -> Value {
    json!({
        "url": public_url(&link.url),
        "label": link.label,
        "target": link_target_token(link.target),
        "style": button_style_token(link.style),
    })
}

fn links_payload(links: &[DdLink]) -> (bool, Vec<Value>) {
    let items: Vec<Value> = links.iter().map(dd_link_json).collect();
    (!items.is_empty(), items)
}

fn navigation_class_token(class: crate::model::NavigationClass) -> &'static str {
    match class {
        crate::model::NavigationClass::MainMenu => "-main-menu",
        crate::model::NavigationClass::MenuSecondary => "-menu-secondary",
        crate::model::NavigationClass::MenuTertiary => "-menu-tertiary",
        crate::model::NavigationClass::FooterMenu => "-footer-menu",
        crate::model::NavigationClass::FooterMenuSecondary => "-footer-menu-secondary",
        crate::model::NavigationClass::FooterMenuTertiary => "-footer-menu-tertiary",
        crate::model::NavigationClass::SocialMenu => "-social-menu",
    }
}

fn hero_to_json(hero: &DdHero) -> Value {
    let resolved_media = hero.resolved_media();
    let mut media = media_to_json(&resolved_media);
    if let Some(obj) = media.as_object_mut() {
        obj.insert("skip_lazy".to_string(), json!(true));
        if let Some(s) = hero
            .parent_image_mobile
            .as_deref()
            .filter(|v| !v.trim().is_empty())
        {
            obj.insert("source_mobile".to_string(), json!(public_url(s)));
        }
        if let Some(s) = hero
            .parent_image_tablet
            .as_deref()
            .filter(|v| !v.trim().is_empty())
        {
            obj.insert("source_tablet".to_string(), json!(public_url(s)));
        }
        if let Some(s) = hero
            .parent_image_desktop
            .as_deref()
            .filter(|v| !v.trim().is_empty())
        {
            obj.insert("source_desktop".to_string(), json!(public_url(s)));
        }
    }
    let image_url = match &resolved_media {
        Media::Image { url, .. } => url.as_str(),
        _ => hero.parent_image_url.trim(),
    };
    let subtitle = hero.parent_subtitle.trim();
    let parent_class = hero
        .parent_class
        .as_ref()
        .and_then(|v| serde_json::to_value(v).ok())
        .map(|v| stringify_json(&v));
    let sal_style = hero.sal.unwrap_or(crate::model::SalAnimation::Fade);
    let sal_attr = sal_html_attrs(sal_style, hero.sal_duration, hero.sal_delay);
    let parent_custom_css = hero
        .parent_custom_css
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let parent_copy_html = hero
        .parent_copy
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .map(markdown_to_html);
    let resolved_links = hero.resolved_links();
    let (has_links, links) = links_payload(&resolved_links);
    let bg_mobile = hero
        .parent_image_mobile
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(image_url);
    let bg_desktop = hero
        .parent_image_desktop
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(image_url);
    let parent_image_class = hero
        .parent_image_class
        .as_ref()
        .and_then(|v| serde_json::to_value(v).ok())
        .map(|v| stringify_json(&v))
        .unwrap_or_else(|| "-full-full".to_string());
    let has_body = hero
        .parent_copy
        .as_deref()
        .is_some_and(|v| !v.trim().is_empty())
        || has_links;
    let overlay = hero
        .overlay
        .and_then(|v| serde_json::to_value(v).ok())
        .map(|v| stringify_json(&v));
    let copy_position = hero
        .copy_position
        .and_then(|v| serde_json::to_value(v).ok())
        .map(|v| stringify_json(&v));
    let id = hero.id.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let aria_label = hero
        .aria_label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Introduction");

    json!({
        "parent_class": parent_class,
        "overlay": overlay,
        "copy_position": copy_position,
        "id": id,
        "aria_label": aria_label,
        "sal_attr": sal_attr,
        "parent_custom_css": parent_custom_css,
        "parent_title": hero.parent_title,
        "parent_subtitle": if subtitle.is_empty() { None } else { Some(hero.parent_subtitle.clone()) },
        "parent_copy_html": parent_copy_html,
        "links": links,
        "parent_image_class": parent_image_class,
        "media": media,
        "has_body": has_body,
        "has_links": has_links,
        "bg_mobile": public_url(bg_mobile),
        "bg_desktop": public_url(bg_desktop)
    })
}

fn oembed_iframe(url: &str) -> String {
    match parse_oembed_url(url) {
        Some(OembedProvider::Youtube { id }) => format!(
            r#"<iframe src="https://www.youtube-nocookie.com/embed/{id}" title="YouTube video" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen loading="lazy"></iframe>"#
        ),
        Some(OembedProvider::Vimeo { id }) => format!(
            r#"<iframe src="https://player.vimeo.com/video/{id}" title="Vimeo video" allow="autoplay; fullscreen; picture-in-picture" allowfullscreen loading="lazy"></iframe>"#
        ),
        None => {
            let href = html_escape_attr(url);
            format!(r#"<p><a href="{href}">{href}</a></p>"#)
        }
    }
}

fn html_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn media_to_json(media: &Media) -> Value {
    match media {
        Media::None => json!({
            "has_media": false,
            "is_image": false,
            "is_oembed": false,
            "is_local_video": false,
        }),
        Media::Image { url, alt } => json!({
            "has_media": true,
            "is_image": true,
            "is_oembed": false,
            "is_local_video": false,
            "url": public_url(url),
            "alt": alt,
        }),
        Media::Oembed { url } => json!({
            "has_media": true,
            "is_image": false,
            "is_oembed": true,
            "is_local_video": false,
            "oembed_iframe": oembed_iframe(url),
        }),
        Media::LocalVideo {
            lg_mp4,
            sm_mp4,
            poster,
            name,
            loop_playback,
            autoplay,
        } => json!({
            "has_media": true,
            "is_image": false,
            "is_oembed": false,
            "is_local_video": true,
            "lg_mp4": public_url(lg_mp4),
            "sm_mp4": sm_mp4.as_deref().filter(|s| !s.trim().is_empty()).map(public_url),
            "poster": poster.as_deref().filter(|s| !s.trim().is_empty()).map(public_url),
            "name": name,
            "loop": loop_playback,
            "autoplay": autoplay,
        }),
    }
}

/// Prefix site-relative `href`/`src`/`poster`/`srcset`/`data-search-index`
/// values and JSON-LD `url`/`image` strings so nested pages still reach the export root.
/// Meta `content` is skipped: viewport, robots, description, and OG titles are not URLs.
fn prefix_relative_urls(html: &str, prefix: &str) -> String {
    if prefix.is_empty() {
        return html.to_string();
    }
    let mut html = prefix_quoted_attrs(html, prefix, "href");
    html = prefix_quoted_attrs(&html, prefix, "src");
    html = prefix_quoted_attrs(&html, prefix, "poster");
    html = prefix_quoted_attrs(&html, prefix, "data-search-index");
    html = prefix_srcset_attrs(&html, prefix);
    html = prefix_json_string_key(&html, prefix, "url");
    html = prefix_json_string_key(&html, prefix, "image");
    html
}

fn prefix_quoted_attrs(html: &str, prefix: &str, attr: &str) -> String {
    let dquote = format!("{attr}=\"");
    let squote = format!("{attr}='");
    prefix_after_needles(html, prefix, &[&dquote, &squote], false)
}

fn prefix_srcset_attrs(html: &str, prefix: &str) -> String {
    let dquote = "srcset=\"";
    let squote = "srcset='";
    prefix_after_needles(html, prefix, &[dquote, squote], true)
}

fn prefix_json_string_key(html: &str, prefix: &str, key: &str) -> String {
    let tight = format!("\"{key}\":\"");
    let spaced = format!("\"{key}\": \"");
    prefix_after_needles(html, prefix, &[&tight, &spaced], false)
}

fn prefix_after_needles(html: &str, prefix: &str, needles: &[&str], srcset: bool) -> String {
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while i < html.len() {
        let slice = &html[i..];
        let mut best: Option<(usize, usize, char)> = None;
        for needle in needles {
            if let Some(p) = slice.find(*needle) {
                let quote = needle.as_bytes()[needle.len() - 1] as char;
                if best.is_none_or(|(bp, _, _)| p < bp) {
                    best = Some((p, needle.len(), quote));
                }
            }
        }
        let Some((rel, needle_len, quote)) = best else {
            out.push_str(slice);
            break;
        };
        out.push_str(&slice[..rel + needle_len]);
        let val_start = i + rel + needle_len;
        let remainder = &html[val_start..];
        let Some(end) = remainder.find(quote) else {
            out.push_str(remainder);
            break;
        };
        let value = &remainder[..end];
        if srcset {
            out.push_str(&prefix_srcset_value(value, prefix));
        } else if crate::model::is_site_relative_url(value) {
            out.push_str(prefix);
            out.push_str(value);
        } else {
            out.push_str(value);
        }
        out.push(quote);
        i = val_start + end + 1;
    }
    out
}

fn prefix_srcset_value(value: &str, prefix: &str) -> String {
    value
        .split(',')
        .map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return String::new();
            }
            let mut bits = part.splitn(2, char::is_whitespace);
            let url = bits.next().unwrap_or("");
            let rest = bits.next();
            let rewritten = if crate::model::is_site_relative_url(url) {
                format!("{prefix}{url}")
            } else {
                url.to_string()
            };
            match rest {
                Some(r) => format!("{rewritten} {r}"),
                None => rewritten,
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn public_url(stored: &str) -> String {
    let t = stored.trim();
    if t.starts_with("http://")
        || t.starts_with("https://")
        || t.starts_with('#')
        || t.starts_with("mailto:")
        || t.starts_with("tel:")
    {
        t.to_string()
    } else {
        t.trim_start_matches('/').to_string()
    }
}

/// Absolute asset URL when `base_url` is set; otherwise the page's relative prefix.
fn resolve_head_asset_url(stored: &str, site: &Site, page: &Page) -> String {
    let u = public_url(stored);
    if crate::model::is_absolute_http_url(&u) {
        return u;
    }
    if let Some(abs) = crate::model::absolute_url(site.base_url.as_deref(), &u) {
        return abs;
    }
    let prefix =
        crate::model::url_prefix_for_page(&page.slug, site.pretty_urls, site.base_url.as_deref());
    format!("{prefix}{u}")
}

fn markdown_to_html(input: &str) -> String {
    crate::markdown::to_html(input)
}

fn inject_item_copy_html(items: &mut [Value]) {
    for item in items {
        let Some(obj) = item.as_object_mut() else {
            continue;
        };
        if let Some(Value::String(copy)) = obj.get("child_copy") {
            let html = markdown_to_html(copy);
            obj.insert("child_copy_html".to_string(), Value::String(html));
        }
    }
}

fn stringify_json(value: &Value) -> String {
    match value {
        Value::String(v) => v.clone(),
        _ => String::new(),
    }
}

fn html_id_safe_from_title(title: &str, fallback: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in title.trim().to_lowercase().chars() {
        let keep = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-';
        if keep {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let mut out = out.trim_matches('-').to_string();
    if out.is_empty() {
        out = fallback.to_string();
    }
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out = format!("modal-{out}");
    }
    out
}

fn stable_uid_from_title(title: &str) -> String {
    let mut hash: u64 = 5381;
    for b in title.as_bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(*b as u64);
    }
    format!("uid-{:06}", hash % 1_000_000)
}

#[cfg(test)]
mod tests {
    use super::render_page_html;
    use crate::model::Site;

    #[test]
    fn renders_page_with_hero_and_section() {
        let site = Site::starter();
        let page = &site.pages[0];
        let html = render_page_html(page).expect("page should render");
        assert!(html.contains("dd-hero"));
        assert!(html.contains("dd-section"));
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("assets/css/style.min.css"));
        assert!(html.contains("lang=\"en\""));
        assert!(!html.contains("/assets/css/style.min.css"));
        assert!(html.contains("data-sal="));
        assert!(!html.contains("data-aos"));
        assert!(html.contains("dd-button -primary"));
        assert!(html.contains("dd-button -ghost"));
        assert!(html.contains("Get Started"));
        assert!(html.contains("Learn More"));
        assert!(html.contains("id=\"section-1\""));
        assert!(html.contains("aria-label=\"Ready to publish?\""));
        assert!(html.contains("class=\"dd-hero -full-full -left\""));
        assert!(html.contains("aria-label=\"Introduction\""));
    }

    #[test]
    fn spacer_tabs_and_timeline_render() {
        use crate::model::*;
        let html = render_page_html(&page_with_component(SectionComponent::Spacer(DdSpacer {
            size: SpacerSize::Xxl,
            divider: true,
        })))
        .expect("spacer");
        assert!(
            html.contains(r#"class="dd-spacer -xxl -divider""#),
            "{html}"
        );
        assert!(html.contains("aria-hidden=\"true\""), "{html}");

        let html = render_page_html(&page_with_component(SectionComponent::Tabs(DdTabs {
            parent_id: "features".to_string(),
            parent_class: TabsOrientation::Vertical,
            aria_label: Some("Features".to_string()),
            sal: SalAnimation::Fade,
            sal_duration: None,
            sal_delay: None,
            items: vec![
                TabsItem {
                    child_title: "One".to_string(),
                    child_copy: "First".to_string(),
                },
                TabsItem {
                    child_title: "Two".to_string(),
                    child_copy: "Second".to_string(),
                },
            ],
        })))
        .expect("tabs");
        assert!(html.contains("class=\"dd-tabs -vertical\""), "{html}");
        assert!(html.contains("data-id=\"features\""), "{html}");
        assert!(html.contains("aria-label=\"Features\""), "{html}");
        assert!(html.contains("role=\"tablist\""), "{html}");
        assert!(html.contains("<button type=\"button\""), "{html}");
        assert!(html.contains("id=\"features-tab-1\""), "{html}");
        assert!(
            html.contains("aria-controls=\"features-panel-1\""),
            "{html}"
        );
        assert!(html.contains("aria-selected=\"true\""), "{html}");
        assert!(html.contains("aria-selected=\"false\""), "{html}");
        assert!(html.contains("id=\"features-panel-2\""), "{html}");
        assert!(html.contains(" hidden"), "{html}");

        let html = render_page_html(&page_with_component(SectionComponent::Timeline(
            DdTimeline {
                aria_label: None,
                sal: SalAnimation::Fade,
                sal_duration: None,
                sal_delay: None,
                items: vec![
                    TimelineItem {
                        child_year: "2022".to_string(),
                        child_datetime: None,
                        child_title: "Start".to_string(),
                        heading_level: 3,
                        child_copy: "Began".to_string(),
                        child_image_url: Some("/assets/images/v1.jpg".to_string()),
                        child_image_alt: Some("v1".to_string()),
                    },
                    TimelineItem {
                        child_year: "Q3 2024".to_string(),
                        child_datetime: Some("2024-07-01".to_string()),
                        child_title: "Next".to_string(),
                        heading_level: 4,
                        child_copy: "Later".to_string(),
                        child_image_url: None,
                        child_image_alt: None,
                    },
                ],
            },
        )))
        .expect("timeline");
        assert!(html.contains("aria-label=\"Timeline\""), "{html}");
        assert!(html.contains("<ol class=\"dd-timeline__items"), "{html}");
        assert!(html.contains("<h3>Start</h3>"), "{html}");
        assert!(html.contains("<h4>Next</h4>"), "{html}");
        assert!(html.contains("datetime=\"2024-07-01\""), "{html}");
        assert!(html.contains("dd-timeline__text"), "{html}");
        assert!(html.contains("src=\"assets/images/v1.jpg\""), "{html}");
        assert!(html.contains("data-sal=\"fade\""), "{html}");
        assert!(html.contains("data-sal-delay=\"100\""), "{html}");
    }

    #[test]
    fn search_results_renders_page_form() {
        use crate::model::*;
        let html = render_page_html(&page_with_component(SectionComponent::SearchResults(
            DdSearchResults {
                sal: SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
            },
        )))
        .expect("search_results");
        assert!(html.contains("class=\"dd-search-page\""), "{html}");
        assert!(html.contains("dd-search-page__title"), "{html}");
        assert!(html.contains("id=\"dd-page-search-q\""), "{html}");
        assert!(html.contains("role=\"search\""), "{html}");
        assert!(html.contains("dd-search-page__results"), "{html}");
        assert!(!html.contains("data-sal="), "{html}");
    }

    #[test]
    fn headline_renders_chosen_level_class_and_escapes_text() {
        use crate::model::*;
        let html = render_page_html(&page_with_component(SectionComponent::Headline(
            DdHeadline {
                text: "A & B".to_string(),
                heading_level: HeadingLevel::H3,
                custom_css: Some("-center".to_string()),
                sal: SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
            },
        )))
        .expect("headline");
        assert!(
            html.contains("<h3 id=\"a-b\" class=\"dd-headline -center\">A &amp; B</h3>"),
            "{html}"
        );
        assert!(!html.contains("data-sal="), "{html}");
        assert!(!html.contains("<h1"), "{html}");
    }

    #[test]
    fn headline_renders_sal_on_the_heading() {
        use crate::model::*;
        let html = render_page_html(&page_with_component(SectionComponent::Headline(
            DdHeadline {
                text: "Hello".to_string(),
                heading_level: HeadingLevel::H2,
                custom_css: Some("  ".to_string()),
                sal: SalAnimation::Fade,
                sal_duration: Some(500),
                sal_delay: None,
            },
        )))
        .expect("headline sal");
        assert!(
            html.contains(
                "<h2 id=\"hello\" class=\"dd-headline\" data-sal=\"fade\" data-sal-duration=\"500\">Hello</h2>"
            ),
            "{html}"
        );
    }

    #[test]
    fn headline_id_matches_text_and_avoids_ids_already_on_the_page() {
        use crate::model::*;
        let headline = |text: &str| {
            SectionComponent::Headline(DdHeadline {
                text: text.to_string(),
                heading_level: HeadingLevel::H2,
                custom_css: None,
                sal: SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
            })
        };
        let mut page = page_with_component(headline("Services"));
        let PageNode::Section(section) = &mut page.nodes[0] else {
            panic!("expected section");
        };
        section.id = "services".to_string();
        section.columns[0].components.push(headline("Services"));
        section.columns[0].components.push(headline("A & B"));
        let html = render_page_html(&page).expect("headline ids");
        assert!(
            html.contains(r#"<h2 id="services-2" class="dd-headline">Services</h2>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<h2 id="services-3" class="dd-headline">Services</h2>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<h2 id="a-b" class="dd-headline">A &amp; B</h2>"#),
            "{html}"
        );
        assert!(html.contains(r#"id="services" aria-label="#), "{html}");
    }

    #[test]
    fn data_table_renders_caption_badge_sort_and_empty_state() {
        use crate::model::*;
        let html = render_page_html(&page_with_component(SectionComponent::DataTable(
            DdDataTable {
                caption: "Prioritized remediation tasks".to_string(),
                dense: true,
                scroll_label: None,
                empty_message: None,
                columns: vec![
                    DataTableColumn {
                        label: "ID".to_string(),
                        align: DataTableAlign::Start,
                        sortable: false,
                    },
                    DataTableColumn {
                        label: "Priority".to_string(),
                        align: DataTableAlign::Start,
                        sortable: true,
                    },
                    DataTableColumn {
                        label: "Effort".to_string(),
                        align: DataTableAlign::End,
                        sortable: true,
                    },
                ],
                rows: vec![DataTableRow {
                    cells: vec![
                        DataTableCell {
                            kind: DataTableCellType::Text,
                            text: "SEO-001".to_string(),
                            badge: DataTableBadge::Info,
                        },
                        DataTableCell {
                            kind: DataTableCellType::Badge,
                            text: "Critical".to_string(),
                            badge: DataTableBadge::Critical,
                        },
                        DataTableCell {
                            kind: DataTableCellType::Text,
                            text: "2h".to_string(),
                            badge: DataTableBadge::Info,
                        },
                    ],
                }],
            },
        )))
        .expect("data table");
        assert!(html.contains("class=\"dd-data-table -dense\""), "{html}");
        assert!(
            html.contains("data-label=\"Prioritized remediation tasks, scrollable\""),
            "{html}"
        );
        assert!(
            html.contains(
                "<caption class=\"dd-data-table__caption\">Prioritized remediation tasks</caption>"
            ),
            "{html}"
        );
        assert!(html.contains("scope=\"col\""), "{html}");
        assert!(html.contains("scope=\"row\""), "{html}");
        assert!(html.contains(">SEO-001</th>"), "{html}");
        assert!(
            html.contains("<span class=\"dd-badge -critical\"><span class=\"dd-badge__label\">Critical</span></span>"),
            "{html}"
        );
        assert!(html.contains("data-align=\"end\""), "{html}");
        assert!(html.contains("aria-sort=\"none\""), "{html}");
        assert!(html.contains("data-sort-key=\"priority\""), "{html}");
        assert!(html.contains("data-sort-key=\"effort\""), "{html}");
        assert!(html.contains("class=\"dd-data-table__sort\""), "{html}");
        assert!(!html.contains("tabindex="), "{html}");
        assert!(!html.contains("role=\"region\""), "{html}");
        assert!(!html.contains("id=\"task-table\""), "{html}");

        let empty = render_page_html(&page_with_component(SectionComponent::DataTable(
            DdDataTable {
                caption: "Empty table".to_string(),
                dense: false,
                scroll_label: Some("Empty table, scrollable".to_string()),
                empty_message: None,
                columns: vec![DataTableColumn {
                    label: "Name".to_string(),
                    align: DataTableAlign::Start,
                    sortable: false,
                }],
                rows: Vec::new(),
            },
        )))
        .expect("empty data table");
        assert!(
            empty.contains("class=\"dd-data-table__row -empty\""),
            "{empty}"
        );
        assert!(empty.contains("colspan=\"1\""), "{empty}");
        assert!(empty.contains("No data to display."), "{empty}");
        assert!(!empty.contains("class=\"dd-data-table -dense\""), "{empty}");
    }

    #[test]
    fn section_visual_options_emit_classes_id_and_sal() {
        use crate::model::{
            DdSection, Page, PageNode, SalAnimation, SectionBg, SectionClass, SectionColumn,
            SectionItemBoxClass, SectionPadding,
        };
        let mut section = DdSection {
            id: "work".to_string(),
            section_title: None,
            section_class: Some(SectionClass::FullFull),
            item_box_class: Some(SectionItemBoxClass::LBox),
            bg: Some(SectionBg::Muted),
            padding: Some(SectionPadding::NoPadding),
            custom_css: Some("extra-class".to_string()),
            aria_label: Some("Our work".to_string()),
            sal: SalAnimation::Fade,
            sal_duration: Some(600),
            sal_delay: None,
            columns: vec![SectionColumn {
                id: "column-1".to_string(),
                width_class: "dd-u-1-1".to_string(),
                components: Vec::new(),
            }],
        };
        let page = Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(section.clone())],
        };
        let html = render_page_html(&page).expect("section should render");
        assert!(html.contains("id=\"work\""), "{html}");
        assert!(html.contains("id=\"column-1\""), "{html}");
        assert!(html.contains("aria-label=\"Our work\""), "{html}");
        assert!(html.contains("-full-full"), "{html}");
        assert!(html.contains("-bg-muted"), "{html}");
        assert!(html.contains("-no-padding"), "{html}");
        assert!(html.contains("extra-class"), "{html}");
        assert!(html.contains("data-sal=\"fade\""), "{html}");
        assert!(html.contains("data-sal-duration=\"600\""), "{html}");

        section.aria_label = None;
        let page = Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(section)],
        };
        let html = render_page_html(&page).expect("section should render");
        assert!(html.contains("aria-label=\"Content section\""), "{html}");
    }

    #[test]
    fn hero_overlay_copy_position_id_and_aria() {
        use crate::model::{HeroCopyPosition, HeroOverlay, PageNode};
        let mut site = Site::starter();
        let PageNode::Hero(hero) = &mut site.pages[0].nodes[0] else {
            panic!("expected hero");
        };
        hero.overlay = Some(HeroOverlay::Dark);
        hero.copy_position = Some(HeroCopyPosition::Center);
        hero.id = Some("intro".to_string());
        hero.aria_label = Some("Welcome".to_string());
        let html = render_page_html(&site.pages[0]).expect("hero should render");
        assert!(
            html.contains("class=\"dd-hero -full-full -overlay-dark -center\""),
            "{html}"
        );
        assert!(html.contains("id=\"intro\""), "{html}");
        assert!(html.contains("aria-label=\"Welcome\""), "{html}");

        let PageNode::Hero(hero) = &mut site.pages[0].nodes[0] else {
            panic!("expected hero");
        };
        hero.overlay = None;
        hero.copy_position = None;
        hero.id = None;
        hero.aria_label = None;
        let html = render_page_html(&site.pages[0]).expect("hero should render");
        assert!(html.contains("class=\"dd-hero -full-full\""), "{html}");
        assert!(!html.contains("id=\"intro\""), "{html}");
        assert!(html.contains("aria-label=\"Introduction\""), "{html}");
        assert!(!html.contains("-overlay-"), "{html}");
    }

    #[test]
    fn cta_renders_authored_button_styles() {
        use crate::model::{
            ButtonStyle, CardLinkTarget, CtaClass, DdCta, DdLink, Page, PageNode, SalAnimation,
            SectionClass, SectionColumn, SectionComponent, SectionItemBoxClass,
        };
        let cta = DdCta {
            parent_class: CtaClass::TopLeft,
            parent_image_url: "/c.jpg".to_string(),
            parent_image_alt: "alt".to_string(),
            sal: SalAnimation::Fade,
            sal_duration: None,
            sal_delay: None,
            parent_title: "T".to_string(),
            parent_subtitle: "S".to_string(),
            parent_copy: "C".to_string(),
            links: vec![
                DdLink {
                    url: "/a".to_string(),
                    label: "One".to_string(),
                    target: CardLinkTarget::SelfTarget,
                    style: ButtonStyle::Primary,
                },
                DdLink {
                    url: "/b".to_string(),
                    label: "Two".to_string(),
                    target: CardLinkTarget::Blank,
                    style: ButtonStyle::Tertiary,
                },
            ],
            parent_link_url: None,
            parent_link_target: None,
            parent_link_label: None,
        };
        let page = Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(crate::model::DdSection {
                id: "s1".to_string(),
                section_title: None,
                section_class: Some(SectionClass::FullContained),
                item_box_class: Some(SectionItemBoxClass::LBox),
                bg: None,
                padding: None,
                custom_css: None,
                aria_label: None,
                sal: crate::model::SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
                columns: vec![SectionColumn {
                    id: "c1".to_string(),
                    width_class: "dd-u-1-1".to_string(),
                    components: vec![SectionComponent::Cta(cta)],
                }],
            })],
        };
        let html = render_page_html(&page).expect("cta page should render");
        assert!(html.contains("dd-button -primary"), "{html}");
        assert!(html.contains("dd-button -tertiary"), "{html}");
        assert!(html.contains(">One<"), "{html}");
        assert!(html.contains(">Two<"), "{html}");
        assert!(html.contains("target=\"_blank\""), "{html}");
    }

    #[test]
    fn banner_oembed_renders_youtube_iframe() {
        use crate::model::{
            BannerClass, DdBanner, Media, Page, PageNode, SalAnimation, SectionClass,
            SectionColumn, SectionComponent, SectionItemBoxClass,
        };
        let banner = DdBanner {
            parent_class: BannerClass::BgCenterCenter,
            sal: SalAnimation::Fade,
            sal_duration: None,
            sal_delay: None,
            parent_image_url: String::new(),
            parent_image_alt: String::new(),
            media: Media::Oembed {
                url: "https://youtu.be/dQw4w9wgGcQ".to_string(),
            },
        };
        let page = Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(crate::model::DdSection {
                id: "s1".to_string(),
                section_title: None,
                section_class: Some(SectionClass::FullContained),
                item_box_class: Some(SectionItemBoxClass::LBox),
                bg: None,
                padding: None,
                custom_css: None,
                aria_label: None,
                sal: crate::model::SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
                columns: vec![SectionColumn {
                    id: "c1".to_string(),
                    width_class: "dd-u-1-1".to_string(),
                    components: vec![SectionComponent::Banner(banner)],
                }],
            })],
        };
        let html = render_page_html(&page).expect("banner should render");
        assert!(
            html.contains("youtube-nocookie.com/embed/dQw4w9wgGcQ"),
            "{html}"
        );
        assert!(!html.contains("background-image"), "{html}");
    }

    #[test]
    fn card_items_stagger_sal_delay() {
        use crate::model::{
            CardItem, CardLinkTarget, CardType, DdCard, Page, PageNode, SalAnimation, SectionClass,
            SectionColumn, SectionComponent, SectionItemBoxClass,
        };

        let card = DdCard {
            parent_type: CardType::Default,
            sal: SalAnimation::SlideUp,
            sal_duration: None,
            sal_delay: None,
            parent_width: "dd-u-1-1".to_string(),
            items: vec![
                CardItem {
                    child_image_url: "/a.jpg".to_string(),
                    child_image_alt: "a".to_string(),
                    child_title: "A".to_string(),
                    child_subtitle: String::new(),
                    child_copy: "one".to_string(),
                    child_link_url: None,
                    child_link_target: Some(CardLinkTarget::SelfTarget),
                    child_link_label: None,
                    child_link_style: crate::model::ButtonStyle::Primary,
                },
                CardItem {
                    child_image_url: "/b.jpg".to_string(),
                    child_image_alt: "b".to_string(),
                    child_title: "B".to_string(),
                    child_subtitle: String::new(),
                    child_copy: "two".to_string(),
                    child_link_url: None,
                    child_link_target: Some(CardLinkTarget::SelfTarget),
                    child_link_label: None,
                    child_link_style: crate::model::ButtonStyle::Primary,
                },
            ],
        };
        let page = Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(crate::model::DdSection {
                id: "s1".to_string(),
                section_title: None,
                section_class: Some(SectionClass::FullContained),
                item_box_class: Some(SectionItemBoxClass::LBox),
                bg: None,
                padding: None,
                custom_css: None,
                aria_label: None,
                sal: crate::model::SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
                columns: vec![SectionColumn {
                    id: "c1".to_string(),
                    width_class: "dd-u-1-1".to_string(),
                    components: vec![SectionComponent::Card(card)],
                }],
            })],
        };
        let html = render_page_html(&page).expect("card page should render");
        assert!(html.contains("data-sal=\"slide-up\""));
        assert!(!html.contains("data-aos"));
        assert!(html.contains("data-sal-delay=\"100\""));
        let first = html.find("dd-card__item").expect("item");
        let delay = html.find("data-sal-delay=\"100\"").expect("stagger");
        assert!(delay > first, "delay should land on a later card item");
    }

    #[test]
    fn no_animation_omits_data_sal_attributes() {
        use crate::model::{
            BannerClass, DdBanner, Page, PageNode, SalAnimation, SectionClass, SectionColumn,
            SectionComponent, SectionItemBoxClass,
        };
        let banner = DdBanner {
            parent_class: BannerClass::BgCenterCenter,
            sal: SalAnimation::NoAnimation,
            sal_duration: Some(600),
            sal_delay: Some(200),
            parent_image_url: "/b.jpg".to_string(),
            parent_image_alt: "b".to_string(),
            media: crate::model::Media::None,
        };
        let page = Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(crate::model::DdSection {
                id: "s1".to_string(),
                section_title: None,
                section_class: Some(SectionClass::FullContained),
                item_box_class: Some(SectionItemBoxClass::LBox),
                bg: None,
                padding: None,
                custom_css: None,
                aria_label: None,
                sal: crate::model::SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
                columns: vec![SectionColumn {
                    id: "c1".to_string(),
                    width_class: "dd-u-1-1".to_string(),
                    components: vec![SectionComponent::Banner(banner)],
                }],
            })],
        };
        let html = render_page_html(&page).expect("banner page should render");
        assert!(!html.contains("data-sal"), "{html}");
        assert!(!html.contains("data-sal-duration"), "{html}");
        assert!(!html.contains("data-sal-delay"), "{html}");
    }

    #[test]
    fn sal_duration_emits_when_not_css_default() {
        use crate::model::{
            BannerClass, DdBanner, Page, PageNode, SalAnimation, SectionClass, SectionColumn,
            SectionComponent, SectionItemBoxClass,
        };
        let banner = DdBanner {
            parent_class: BannerClass::BgCenterCenter,
            sal: SalAnimation::Fade,
            sal_duration: Some(600),
            sal_delay: Some(150),
            parent_image_url: "/b.jpg".to_string(),
            parent_image_alt: "b".to_string(),
            media: crate::model::Media::None,
        };
        let page = Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(crate::model::DdSection {
                id: "s1".to_string(),
                section_title: None,
                section_class: Some(SectionClass::FullContained),
                item_box_class: Some(SectionItemBoxClass::LBox),
                bg: None,
                padding: None,
                custom_css: None,
                aria_label: None,
                sal: crate::model::SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
                columns: vec![SectionColumn {
                    id: "c1".to_string(),
                    width_class: "dd-u-1-1".to_string(),
                    components: vec![SectionComponent::Banner(banner)],
                }],
            })],
        };
        let html = render_page_html(&page).expect("banner page should render");
        assert!(html.contains("data-sal=\"fade\""), "{html}");
        assert!(html.contains("data-sal-duration=\"600\""), "{html}");
        assert!(html.contains("data-sal-delay=\"150\""), "{html}");
    }

    #[test]
    fn card_stagger_adds_author_delay() {
        use crate::model::{
            CardItem, CardLinkTarget, CardType, DdCard, Page, PageNode, SalAnimation, SectionClass,
            SectionColumn, SectionComponent, SectionItemBoxClass,
        };

        let card = DdCard {
            parent_type: CardType::Default,
            sal: SalAnimation::Fade,
            sal_duration: None,
            sal_delay: Some(200),
            parent_width: "dd-u-1-1".to_string(),
            items: vec![
                CardItem {
                    child_image_url: "/a.jpg".to_string(),
                    child_image_alt: "a".to_string(),
                    child_title: "A".to_string(),
                    child_subtitle: String::new(),
                    child_copy: "one".to_string(),
                    child_link_url: None,
                    child_link_target: Some(CardLinkTarget::SelfTarget),
                    child_link_label: None,
                    child_link_style: crate::model::ButtonStyle::Primary,
                },
                CardItem {
                    child_image_url: "/b.jpg".to_string(),
                    child_image_alt: "b".to_string(),
                    child_title: "B".to_string(),
                    child_subtitle: String::new(),
                    child_copy: "two".to_string(),
                    child_link_url: None,
                    child_link_target: Some(CardLinkTarget::SelfTarget),
                    child_link_label: None,
                    child_link_style: crate::model::ButtonStyle::Primary,
                },
            ],
        };
        let page = Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(crate::model::DdSection {
                id: "s1".to_string(),
                section_title: None,
                section_class: Some(SectionClass::FullContained),
                item_box_class: Some(SectionItemBoxClass::LBox),
                bg: None,
                padding: None,
                custom_css: None,
                aria_label: None,
                sal: crate::model::SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
                columns: vec![SectionColumn {
                    id: "c1".to_string(),
                    width_class: "dd-u-1-1".to_string(),
                    components: vec![SectionComponent::Card(card)],
                }],
            })],
        };
        let html = render_page_html(&page).expect("card page should render");
        assert!(html.contains("data-sal-delay=\"200\""), "{html}");
        assert!(html.contains("data-sal-delay=\"300\""), "{html}");
    }

    #[test]
    fn auto_canonical_and_og_url_from_base_url() {
        let mut site = Site::starter();
        site.base_url = Some("https://ex.com".to_string());
        site.lang = "fr".to_string();
        let r = crate::templates::Renderer::bundled_only().unwrap();
        let header = super::render_header(&r, &site.header).unwrap();
        let footer = super::render_footer(&r, &site.footer, &site.name).unwrap();
        assert!(
            !header.contains("dd-alert"),
            "starter header must not render dd-alert: {header}"
        );
        let html = super::render_page_html_with_chrome(&r, &site.pages[0], &header, &footer, &site)
            .unwrap();
        assert!(html.contains("lang=\"fr\""));
        assert!(html.contains("rel=\"canonical\" href=\"https://ex.com/\""));
        assert!(html.contains("property=\"og:url\" content=\"https://ex.com/\""));
        assert!(html.contains("property=\"og:title\" content=\"Home\""));
        assert!(html.contains("<title>Home</title>"));
        assert!(html.contains("©"));
        assert!(html.contains("My Site"));
        assert!(!html.contains("dd-header__cta"));
        assert!(!html.contains("googletagmanager.com"));
    }

    #[test]
    fn pretty_nested_page_prefixes_assets_and_canonical() {
        let mut site = Site::starter();
        site.pretty_urls = true;
        site.base_url = Some("https://ex.com".to_string());
        site.pages[0].slug = "blog/entry".to_string();
        let r = crate::templates::Renderer::bundled_only().unwrap();
        let html = super::render_page_html_with_chrome(&r, &site.pages[0], "", "", &site).unwrap();
        assert!(html.contains("../../assets/css/style.min.css"), "{html}");
        assert!(html.contains("../../assets/js/main.min.js"), "{html}");
        assert!(
            html.contains("data-search-index=\"/search-index.json\"")
                || html.contains("data-search-index=\"../../search-index.json\""),
            "{html}"
        );
        assert!(html.contains("class=\"page page-blog-entry\""));
        assert!(html.contains("https://ex.com/blog/entry/"));
        assert!(!html.contains("blog/entry/index.html"), "{html}");
        assert!(
            html.contains("content=\"width=device-width, initial-scale=1.0\""),
            "viewport must not get a ../ prefix: {html}"
        );
        assert!(
            html.contains("content=\"index, follow\""),
            "robots must not get a ../ prefix: {html}"
        );
        assert!(
            html.contains("property=\"og:title\" content=\"Home\""),
            "og:title must not get a ../ prefix: {html}"
        );
        assert!(
            html.contains("property=\"og:type\" content=\"website\""),
            "og:type must not get a ../ prefix: {html}"
        );
        assert!(
            html.contains("name=\"theme-color\" content=\"#ffffff\""),
            "{html}"
        );
    }

    #[test]
    fn nested_page_joins_typed_canonical_path_with_base_url() {
        let mut site = Site::starter();
        site.pretty_urls = true;
        site.base_url = Some("https://www.ldnddev.com".to_string());
        site.pages[0].slug = "services".to_string();
        site.pages[0].head.canonical_url = Some("/page/".to_string());
        site.pages[0].head.meta_description = Some("We build sites.".to_string());
        site.pages[0].head.og_image = Some("assets/images/og.jpg".to_string());
        let r = crate::templates::Renderer::bundled_only().unwrap();
        let html = super::render_page_html_with_chrome(&r, &site.pages[0], "", "", &site).unwrap();
        assert!(
            html.contains("rel=\"canonical\" href=\"https://www.ldnddev.com/page/\""),
            "{html}"
        );
        assert!(
            html.contains("property=\"og:url\" content=\"https://www.ldnddev.com/page/\""),
            "{html}"
        );
        assert!(
            html.contains("content=\"We build sites.\""),
            "description must not get a ../ prefix: {html}"
        );
        assert!(
            html.contains(
                "property=\"og:image\" content=\"https://www.ldnddev.com/assets/images/og.jpg\""
            ),
            "{html}"
        );
    }

    #[test]
    fn prefix_relative_urls_skips_absolute_and_hash() {
        let html = r##"<a href="contact.html">c</a><a href="https://x.com">x</a><a href="#top">t</a><img src="assets/a.jpg"><html data-search-index="search-index.json"><meta name="viewport" content="width=device-width, initial-scale=1.0"><meta name="robots" content="index, follow"><meta name="description" content="Hello world"><meta property="og:type" content="website">"##;
        let out = super::prefix_relative_urls(html, "../");
        assert!(out.contains("href=\"../contact.html\""));
        assert!(out.contains("href=\"https://x.com\""));
        assert!(out.contains("href=\"#top\""));
        assert!(out.contains("src=\"../assets/a.jpg\""));
        assert!(out.contains("data-search-index=\"../search-index.json\""));
        assert!(out.contains("content=\"width=device-width, initial-scale=1.0\""));
        assert!(out.contains("content=\"index, follow\""));
        assert!(out.contains("content=\"Hello world\""));
        assert!(out.contains("content=\"website\""));
        assert!(!out.contains("content=\"../"));
    }

    #[test]
    fn header_cta_banner_footer_socials_and_gtm_render() {
        let mut site = Site::starter();
        site.header.cta_url = Some("/contact.html".to_string());
        site.header.cta_label = Some("Contact us".to_string());
        site.header.banner = Some("Office closed Friday".to_string());
        site.footer.blurb = Some("Hello\nworld".to_string());
        site.footer.copyright = Some("© Custom Co".to_string());
        site.footer.social_github = Some("https://github.com/ldnddev".to_string());
        site.header_gtm_tag = Some(
            "<script src=\"https://www.googletagmanager.com/gtm.js?id=GTM-ABC123\"></script>"
                .to_string(),
        );
        site.body_gtm_tag = Some("id=GTM-ABC123".to_string());
        let r = crate::templates::Renderer::bundled_only().unwrap();
        let header = super::render_header(&r, &site.header).unwrap();
        let footer = super::render_footer(&r, &site.footer, &site.name).unwrap();
        assert!(header.contains("class=\"dd-header__banner\""), "{header}");
        assert!(header.contains("Office closed Friday"), "{header}");
        assert!(header.contains("class=\"dd-header__cta\""), "{header}");
        assert!(
            header.contains("href=\"/contact.html\"") && header.contains("Contact us"),
            "{header}"
        );
        assert!(footer.contains("Hello<br>world"), "{footer}");
        assert!(footer.contains("© Custom Co"), "{footer}");
        assert!(footer.contains("fa-github"), "{footer}");
        assert!(footer.contains("aria-label=\"Social\""), "{footer}");
        assert!(!footer.contains("fa-linkedin"), "{footer}");

        let html = super::render_page_html_with_chrome(&r, &site.pages[0], &header, &footer, &site)
            .unwrap();
        assert!(
            html.contains("https://www.googletagmanager.com/gtm.js?id='+i"),
            "{html}"
        );
        assert!(html.contains("'GTM-ABC123'"), "{html}");
        assert!(
            html.contains("https://www.googletagmanager.com/ns.html?id=GTM-ABC123"),
            "{html}"
        );
        assert!(html.contains("title=\"Google Tag Manager\""), "{html}");
    }

    #[test]
    fn html_title_uses_meta_title_when_set() {
        let mut site = Site::starter();
        site.pages[0].head.title = "Home".to_string();
        site.pages[0].head.meta_title =
            Some("Custom Drupal & WordPress Development | ldnddev".to_string());
        let r = crate::templates::Renderer::bundled_only().unwrap();
        let html = super::render_page_html_with_chrome(&r, &site.pages[0], "", "", &site).unwrap();
        assert!(
            html.contains("<title>Custom Drupal &amp; WordPress Development | ldnddev</title>")
                || html.contains("<title>Custom Drupal & WordPress Development | ldnddev</title>")
        );
        assert!(!html.contains("<title>Home</title>"));
        assert!(html.contains(
            "property=\"og:title\" content=\"Custom Drupal &amp; WordPress Development | ldnddev\""
        ) || html.contains(
            "property=\"og:title\" content=\"Custom Drupal & WordPress Development | ldnddev\""
        ));
    }

    #[test]
    fn og_description_uses_meta_description_when_og_is_empty() {
        let mut site = Site::starter();
        site.pages[0].head.og_description = None;
        site.pages[0].head.meta_description = Some("We build sites.".to_string());
        let r = crate::templates::Renderer::bundled_only().unwrap();
        let html = super::render_page_html_with_chrome(&r, &site.pages[0], "", "", &site).unwrap();
        assert!(
            html.contains("property=\"og:description\" content=\"We build sites.\""),
            "{html}"
        );
    }

    #[test]
    fn og_fields_prefer_explicit_values_over_meta() {
        let mut site = Site::starter();
        site.pages[0].head.meta_title = Some("Meta Title".to_string());
        site.pages[0].head.meta_description = Some("Meta Description".to_string());
        site.pages[0].head.og_title = Some("OG Title".to_string());
        site.pages[0].head.og_description = Some("OG Description".to_string());
        let r = crate::templates::Renderer::bundled_only().unwrap();
        let html = super::render_page_html_with_chrome(&r, &site.pages[0], "", "", &site).unwrap();
        assert!(
            html.contains("property=\"og:title\" content=\"OG Title\""),
            "{html}"
        );
        assert!(
            html.contains("property=\"og:description\" content=\"OG Description\""),
            "{html}"
        );
        assert!(!html.contains("property=\"og:title\" content=\"Meta Title\""));
        assert!(!html.contains("property=\"og:description\" content=\"Meta Description\""));
    }

    fn page_with_image(dark: Option<&str>, link: Option<&str>) -> crate::model::Page {
        use crate::model::{
            CardLinkTarget, DdImage, DdSection, Page, PageNode, SalAnimation, SectionClass,
            SectionColumn, SectionComponent, SectionItemBoxClass,
        };
        Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(DdSection {
                id: "s1".to_string(),
                section_title: None,
                section_class: Some(SectionClass::FullContained),
                item_box_class: Some(SectionItemBoxClass::LBox),
                bg: None,
                padding: None,
                custom_css: None,
                aria_label: None,
                sal: crate::model::SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
                columns: vec![SectionColumn {
                    id: "c1".to_string(),
                    width_class: "dd-u-1-1".to_string(),
                    components: vec![SectionComponent::Image(DdImage {
                        sal: SalAnimation::Fade,
                        sal_duration: None,
                        sal_delay: None,
                        parent_image_url: "/light.jpg".to_string(),
                        parent_image_url_dark: dark.map(str::to_string),
                        parent_image_alt: "Alt text".to_string(),
                        parent_link_url: link.map(str::to_string),
                        parent_link_target: link.map(|_| CardLinkTarget::SelfTarget),
                    })],
                }],
            })],
        }
    }

    #[test]
    fn image_renders_picture_with_dark_source() {
        let html = render_page_html(&page_with_image(Some("/dark.jpg"), None))
            .expect("image page should render");
        assert!(html.contains("<picture>"));
        assert!(
            html.contains(r#"<source srcset="/dark.jpg" media="(prefers-color-scheme: dark)">"#)
        );
        assert!(html.contains(r#"<img src="/light.jpg" alt="Alt text" class="dd-img""#));
        assert!(!html.contains("<a href="));
    }

    #[test]
    fn image_without_dark_omits_source_but_keeps_picture() {
        let html =
            render_page_html(&page_with_image(None, None)).expect("image page should render");
        assert!(html.contains("<picture>"));
        assert!(!html.contains("prefers-color-scheme: dark"));
        assert!(html.contains(r#"<img src="/light.jpg" alt="Alt text" class="dd-img""#));
    }

    #[test]
    fn image_with_link_wraps_picture() {
        let html = render_page_html(&page_with_image(Some("/dark.jpg"), Some("/about.html")))
            .expect("linked image page should render");
        let link_at = html.find(r#"<a href="/about.html""#).expect("link");
        let picture_at = html.find("<picture>").expect("picture");
        let source_at = html
            .find(r#"<source srcset="/dark.jpg" media="(prefers-color-scheme: dark)">"#)
            .expect("dark source");
        assert!(picture_at > link_at, "picture should sit inside the link");
        assert!(source_at > picture_at);
    }

    fn page_with_alternating(subtitle: &str) -> crate::model::Page {
        use crate::model::{
            AlternatingItem, AlternatingType, DdAlternating, DdSection, Page, PageNode,
            SalAnimation, SectionClass, SectionColumn, SectionComponent, SectionItemBoxClass,
        };
        Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(DdSection {
                id: "s1".to_string(),
                section_title: None,
                section_class: Some(SectionClass::FullContained),
                item_box_class: Some(SectionItemBoxClass::LBox),
                bg: None,
                padding: None,
                custom_css: None,
                aria_label: None,
                sal: crate::model::SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
                columns: vec![SectionColumn {
                    id: "c1".to_string(),
                    width_class: "dd-u-1-1".to_string(),
                    components: vec![SectionComponent::Alternating(DdAlternating {
                        parent_type: AlternatingType::Default,
                        parent_class: "-default".to_string(),
                        sal: SalAnimation::Fade,
                        sal_duration: None,
                        sal_delay: None,
                        items: vec![AlternatingItem {
                            child_image_url: "/a.jpg".to_string(),
                            child_image_alt: "a".to_string(),
                            child_title: "Title".to_string(),
                            child_subtitle: subtitle.to_string(),
                            child_copy: "Copy".to_string(),
                            links: Vec::new(),
                            media: crate::model::Media::None,
                        }],
                    })],
                }],
            })],
        }
    }

    #[test]
    fn alternating_renders_subtitle_when_set() {
        let html = render_page_html(&page_with_alternating("A kicker"))
            .expect("alternating page should render");
        assert!(html.contains("dd-alternating__subtitle"));
        assert!(html.contains("A kicker"));
    }

    #[test]
    fn alternating_omits_subtitle_when_empty() {
        let html =
            render_page_html(&page_with_alternating("")).expect("alternating page should render");
        assert!(!html.contains("dd-alternating__subtitle"));
    }

    fn page_with_navigation(items: Vec<crate::model::NavigationItem>) -> crate::model::Page {
        use crate::model::{
            DdNavigation, DdSection, NavigationClass, NavigationType, Page, PageNode, SalAnimation,
            SectionClass, SectionColumn, SectionComponent, SectionItemBoxClass,
        };
        Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(DdSection {
                id: "s1".to_string(),
                section_title: None,
                section_class: Some(SectionClass::FullContained),
                item_box_class: Some(SectionItemBoxClass::LBox),
                bg: None,
                padding: None,
                custom_css: None,
                aria_label: None,
                sal: crate::model::SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
                columns: vec![SectionColumn {
                    id: "c1".to_string(),
                    width_class: "dd-u-1-1".to_string(),
                    components: vec![SectionComponent::Navigation(DdNavigation {
                        parent_type: NavigationType::HeaderNav,
                        parent_class: NavigationClass::MainMenu,
                        sal: SalAnimation::Fade,
                        sal_duration: None,
                        sal_delay: None,
                        items,
                    })],
                }],
            })],
        }
    }

    fn nav_link(
        label: &str,
        url: &str,
        children: Vec<crate::model::NavigationItem>,
    ) -> crate::model::NavigationItem {
        crate::model::NavigationItem {
            child_kind: crate::model::NavigationKind::Link,
            child_link_label: label.to_string(),
            child_link_url: Some(url.to_string()),
            child_link_target: Some(crate::model::CardLinkTarget::SelfTarget),
            child_link_css: None,
            items: children,
        }
    }

    fn nav_button(
        label: &str,
        url: &str,
        children: Vec<crate::model::NavigationItem>,
    ) -> crate::model::NavigationItem {
        crate::model::NavigationItem {
            child_kind: crate::model::NavigationKind::Button,
            child_link_label: label.to_string(),
            child_link_url: Some(url.to_string()),
            child_link_target: Some(crate::model::CardLinkTarget::SelfTarget),
            child_link_css: None,
            items: children,
        }
    }

    #[test]
    fn navigation_renders_link_item() {
        let html = render_page_html(&page_with_navigation(vec![nav_link("Home", "/", vec![])]))
            .expect("nav page should render");
        assert!(html.contains(r#"<li class="menu-item">"#));
        assert!(html.contains(r#"<a href="/" target="_self" class="">Home</a>"#));
        assert!(!html.contains("sub-menu"));
        assert!(!html.contains("-has-children"));
    }

    #[test]
    fn navigation_renders_nested_button_and_escapes_label() {
        let html = render_page_html(&page_with_navigation(vec![nav_button(
            "More & Extra",
            "/more",
            vec![nav_link("About", "/about.html", vec![])],
        )]))
        .expect("nested nav should render");
        assert!(html.contains(r#"<li class="menu-item -has-children">"#));
        assert!(html.contains(
            r#"<a href="/more" target="_self" class=""><span class="dd-button" role="presentation">More &amp; Extra</span></a>"#
        ));
        assert!(html.contains(r#"<ul class="sub-menu">"#));
        assert!(html.contains(r#"<a href="/about.html" target="_self" class="">About</a>"#));
    }

    #[test]
    fn markdown_image_attribute_sets_class() {
        let html = super::markdown_to_html(
            "![Sample estimate table listing the agency fee by role with an amount for each](/assets/images/projectscope/reading-your-estimate/reading-your-estimate-01-fee-by-role.png){.dd-img}",
        );
        assert!(
            html.contains(
                r#"<img src="/assets/images/projectscope/reading-your-estimate/reading-your-estimate-01-fee-by-role.png" alt="Sample estimate table listing the agency fee by role with an amount for each" class="dd-img" />"#
            ),
            "{html}"
        );
        assert!(!html.contains("{.dd-img}"), "{html}");
    }

    #[test]
    fn markdown_renders_headings_lists_rules_and_inline() {
        let html = super::markdown_to_html(
            "# Hello\n\n## World\n\n### Three\n\n#### Four\n\n##### Five\n\n###### Six\n\n- a\n- b\n\n1. one\n2. two\n\n---\n\n**bold** *italic* `code` [x](https://example.com)\n\n~~strike~~",
        );
        assert!(html.contains("<h1>Hello</h1>"), "{html}");
        assert!(html.contains("<h2>World</h2>"), "{html}");
        assert!(html.contains("<h3>Three</h3>"), "{html}");
        assert!(html.contains("<h4>Four</h4>"), "{html}");
        assert!(html.contains("<h5>Five</h5>"), "{html}");
        assert!(html.contains("<h6>Six</h6>"), "{html}");
        assert!(html.contains("<ul>"), "{html}");
        assert!(html.contains("<li>a</li>"), "{html}");
        assert!(html.contains("<ol>"), "{html}");
        assert!(html.contains("<li>one</li>"), "{html}");
        assert!(html.contains("<hr"), "{html}");
        assert!(html.contains("<strong>bold</strong>"), "{html}");
        assert!(html.contains("<em>italic</em>"), "{html}");
        assert!(html.contains("<code>code</code>"), "{html}");
        assert!(
            html.contains(r#"<a href="https://example.com">x</a>"#),
            "{html}"
        );
        assert!(html.contains("<del>strike</del>"), "{html}");
    }

    #[test]
    fn markdown_renders_setext_headings_plus_lists_and_rules_without_blank_lines() {
        let html = super::markdown_to_html(
            "This is a copy block for you.  \n# h1\n## h2\n### h3\nh1  \n==\nh2  \n--\n**bold**\n*italic*\n`code`\n- item 1\n- item 2\n+ item 5\n1. item 7\n2. item 8\n---  \n[test](https://www.google.com/)  \n***\n",
        );
        assert!(html.contains("<h1>h1</h1>"), "{html}");
        assert!(html.contains("<h2>h2</h2>"), "{html}");
        assert!(html.contains("<h3>h3</h3>"), "{html}");
        assert!(html.contains("<ul>"), "{html}");
        assert!(html.contains("<li>item 1</li>"), "{html}");
        assert!(html.contains("<li>item 5</li>"), "{html}");
        assert!(html.contains("<ol>"), "{html}");
        assert!(html.contains("<li>item 7</li>"), "{html}");
        assert!(html.contains("<hr"), "{html}");
        assert!(
            html.contains(r#"<a href="https://www.google.com/">test</a>"#),
            "{html}"
        );
        assert!(!html.contains("<p># h1</p>"), "{html}");
    }

    #[test]
    fn markdown_passthrough_html_blocks() {
        let html = super::markdown_to_html("<div class=\"note\">raw</div>\n\nNext");
        assert!(html.contains(r#"<div class="note">raw</div>"#), "{html}");
        assert!(html.contains("<p>Next</p>"), "{html}");
    }

    fn page_with_component(component: crate::model::SectionComponent) -> crate::model::Page {
        use crate::model::{
            DdSection, Page, PageNode, SectionClass, SectionColumn, SectionItemBoxClass,
        };
        Page {
            id: "p".to_string(),
            slug: "index".to_string(),
            slug_locked: false,
            head: crate::model::Site::starter().pages[0].head.clone(),
            nodes: vec![PageNode::Section(DdSection {
                id: "s1".to_string(),
                section_title: None,
                section_class: Some(SectionClass::FullContained),
                item_box_class: Some(SectionItemBoxClass::LBox),
                bg: None,
                padding: None,
                custom_css: None,
                aria_label: None,
                sal: crate::model::SalAnimation::NoAnimation,
                sal_duration: None,
                sal_delay: None,
                columns: vec![SectionColumn {
                    id: "c1".to_string(),
                    width_class: "dd-u-1-1".to_string(),
                    components: vec![component],
                }],
            })],
        }
    }

    fn assert_unescaped_markdown_copy(html: &str, label: &str) {
        assert!(
            html.contains("<h1>Hello</h1>"),
            "{label} missing heading: {html}"
        );
        assert!(html.contains("<ul>"), "{label} missing list: {html}");
        assert!(
            html.contains("<li>one</li>"),
            "{label} missing item: {html}"
        );
        assert!(
            !html.contains("&lt;h1&gt;"),
            "{label} escaped markdown HTML: {html}"
        );
    }

    #[test]
    fn rich_text_copy_emits_unescaped_markdown_html() {
        use crate::model::{DdRichText, SalAnimation, SectionComponent};
        let html = render_page_html(&page_with_component(SectionComponent::RichText(
            DdRichText {
                parent_class: None,
                sal: SalAnimation::Fade,
                sal_duration: None,
                sal_delay: None,
                parent_copy: "# Hello\n\n- one\n- two\n\n---\n".to_string(),
            },
        )))
        .expect("rich text page should render");
        assert!(
            html.contains(r#"<div class="dd-rich_text__copy">"#),
            "{html}"
        );
        assert_unescaped_markdown_copy(&html, "dd-rich_text");
        assert!(html.contains("<hr"), "{html}");
    }

    #[test]
    fn expand_textarea_components_render_markdown_copy() {
        use crate::model::*;
        let md = "# Hello\n\n- one\n".to_string();
        let components = [
            (
                "dd-cta",
                SectionComponent::Cta(DdCta {
                    parent_class: CtaClass::TopLeft,
                    parent_image_url: "/a.jpg".to_string(),
                    parent_image_alt: "alt".to_string(),
                    sal: SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_title: "Title".to_string(),
                    parent_subtitle: "Subtitle".to_string(),
                    parent_copy: md.clone(),
                    links: Vec::new(),
                    parent_link_url: None,
                    parent_link_target: None,
                    parent_link_label: None,
                }),
            ),
            (
                "dd-card",
                SectionComponent::Card(DdCard {
                    parent_type: CardType::Default,
                    sal: SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_width: "dd-u-1-1".to_string(),
                    items: vec![CardItem {
                        child_image_url: "/a.jpg".to_string(),
                        child_image_alt: "alt".to_string(),
                        child_title: "Title".to_string(),
                        child_subtitle: "Subtitle".to_string(),
                        child_copy: md.clone(),
                        child_link_url: None,
                        child_link_target: None,
                        child_link_label: None,
                        child_link_style: crate::model::ButtonStyle::Primary,
                    }],
                }),
            ),
            (
                "dd-alert",
                SectionComponent::Alert(DdAlert {
                    parent_type: AlertType::Default,
                    parent_class: AlertClass::Default,
                    sal: SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_title: Some("Title".to_string()),
                    parent_copy: md.clone(),
                }),
            ),
            (
                "dd-modal",
                SectionComponent::Modal(DdModal {
                    parent_title: "Title".to_string(),
                    parent_copy: md.clone(),
                }),
            ),
            (
                "dd-blockquote",
                SectionComponent::Blockquote(DdBlockquote {
                    sal: SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_image_url: "/a.jpg".to_string(),
                    parent_image_alt: "alt".to_string(),
                    parent_name: "Name".to_string(),
                    parent_role: "Role".to_string(),
                    parent_copy: md.clone(),
                }),
            ),
            (
                "dd-accordion",
                SectionComponent::Accordion(DdAccordion {
                    parent_type: AccordionType::Default,
                    parent_class: AccordionClass::Primary,
                    sal: SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_group_name: "group1".to_string(),
                    items: vec![AccordionItem {
                        child_title: "Title".to_string(),
                        child_copy: md.clone(),
                    }],
                    multiple: Some(false),
                }),
            ),
            (
                "dd-alternating",
                SectionComponent::Alternating(DdAlternating {
                    parent_type: AlternatingType::Default,
                    parent_class: "-default".to_string(),
                    sal: SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    items: vec![AlternatingItem {
                        child_image_url: "/a.jpg".to_string(),
                        child_image_alt: "alt".to_string(),
                        child_title: "Title".to_string(),
                        child_subtitle: "Subtitle".to_string(),
                        child_copy: md.clone(),
                        links: Vec::new(),
                        media: Media::None,
                    }],
                }),
            ),
            (
                "dd-milestones",
                SectionComponent::Milestones(DdMilestones {
                    sal: SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_width: "dd-u-1-1".to_string(),
                    items: vec![MilestonesItem {
                        child_percentage: "70".to_string(),
                        child_title: "Title".to_string(),
                        child_subtitle: "Subtitle".to_string(),
                        child_copy: md.clone(),
                        child_link_url: None,
                        child_link_target: None,
                        child_link_label: None,
                        child_link_style: crate::model::ButtonStyle::Primary,
                    }],
                }),
            ),
            (
                "dd-slider",
                SectionComponent::Slider(DdSlider {
                    parent_title: "Title".to_string(),
                    items: vec![SliderItem {
                        child_title: "Title".to_string(),
                        child_copy: md.clone(),
                        links: Vec::new(),
                        child_link_url: None,
                        child_link_target: None,
                        child_link_label: None,
                        child_image_url: "/a.jpg".to_string(),
                        child_image_alt: "alt".to_string(),
                        media: crate::model::Media::None,
                    }],
                }),
            ),
            (
                "dd-tabs",
                SectionComponent::Tabs(DdTabs {
                    parent_id: "tabs".to_string(),
                    parent_class: TabsOrientation::Horizontal,
                    aria_label: None,
                    sal: SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    items: vec![TabsItem {
                        child_title: "Tab".to_string(),
                        child_copy: md.clone(),
                    }],
                }),
            ),
            (
                "dd-timeline",
                SectionComponent::Timeline(DdTimeline {
                    aria_label: None,
                    sal: SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    items: vec![TimelineItem {
                        child_year: "2024".to_string(),
                        child_datetime: None,
                        child_title: "Title".to_string(),
                        heading_level: 3,
                        child_copy: md.clone(),
                        child_image_url: None,
                        child_image_alt: None,
                    }],
                }),
            ),
        ];
        for (label, component) in components {
            let html = render_page_html(&page_with_component(component))
                .unwrap_or_else(|e| panic!("{label} should render: {e}"));
            assert_unescaped_markdown_copy(&html, label);
        }
    }
}
