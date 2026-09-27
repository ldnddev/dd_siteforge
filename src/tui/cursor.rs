//! Region-aware mutation cursor.
//!
//! A `Cursor` identifies any editable node anywhere in the site — header,
//! footer, or page. `resolve_mut()` converts a cursor into a mutable
//! reference to the underlying model node. `apply_edit_form_to_component()`
//! is the single entry point the editor calls on Ctrl+S; it writes every
//! visible field of an `EditFormState` back into the target, correctly
//! routing to the header/footer/page region.
//!
//! This module is the structural fix for the "header/footer edits
//! silently target the current page" bug: every write path funnels through
//! `resolve_mut`, which knows every region.

use anyhow::{Context, Result, anyhow};

use crate::model::{
    AccordionClass, AccordionItem, AccordionType, AlertClass, AlertType, AlternatingItem,
    AlternatingType, BannerClass, ButtonStyle, CardItem, CardLinkTarget, CardType, CtaClass,
    DdAccordion, DdAlert, DdAlternating, DdBanner, DdBlockquote, DdCard, DdCta, DdFilmstrip,
    DdFooter, DdHead, DdHeader, DdHeaderMenu, DdHeaderSearch, DdHero, DdImage, DdLink,
    DdMilestones, DdModal, DdNavigation, DdRichText, DdSection, DdSlider, DdSpacer, DdTabs,
    DdTimeline, FilmstripItem, FilmstripType, HeroCopyPosition, HeroImageClass, HeroOverlay, Media,
    MilestonesItem, NavigationClass, NavigationItem, NavigationKind, NavigationType, PageNode,
    SalAnimation, SectionBg, SectionClass, SectionColumn, SectionComponent, SectionItemBoxClass,
    SectionPadding, Site, SliderItem, SpacerSize, TabsItem, TabsOrientation, TimelineItem,
};
use crate::tui::editform::{self, EditFormState, FieldKind};

/// Address of any editable node in the site.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cursor {
    // --- Header region ---
    HeaderRoot,
    HeaderSection {
        sec: usize,
    },
    HeaderComponent {
        sec: usize,
        col: usize,
        comp: usize,
        items: Vec<usize>,
    },
    // --- Footer region ---
    FooterRoot,
    FooterSection {
        sec: usize,
    },
    FooterComponent {
        sec: usize,
        col: usize,
        comp: usize,
        items: Vec<usize>,
    },
    // --- Site settings ---
    Site,
    // --- Page region ---
    PageHead {
        page: usize,
    },
    PageHero {
        page: usize,
        node: usize,
    },
    PageSection {
        page: usize,
        node: usize,
    },
    PageComponent {
        page: usize,
        node: usize,
        col: usize,
        comp: usize,
        items: Vec<usize>,
    },
}

/// Typed mutable reference to whichever node a `Cursor` resolved to.
#[allow(dead_code)] // most variants unused until Tier A/B/C/D migrations
pub enum CursorRef<'a> {
    Hero(&'a mut DdHero),
    Section(&'a mut DdSection),
    Component(&'a mut SectionComponent),
    Head(&'a mut DdHead),
    HeaderRoot(&'a mut DdHeader),
    FooterRoot(&'a mut DdFooter),
}

/// Resolve a cursor to a mutable typed reference inside the site.
pub fn resolve_mut<'a>(site: &'a mut Site, cursor: &Cursor) -> Result<CursorRef<'a>> {
    match cursor {
        Cursor::Site => Err(anyhow!("site settings are applied via dedicated path")),
        Cursor::HeaderRoot => Ok(CursorRef::HeaderRoot(&mut site.header)),
        Cursor::HeaderSection { sec } => {
            let s = site
                .header
                .sections
                .get_mut(*sec)
                .context("header section index out of bounds")?;
            Ok(CursorRef::Section(s))
        }
        Cursor::HeaderComponent { sec, col, comp, .. } => {
            let column = resolve_column_mut(&mut site.header.sections, *sec, *col)?;
            let c = column
                .components
                .get_mut(*comp)
                .context("header component index out of bounds")?;
            Ok(CursorRef::Component(c))
        }
        Cursor::FooterRoot => Ok(CursorRef::FooterRoot(&mut site.footer)),
        Cursor::FooterSection { sec } => {
            let s = site
                .footer
                .sections
                .get_mut(*sec)
                .context("footer section index out of bounds")?;
            Ok(CursorRef::Section(s))
        }
        Cursor::FooterComponent { sec, col, comp, .. } => {
            let column = resolve_column_mut(&mut site.footer.sections, *sec, *col)?;
            let c = column
                .components
                .get_mut(*comp)
                .context("footer component index out of bounds")?;
            Ok(CursorRef::Component(c))
        }
        Cursor::PageHead { page } => {
            let p = site
                .pages
                .get_mut(*page)
                .context("page index out of bounds")?;
            Ok(CursorRef::Head(&mut p.head))
        }
        Cursor::PageHero { page, node } => {
            let p = site
                .pages
                .get_mut(*page)
                .context("page index out of bounds")?;
            let n = p
                .nodes
                .get_mut(*node)
                .context("page node index out of bounds")?;
            match n {
                PageNode::Hero(h) => Ok(CursorRef::Hero(h)),
                _ => Err(anyhow!("cursor points at hero but node is not a Hero")),
            }
        }
        Cursor::PageSection { page, node } => {
            let p = site
                .pages
                .get_mut(*page)
                .context("page index out of bounds")?;
            let n = p
                .nodes
                .get_mut(*node)
                .context("page node index out of bounds")?;
            match n {
                PageNode::Section(s) => Ok(CursorRef::Section(s)),
                _ => Err(anyhow!(
                    "cursor points at section but node is not a Section"
                )),
            }
        }
        Cursor::PageComponent {
            page,
            node,
            col,
            comp,
            ..
        } => {
            let p = site
                .pages
                .get_mut(*page)
                .context("page index out of bounds")?;
            let n = p
                .nodes
                .get_mut(*node)
                .context("page node index out of bounds")?;
            let section = match n {
                PageNode::Section(s) => s,
                _ => return Err(anyhow!("component cursor does not address a Section node")),
            };
            let column = section
                .columns
                .get_mut(*col)
                .context("column index out of bounds")?;
            let c = column
                .components
                .get_mut(*comp)
                .context("component index out of bounds")?;
            Ok(CursorRef::Component(c))
        }
    }
}

fn resolve_column_mut<'a>(
    sections: &'a mut [DdSection],
    sec_idx: usize,
    col_idx: usize,
) -> Result<&'a mut SectionColumn> {
    let s = sections
        .get_mut(sec_idx)
        .context("section index out of bounds")?;
    s.columns
        .get_mut(col_idx)
        .context("column index out of bounds")
}

/// Apply every visible field of `state` back into the model node at `cursor`.
/// The single entry point for Ctrl+S from the form editor.
pub fn apply_edit_form_to_component(
    site: &mut Site,
    cursor: &Cursor,
    state: &EditFormState,
) -> Result<()> {
    // Special roots/heads have dedicated cursors and may involve page-level fields like slug.
    match cursor {
        Cursor::PageHead { page } => {
            let p = site
                .pages
                .get_mut(*page)
                .context("page index out of bounds")?;
            let orig_title = p.head.title.clone();
            let orig_slug = p.slug.clone();
            apply_head_values(&mut p.head, state)?;
            let slug_val = state.get("slug").trim().to_string();
            if !slug_val.is_empty() && slug_val != p.slug {
                p.slug = crate::model::slug_from_title(&slug_val);
                p.slug_locked = true;
            }
            let title_changed = p.head.title != orig_title;
            let slug_user_edited = p.slug != orig_slug;
            if title_changed && !slug_user_edited && !p.slug_locked {
                let derived = crate::model::slug_from_title(&p.head.title);
                if !derived.is_empty() {
                    p.slug = derived;
                }
            }
            return Ok(());
        }
        Cursor::Site => {
            apply_site_values(site, state)?;
            return Ok(());
        }
        Cursor::HeaderRoot => {
            apply_header_root_values(&mut site.header, state)?;
            return Ok(());
        }
        Cursor::FooterRoot => {
            apply_footer_values(&mut site.footer, state)?;
            return Ok(());
        }
        _ => {}
    }

    let target = resolve_mut(site, cursor)?;
    match target {
        CursorRef::Component(component) => match component {
            SectionComponent::Cta(cta) => apply_cta_values(cta, state),
            SectionComponent::Banner(b) => apply_banner_values(b, state),
            SectionComponent::Image(i) => apply_image_values(i, state),
            SectionComponent::HeaderSearch(h) => apply_header_search_values(h, state),
            SectionComponent::HeaderMenu(h) => apply_header_menu_values(h, state),
            SectionComponent::RichText(r) => apply_rich_text_values(r, state),
            SectionComponent::Alert(a) => apply_alert_values(a, state),
            SectionComponent::Modal(m) => apply_modal_values(m, state),
            SectionComponent::Blockquote(bq) => apply_blockquote_values(bq, state),
            SectionComponent::Card(c) => apply_card_values(c, state),
            SectionComponent::Filmstrip(f) => apply_filmstrip_values(f, state),
            SectionComponent::Milestones(m) => apply_milestones_values(m, state),
            SectionComponent::Slider(s) => apply_slider_values(s, state),
            SectionComponent::Accordion(a) => apply_accordion_values(a, state),
            SectionComponent::Alternating(a) => apply_alternating_values(a, state),
            SectionComponent::Navigation(n) => apply_navigation_values(n, state),
            SectionComponent::Spacer(s) => apply_spacer_values(s, state),
            SectionComponent::Tabs(t) => apply_tabs_values(t, state),
            SectionComponent::Timeline(t) => apply_timeline_values(t, state),
        },
        CursorRef::Hero(hero) => apply_hero_values(hero, state),
        CursorRef::Section(section) => apply_section_values(section, state),
        CursorRef::Head(head) => apply_head_values(head, state),
        CursorRef::HeaderRoot(h) => apply_header_root_values(h, state),
        CursorRef::FooterRoot(f) => apply_footer_values(f, state),
    }
}

fn apply_cta_values(cta: &mut DdCta, state: &EditFormState) -> Result<()> {
    cta.parent_class =
        parse_enum::<CtaClass>(state.get("parent_class")).context("invalid parent_class")?;
    cta.parent_image_url = state.get("parent_image_url").trim().to_string();
    cta.parent_image_alt = state.get("parent_image_alt").trim().to_string();
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    cta.sal = sal;
    cta.sal_duration = sal_duration;
    cta.sal_delay = sal_delay;
    cta.parent_title = state.get("parent_title").to_string();
    cta.parent_subtitle = state.get("parent_subtitle").to_string();
    cta.parent_copy = state.get("parent_copy").to_string();
    cta.links = apply_links_subform(state, "links")?;
    cta.parent_link_url = None;
    cta.parent_link_target = None;
    cta.parent_link_label = None;
    Ok(())
}

/// Entry point used by the tui to turn a live component into an `EditFormState`.
/// Returns None when the component type hasn't been migrated to the unified
/// editor yet.
pub fn component_to_form_state(component: &SectionComponent) -> Option<EditFormState> {
    match component {
        SectionComponent::Cta(c) => Some(cta_to_form_state(c)),
        SectionComponent::Banner(b) => Some(banner_to_form_state(b)),
        SectionComponent::Image(i) => Some(image_to_form_state(i)),
        SectionComponent::HeaderSearch(h) => Some(header_search_to_form_state(h)),
        SectionComponent::HeaderMenu(h) => Some(header_menu_to_form_state(h)),
        SectionComponent::RichText(r) => Some(rich_text_to_form_state(r)),
        SectionComponent::Alert(a) => Some(alert_to_form_state(a)),
        SectionComponent::Modal(m) => Some(modal_to_form_state(m)),
        SectionComponent::Blockquote(bq) => Some(blockquote_to_form_state(bq)),
        SectionComponent::Card(c) => Some(card_to_form_state(c)),
        SectionComponent::Filmstrip(f) => Some(filmstrip_to_form_state(f)),
        SectionComponent::Milestones(m) => Some(milestones_to_form_state(m)),
        SectionComponent::Slider(s) => Some(slider_to_form_state(s)),
        SectionComponent::Accordion(a) => Some(accordion_to_form_state(a)),
        SectionComponent::Alternating(a) => Some(alternating_to_form_state(a)),
        SectionComponent::Navigation(n) => Some(navigation_to_form_state(n)),
        SectionComponent::Spacer(s) => Some(spacer_to_form_state(s)),
        SectionComponent::Tabs(t) => Some(tabs_to_form_state(t)),
        SectionComponent::Timeline(t) => Some(timeline_to_form_state(t)),
    }
}

/// Seed an `EditFormState` with current values from a `DdCta`.
pub fn cta_to_form_state(cta: &DdCta) -> EditFormState {
    let mut state = EditFormState::new(&crate::tui::editform::CTA_FORM);
    state.set("parent_class", enum_serde_str(cta.parent_class));
    state.set("parent_image_url", cta.parent_image_url.clone());
    state.set("parent_image_alt", cta.parent_image_alt.clone());
    set_sal_fields(&mut state, cta.sal, cta.sal_duration, cta.sal_delay);
    state.set("parent_title", cta.parent_title.clone());
    state.set("parent_subtitle", cta.parent_subtitle.clone());
    state.set("parent_copy", cta.parent_copy.clone());
    set_links_subform(&mut state, "links", &cta.resolved_links());
    state
}

// ==================== Tier A populate + apply ====================

pub fn banner_to_form_state(b: &DdBanner) -> EditFormState {
    let mut s = EditFormState::new(&editform::BANNER_FORM);
    s.set("parent_class", enum_serde_str(b.parent_class));
    set_sal_fields(&mut s, b.sal, b.sal_duration, b.sal_delay);
    set_media_fields(&mut s, &b.resolved_media());
    s
}
fn apply_banner_values(b: &mut DdBanner, state: &EditFormState) -> Result<()> {
    b.parent_class =
        parse_enum::<BannerClass>(state.get("parent_class")).context("invalid parent_class")?;
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    b.sal = sal;
    b.sal_duration = sal_duration;
    b.sal_delay = sal_delay;
    b.media = apply_media_fields(state)?;
    let (url, alt) = legacy_image_from_media(&b.media);
    b.parent_image_url = url;
    b.parent_image_alt = alt;
    Ok(())
}

pub fn image_to_form_state(i: &DdImage) -> EditFormState {
    let mut s = EditFormState::new(&editform::IMAGE_FORM);
    set_sal_fields(&mut s, i.sal, i.sal_duration, i.sal_delay);
    s.set("parent_image_url", i.parent_image_url.clone());
    s.set(
        "parent_image_url_dark",
        i.parent_image_url_dark.clone().unwrap_or_default(),
    );
    s.set("parent_image_alt", i.parent_image_alt.clone());
    s.set(
        "parent_link_url",
        i.parent_link_url.clone().unwrap_or_default(),
    );
    s.set(
        "parent_link_target",
        i.parent_link_target
            .map(enum_serde_str)
            .unwrap_or_else(|| "_self".to_string()),
    );
    s
}
fn apply_image_values(i: &mut DdImage, state: &EditFormState) -> Result<()> {
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    i.sal = sal;
    i.sal_duration = sal_duration;
    i.sal_delay = sal_delay;
    i.parent_image_url = state.get("parent_image_url").trim().to_string();
    let dark = state.get("parent_image_url_dark").trim().to_string();
    i.parent_image_url_dark = if dark.is_empty() { None } else { Some(dark) };
    i.parent_image_alt = state.get("parent_image_alt").trim().to_string();
    let link = state.get("parent_link_url").trim().to_string();
    if link.is_empty() {
        i.parent_link_url = None;
        i.parent_link_target = None;
    } else {
        i.parent_link_url = Some(link);
        i.parent_link_target = Some(
            parse_enum::<CardLinkTarget>(state.get("parent_link_target"))
                .context("invalid parent_link_target")?,
        );
    }
    Ok(())
}

pub fn header_search_to_form_state(h: &DdHeaderSearch) -> EditFormState {
    let mut s = EditFormState::new(&editform::HEADER_SEARCH_FORM);
    s.set("parent_width", h.parent_width.clone());
    set_sal_fields(&mut s, h.sal, h.sal_duration, h.sal_delay);
    s
}
fn apply_header_search_values(h: &mut DdHeaderSearch, state: &EditFormState) -> Result<()> {
    h.parent_width = state.get("parent_width").trim().to_string();
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    h.sal = sal;
    h.sal_duration = sal_duration;
    h.sal_delay = sal_delay;
    Ok(())
}

pub fn header_menu_to_form_state(h: &DdHeaderMenu) -> EditFormState {
    let mut s = EditFormState::new(&editform::HEADER_MENU_FORM);
    s.set("parent_width", h.parent_width.clone());
    set_sal_fields(&mut s, h.sal, h.sal_duration, h.sal_delay);
    s
}
fn apply_header_menu_values(h: &mut DdHeaderMenu, state: &EditFormState) -> Result<()> {
    h.parent_width = state.get("parent_width").trim().to_string();
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    h.sal = sal;
    h.sal_duration = sal_duration;
    h.sal_delay = sal_delay;
    Ok(())
}

pub fn rich_text_to_form_state(r: &DdRichText) -> EditFormState {
    let mut s = EditFormState::new(&editform::RICH_TEXT_FORM);
    s.set("parent_class", r.parent_class.clone().unwrap_or_default());
    set_sal_fields(&mut s, r.sal, r.sal_duration, r.sal_delay);
    s.set("parent_copy", r.parent_copy.clone());
    s
}
fn apply_rich_text_values(r: &mut DdRichText, state: &EditFormState) -> Result<()> {
    let class = state.get("parent_class").trim().to_string();
    r.parent_class = if class.is_empty() { None } else { Some(class) };
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    r.sal = sal;
    r.sal_duration = sal_duration;
    r.sal_delay = sal_delay;
    r.parent_copy = state.get("parent_copy").to_string();
    Ok(())
}

pub fn alert_to_form_state(a: &DdAlert) -> EditFormState {
    let mut s = EditFormState::new(&editform::ALERT_FORM);
    s.set("parent_type", enum_serde_str(a.parent_type));
    s.set("parent_class", enum_serde_str(a.parent_class));
    set_sal_fields(&mut s, a.sal, a.sal_duration, a.sal_delay);
    s.set("parent_title", a.parent_title.clone().unwrap_or_default());
    s.set("parent_copy", a.parent_copy.clone());
    s
}
fn apply_alert_values(a: &mut DdAlert, state: &EditFormState) -> Result<()> {
    a.parent_type =
        parse_enum::<AlertType>(state.get("parent_type")).context("invalid parent_type")?;
    a.parent_class =
        parse_enum::<AlertClass>(state.get("parent_class")).context("invalid parent_class")?;
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    a.sal = sal;
    a.sal_duration = sal_duration;
    a.sal_delay = sal_delay;
    let title = state.get("parent_title").trim().to_string();
    a.parent_title = if title.is_empty() { None } else { Some(title) };
    a.parent_copy = state.get("parent_copy").to_string();
    Ok(())
}

pub fn spacer_to_form_state(s: &DdSpacer) -> EditFormState {
    let mut state = EditFormState::new(&editform::SPACER_FORM);
    state.set("size", enum_serde_str(s.size));
    state.set("divider", if s.divider { "true" } else { "false" });
    state
}
fn apply_spacer_values(s: &mut DdSpacer, state: &EditFormState) -> Result<()> {
    s.size = parse_enum::<SpacerSize>(state.get("size")).context("invalid size")?;
    s.divider = matches!(state.get("divider"), "true" | "1");
    Ok(())
}

pub fn tabs_to_form_state(t: &DdTabs) -> EditFormState {
    let mut s = EditFormState::new(&editform::TABS_FORM);
    s.set("parent_id", t.parent_id.clone());
    s.set("parent_class", enum_serde_str(t.parent_class));
    s.set("aria_label", t.aria_label.clone().unwrap_or_default());
    set_sal_fields(&mut s, t.sal, t.sal_duration, t.sal_delay);
    let mut items = Vec::new();
    for it in &t.items {
        let mut item = EditFormState::new(&editform::TABS_ITEM_FORM);
        item.set("child_title", it.child_title.clone());
        item.set("child_copy", it.child_copy.clone());
        items.push(item);
    }
    s.sub_state.insert("items".to_string(), items);
    s.selected_sub_item.insert("items".to_string(), 0);
    s
}
fn apply_tabs_values(t: &mut DdTabs, state: &EditFormState) -> Result<()> {
    t.parent_id = state.get("parent_id").trim().to_string();
    t.parent_class =
        parse_enum::<TabsOrientation>(state.get("parent_class")).context("invalid orientation")?;
    let aria = state.get("aria_label").trim().to_string();
    t.aria_label = if aria.is_empty() { None } else { Some(aria) };
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    t.sal = sal;
    t.sal_duration = sal_duration;
    t.sal_delay = sal_delay;
    t.items.clear();
    if let Some(items) = state.sub_state.get("items") {
        for item_s in items {
            t.items.push(TabsItem {
                child_title: item_s.get("child_title").to_string(),
                child_copy: item_s.get("child_copy").to_string(),
            });
        }
    }
    Ok(())
}

pub fn timeline_to_form_state(t: &DdTimeline) -> EditFormState {
    let mut s = EditFormState::new(&editform::TIMELINE_FORM);
    s.set("aria_label", t.aria_label.clone().unwrap_or_default());
    set_sal_fields(&mut s, t.sal, t.sal_duration, t.sal_delay);
    let mut items = Vec::new();
    for it in &t.items {
        let mut item = EditFormState::new(&editform::TIMELINE_ITEM_FORM);
        item.set("child_year", it.child_year.clone());
        item.set(
            "child_datetime",
            it.child_datetime.clone().unwrap_or_default(),
        );
        item.set("child_title", it.child_title.clone());
        item.set("heading_level", it.heading_level.to_string());
        item.set("child_copy", it.child_copy.clone());
        item.set(
            "child_image_url",
            it.child_image_url.clone().unwrap_or_default(),
        );
        item.set(
            "child_image_alt",
            it.child_image_alt.clone().unwrap_or_default(),
        );
        items.push(item);
    }
    s.sub_state.insert("items".to_string(), items);
    s.selected_sub_item.insert("items".to_string(), 0);
    s
}
fn apply_timeline_values(t: &mut DdTimeline, state: &EditFormState) -> Result<()> {
    let aria = state.get("aria_label").trim().to_string();
    t.aria_label = if aria.is_empty() { None } else { Some(aria) };
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    t.sal = sal;
    t.sal_duration = sal_duration;
    t.sal_delay = sal_delay;
    t.items.clear();
    if let Some(items) = state.sub_state.get("items") {
        for item_s in items {
            let datetime = item_s.get("child_datetime").trim().to_string();
            let image_url = item_s.get("child_image_url").trim().to_string();
            let image_alt = item_s.get("child_image_alt").trim().to_string();
            let heading = item_s
                .get("heading_level")
                .trim()
                .parse::<u8>()
                .unwrap_or(3);
            t.items.push(TimelineItem {
                child_year: item_s.get("child_year").to_string(),
                child_datetime: if datetime.is_empty() {
                    None
                } else {
                    Some(datetime)
                },
                child_title: item_s.get("child_title").to_string(),
                heading_level: heading,
                child_copy: item_s.get("child_copy").to_string(),
                child_image_url: if image_url.is_empty() {
                    None
                } else {
                    Some(image_url)
                },
                child_image_alt: if image_alt.is_empty() {
                    None
                } else {
                    Some(image_alt)
                },
            });
        }
    }
    Ok(())
}

pub fn modal_to_form_state(m: &DdModal) -> EditFormState {
    let mut s = EditFormState::new(&editform::MODAL_FORM);
    s.set("parent_title", m.parent_title.clone());
    s.set("parent_copy", m.parent_copy.clone());
    s
}
fn apply_modal_values(m: &mut DdModal, state: &EditFormState) -> Result<()> {
    m.parent_title = state.get("parent_title").to_string();
    m.parent_copy = state.get("parent_copy").to_string();
    Ok(())
}

pub fn blockquote_to_form_state(bq: &DdBlockquote) -> EditFormState {
    let mut s = EditFormState::new(&editform::BLOCKQUOTE_FORM);
    set_sal_fields(&mut s, bq.sal, bq.sal_duration, bq.sal_delay);
    s.set("parent_image_url", bq.parent_image_url.clone());
    s.set("parent_image_alt", bq.parent_image_alt.clone());
    s.set("parent_name", bq.parent_name.clone());
    s.set("parent_role", bq.parent_role.clone());
    s.set("parent_copy", bq.parent_copy.clone());
    s
}
fn apply_blockquote_values(bq: &mut DdBlockquote, state: &EditFormState) -> Result<()> {
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    bq.sal = sal;
    bq.sal_duration = sal_duration;
    bq.sal_delay = sal_delay;
    bq.parent_image_url = state.get("parent_image_url").trim().to_string();
    bq.parent_image_alt = state.get("parent_image_alt").trim().to_string();
    bq.parent_name = state.get("parent_name").to_string();
    bq.parent_role = state.get("parent_role").to_string();
    bq.parent_copy = state.get("parent_copy").to_string();
    Ok(())
}

// ==================== Tier B populate + apply ====================

pub fn card_to_form_state(c: &DdCard) -> EditFormState {
    let mut s = EditFormState::new(&editform::CARD_FORM);
    s.set("parent_type", enum_serde_str(c.parent_type));
    set_sal_fields(&mut s, c.sal, c.sal_duration, c.sal_delay);
    s.set("parent_width", c.parent_width.clone());
    let mut items = Vec::new();
    for it in &c.items {
        let mut item = EditFormState::new(&editform::CARD_ITEM_FORM);
        item.set("child_image_url", it.child_image_url.clone());
        item.set("child_image_alt", it.child_image_alt.clone());
        item.set("child_title", it.child_title.clone());
        item.set("child_subtitle", it.child_subtitle.clone());
        item.set("child_copy", it.child_copy.clone());
        item.set(
            "child_link_url",
            it.child_link_url.clone().unwrap_or_default(),
        );
        item.set(
            "child_link_target",
            it.child_link_target
                .map(enum_serde_str)
                .unwrap_or_else(|| "_self".to_string()),
        );
        item.set(
            "child_link_label",
            it.child_link_label.clone().unwrap_or_default(),
        );
        item.set("child_link_style", enum_serde_str(it.child_link_style));
        items.push(item);
    }
    s.sub_state.insert("items".to_string(), items);
    s.selected_sub_item.insert("items".to_string(), 0);
    s
}
fn apply_card_values(c: &mut DdCard, state: &EditFormState) -> Result<()> {
    c.parent_type = parse_enum::<CardType>(state.get("parent_type"))?;
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    c.sal = sal;
    c.sal_duration = sal_duration;
    c.sal_delay = sal_delay;
    c.parent_width = state.get("parent_width").trim().to_string();
    c.items.clear();
    if let Some(items) = state.sub_state.get("items") {
        for item_s in items {
            let (link_url_opt, link_target_opt, link_label_opt, link_style) =
                apply_optional_child_link(item_s)?;
            c.items.push(CardItem {
                child_image_url: item_s.get("child_image_url").trim().to_string(),
                child_image_alt: item_s.get("child_image_alt").trim().to_string(),
                child_title: item_s.get("child_title").to_string(),
                child_subtitle: item_s.get("child_subtitle").to_string(),
                child_copy: item_s.get("child_copy").to_string(),
                child_link_url: link_url_opt,
                child_link_target: link_target_opt,
                child_link_label: link_label_opt,
                child_link_style: link_style,
            });
        }
    }
    Ok(())
}

pub fn filmstrip_to_form_state(f: &DdFilmstrip) -> EditFormState {
    let mut s = EditFormState::new(&editform::FILMSTRIP_FORM);
    s.set("parent_type", enum_serde_str(f.parent_type));
    set_sal_fields(&mut s, f.sal, f.sal_duration, f.sal_delay);
    let mut items = Vec::new();
    for it in &f.items {
        let mut item = EditFormState::new(&editform::FILMSTRIP_ITEM_FORM);
        item.set("child_image_url", it.child_image_url.clone());
        item.set("child_image_alt", it.child_image_alt.clone());
        item.set("child_title", it.child_title.clone());
        items.push(item);
    }
    s.sub_state.insert("items".to_string(), items);
    s.selected_sub_item.insert("items".to_string(), 0);
    s
}
fn apply_filmstrip_values(f: &mut DdFilmstrip, state: &EditFormState) -> Result<()> {
    f.parent_type = parse_enum::<FilmstripType>(state.get("parent_type"))?;
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    f.sal = sal;
    f.sal_duration = sal_duration;
    f.sal_delay = sal_delay;
    f.items.clear();
    if let Some(items) = state.sub_state.get("items") {
        for item_s in items {
            f.items.push(FilmstripItem {
                child_image_url: item_s.get("child_image_url").trim().to_string(),
                child_image_alt: item_s.get("child_image_alt").trim().to_string(),
                child_title: item_s.get("child_title").to_string(),
            });
        }
    }
    Ok(())
}

pub fn milestones_to_form_state(m: &DdMilestones) -> EditFormState {
    let mut s = EditFormState::new(&editform::MILESTONES_FORM);
    set_sal_fields(&mut s, m.sal, m.sal_duration, m.sal_delay);
    s.set("parent_width", m.parent_width.clone());
    let mut items = Vec::new();
    for it in &m.items {
        let mut item = EditFormState::new(&editform::MILESTONES_ITEM_FORM);
        item.set("child_percentage", it.child_percentage.clone());
        item.set("child_title", it.child_title.clone());
        item.set("child_subtitle", it.child_subtitle.clone());
        item.set("child_copy", it.child_copy.clone());
        item.set(
            "child_link_url",
            it.child_link_url.clone().unwrap_or_default(),
        );
        item.set(
            "child_link_target",
            it.child_link_target
                .map(enum_serde_str)
                .unwrap_or_else(|| "_self".to_string()),
        );
        item.set(
            "child_link_label",
            it.child_link_label.clone().unwrap_or_default(),
        );
        item.set("child_link_style", enum_serde_str(it.child_link_style));
        items.push(item);
    }
    s.sub_state.insert("items".to_string(), items);
    s.selected_sub_item.insert("items".to_string(), 0);
    s
}
fn apply_milestones_values(m: &mut DdMilestones, state: &EditFormState) -> Result<()> {
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    m.sal = sal;
    m.sal_duration = sal_duration;
    m.sal_delay = sal_delay;
    m.parent_width = state.get("parent_width").trim().to_string();
    m.items.clear();
    if let Some(items) = state.sub_state.get("items") {
        for item_s in items {
            let (link_url_opt, link_target_opt, link_label_opt, link_style) =
                apply_optional_child_link(item_s)?;
            m.items.push(MilestonesItem {
                child_percentage: item_s.get("child_percentage").trim().to_string(),
                child_title: item_s.get("child_title").to_string(),
                child_subtitle: item_s.get("child_subtitle").to_string(),
                child_copy: item_s.get("child_copy").to_string(),
                child_link_url: link_url_opt,
                child_link_target: link_target_opt,
                child_link_label: link_label_opt,
                child_link_style: link_style,
            });
        }
    }
    Ok(())
}

pub fn slider_to_form_state(sl: &DdSlider) -> EditFormState {
    let mut s = EditFormState::new(&editform::SLIDER_FORM);
    s.set("parent_title", sl.parent_title.clone());
    let mut items = Vec::new();
    for it in &sl.items {
        let mut item = EditFormState::new(&editform::SLIDER_ITEM_FORM);
        item.set("child_title", it.child_title.clone());
        item.set("child_copy", it.child_copy.clone());
        set_media_fields(&mut item, &it.resolved_media());
        set_links_subform(&mut item, "links", &it.resolved_links());
        items.push(item);
    }
    s.sub_state.insert("items".to_string(), items);
    s.selected_sub_item.insert("items".to_string(), 0);
    s
}
fn apply_slider_values(sl: &mut DdSlider, state: &EditFormState) -> Result<()> {
    sl.parent_title = state.get("parent_title").to_string();
    sl.items.clear();
    if let Some(items) = state.sub_state.get("items") {
        for item_s in items {
            let media = apply_media_fields(item_s)?;
            let (url, alt) = legacy_image_from_media(&media);
            sl.items.push(SliderItem {
                child_title: item_s.get("child_title").to_string(),
                child_copy: item_s.get("child_copy").to_string(),
                child_image_url: url,
                child_image_alt: alt,
                links: apply_links_subform(item_s, "links")?,
                child_link_url: None,
                child_link_target: None,
                child_link_label: None,
                media,
            });
        }
    }
    Ok(())
}

pub fn accordion_to_form_state(a: &DdAccordion) -> EditFormState {
    let mut s = EditFormState::new(&editform::ACCORDION_FORM);
    s.set("parent_type", enum_serde_str(a.parent_type));
    s.set("parent_class", enum_serde_str(a.parent_class));
    set_sal_fields(&mut s, a.sal, a.sal_duration, a.sal_delay);
    s.set("parent_group_name", a.parent_group_name.clone());
    let mut items = Vec::new();
    for it in &a.items {
        let mut item = EditFormState::new(&editform::ACCORDION_ITEM_FORM);
        item.set("child_title", it.child_title.clone());
        item.set("child_copy", it.child_copy.clone());
        items.push(item);
    }
    s.sub_state.insert("items".to_string(), items);
    s.selected_sub_item.insert("items".to_string(), 0);
    s
}
fn apply_accordion_values(a: &mut DdAccordion, state: &EditFormState) -> Result<()> {
    a.parent_type = parse_enum::<AccordionType>(state.get("parent_type"))?;
    a.parent_class = parse_enum::<AccordionClass>(state.get("parent_class"))?;
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    a.sal = sal;
    a.sal_duration = sal_duration;
    a.sal_delay = sal_delay;
    a.parent_group_name = state.get("parent_group_name").trim().to_string();
    a.items.clear();
    if let Some(items) = state.sub_state.get("items") {
        for item_s in items {
            a.items.push(AccordionItem {
                child_title: item_s.get("child_title").to_string(),
                child_copy: item_s.get("child_copy").to_string(),
            });
        }
    }
    Ok(())
}

pub fn alternating_to_form_state(a: &DdAlternating) -> EditFormState {
    let mut s = EditFormState::new(&editform::ALTERNATING_FORM);
    s.set("parent_type", enum_serde_str(a.parent_type));
    s.set("parent_class", a.parent_class.clone());
    set_sal_fields(&mut s, a.sal, a.sal_duration, a.sal_delay);
    let mut items = Vec::new();
    for it in &a.items {
        let mut item = EditFormState::new(&editform::ALTERNATING_ITEM_FORM);
        set_media_fields(&mut item, &it.resolved_media());
        item.set("child_title", it.child_title.clone());
        item.set("child_subtitle", it.child_subtitle.clone());
        item.set("child_copy", it.child_copy.clone());
        set_links_subform(&mut item, "links", &it.links);
        items.push(item);
    }
    s.sub_state.insert("items".to_string(), items);
    s.selected_sub_item.insert("items".to_string(), 0);
    s
}
fn apply_alternating_values(a: &mut DdAlternating, state: &EditFormState) -> Result<()> {
    a.parent_type = parse_enum::<AlternatingType>(state.get("parent_type"))?;
    a.parent_class = state.get("parent_class").to_string();
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    a.sal = sal;
    a.sal_duration = sal_duration;
    a.sal_delay = sal_delay;
    a.items.clear();
    if let Some(items) = state.sub_state.get("items") {
        for item_s in items {
            let media = apply_media_fields(item_s)?;
            let (url, alt) = legacy_image_from_media(&media);
            a.items.push(AlternatingItem {
                child_image_url: url,
                child_image_alt: alt,
                child_title: item_s.get("child_title").to_string(),
                child_subtitle: item_s.get("child_subtitle").to_string(),
                child_copy: item_s.get("child_copy").to_string(),
                links: apply_links_subform(item_s, "links")?,
                media,
            });
        }
    }
    Ok(())
}

// ==================== Tier C: Hero + Section ====================

pub fn hero_to_form_state(hero: &DdHero) -> EditFormState {
    let mut s = EditFormState::new(&editform::HERO_FORM);
    s.set("parent_title", hero.parent_title.clone());
    s.set("parent_subtitle", hero.parent_subtitle.clone());
    s.set("parent_copy", hero.parent_copy.clone().unwrap_or_default());
    s.set(
        "parent_class",
        hero.parent_class
            .map(enum_serde_str)
            .unwrap_or_else(|| "-full-full".to_string()),
    );
    set_sal_fields(
        &mut s,
        hero.sal.unwrap_or(SalAnimation::Fade),
        hero.sal_duration,
        hero.sal_delay,
    );
    s.set(
        "parent_custom_css",
        hero.parent_custom_css.clone().unwrap_or_default(),
    );
    set_media_fields(&mut s, &hero.resolved_media());
    s.set(
        "parent_image_class",
        hero.parent_image_class
            .map(enum_serde_str)
            .unwrap_or_else(|| "-full-full".to_string()),
    );
    s.set(
        "parent_image_mobile",
        hero.parent_image_mobile.clone().unwrap_or_default(),
    );
    s.set(
        "parent_image_tablet",
        hero.parent_image_tablet.clone().unwrap_or_default(),
    );
    s.set(
        "parent_image_desktop",
        hero.parent_image_desktop.clone().unwrap_or_default(),
    );
    set_links_subform(&mut s, "links", &hero.resolved_links());
    s.set(
        "overlay",
        hero.overlay
            .map(enum_serde_str)
            .unwrap_or_else(|| "none".to_string()),
    );
    s.set(
        "copy_position",
        hero.copy_position
            .map(enum_serde_str)
            .unwrap_or_else(|| "-left".to_string()),
    );
    s.set("id", hero.id.clone().unwrap_or_default());
    s.set("aria_label", hero.aria_label.clone().unwrap_or_default());
    s
}
fn apply_hero_values(hero: &mut DdHero, state: &EditFormState) -> Result<()> {
    hero.parent_title = state.get("parent_title").to_string();
    hero.parent_subtitle = state.get("parent_subtitle").to_string();
    let copy = state.get("parent_copy").to_string();
    hero.parent_copy = if copy.trim().is_empty() {
        None
    } else {
        Some(copy)
    };
    hero.parent_class = Some(parse_enum::<HeroImageClass>(state.get("parent_class"))?);
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    hero.sal = Some(sal);
    hero.sal_duration = sal_duration;
    hero.sal_delay = sal_delay;
    let css = state.get("parent_custom_css").trim().to_string();
    hero.parent_custom_css = if css.is_empty() { None } else { Some(css) };
    hero.media = apply_media_fields(state)?;
    let (url, alt) = legacy_image_from_media(&hero.media);
    hero.parent_image_url = url;
    hero.parent_image_alt = if alt.is_empty() { None } else { Some(alt) };
    hero.parent_image_class = Some(parse_enum::<HeroImageClass>(
        state.get("parent_image_class"),
    )?);
    for (field_id, slot) in [
        ("parent_image_mobile", &mut hero.parent_image_mobile),
        ("parent_image_tablet", &mut hero.parent_image_tablet),
        ("parent_image_desktop", &mut hero.parent_image_desktop),
    ] {
        let v = state.get(field_id).trim().to_string();
        *slot = if v.is_empty() { None } else { Some(v) };
    }
    hero.links = apply_links_subform(state, "links")?;
    if hero.links.len() > 2 {
        hero.links.truncate(2);
    }
    hero.link_1_label = None;
    hero.link_1_url = None;
    hero.link_1_target = None;
    hero.link_2_label = None;
    hero.link_2_url = None;
    hero.link_2_target = None;
    hero.overlay = match state.get("overlay") {
        "none" | "" => None,
        v => Some(parse_enum::<HeroOverlay>(v).context("invalid overlay")?),
    };
    hero.copy_position = Some(
        parse_enum::<HeroCopyPosition>(state.get("copy_position"))
            .context("invalid copy position")?,
    );
    let id = state.get("id").trim().to_string();
    hero.id = if id.is_empty() { None } else { Some(id) };
    let aria = state.get("aria_label").trim().to_string();
    hero.aria_label = if aria.is_empty() { None } else { Some(aria) };
    Ok(())
}

pub fn section_to_form_state(section: &DdSection) -> EditFormState {
    let mut s = EditFormState::new(&editform::SECTION_FORM);
    s.set("id", section.id.clone());
    s.set(
        "section_title",
        section.section_title.clone().unwrap_or_default(),
    );
    s.set("aria_label", section.aria_label.clone().unwrap_or_default());
    s.set(
        "section_class",
        section
            .section_class
            .map(enum_serde_str)
            .unwrap_or_else(|| "-full-contained".to_string()),
    );
    s.set(
        "bg",
        section
            .bg
            .map(enum_serde_str)
            .unwrap_or_else(|| "none".to_string()),
    );
    s.set(
        "padding",
        section
            .padding
            .map(enum_serde_str)
            .unwrap_or_else(|| "default".to_string()),
    );
    s.set("custom_css", section.custom_css.clone().unwrap_or_default());
    set_sal_fields(&mut s, section.sal, section.sal_duration, section.sal_delay);
    s.set(
        "item_box_class",
        section
            .item_box_class
            .map(enum_serde_str)
            .unwrap_or_else(|| "l-box".to_string()),
    );
    let mut columns = Vec::new();
    for col in &section.columns {
        let mut item = EditFormState::new(&editform::COLUMN_ITEM_FORM);
        item.set("id", col.id.clone());
        item.set("width_class", col.width_class.clone());
        columns.push(item);
    }
    s.sub_state.insert("columns".to_string(), columns);
    s.selected_sub_item.insert("columns".to_string(), 0);
    s
}
fn apply_section_values(section: &mut DdSection, state: &EditFormState) -> Result<()> {
    section.id = state.get("id").trim().to_string();
    let title = state.get("section_title").trim().to_string();
    section.section_title = if title.is_empty() { None } else { Some(title) };
    let aria = state.get("aria_label").trim().to_string();
    section.aria_label = if aria.is_empty() { None } else { Some(aria) };
    section.section_class = Some(parse_enum::<SectionClass>(state.get("section_class"))?);
    section.bg = match state.get("bg") {
        "none" | "" => None,
        v => Some(parse_enum::<SectionBg>(v).context("invalid bg")?),
    };
    section.padding = match state.get("padding") {
        "default" | "" => None,
        v => Some(parse_enum::<SectionPadding>(v).context("invalid padding")?),
    };
    let css = state.get("custom_css").trim().to_string();
    section.custom_css = if css.is_empty() { None } else { Some(css) };
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    section.sal = sal;
    section.sal_duration = sal_duration;
    section.sal_delay = sal_delay;
    section.item_box_class = Some(parse_enum::<SectionItemBoxClass>(
        state.get("item_box_class"),
    )?);

    // Reconcile columns: match existing columns by ID so components aren't
    // dropped when columns are merely renamed or reordered.
    let form_items = state.sub_state.get("columns").cloned().unwrap_or_default();
    let mut new_columns: Vec<SectionColumn> = Vec::with_capacity(form_items.len());
    for form_col in form_items {
        let new_id = form_col.get("id").trim().to_string();
        let new_width = form_col.get("width_class").trim().to_string();
        // Try to find an existing column with the same ID and steal its components.
        let existing = section.columns.iter().position(|c| c.id == new_id);
        let components = match existing {
            Some(pos) => section.columns.remove(pos).components,
            None => Vec::new(),
        };
        new_columns.push(SectionColumn {
            id: new_id,
            width_class: new_width,
            components,
        });
    }
    section.columns = new_columns;
    Ok(())
}

fn apply_head_values(head: &mut crate::model::DdHead, state: &EditFormState) -> Result<()> {
    head.title = state.get("title").to_string();
    let meta_title = state.get("meta_title").trim().to_string();
    head.meta_title = if meta_title.is_empty() {
        None
    } else {
        Some(meta_title)
    };
    let meta = state.get("meta_description").trim().to_string();
    head.meta_description = if meta.is_empty() { None } else { Some(meta) };
    let canon = state.get("canonical_url").trim().to_string();
    head.canonical_url = if canon.is_empty() { None } else { Some(canon) };
    let robots_str = state.get("robots").to_string();
    head.robots = match robots_str.trim() {
        "index, follow" | "index,follow" => crate::model::RobotsDirective::IndexFollow,
        "noindex, follow" | "noindex,follow" => crate::model::RobotsDirective::NoindexFollow,
        "index, nofollow" | "index,nofollow" => crate::model::RobotsDirective::IndexNofollow,
        "noindex, nofollow" | "noindex,nofollow" => crate::model::RobotsDirective::NoindexNofollow,
        _ => crate::model::RobotsDirective::IndexFollow,
    };
    let schema_str = state.get("schema_type").to_string();
    head.schema_type = match schema_str.trim() {
        "WebPage" => crate::model::SchemaType::WebPage,
        "Article" => crate::model::SchemaType::Article,
        "AboutPage" => crate::model::SchemaType::AboutPage,
        "ContactPage" => crate::model::SchemaType::ContactPage,
        "CollectionPage" => crate::model::SchemaType::CollectionPage,
        "Organization" => crate::model::SchemaType::Organization,
        "LocalBusiness" => crate::model::SchemaType::LocalBusiness,
        "Product" => crate::model::SchemaType::Product,
        "Service" => crate::model::SchemaType::Service,
        _ => crate::model::SchemaType::WebPage,
    };
    let og_t = state.get("og_title").trim().to_string();
    head.og_title = if og_t.is_empty() { None } else { Some(og_t) };
    let og_d = state.get("og_description").trim().to_string();
    head.og_description = if og_d.is_empty() { None } else { Some(og_d) };
    let og_i = state.get("og_image").trim().to_string();
    head.og_image = if og_i.is_empty() { None } else { Some(og_i) };
    Ok(())
}

fn require_css_hex(raw: &str, field: &str) -> Result<String> {
    let trimmed = raw.trim().to_string();
    super::theme::parse_hex_color(&trimmed)
        .with_context(|| format!("invalid {field}: expected hex color like '#RRGGBB'"))?;
    Ok(trimmed)
}

fn apply_site_values(site: &mut Site, state: &EditFormState) -> Result<()> {
    // Validate fallible fields first so a bad hex cannot leave name/lang/urls written.
    let primary = require_css_hex(state.get("primary_color"), "primary_color")?;
    let secondary = require_css_hex(state.get("secondary_color"), "secondary_color")?;
    let tertiary = require_css_hex(state.get("tertiary_color"), "tertiary_color")?;
    let support = require_css_hex(state.get("support_color"), "support_color")?;
    site.name = state.get("name").to_string();
    site.lang = state.get("lang").trim().to_string();
    let base = state.get("base_url").trim().to_string();
    site.base_url = if base.is_empty() { None } else { Some(base) };
    let export = state.get("export_dir").trim().to_string();
    site.export_dir = if export.is_empty() {
        None
    } else {
        Some(export)
    };
    site.theme.primary_color = primary;
    site.theme.secondary_color = secondary;
    site.theme.tertiary_color = tertiary;
    site.theme.support_color = support;
    site.header_gtm_tag = opt_trimmed(state.get("header_gtm_tag"));
    site.body_gtm_tag = opt_trimmed(state.get("body_gtm_tag"));
    Ok(())
}

fn apply_header_root_values(
    header: &mut crate::model::DdHeader,
    state: &EditFormState,
) -> Result<()> {
    header.id = state.get("id").to_string();
    let css = state.get("custom_css").trim().to_string();
    header.custom_css = if css.is_empty() { None } else { Some(css) };
    header.cta_label = opt_trimmed(state.get("cta_label"));
    header.cta_url = opt_trimmed(state.get("cta_url"));
    header.banner = opt_trimmed(state.get("banner"));
    Ok(())
}

fn apply_footer_values(footer: &mut crate::model::DdFooter, state: &EditFormState) -> Result<()> {
    footer.id = state.get("id").to_string();
    let css = state.get("custom_css").trim().to_string();
    footer.custom_css = if css.is_empty() { None } else { Some(css) };
    footer.blurb = opt_trimmed(state.get("blurb"));
    footer.copyright = opt_trimmed(state.get("copyright"));
    footer.social_linkedin = opt_trimmed(state.get("social_linkedin"));
    footer.social_x = opt_trimmed(state.get("social_x"));
    footer.social_github = opt_trimmed(state.get("social_github"));
    Ok(())
}

fn opt_trimmed(value: &str) -> Option<String> {
    let t = value.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

pub fn page_head_to_form_state(page: &crate::model::Page) -> EditFormState {
    let mut s = EditFormState::new(&editform::PAGE_HEAD_FORM);
    let head = &page.head;
    s.set("title", head.title.clone());
    s.set("slug", page.slug.clone());
    s.set("meta_title", head.meta_title.clone().unwrap_or_default());
    s.set(
        "meta_description",
        head.meta_description.clone().unwrap_or_default(),
    );
    let canon = head.canonical_url.clone().unwrap_or_default();
    s.set("canonical_url", canon);
    let robots = match head.robots {
        crate::model::RobotsDirective::IndexFollow => "index, follow",
        crate::model::RobotsDirective::NoindexFollow => "noindex, follow",
        crate::model::RobotsDirective::IndexNofollow => "index, nofollow",
        crate::model::RobotsDirective::NoindexNofollow => "noindex, nofollow",
    };
    s.set("robots", robots.to_string());
    let schema = match head.schema_type {
        crate::model::SchemaType::WebPage => "WebPage",
        crate::model::SchemaType::Article => "Article",
        crate::model::SchemaType::AboutPage => "AboutPage",
        crate::model::SchemaType::ContactPage => "ContactPage",
        crate::model::SchemaType::CollectionPage => "CollectionPage",
        crate::model::SchemaType::Organization => "Organization",
        crate::model::SchemaType::LocalBusiness => "LocalBusiness",
        crate::model::SchemaType::Product => "Product",
        crate::model::SchemaType::Service => "Service",
    };
    s.set("schema_type", schema.to_string());
    s.set(
        "og_title",
        head.og_title
            .clone()
            .unwrap_or_else(|| head.html_title().to_string()),
    );
    s.set(
        "og_description",
        head.og_description.clone().unwrap_or_default(),
    );
    s.set("og_image", head.og_image.clone().unwrap_or_default());
    s
}

pub fn site_to_form_state(site: &Site) -> EditFormState {
    let mut s = EditFormState::new(&editform::SITE_FORM);
    s.set("name", site.name.clone());
    s.set("lang", site.lang.clone());
    s.set("base_url", site.base_url.clone().unwrap_or_default());
    s.set("export_dir", site.export_dir.clone().unwrap_or_default());
    s.set("primary_color", site.theme.primary_color.clone());
    s.set("secondary_color", site.theme.secondary_color.clone());
    s.set("tertiary_color", site.theme.tertiary_color.clone());
    s.set("support_color", site.theme.support_color.clone());
    s.set(
        "header_gtm_tag",
        site.header_gtm_tag.clone().unwrap_or_default(),
    );
    s.set(
        "body_gtm_tag",
        site.body_gtm_tag.clone().unwrap_or_default(),
    );
    s
}

pub fn header_root_to_form_state(header: &crate::model::DdHeader) -> EditFormState {
    let mut s = EditFormState::new(&editform::HEADER_ROOT_FORM);
    s.set("id", header.id.clone());
    s.set("custom_css", header.custom_css.clone().unwrap_or_default());
    s.set("cta_label", header.cta_label.clone().unwrap_or_default());
    s.set("cta_url", header.cta_url.clone().unwrap_or_default());
    s.set("banner", header.banner.clone().unwrap_or_default());
    s
}

pub fn footer_to_form_state(footer: &crate::model::DdFooter) -> EditFormState {
    let mut s = EditFormState::new(&editform::FOOTER_FORM);
    s.set("id", footer.id.clone());
    s.set("custom_css", footer.custom_css.clone().unwrap_or_default());
    s.set("blurb", footer.blurb.clone().unwrap_or_default());
    s.set("copyright", footer.copyright.clone().unwrap_or_default());
    s.set(
        "social_linkedin",
        footer.social_linkedin.clone().unwrap_or_default(),
    );
    s.set("social_x", footer.social_x.clone().unwrap_or_default());
    s.set(
        "social_github",
        footer.social_github.clone().unwrap_or_default(),
    );
    s
}

// ==================== Tier D: dd-navigation (recursive) ====================

pub fn navigation_to_form_state(nav: &DdNavigation) -> EditFormState {
    let mut s = EditFormState::new(&editform::NAVIGATION_FORM);
    s.set("parent_type", enum_serde_str(nav.parent_type));
    s.set("parent_class", enum_serde_str(nav.parent_class));
    set_sal_fields(&mut s, nav.sal, nav.sal_duration, nav.sal_delay);
    s.set("parent_width", nav.parent_width.clone());
    let mut items = Vec::new();
    for it in &nav.items {
        items.push(nav_item_to_form_state(it));
    }
    s.sub_state.insert("items".to_string(), items);
    s.selected_sub_item.insert("items".to_string(), 0);
    s
}

fn nav_item_to_form_state(item: &NavigationItem) -> EditFormState {
    let mut s = EditFormState::new(&editform::NAV_ITEM_FORM);
    s.set("child_kind", enum_serde_str(item.child_kind));
    s.set("child_link_label", item.child_link_label.clone());
    s.set(
        "child_link_url",
        item.child_link_url.clone().unwrap_or_default(),
    );
    s.set(
        "child_link_target",
        item.child_link_target
            .map(enum_serde_str)
            .unwrap_or_else(|| "_self".to_string()),
    );
    s.set(
        "child_link_css",
        item.child_link_css.clone().unwrap_or_default(),
    );
    let mut nested = Vec::new();
    for inner in &item.items {
        nested.push(nav_item_to_form_state(inner));
    }
    s.sub_state.insert("items".to_string(), nested);
    s.selected_sub_item.insert("items".to_string(), 0);
    s
}

fn apply_navigation_values(nav: &mut DdNavigation, state: &EditFormState) -> Result<()> {
    nav.parent_type = parse_enum::<NavigationType>(state.get("parent_type"))?;
    nav.parent_class = parse_enum::<NavigationClass>(state.get("parent_class"))?;
    let (sal, sal_duration, sal_delay) = apply_sal_fields(state)?;
    nav.sal = sal;
    nav.sal_duration = sal_duration;
    nav.sal_delay = sal_delay;
    nav.parent_width = state.get("parent_width").trim().to_string();
    nav.items.clear();
    if let Some(items) = state.sub_state.get("items") {
        for item_s in items {
            nav.items.push(apply_nav_item(item_s)?);
        }
    }
    Ok(())
}

fn apply_nav_item(state: &EditFormState) -> Result<NavigationItem> {
    let kind = parse_enum::<NavigationKind>(state.get("child_kind"))?;
    let label = state.get("child_link_label").to_string();
    let url = state.get("child_link_url").trim().to_string();
    let target = state.get("child_link_target").to_string();
    let css = state.get("child_link_css").trim().to_string();

    let (child_link_url, child_link_target) = match kind {
        NavigationKind::Link => {
            let url_opt = if url.is_empty() { None } else { Some(url) };
            let target_opt = if url_opt.is_some() {
                Some(parse_enum::<CardLinkTarget>(&target)?)
            } else {
                None
            };
            (url_opt, target_opt)
        }
        NavigationKind::Button => (None, None),
    };

    let mut nested = Vec::new();
    if let Some(children) = state.sub_state.get("items") {
        for child in children {
            nested.push(apply_nav_item(child)?);
        }
    }

    Ok(NavigationItem {
        child_kind: kind,
        child_link_label: label,
        child_link_url,
        child_link_target,
        child_link_css: if css.is_empty() { None } else { Some(css) },
        items: nested,
    })
}

/// Serialize a serde enum to its `#[serde(rename = ...)]` string form.
fn enum_serde_str<T: serde::Serialize>(value: T) -> String {
    serde_json::to_value(&value)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default()
}

/// Parse a serde enum from its wire string form.
fn parse_enum<T: for<'de> serde::Deserialize<'de>>(input: &str) -> Result<T> {
    serde_json::from_value::<T>(serde_json::Value::String(input.to_string()))
        .map_err(|e| anyhow!("failed to parse '{}': {}", input, e))
}

fn apply_sal_fields(state: &EditFormState) -> Result<(SalAnimation, Option<u16>, Option<u16>)> {
    let sal = parse_enum::<SalAnimation>(state.get("sal")).context("invalid sal")?;
    Ok((
        sal,
        parse_sal_ms(state.get("sal_duration"), 400)?,
        parse_sal_ms(state.get("sal_delay"), 0)?,
    ))
}

fn parse_sal_ms(raw: &str, omit_value: u16) -> Result<Option<u16>> {
    let s = raw.trim();
    if s.is_empty() {
        return Ok(None);
    }
    let v: u16 = s
        .parse()
        .map_err(|_| anyhow!("invalid SAL milliseconds '{s}'"))?;
    if v == omit_value {
        Ok(None)
    } else {
        Ok(Some(v))
    }
}

fn set_sal_fields(
    state: &mut EditFormState,
    sal: SalAnimation,
    duration: Option<u16>,
    delay: Option<u16>,
) {
    state.set("sal", enum_serde_str(sal));
    state.set("sal_duration", duration.unwrap_or(400).to_string());
    state.set("sal_delay", delay.unwrap_or(0).to_string());
}

fn link_to_form_state(link: &DdLink) -> EditFormState {
    let mut s = EditFormState::new(&editform::LINK_ITEM_FORM);
    s.set("url", link.url.clone());
    s.set("label", link.label.clone());
    s.set("target", enum_serde_str(link.target));
    s.set("style", enum_serde_str(link.style));
    s
}

fn set_links_subform(state: &mut EditFormState, field_id: &str, links: &[DdLink]) {
    let items: Vec<EditFormState> = links.iter().map(link_to_form_state).collect();
    state.sub_state.insert(field_id.to_string(), items);
    state.selected_sub_item.insert(field_id.to_string(), 0);
}

fn apply_links_subform(state: &EditFormState, field_id: &str) -> Result<Vec<DdLink>> {
    let mut out = Vec::new();
    let Some(items) = state.sub_state.get(field_id) else {
        return Ok(out);
    };
    for (idx, item) in items.iter().enumerate() {
        let url = item.get("url").trim().to_string();
        let label = item.get("label").trim().to_string();
        if url.is_empty() && label.is_empty() {
            continue;
        }
        if url.is_empty() || label.is_empty() {
            return Err(anyhow!("link {} requires both URL and label", idx + 1));
        }
        out.push(DdLink {
            url,
            label,
            target: parse_enum(item.get("target")).context("invalid link target")?,
            style: parse_enum(item.get("style")).context("invalid button style")?,
        });
    }
    Ok(out)
}

fn set_media_fields(state: &mut EditFormState, media: &Media) {
    state.set("media_kind", media.kind_str());
    match media {
        Media::None => {}
        Media::Image { url, alt } => {
            state.set("media_image_url", url.clone());
            state.set("media_image_alt", alt.clone());
        }
        Media::Oembed { url } => {
            state.set("media_oembed_url", url.clone());
        }
        Media::LocalVideo {
            lg_mp4,
            sm_mp4,
            poster,
            name,
            loop_playback,
            autoplay,
        } => {
            state.set("media_lg_mp4", lg_mp4.clone());
            state.set("media_sm_mp4", sm_mp4.clone().unwrap_or_default());
            state.set("media_poster", poster.clone().unwrap_or_default());
            state.set("media_name", name.clone());
            state.set("media_loop", if *loop_playback { "true" } else { "false" });
            state.set("media_autoplay", if *autoplay { "true" } else { "false" });
        }
    }
}

fn apply_media_fields(state: &EditFormState) -> Result<Media> {
    match state.get("media_kind") {
        "none" | "" => Ok(Media::None),
        "image" => {
            let url = state.get("media_image_url").trim().to_string();
            let alt = state.get("media_image_alt").trim().to_string();
            if url.is_empty() {
                Ok(Media::None)
            } else {
                Ok(Media::Image { url, alt })
            }
        }
        "oembed" => {
            let url = state.get("media_oembed_url").trim().to_string();
            if url.is_empty() {
                Ok(Media::None)
            } else {
                Ok(Media::Oembed { url })
            }
        }
        "local-video" => {
            let lg_mp4 = state.get("media_lg_mp4").trim().to_string();
            if lg_mp4.is_empty() {
                return Ok(Media::None);
            }
            let sm = state.get("media_sm_mp4").trim().to_string();
            let poster = state.get("media_poster").trim().to_string();
            Ok(Media::LocalVideo {
                lg_mp4,
                sm_mp4: if sm.is_empty() { None } else { Some(sm) },
                poster: if poster.is_empty() {
                    None
                } else {
                    Some(poster)
                },
                name: state.get("media_name").trim().to_string(),
                loop_playback: state.get("media_loop") == "true",
                autoplay: state.get("media_autoplay") == "true",
            })
        }
        other => Err(anyhow!("invalid media kind '{other}'")),
    }
}

fn legacy_image_from_media(media: &Media) -> (String, String) {
    match media {
        Media::Image { url, alt } => (url.clone(), alt.clone()),
        _ => (String::new(), String::new()),
    }
}

fn apply_optional_child_link(
    item_s: &EditFormState,
) -> Result<(
    Option<String>,
    Option<CardLinkTarget>,
    Option<String>,
    ButtonStyle,
)> {
    let url = item_s.get("child_link_url").trim().to_string();
    let label = item_s.get("child_link_label").trim().to_string();
    let style = parse_enum::<ButtonStyle>(item_s.get("child_link_style")).unwrap_or_default();
    if url.is_empty() && label.is_empty() {
        Ok((None, None, None, style))
    } else {
        Ok((
            Some(url),
            Some(parse_enum::<CardLinkTarget>(
                item_s.get("child_link_target"),
            )?),
            Some(label),
            style,
        ))
    }
}

/// Remove the last `items` index from a cursor — used to navigate out of a
/// nested navigation item when editing its parent. Returns false if already
/// at component level.
#[allow(dead_code)] // exercised during Tier D (dd-navigation)
pub fn pop_items(cursor: &mut Cursor) -> bool {
    let items = match cursor {
        Cursor::HeaderComponent { items, .. }
        | Cursor::FooterComponent { items, .. }
        | Cursor::PageComponent { items, .. } => items,
        _ => return false,
    };
    items.pop().is_some()
}

/// Skip-counts FormField writes using the form's own declared kind. Returns
/// the number of values the form would attempt to round-trip — used by tests
/// to assert completeness of the wedge form wiring.
#[allow(dead_code)]
pub fn form_field_value_count(state: &EditFormState) -> usize {
    let mut count = 0usize;
    for field in state.form.fields {
        if !state.field_visible(field) {
            continue;
        }
        count += match &field.kind {
            FieldKind::OptionalLinkTriple { .. } => 3,
            _ => 1,
        };
    }
    count
}
