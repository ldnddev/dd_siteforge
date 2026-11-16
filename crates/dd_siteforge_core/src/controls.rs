//! Inspector control for each `EditForm` field. Spec 04 §2.
//!
//! The GUI draws these. It does not add fields, options, or validation.

use crate::editform::{FieldKind, FormField};

/// The control spec 04 maps a TUI field onto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldControl {
    Text,
    Markdown,
    Image {
        video: bool,
    },
    PagePicker,
    Toggle,
    Select,
    /// Nine-cell position enum (`-top-left` … or `-bg-top-left` …).
    Position,
    /// `media_kind` options `none` / `image` / `oembed` / `local-video`.
    Media,
    /// Free-text `dd-u-*` class string (`width_class`, `parent_width`).
    Width,
    Collection(CollectionRows),
    /// `OptionalLinkTriple` is reserved and unused.
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectionRows {
    Generic,
    Links,
    Nav,
}

impl FieldControl {
    pub fn is_toggle(self) -> bool {
        matches!(self, Self::Toggle)
    }
}

pub fn control_for(field: &FormField) -> FieldControl {
    match &field.kind {
        FieldKind::Text { .. } => {
            if field.id == "width_class" || field.id == "parent_width" {
                FieldControl::Width
            } else {
                FieldControl::Text
            }
        }
        FieldKind::Textarea { .. } => FieldControl::Markdown,
        FieldKind::Url { .. } => {
            if field.id.contains("mp4") {
                FieldControl::Image { video: true }
            } else if field.id.contains("image") || field.id.contains("poster") {
                FieldControl::Image { video: false }
            } else if field.id.contains("link") {
                FieldControl::PagePicker
            } else {
                FieldControl::Text
            }
        }
        FieldKind::Enum { options, .. } => {
            if is_toggle(options) {
                FieldControl::Toggle
            } else if is_media(options) {
                FieldControl::Media
            } else if is_position(options) {
                FieldControl::Position
            } else {
                FieldControl::Select
            }
        }
        FieldKind::SubForm { .. } => {
            if field.id == "links" {
                FieldControl::Collection(CollectionRows::Links)
            } else if field.label.contains("Menu") || field.label.contains("Nested") {
                FieldControl::Collection(CollectionRows::Nav)
            } else {
                FieldControl::Collection(CollectionRows::Generic)
            }
        }
        FieldKind::OptionalLinkTriple { .. } => FieldControl::None,
    }
}

fn is_toggle(options: &[&str]) -> bool {
    let mut lower: Vec<String> = options.iter().map(|opt| opt.to_ascii_lowercase()).collect();
    lower.sort();
    lower == ["false", "true"] || lower == ["off", "on"]
}

fn is_media(options: &[&str]) -> bool {
    let mut set: Vec<&str> = options.to_vec();
    set.sort_unstable();
    set == ["image", "local-video", "none", "oembed"]
}

fn is_position(options: &[&str]) -> bool {
    options.len() == 9
        && options.iter().any(|opt| opt.contains("top"))
        && options.iter().any(|opt| opt.contains("bottom"))
        && options.iter().any(|opt| opt.contains("left"))
}

/// Markdown toolbar inserts. Spec 04 §2.4. No preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkdownInsert {
    H2,
    Bold,
    Italic,
    Link,
    List,
}

impl MarkdownInsert {
    pub const BUTTONS: &'static [Self] =
        &[Self::H2, Self::Bold, Self::Italic, Self::Link, Self::List];

    pub fn label(self) -> &'static str {
        match self {
            Self::H2 => "H2",
            Self::Bold => "B",
            Self::Italic => "I",
            Self::Link => "Link",
            Self::List => "List",
        }
    }
}

/// Insert the TUI-less toolbar snippet. The buttons add Markdown text only.
pub fn apply_markdown_insert(value: &str, insert: MarkdownInsert) -> String {
    match insert {
        MarkdownInsert::H2 => {
            if value.is_empty() {
                "## ".to_string()
            } else {
                format!("{value}\n## ")
            }
        }
        MarkdownInsert::Bold => format!("**{value}**"),
        MarkdownInsert::Italic => format!("*{value}*"),
        MarkdownInsert::Link => format!("[{value}](url)"),
        MarkdownInsert::List => {
            if value.is_empty() {
                "- ".to_string()
            } else {
                format!("- {value}")
            }
        }
    }
}

/// Which inspector tab a field belongs to. Empty tabs are not shown.
/// Content keeps TUI order; Media and Animation are the shared groups.
pub fn inspector_tab(id: &str) -> &'static str {
    if id == "sal" || id == "sal_duration" || id == "sal_delay" {
        "Animation"
    } else if id.starts_with("media_") || id.starts_with("parent_image") {
        "Media"
    } else {
        "Content"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editform::{
        ACCORDION_FORM, ACCORDION_ITEM_FORM, ALERT_FORM, ALTERNATING_FORM, ALTERNATING_ITEM_FORM,
        BANNER_FORM, BLOCKQUOTE_FORM, CARD_FORM, CARD_ITEM_FORM, COLUMN_ITEM_FORM, CTA_FORM,
        DATA_TABLE_CELL_FORM, DATA_TABLE_COLUMN_FORM, DATA_TABLE_FORM, DATA_TABLE_ROW_FORM,
        EditForm, FILMSTRIP_FORM, FILMSTRIP_ITEM_FORM, FOOTER_FORM, FieldKind, HEADER_MENU_FORM,
        HEADER_ROOT_FORM, HEADER_SEARCH_FORM, HEADLINE_FORM, HERO_FORM, IMAGE_FORM, LINK_ITEM_FORM,
        MILESTONES_FORM, MILESTONES_ITEM_FORM, MODAL_FORM, NAV_ITEM_FORM, NAVIGATION_FORM,
        PAGE_HEAD_FORM, RICH_TEXT_FORM, SEARCH_RESULTS_FORM, SECTION_FORM, SITE_FORM, SLIDER_FORM,
        SLIDER_ITEM_FORM, SPACER_FORM, TABS_FORM, TABS_ITEM_FORM, TIMELINE_FORM,
        TIMELINE_ITEM_FORM,
    };
    use std::collections::BTreeSet;

    fn forms() -> Vec<&'static EditForm> {
        vec![
            &HERO_FORM,
            &COLUMN_ITEM_FORM,
            &SECTION_FORM,
            &PAGE_HEAD_FORM,
            &HEADER_ROOT_FORM,
            &FOOTER_FORM,
            &SITE_FORM,
            &NAV_ITEM_FORM,
            &NAVIGATION_FORM,
            &CTA_FORM,
            &BANNER_FORM,
            &IMAGE_FORM,
            &HEADER_SEARCH_FORM,
            &HEADER_MENU_FORM,
            &SEARCH_RESULTS_FORM,
            &HEADLINE_FORM,
            &RICH_TEXT_FORM,
            &SPACER_FORM,
            &ALERT_FORM,
            &MODAL_FORM,
            &BLOCKQUOTE_FORM,
            &LINK_ITEM_FORM,
            &CARD_ITEM_FORM,
            &FILMSTRIP_ITEM_FORM,
            &MILESTONES_ITEM_FORM,
            &SLIDER_ITEM_FORM,
            &ACCORDION_ITEM_FORM,
            &ALTERNATING_ITEM_FORM,
            &CARD_FORM,
            &FILMSTRIP_FORM,
            &MILESTONES_FORM,
            &SLIDER_FORM,
            &ACCORDION_FORM,
            &TABS_ITEM_FORM,
            &TABS_FORM,
            &TIMELINE_ITEM_FORM,
            &TIMELINE_FORM,
            &DATA_TABLE_COLUMN_FORM,
            &DATA_TABLE_CELL_FORM,
            &DATA_TABLE_ROW_FORM,
            &DATA_TABLE_FORM,
            &ALTERNATING_FORM,
        ]
    }

    #[test]
    fn every_drawn_control_is_used_and_optional_link_triple_is_not() {
        let mut seen = BTreeSet::new();
        let mut kinds = BTreeSet::new();
        for form in forms() {
            for field in form.fields {
                let control = control_for(field);
                seen.insert(format!("{control:?}"));
                kinds.insert(format!("{:?}", std::mem::discriminant(&field.kind)));
                assert_ne!(
                    control,
                    FieldControl::None,
                    "{} {} should have a control",
                    form.title,
                    field.id
                );
                if field.id == "og_image_width" || field.id == "default_og_image_width" {
                    assert_eq!(control, FieldControl::Text);
                }
            }
        }
        for expected in [
            "Text",
            "Markdown",
            "Image { video: false }",
            "Image { video: true }",
            "PagePicker",
            "Toggle",
            "Select",
            "Position",
            "Media",
            "Width",
            "Collection(Generic)",
            "Collection(Links)",
            "Collection(Nav)",
        ] {
            assert!(seen.contains(expected), "missing {expected} in {seen:?}");
        }
        assert!(kinds.len() >= 5, "field kinds: {kinds:?}");
        let unused = FormField {
            id: "unused",
            label: "Unused",
            kind: FieldKind::OptionalLinkTriple {
                url_id: "url",
                target_id: "target",
                label_id: "label",
            },
            required: false,
            visible_when: None,
        };
        assert_eq!(control_for(&unused), FieldControl::None);
    }

    #[test]
    fn markdown_inserts_have_no_preview_and_wrap_text() {
        let labels: Vec<_> = MarkdownInsert::BUTTONS
            .iter()
            .map(|button| button.label())
            .collect();
        assert_eq!(labels, ["H2", "B", "I", "Link", "List"]);
        assert!(
            !labels
                .iter()
                .any(|label| label.eq_ignore_ascii_case("preview"))
        );
        assert_eq!(
            apply_markdown_insert("Copy", MarkdownInsert::Bold),
            "**Copy**"
        );
        assert_eq!(apply_markdown_insert("", MarkdownInsert::H2), "## ");
        assert_eq!(
            apply_markdown_insert("docs", MarkdownInsert::Link),
            "[docs](url)"
        );
    }

    #[test]
    fn tabs_follow_the_sal_and_media_groups() {
        assert_eq!(inspector_tab("sal_duration"), "Animation");
        assert_eq!(inspector_tab("media_kind"), "Media");
        assert_eq!(inspector_tab("parent_image_url"), "Media");
        assert_eq!(inspector_tab("parent_title"), "Content");
        assert_eq!(inspector_tab("parent_image_class"), "Media");
    }
}
