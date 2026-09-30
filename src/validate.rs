use crate::model::{
    DdLink, DdSection, Media, NavigationItem, NavigationKind, PageNode, SalAnimation,
    SectionComponent, Site, parse_oembed_url,
};

pub fn validate_site(site: &Site) -> Vec<String> {
    let mut errors = Vec::new();
    let mut slugs = std::collections::HashSet::new();
    let mut outputs = std::collections::HashSet::new();

    if site.pages.is_empty() {
        errors.push("Site must include at least one page.".to_string());
    }

    validate_header(&site.header, &mut errors);
    validate_footer(&site.footer, &mut errors);
    validate_gtm_snippet(
        site.header_gtm_tag.as_deref(),
        "header GTM snippet",
        &mut errors,
    );
    validate_gtm_snippet(
        site.body_gtm_tag.as_deref(),
        "body GTM snippet",
        &mut errors,
    );

    for page in &site.pages {
        if page.head.title.trim().is_empty() {
            errors.push(format!("Page '{}' is missing a head title.", page.id));
        }
        if page.slug.trim().is_empty() {
            errors.push(format!("Page '{}' has an empty slug.", page.id));
        } else if !crate::model::is_safe_page_slug(&page.slug) {
            errors.push(format!(
                "Page '{}' has an unsafe slug '{}'. Use lowercase letters, numbers, hyphens, and / between folders. Do not use 'index' as a folder name.",
                page.id, page.slug
            ));
        }
        if !page.slug.trim().is_empty() && !slugs.insert(page.slug.clone()) {
            errors.push(format!("Duplicate page slug '{}'.", page.slug));
        }
        if crate::model::is_safe_page_slug(&page.slug) {
            let out = crate::model::page_file_name(&page.slug, site.pretty_urls);
            if !outputs.insert(out.clone()) {
                errors.push(format!("Duplicate page output path '{}'.", out));
            }
        }
        if page.nodes.is_empty() {
            errors.push(format!("Page '{}' has no page nodes.", page.id));
        }
        let mut html_ids = std::collections::HashSet::new();

        for node in &page.nodes {
            match node {
                PageNode::Hero(hero) => {
                    if hero.parent_title.trim().is_empty() {
                        errors.push(format!("Page '{}' hero is missing parent_title.", page.id));
                    }
                    if let Some(id) = hero.id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                        if !crate::model::is_safe_slug(id) {
                            errors.push(format!(
                                "Page '{}' hero has an unsafe id '{}'. Use lowercase letters, numbers, and hyphens only.",
                                page.id, id
                            ));
                        } else if !html_ids.insert(id.to_string()) {
                            errors.push(format!("Page '{}' has duplicate id '{}'.", page.id, id));
                        }
                    }
                    validate_media(
                        &hero.resolved_media(),
                        &format!("Page '{}' hero", page.id),
                        &mut errors,
                    );
                    validate_dd_links(
                        &hero.resolved_links(),
                        2,
                        &format!("Page '{}' hero", page.id),
                        &mut errors,
                    );
                    validate_sal_fields(
                        hero.sal.unwrap_or(SalAnimation::Fade),
                        hero.sal_duration,
                        hero.sal_delay,
                        &format!("Page '{}' hero", page.id),
                        &mut errors,
                    );
                }
                PageNode::Section(section) => {
                    if section.id.trim().is_empty() {
                        errors.push(format!("Page '{}' has section with empty id.", page.id));
                    } else if !html_ids.insert(section.id.clone()) {
                        errors.push(format!(
                            "Page '{}' has duplicate section id '{}'.",
                            page.id, section.id
                        ));
                    }
                    if section.columns.is_empty() {
                        errors.push(format!("Section '{}' has no columns.", section.id));
                    }
                    validate_sal_fields(
                        section.sal,
                        section.sal_duration,
                        section.sal_delay,
                        &format!("Page '{}' section '{}'", page.id, section.id),
                        &mut errors,
                    );
                    let mut column_ids = std::collections::HashSet::new();
                    for column in &section.columns {
                        if column.id.trim().is_empty() {
                            errors.push(format!(
                                "Page '{}' section '{}' has a column with empty id.",
                                page.id, section.id
                            ));
                        } else if !column_ids.insert(column.id.clone()) {
                            errors.push(format!(
                                "Page '{}' section '{}' has duplicate column id '{}'.",
                                page.id, section.id, column.id
                            ));
                        }
                        if column.width_class.trim().is_empty() {
                            errors.push(format!(
                                "Page '{}' section '{}' column '{}' missing width_class.",
                                page.id, section.id, column.id
                            ));
                        }
                        for component in &column.components {
                            validate_section_component(
                                component,
                                page.id.as_str(),
                                section.id.as_str(),
                                &mut errors,
                            );
                        }
                    }
                }
            }
        }
    }

    errors
}

fn validate_section_component(
    component: &SectionComponent,
    page_id: &str,
    section_id: &str,
    errors: &mut Vec<String>,
) {
    if let Some((sal, duration, delay)) = component_sal(component) {
        validate_sal_fields(
            sal,
            duration,
            delay,
            &format!("Page '{}' section '{}'", page_id, section_id),
            errors,
        );
    }
    match component {
        SectionComponent::Alternating(alternating) => {
            if alternating.items.is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' has dd-alternating with no items.",
                    page_id, section_id
                ));
            }
            for (idx, item) in alternating.items.iter().enumerate() {
                if item.child_title.trim().is_empty() || item.child_copy.trim().is_empty() {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-alternating item {} has missing required fields.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
                validate_media(
                    &item.resolved_media(),
                    &format!(
                        "Page '{}' section '{}' dd-alternating item {}",
                        page_id,
                        section_id,
                        idx + 1
                    ),
                    errors,
                );
                validate_dd_links(
                    &item.links,
                    4,
                    &format!(
                        "Page '{}' section '{}' dd-alternating item {}",
                        page_id,
                        section_id,
                        idx + 1
                    ),
                    errors,
                );
            }
        }
        SectionComponent::Card(card) => {
            if card.parent_width.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' has dd-card with empty parent_width.",
                    page_id, section_id
                ));
            }
            if card.items.is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' has dd-card with no items.",
                    page_id, section_id
                ));
            }
            for (idx, item) in card.items.iter().enumerate() {
                if item.child_image_url.trim().is_empty()
                    || item.child_image_alt.trim().is_empty()
                    || item.child_title.trim().is_empty()
                    || item.child_subtitle.trim().is_empty()
                    || item.child_copy.trim().is_empty()
                {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-card item {} has missing required fields.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
                if !is_valid_url(&item.child_image_url) {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-card item {} child_image_url is invalid.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
                let has_link_url = item
                    .child_link_url
                    .as_deref()
                    .is_some_and(|v| !v.trim().is_empty());
                let has_link_label = item
                    .child_link_label
                    .as_deref()
                    .is_some_and(|v| !v.trim().is_empty());
                if has_link_url ^ has_link_label {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-card item {} must provide both child_link_url and child_link_label together.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
                if let Some(url) = item.child_link_url.as_deref()
                    && !url.trim().is_empty()
                    && !is_valid_url(url)
                {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-card item {} child_link_url is invalid.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
            }
        }
        SectionComponent::Banner(banner) => {
            validate_media(
                &banner.resolved_media(),
                &format!("Page '{}' section '{}' dd-banner", page_id, section_id),
                errors,
            );
        }
        SectionComponent::Cta(cta) => {
            if cta.parent_image_url.trim().is_empty()
                || cta.parent_image_alt.trim().is_empty()
                || cta.parent_title.trim().is_empty()
                || cta.parent_subtitle.trim().is_empty()
                || cta.parent_copy.trim().is_empty()
            {
                errors.push(format!(
                    "Page '{}' section '{}' dd-cta has missing required fields.",
                    page_id, section_id
                ));
            }
            if !is_valid_url(&cta.parent_image_url) {
                errors.push(format!(
                    "Page '{}' section '{}' dd-cta parent_image_url is invalid.",
                    page_id, section_id
                ));
            }
            validate_dd_links(
                &cta.resolved_links(),
                4,
                &format!("Page '{}' section '{}' dd-cta", page_id, section_id),
                errors,
            );
        }
        SectionComponent::Filmstrip(filmstrip) => {
            if filmstrip.items.is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' has dd-filmstrip with no items.",
                    page_id, section_id
                ));
            }
            for (idx, item) in filmstrip.items.iter().enumerate() {
                if item.child_image_url.trim().is_empty()
                    || item.child_image_alt.trim().is_empty()
                    || item.child_title.trim().is_empty()
                {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-filmstrip item {} has missing required fields.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
                if !is_valid_url(&item.child_image_url) {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-filmstrip item {} child_image_url is invalid.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
            }
        }
        SectionComponent::Milestones(milestones) => {
            if milestones.parent_width.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' has dd-milestones with empty parent_width.",
                    page_id, section_id
                ));
            }
            if milestones.items.is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' has dd-milestones with no items.",
                    page_id, section_id
                ));
            }
            for (idx, item) in milestones.items.iter().enumerate() {
                if item.child_percentage.trim().is_empty()
                    || item.child_title.trim().is_empty()
                    || item.child_subtitle.trim().is_empty()
                    || item.child_copy.trim().is_empty()
                {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-milestones item {} has missing required fields.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
                let has_link_url = item
                    .child_link_url
                    .as_deref()
                    .is_some_and(|v| !v.trim().is_empty());
                let has_link_label = item
                    .child_link_label
                    .as_deref()
                    .is_some_and(|v| !v.trim().is_empty());
                if has_link_url ^ has_link_label {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-milestones item {} must provide both child_link_url and child_link_label together.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
                if let Some(url) = item.child_link_url.as_deref()
                    && !url.trim().is_empty()
                    && !is_valid_url(url)
                {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-milestones item {} child_link_url is invalid.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
            }
        }
        SectionComponent::Modal(modal) => {
            if modal.parent_title.trim().is_empty() || modal.parent_copy.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-modal has missing required fields.",
                    page_id, section_id
                ));
            }
        }
        SectionComponent::Slider(slider) => {
            if slider.items.is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' has dd-slider with no items.",
                    page_id, section_id
                ));
            }
            for (idx, item) in slider.items.iter().enumerate() {
                if item.child_title.trim().is_empty() || item.child_copy.trim().is_empty() {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-slider item {} has missing required fields.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
                validate_media(
                    &item.resolved_media(),
                    &format!(
                        "Page '{}' section '{}' dd-slider item {}",
                        page_id,
                        section_id,
                        idx + 1
                    ),
                    errors,
                );
                validate_dd_links(
                    &item.resolved_links(),
                    4,
                    &format!(
                        "Page '{}' section '{}' dd-slider item {}",
                        page_id,
                        section_id,
                        idx + 1
                    ),
                    errors,
                );
            }
        }
        SectionComponent::Accordion(accordion) => {
            if accordion.parent_group_name.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-accordion missing parent_group_name.",
                    page_id, section_id
                ));
            }
            if accordion.items.is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' has dd-accordion with no items.",
                    page_id, section_id
                ));
            }
            for (idx, item) in accordion.items.iter().enumerate() {
                if item.child_title.trim().is_empty() || item.child_copy.trim().is_empty() {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-accordion item {} has missing child_title/child_copy.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
            }
        }
        SectionComponent::Blockquote(blockquote) => {
            if blockquote.parent_image_url.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' has a dd-blockquote with empty parent_image_url.",
                    page_id, section_id
                ));
            }
            if blockquote.parent_image_alt.trim().is_empty()
                || blockquote.parent_name.trim().is_empty()
                || blockquote.parent_role.trim().is_empty()
                || blockquote.parent_copy.trim().is_empty()
            {
                errors.push(format!(
                    "Page '{}' section '{}' dd-blockquote has missing required fields.",
                    page_id, section_id
                ));
            }
            if !is_valid_url(&blockquote.parent_image_url) {
                errors.push(format!(
                    "Page '{}' section '{}' dd-blockquote parent_image_url is invalid.",
                    page_id, section_id
                ));
            }
        }
        SectionComponent::Alert(alert) => {
            if alert.parent_copy.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-alert has missing required parent_copy.",
                    page_id, section_id
                ));
            }
        }
        SectionComponent::Image(image) => {
            if image.parent_image_url.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-image is missing parent_image_url.",
                    page_id, section_id
                ));
            } else if !is_valid_url(&image.parent_image_url) {
                errors.push(format!(
                    "Page '{}' section '{}' dd-image parent_image_url is invalid.",
                    page_id, section_id
                ));
            }
            if let Some(dark) = image.parent_image_url_dark.as_deref()
                && !dark.trim().is_empty()
                && !is_valid_url(dark)
            {
                errors.push(format!(
                    "Page '{}' section '{}' dd-image parent_image_url_dark is invalid.",
                    page_id, section_id
                ));
            }
            if image.parent_image_alt.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-image is missing parent_image_alt.",
                    page_id, section_id
                ));
            }
            if let Some(url) = image.parent_link_url.as_deref()
                && !url.trim().is_empty()
                && !is_valid_url(url)
            {
                errors.push(format!(
                    "Page '{}' section '{}' dd-image parent_link_url is invalid.",
                    page_id, section_id
                ));
            }
        }
        SectionComponent::RichText(rt) => {
            if rt.parent_copy.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-rich_text is missing parent_copy.",
                    page_id, section_id
                ));
            }
        }
        SectionComponent::Navigation(nav) => {
            if nav.items.is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-navigation has no items.",
                    page_id, section_id
                ));
            }
            for (idx, item) in nav.items.iter().enumerate() {
                validate_navigation_item(
                    item,
                    &format!("{}", idx + 1),
                    page_id,
                    section_id,
                    errors,
                );
            }
        }
        SectionComponent::HeaderSearch(_)
        | SectionComponent::HeaderMenu(_)
        | SectionComponent::Spacer(_) => {}
        SectionComponent::DataTable(table) => {
            if table.caption.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-data-table is missing caption.",
                    page_id, section_id
                ));
            }
            if table.columns.is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-data-table has no columns.",
                    page_id, section_id
                ));
            } else if table.columns.len() > crate::model::DATA_TABLE_MAX_COLUMNS {
                errors.push(format!(
                    "Page '{}' section '{}' dd-data-table has more than {} columns.",
                    page_id,
                    section_id,
                    crate::model::DATA_TABLE_MAX_COLUMNS
                ));
            }
            for (idx, col) in table.columns.iter().enumerate() {
                if col.label.trim().is_empty() {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-data-table column {} is missing a label.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
            }
        }
        SectionComponent::Tabs(tabs) => {
            if tabs.parent_id.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-tabs is missing parent_id.",
                    page_id, section_id
                ));
            } else if !crate::model::is_safe_slug(&tabs.parent_id) {
                errors.push(format!(
                    "Page '{}' section '{}' dd-tabs has an unsafe parent_id '{}'. Use lowercase letters, numbers, and hyphens only.",
                    page_id, section_id, tabs.parent_id
                ));
            }
            if tabs.items.is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' has dd-tabs with no items.",
                    page_id, section_id
                ));
            }
            for (idx, item) in tabs.items.iter().enumerate() {
                if item.child_title.trim().is_empty() || item.child_copy.trim().is_empty() {
                    errors.push(format!(
                        "Page '{}' section '{}' dd-tabs item {} has missing title/copy.",
                        page_id,
                        section_id,
                        idx + 1
                    ));
                }
            }
        }
        SectionComponent::Timeline(timeline) => {
            if timeline.items.is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' has dd-timeline with no items.",
                    page_id, section_id
                ));
            }
            for (idx, item) in timeline.items.iter().enumerate() {
                let ctx = format!(
                    "Page '{}' section '{}' dd-timeline item {}",
                    page_id,
                    section_id,
                    idx + 1
                );
                if item.child_year.trim().is_empty()
                    || item.child_title.trim().is_empty()
                    || item.child_copy.trim().is_empty()
                {
                    errors.push(format!("{ctx} has missing year/title/copy."));
                }
                if !(2..=6).contains(&item.heading_level) {
                    errors.push(format!(
                        "{ctx} heading_level {} is invalid; use 2–6.",
                        item.heading_level
                    ));
                }
                let datetime = item
                    .child_datetime
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty());
                if !crate::model::is_parseable_year_label(&item.child_year) {
                    match datetime {
                        None => errors.push(format!(
                            "{ctx} year '{}' is not a parseable date; set datetime (YYYY, YYYY-MM, or YYYY-MM-DD).",
                            item.child_year
                        )),
                        Some(dt) if !crate::model::is_parseable_year_label(dt) => {
                            errors.push(format!(
                                "{ctx} datetime '{dt}' is invalid; use YYYY, YYYY-MM, or YYYY-MM-DD."
                            ));
                        }
                        Some(_) => {}
                    }
                }
                let image = item
                    .child_image_url
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty());
                if image.is_some() {
                    // Decorative images may use empty alt; URL-only is enough.
                }
            }
        }
    }
}

fn validate_navigation_item(
    item: &NavigationItem,
    path: &str,
    page_id: &str,
    section_id: &str,
    errors: &mut Vec<String>,
) {
    if item.child_link_label.trim().is_empty() {
        errors.push(format!(
            "Page '{}' section '{}' dd-navigation item {} missing child_link_label.",
            page_id, section_id, path
        ));
    }
    match item.child_kind {
        NavigationKind::Link => {
            let url = item.child_link_url.as_deref().unwrap_or("");
            if url.trim().is_empty() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-navigation item {} kind=link requires child_link_url.",
                    page_id, section_id, path
                ));
            } else if !is_valid_url(url) {
                errors.push(format!(
                    "Page '{}' section '{}' dd-navigation item {} child_link_url is invalid.",
                    page_id, section_id, path
                ));
            }
        }
        NavigationKind::Button => {
            if item
                .child_link_url
                .as_deref()
                .is_some_and(|v| !v.trim().is_empty())
            {
                errors.push(format!(
                    "Page '{}' section '{}' dd-navigation item {} kind=button must not provide child_link_url.",
                    page_id, section_id, path
                ));
            }
            if item.child_link_target.is_some() {
                errors.push(format!(
                    "Page '{}' section '{}' dd-navigation item {} kind=button must not provide child_link_target.",
                    page_id, section_id, path
                ));
            }
        }
    }
    for (idx, child) in item.items.iter().enumerate() {
        validate_navigation_item(
            child,
            &format!("{}.{}", path, idx + 1),
            page_id,
            section_id,
            errors,
        );
    }
}

fn validate_media(media: &Media, ctx: &str, errors: &mut Vec<String>) {
    match media {
        Media::None => {}
        Media::Image { url, alt } => {
            if url.trim().is_empty() {
                errors.push(format!("{ctx} image is missing url."));
            } else if !is_valid_url(url) {
                errors.push(format!("{ctx} image url is invalid."));
            }
            if alt.trim().is_empty() {
                errors.push(format!("{ctx} image is missing alt text."));
            }
        }
        Media::Oembed { url } => {
            if url.trim().is_empty() {
                errors.push(format!("{ctx} video URL is empty."));
            } else if parse_oembed_url(url).is_none() {
                errors.push(format!(
                    "{ctx} video URL is not a recognized YouTube or Vimeo link."
                ));
            }
        }
        Media::LocalVideo {
            lg_mp4,
            sm_mp4,
            poster,
            name,
            ..
        } => {
            if lg_mp4.trim().is_empty() {
                errors.push(format!("{ctx} local video is missing large MP4."));
            } else if !is_valid_url(lg_mp4) {
                errors.push(format!("{ctx} large MP4 url is invalid."));
            }
            if let Some(sm) = sm_mp4 {
                if !sm.trim().is_empty() && !is_valid_url(sm) {
                    errors.push(format!("{ctx} small MP4 url is invalid."));
                }
            }
            if let Some(p) = poster {
                if !p.trim().is_empty() && !is_valid_url(p) {
                    errors.push(format!("{ctx} poster url is invalid."));
                }
            }
            if name.trim().is_empty() {
                errors.push(format!("{ctx} local video is missing accessible name."));
            }
        }
    }
}

fn validate_dd_links(links: &[DdLink], max: usize, ctx: &str, errors: &mut Vec<String>) {
    if links.len() > max {
        errors.push(format!(
            "{ctx} has {} links; maximum is {max}.",
            links.len()
        ));
    }
    for (idx, link) in links.iter().enumerate() {
        if link.url.trim().is_empty() || link.label.trim().is_empty() {
            errors.push(format!(
                "{ctx} link {} requires both url and label.",
                idx + 1
            ));
        } else if !is_valid_url(&link.url) {
            errors.push(format!("{ctx} link {} url is invalid.", idx + 1));
        }
    }
}

fn validate_sal_fields(
    _sal: SalAnimation,
    duration: Option<u16>,
    delay: Option<u16>,
    ctx: &str,
    errors: &mut Vec<String>,
) {
    if let Some(ms) = duration {
        if !crate::model::is_valid_sal_duration(ms) {
            errors.push(format!(
                "{ctx} sal_duration {ms} is invalid; use 200–2000 in steps of 50."
            ));
        }
    }
    if let Some(ms) = delay {
        if !crate::model::is_valid_sal_delay(ms) {
            errors.push(format!(
                "{ctx} sal_delay {ms} is invalid; use 0–1000 in steps of 50."
            ));
        }
    }
}

fn component_sal(component: &SectionComponent) -> Option<(SalAnimation, Option<u16>, Option<u16>)> {
    match component {
        SectionComponent::Alternating(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Card(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Cta(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Filmstrip(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Milestones(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Banner(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Accordion(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Blockquote(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Alert(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Image(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::RichText(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Navigation(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::HeaderSearch(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::HeaderMenu(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Tabs(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Timeline(c) => Some((c.sal, c.sal_duration, c.sal_delay)),
        SectionComponent::Slider(_)
        | SectionComponent::Modal(_)
        | SectionComponent::Spacer(_)
        | SectionComponent::DataTable(_) => None,
    }
}

fn validate_gtm_snippet(snippet: Option<&str>, label: &str, errors: &mut Vec<String>) {
    let Some(raw) = snippet.map(str::trim).filter(|s| !s.is_empty()) else {
        return;
    };
    if crate::model::extract_gtm_id(raw).is_none() {
        errors.push(format!(
            "site.{label} must include a GTM-XXXX container id (only googletagmanager.com is exported)."
        ));
    }
}

fn validate_header(header: &crate::model::DdHeader, errors: &mut Vec<String>) {
    if header.id.trim().is_empty() {
        errors.push("site.header has empty id.".to_string());
    }
    if header.sections.is_empty() {
        errors.push("site.header must have at least one section.".to_string());
    }
    if let Some(url) = header
        .cta_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !is_valid_url(url) {
            errors.push(format!("site.header CTA URL '{url}' is not a valid URL."));
        }
    }
    if let Some(alert) = &header.alert {
        validate_sal_fields(
            alert.sal,
            alert.sal_duration,
            alert.sal_delay,
            "site.header alert",
            errors,
        );
    }
    for section in &header.sections {
        validate_section_context(
            section,
            "header",
            &[
                "dd-image",
                "dd-rich_text",
                "dd-navigation",
                "dd-header-search",
                "dd-header-menu",
            ],
            errors,
        );
    }
}

fn validate_footer(footer: &crate::model::DdFooter, errors: &mut Vec<String>) {
    if footer.id.trim().is_empty() {
        errors.push("site.footer has empty id.".to_string());
    }
    if footer.sections.is_empty() {
        errors.push("site.footer must have at least one section.".to_string());
    }
    for (label, url) in [
        ("LinkedIn", footer.social_linkedin.as_deref()),
        ("X", footer.social_x.as_deref()),
        ("GitHub", footer.social_github.as_deref()),
    ] {
        if let Some(url) = url.map(str::trim).filter(|s| !s.is_empty()) {
            if !is_valid_url(url) {
                errors.push(format!(
                    "site.footer {label} URL '{url}' is not a valid URL."
                ));
            }
        }
    }
    for section in &footer.sections {
        validate_section_context(
            section,
            "footer",
            &["dd-image", "dd-rich_text", "dd-navigation"],
            errors,
        );
    }
}

fn validate_section_context(
    section: &DdSection,
    scope: &str,
    allowed_types: &[&str],
    errors: &mut Vec<String>,
) {
    validate_sal_fields(
        section.sal,
        section.sal_duration,
        section.sal_delay,
        &format!("site.{} section '{}'", scope, section.id),
        errors,
    );
    for column in &section.columns {
        for component in &column.components {
            let ty = section_component_type_name(component);
            if !allowed_types.contains(&ty) {
                errors.push(format!(
                    "site.{} section '{}' column '{}' contains disallowed component type '{}'; allowed: {:?}",
                    scope, section.id, column.id, ty, allowed_types
                ));
            }
            if let Some((sal, duration, delay)) = component_sal(component) {
                validate_sal_fields(
                    sal,
                    duration,
                    delay,
                    &format!("site.{} section '{}'", scope, section.id),
                    errors,
                );
            }
        }
    }
}

fn section_component_type_name(component: &SectionComponent) -> &'static str {
    match component {
        SectionComponent::Alternating(_) => "dd-alternating",
        SectionComponent::Card(_) => "dd-card",
        SectionComponent::Cta(_) => "dd-cta",
        SectionComponent::Filmstrip(_) => "dd-filmstrip",
        SectionComponent::Milestones(_) => "dd-milestones",
        SectionComponent::Slider(_) => "dd-slider",
        SectionComponent::Modal(_) => "dd-modal",
        SectionComponent::Banner(_) => "dd-banner",
        SectionComponent::Accordion(_) => "dd-accordion",
        SectionComponent::Blockquote(_) => "dd-blockquote",
        SectionComponent::Alert(_) => "dd-alert",
        SectionComponent::Image(_) => "dd-image",
        SectionComponent::RichText(_) => "dd-rich_text",
        SectionComponent::Navigation(_) => "dd-navigation",
        SectionComponent::HeaderSearch(_) => "dd-header-search",
        SectionComponent::HeaderMenu(_) => "dd-header-menu",
        SectionComponent::Spacer(_) => "dd-spacer",
        SectionComponent::Tabs(_) => "dd-tabs",
        SectionComponent::Timeline(_) => "dd-timeline",
        SectionComponent::DataTable(_) => "dd-data-table",
    }
}

fn is_valid_url(url: &str) -> bool {
    let v = url.trim();
    !v.is_empty()
        && (v.starts_with('/')
            || v.starts_with('#')
            || v.starts_with("http://")
            || v.starts_with("https://")
            || v.starts_with("mailto:")
            || v.starts_with("tel:")
            || v.starts_with("assets/")
            || v.ends_with(".html"))
}

pub fn validate_site_with_root(site: &Site, root: Option<&std::path::Path>) -> Vec<String> {
    let mut errors = validate_site(site);
    let Some(root) = root else {
        return errors;
    };
    for page in &site.pages {
        let refs = collect_image_refs(page);
        for (label, value) in refs {
            check_local_image(root, &label, &value, &mut errors);
        }
        if let Some(og) = page.head.og_image.as_deref() {
            check_local_image(
                root,
                &format!("page '{}' og_image", page.id),
                og,
                &mut errors,
            );
        }
    }
    collect_region_image_refs(&site.header.sections, "header", root, &mut errors);
    collect_region_image_refs(&site.footer.sections, "footer", root, &mut errors);
    errors
}

fn collect_region_image_refs(
    sections: &[DdSection],
    region: &str,
    root: &std::path::Path,
    errors: &mut Vec<String>,
) {
    let dummy = crate::model::Page {
        id: region.to_string(),
        slug: region.to_string(),
        slug_locked: true,
        head: crate::model::DdHead {
            title: region.to_string(),
            meta_title: None,
            meta_description: None,
            canonical_url: None,
            robots: crate::model::RobotsDirective::NoindexNofollow,
            schema_type: crate::model::SchemaType::WebPage,
            og_title: None,
            og_description: None,
            og_image: None,
        },
        nodes: Vec::new(),
    };
    let mut refs = Vec::new();
    for section in sections {
        for column in &section.columns {
            for component in &column.components {
                collect_component_image_refs(&dummy, component, &mut refs);
            }
        }
    }
    for (label, value) in refs {
        check_local_image(root, &label, &value, errors);
    }
}

fn check_local_image(root: &std::path::Path, label: &str, value: &str, errors: &mut Vec<String>) {
    let prefix = "assets/images/";
    let v = value.trim_start_matches('/');
    let Some(rest) = v.strip_prefix(prefix) else {
        return;
    };
    let resolved = root.join("source").join("images").join(rest);
    if !resolved.exists() {
        errors.push(format!(
            "Missing local image: {} → {} (expected at source/images/{})",
            label, value, rest
        ));
    }
}

fn collect_image_refs(page: &crate::model::Page) -> Vec<(String, String)> {
    let mut refs: Vec<(String, String)> = Vec::new();
    for node in &page.nodes {
        match node {
            crate::model::PageNode::Hero(hero) => {
                for url in hero.resolved_media().local_asset_urls() {
                    refs.push((format!("page '{}' hero media", page.id), url));
                }
                if let Some(s) = hero.parent_image_mobile.as_deref() {
                    refs.push((
                        format!("page '{}' hero parent_image_mobile", page.id),
                        s.to_string(),
                    ));
                }
                if let Some(s) = hero.parent_image_tablet.as_deref() {
                    refs.push((
                        format!("page '{}' hero parent_image_tablet", page.id),
                        s.to_string(),
                    ));
                }
                if let Some(s) = hero.parent_image_desktop.as_deref() {
                    refs.push((
                        format!("page '{}' hero parent_image_desktop", page.id),
                        s.to_string(),
                    ));
                }
            }
            crate::model::PageNode::Section(section) => {
                for col in &section.columns {
                    for comp in &col.components {
                        collect_component_image_refs(page, comp, &mut refs);
                    }
                }
            }
        }
    }
    refs
}

fn collect_component_image_refs(
    page: &crate::model::Page,
    comp: &crate::model::SectionComponent,
    refs: &mut Vec<(String, String)>,
) {
    use crate::model::SectionComponent::*;
    let lbl = |suffix: &str| format!("page '{}' {}", page.id, suffix);
    match comp {
        Banner(b) => {
            for url in b.resolved_media().local_asset_urls() {
                refs.push((lbl("banner media"), url));
            }
        }
        Cta(c) => refs.push((lbl("cta image"), c.parent_image_url.clone())),
        Image(i) => {
            refs.push((lbl("image"), i.parent_image_url.clone()));
            if let Some(dark) = i
                .parent_image_url_dark
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
            {
                refs.push((lbl("image dark"), dark.to_string()));
            }
        }
        Blockquote(b) => refs.push((lbl("blockquote image"), b.parent_image_url.clone())),
        Card(c) => {
            for (n, item) in c.items.iter().enumerate() {
                refs.push((
                    lbl(&format!("card item {} image", n + 1)),
                    item.child_image_url.clone(),
                ));
            }
        }
        Filmstrip(f) => {
            for (n, item) in f.items.iter().enumerate() {
                refs.push((
                    lbl(&format!("filmstrip item {} image", n + 1)),
                    item.child_image_url.clone(),
                ));
            }
        }
        Slider(s) => {
            for (n, item) in s.items.iter().enumerate() {
                for url in item.resolved_media().local_asset_urls() {
                    refs.push((lbl(&format!("slider item {} media", n + 1)), url));
                }
            }
        }
        Alternating(a) => {
            for (n, item) in a.items.iter().enumerate() {
                for url in item.resolved_media().local_asset_urls() {
                    refs.push((lbl(&format!("alternating item {} media", n + 1)), url));
                }
            }
        }
        Timeline(t) => {
            for (n, item) in t.items.iter().enumerate() {
                if let Some(url) = item
                    .child_image_url
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    refs.push((
                        lbl(&format!("timeline item {} image", n + 1)),
                        url.to_string(),
                    ));
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_site, validate_site_with_root};
    use crate::model::{PageNode, Site};

    #[test]
    fn starter_site_is_valid() {
        let site = Site::starter();
        let errors = validate_site(&site);
        assert!(
            errors.is_empty(),
            "expected no validation errors, got {errors:?}"
        );
    }

    #[test]
    fn gtm_snippet_without_container_id_is_invalid() {
        let mut site = Site::starter();
        site.header_gtm_tag = Some("<script>alert(1)</script>".to_string());
        let errors = validate_site(&site);
        assert!(
            errors.iter().any(|e| e.contains("GTM-XXXX")),
            "expected GTM id error, got {errors:?}"
        );
    }

    #[test]
    fn detects_missing_hero_required_fields() {
        let mut site = Site::starter();
        let page = &mut site.pages[0];
        if let PageNode::Hero(hero) = &mut page.nodes[0] {
            hero.parent_title.clear();
            hero.parent_subtitle.clear();
            hero.parent_image_url.clear();
        }
        let errors = validate_site(&site);
        assert!(errors.iter().any(|e| e.contains("missing parent_title")));
        assert!(!errors.iter().any(|e| e.contains("missing parent_subtitle")));
        assert!(
            !errors
                .iter()
                .any(|e| e.contains("missing parent_image_url"))
        );
    }

    #[test]
    fn detects_duplicate_page_slug() {
        let mut site = Site::starter();
        site.pages.push(site.pages[0].clone());
        let errors = validate_site(&site);
        assert!(errors.iter().any(|e| e.contains("Duplicate page slug")));
    }

    #[test]
    fn detects_unsafe_and_duplicate_hero_id() {
        let mut site = Site::starter();
        if let PageNode::Hero(hero) = &mut site.pages[0].nodes[0] {
            hero.id = Some("not a slug".to_string());
        }
        let errors = validate_site(&site);
        assert!(
            errors.iter().any(|e| e.contains("unsafe id")),
            "expected unsafe hero id, got {errors:?}"
        );

        let mut site = Site::starter();
        if let PageNode::Hero(hero) = &mut site.pages[0].nodes[0] {
            hero.id = Some("section-1".to_string());
        }
        let errors = validate_site(&site);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("duplicate section id 'section-1'")),
            "expected hero/section id collision, got {errors:?}"
        );
    }

    #[test]
    fn validate_with_root_flags_missing_local_image() {
        let tmp = std::env::temp_dir().join(format!(
            "dd_missing_img_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let mut site = Site::starter();
        if let PageNode::Hero(hero) = &mut site.pages[0].nodes[0] {
            hero.parent_image_url = "/assets/images/missing.jpg".to_string();
            hero.parent_image_alt = Some("alt".to_string());
        }
        let errors = validate_site_with_root(&site, Some(&tmp));
        assert!(
            errors.iter().any(|e| e.contains("Missing local image")),
            "expected missing-image error, got: {:?}",
            errors
        );
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn validate_with_root_passes_when_image_exists() {
        let tmp = std::env::temp_dir().join(format!(
            "dd_present_img_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let imgs = tmp.join("source").join("images");
        std::fs::create_dir_all(&imgs).unwrap();
        std::fs::write(imgs.join("hero.jpg"), b"fake").unwrap();

        let mut site = Site::starter();
        if let PageNode::Hero(hero) = &mut site.pages[0].nodes[0] {
            hero.parent_image_url = "assets/images/hero.jpg".to_string();
            hero.parent_image_alt = Some("alt".to_string());
        }
        let errors = validate_site_with_root(&site, Some(&tmp));
        assert!(
            errors.iter().all(|e| !e.contains("Missing local image")),
            "no missing-image error expected, got: {:?}",
            errors
        );
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn rejects_unsafe_slug() {
        let mut site = Site::starter();
        site.pages[0].slug = "../etc".to_string();
        let errors = validate_site(&site);
        assert!(
            errors.iter().any(|e| e.contains("unsafe slug")),
            "expected unsafe slug error, got {errors:?}"
        );
    }

    #[test]
    fn allows_nested_page_slug_and_rejects_index_folder() {
        let mut site = Site::starter();
        site.pages[0].slug = "blog/entry".to_string();
        let errors = validate_site(&site);
        assert!(
            errors.iter().all(|e| !e.contains("unsafe slug")),
            "nested slug should be valid, got {errors:?}"
        );
        site.pages[0].slug = "blog/index".to_string();
        let errors = validate_site(&site);
        assert!(
            errors.iter().any(|e| e.contains("unsafe slug")),
            "slug ending in index should fail, got {errors:?}"
        );
    }

    #[test]
    fn rejects_invalid_sal_duration() {
        let mut site = Site::starter();
        if let PageNode::Hero(hero) = &mut site.pages[0].nodes[0] {
            hero.sal_duration = Some(225);
        }
        let errors = validate_site(&site);
        assert!(
            errors.iter().any(|e| e.contains("sal_duration 225")),
            "expected sal_duration error, got {errors:?}"
        );
    }
}
