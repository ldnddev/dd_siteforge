//! Hero, section, page-head, header, footer, and navigation forms.
use super::*;

pub static HERO_FORM: EditForm = EditForm {
    title: "dd-hero",
    fields: &[
        FormField {
            id: "parent_title",
            label: "Title",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "parent_subtitle",
            label: "Subtitle",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "parent_copy",
            label: "Copy (Markdown)",
            kind: FieldKind::Textarea {
                rows: 5,
                default: "",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "parent_class",
            label: "Hero Class",
            kind: FieldKind::Enum {
                options: HERO_CLASS_OPTIONS,
                default: "-full-full",
            },
            required: true,
            visible_when: None,
        },
        SAL_STYLE_FIELD,
        SAL_DURATION_FIELD,
        SAL_DELAY_FIELD,
        FormField {
            id: "parent_custom_css",
            label: "Custom CSS (optional)",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        MEDIA_KIND_FIELD,
        MEDIA_IMAGE_URL_FIELD,
        MEDIA_IMAGE_ALT_FIELD,
        MEDIA_OEMBED_URL_FIELD,
        MEDIA_LG_MP4_FIELD,
        MEDIA_SM_MP4_FIELD,
        MEDIA_POSTER_FIELD,
        MEDIA_NAME_FIELD,
        MEDIA_LOOP_FIELD,
        MEDIA_AUTOPLAY_FIELD,
        FormField {
            id: "parent_image_class",
            label: "Image Class",
            kind: FieldKind::Enum {
                options: HERO_CLASS_OPTIONS,
                default: "-full-full",
            },
            required: true,
            visible_when: Some(FieldPredicate::FieldEquals {
                other_id: "media_kind",
                value: "image",
            }),
        },
        FormField {
            id: "parent_image_mobile",
            label: "Image (mobile, optional)",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: Some(FieldPredicate::FieldEquals {
                other_id: "media_kind",
                value: "image",
            }),
        },
        FormField {
            id: "parent_image_tablet",
            label: "Image (tablet, optional)",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: Some(FieldPredicate::FieldEquals {
                other_id: "media_kind",
                value: "image",
            }),
        },
        FormField {
            id: "parent_image_desktop",
            label: "Image (desktop, optional)",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: Some(FieldPredicate::FieldEquals {
                other_id: "media_kind",
                value: "image",
            }),
        },
        FormField {
            id: "links",
            label: "Links",
            kind: FieldKind::SubForm {
                template: &LINK_ITEM_FORM,
                min_items: 0,
                max_items: Some(2),
                summary_field_id: "label",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "overlay",
            label: "Overlay",
            kind: FieldKind::Enum {
                options: HERO_OVERLAY_OPTIONS,
                default: "none",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "copy_position",
            label: "Copy position",
            kind: FieldKind::Enum {
                options: HERO_COPY_POSITION_OPTIONS,
                default: "-left",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "id",
            label: "Hero ID",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "aria_label",
            label: "ARIA label",
            kind: FieldKind::Text {
                default: "Introduction",
            },
            required: false,
            visible_when: None,
        },
    ],
};

pub static COLUMN_ITEM_FORM: EditForm = EditForm {
    title: "column",
    fields: &[
        FormField {
            id: "id",
            label: "Column ID",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "width_class",
            label: "Width Class (dd-u-*)",
            kind: FieldKind::Text {
                default: "dd-u-1-1",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub static SECTION_FORM: EditForm = EditForm {
    title: "dd-section",
    fields: &[
        FormField {
            id: "id",
            label: "Section ID",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "section_title",
            label: "Section Title (optional)",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "aria_label",
            label: "ARIA label (optional)",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "section_class",
            label: "Width",
            kind: FieldKind::Enum {
                options: SECTION_CLASS_OPTIONS,
                default: "-full-contained",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "bg",
            label: "Background",
            kind: FieldKind::Enum {
                options: SECTION_BG_OPTIONS,
                default: "none",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "padding",
            label: "Padding",
            kind: FieldKind::Enum {
                options: SECTION_PADDING_OPTIONS,
                default: "default",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "custom_css",
            label: "Extra CSS (optional)",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        SAL_STYLE_FIELD,
        SAL_DURATION_FIELD,
        SAL_DELAY_FIELD,
        FormField {
            id: "item_box_class",
            label: "Item Box Class",
            kind: FieldKind::Enum {
                options: ITEM_BOX_CLASS_OPTIONS,
                default: "l-box",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "columns",
            label: "Columns",
            kind: FieldKind::SubForm {
                template: &COLUMN_ITEM_FORM,
                min_items: 1,
                max_items: None,
                summary_field_id: "id",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub static PAGE_HEAD_FORM: EditForm = EditForm {
    title: "page-head",
    fields: &[
        FormField {
            id: "title",
            label: "Title",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "slug",
            label: "Slug / path",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "meta_title",
            label: "Meta Title",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "meta_description",
            label: "Meta Description",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "canonical_url",
            label: "Canonical URL",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "robots",
            label: "Robots",
            kind: FieldKind::Enum {
                options: ROBOTS_OPTIONS,
                default: "index, follow",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "schema_type",
            label: "Schema Type",
            kind: FieldKind::Enum {
                options: SCHEMA_OPTIONS,
                default: "WebPage",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "og_title",
            label: "OG Title",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "og_description",
            label: "OG Description",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "og_image",
            label: "OG Image",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
    ],
};

pub static HEADER_ROOT_FORM: EditForm = EditForm {
    title: "dd-header-root",
    fields: &[
        FormField {
            id: "id",
            label: "Header ID",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "custom_css",
            label: "Custom CSS",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "cta_label",
            label: "CTA label",
            kind: FieldKind::Text { default: "Contact" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "cta_url",
            label: "CTA URL",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "banner",
            label: "Alert banner",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
    ],
};

pub static FOOTER_FORM: EditForm = EditForm {
    title: "dd-footer",
    fields: &[
        FormField {
            id: "id",
            label: "Footer ID",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "custom_css",
            label: "Custom CSS",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "blurb",
            label: "Footer blurb",
            kind: FieldKind::Textarea {
                rows: 3,
                default: "",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "copyright",
            label: "Copyright override",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "social_linkedin",
            label: "LinkedIn URL",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "social_x",
            label: "X URL",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "social_github",
            label: "GitHub URL",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: None,
        },
    ],
};

pub static SITE_FORM: EditForm = EditForm {
    title: "Site settings",
    fields: &[
        FormField {
            id: "name",
            label: "Name",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "lang",
            label: "Lang",
            kind: FieldKind::Text { default: "en" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "base_url",
            label: "Base URL",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "export_dir",
            label: "Export Dir",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "pretty_urls",
            label: "Pretty URLs",
            kind: FieldKind::Enum {
                options: &["off", "on"],
                default: "off",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "primary_color",
            label: "Primary Color",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "secondary_color",
            label: "Secondary Color",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "tertiary_color",
            label: "Tertiary Color",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "support_color",
            label: "Support Color",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "header_gtm_tag",
            label: "Header GTM snippet",
            kind: FieldKind::Textarea {
                rows: 6,
                default: "",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "body_gtm_tag",
            label: "Body GTM snippet",
            kind: FieldKind::Textarea {
                rows: 6,
                default: "",
            },
            required: false,
            visible_when: None,
        },
    ],
};

/// NAV_ITEM_FORM is self-referential — its `items` field is a SubForm whose
/// template is `&NAV_ITEM_FORM`. Rust permits this because the address of a
/// `static` is known at compile time.
pub static NAV_ITEM_FORM: EditForm = EditForm {
    title: "nav item",
    fields: &[
        FormField {
            id: "child_kind",
            label: "Kind",
            kind: FieldKind::Enum {
                options: &["link", "button"],
                default: "link",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_link_label",
            label: "Label",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_link_url",
            label: "URL",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: Some(FieldPredicate::FieldEquals {
                other_id: "child_kind",
                value: "link",
            }),
        },
        FormField {
            id: "child_link_target",
            label: "Target",
            kind: FieldKind::Enum {
                options: LINK_TARGET_OPTIONS,
                default: "_self",
            },
            required: false,
            visible_when: Some(FieldPredicate::FieldEquals {
                other_id: "child_kind",
                value: "link",
            }),
        },
        FormField {
            id: "child_link_css",
            label: "CSS Class (optional)",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "items",
            label: "Nested items",
            kind: FieldKind::SubForm {
                template: &NAV_ITEM_FORM,
                min_items: 0,
                max_items: None,
                summary_field_id: "child_link_label",
            },
            required: false,
            visible_when: None,
        },
    ],
};

pub static NAVIGATION_FORM: EditForm = EditForm {
    title: "dd-navigation",
    fields: &[
        FormField {
            id: "parent_type",
            label: "Type",
            kind: FieldKind::Enum {
                options: &["dd-header__navigation", "dd-footer__navigation"],
                default: "dd-header__navigation",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "parent_class",
            label: "Menu Style",
            kind: FieldKind::Enum {
                options: &[
                    "-main-menu",
                    "-menu-secondary",
                    "-menu-tertiary",
                    "-footer-menu",
                    "-footer-menu-secondary",
                    "-footer-menu-tertiary",
                    "-social-menu",
                ],
                default: "-main-menu",
            },
            required: true,
            visible_when: None,
        },
        SAL_STYLE_FIELD,
        SAL_DURATION_FIELD,
        SAL_DELAY_FIELD,
        FormField {
            id: "items",
            label: "Menu Items",
            kind: FieldKind::SubForm {
                template: &NAV_ITEM_FORM,
                min_items: 1,
                max_items: None,
                summary_field_id: "child_link_label",
            },
            required: true,
            visible_when: None,
        },
    ],
};
