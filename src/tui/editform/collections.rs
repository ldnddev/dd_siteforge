//! Collection component forms and their item templates.
use super::*;

pub static LINK_ITEM_FORM: EditForm = EditForm {
    title: "link",
    fields: &[
        FormField {
            id: "label",
            label: "Label",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "url",
            label: "URL",
            kind: FieldKind::Url { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "target",
            label: "Target",
            kind: FieldKind::Enum {
                options: LINK_TARGET_OPTIONS,
                default: "_self",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "style",
            label: "Button style",
            kind: FieldKind::Enum {
                options: BUTTON_STYLE_OPTIONS,
                default: "-primary",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub static CARD_ITEM_FORM: EditForm = EditForm {
    title: "dd-card item",
    fields: &[
        FormField {
            id: "child_image_url",
            label: "Image URL",
            kind: FieldKind::Url { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_image_alt",
            label: "Image Alt",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_title",
            label: "Title",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_subtitle",
            label: "Subtitle",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_copy",
            label: "Copy (Markdown)",
            kind: FieldKind::Textarea {
                rows: 4,
                default: "",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_link_url",
            label: "Link URL (optional)",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "child_link_target",
            label: "Link Target",
            kind: FieldKind::Enum {
                options: LINK_TARGET_OPTIONS,
                default: "_self",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "child_link_label",
            label: "Link Label (optional)",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "child_link_style",
            label: "Button style",
            kind: FieldKind::Enum {
                options: BUTTON_STYLE_OPTIONS,
                default: "-primary",
            },
            required: false,
            visible_when: None,
        },
    ],
};

pub static FILMSTRIP_ITEM_FORM: EditForm = EditForm {
    title: "dd-filmstrip item",
    fields: &[
        FormField {
            id: "child_image_url",
            label: "Image URL",
            kind: FieldKind::Url { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_image_alt",
            label: "Image Alt",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_title",
            label: "Title",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
    ],
};

pub static MILESTONES_ITEM_FORM: EditForm = EditForm {
    title: "dd-milestones item",
    fields: &[
        FormField {
            id: "child_percentage",
            label: "Percentage",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_title",
            label: "Title",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_subtitle",
            label: "Subtitle",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_copy",
            label: "Copy (Markdown)",
            kind: FieldKind::Textarea {
                rows: 4,
                default: "",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_link_url",
            label: "Link URL (optional)",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "child_link_target",
            label: "Link Target",
            kind: FieldKind::Enum {
                options: LINK_TARGET_OPTIONS,
                default: "_self",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "child_link_label",
            label: "Link Label (optional)",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "child_link_style",
            label: "Button style",
            kind: FieldKind::Enum {
                options: BUTTON_STYLE_OPTIONS,
                default: "-primary",
            },
            required: false,
            visible_when: None,
        },
    ],
};

pub static SLIDER_ITEM_FORM: EditForm = EditForm {
    title: "dd-slider item",
    fields: &[
        FormField {
            id: "child_title",
            label: "Title",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_copy",
            label: "Copy (Markdown)",
            kind: FieldKind::Textarea {
                rows: 4,
                default: "",
            },
            required: true,
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
            id: "links",
            label: "Links",
            kind: FieldKind::SubForm {
                template: &LINK_ITEM_FORM,
                min_items: 0,
                max_items: Some(4),
                summary_field_id: "label",
            },
            required: false,
            visible_when: None,
        },
    ],
};

pub static ACCORDION_ITEM_FORM: EditForm = EditForm {
    title: "dd-accordion item",
    fields: &[
        FormField {
            id: "child_title",
            label: "Title",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_copy",
            label: "Content (Markdown)",
            kind: FieldKind::Textarea {
                rows: 5,
                default: "",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub static ALTERNATING_ITEM_FORM: EditForm = EditForm {
    title: "dd-alternating item",
    fields: &[
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
            id: "child_title",
            label: "Title",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_subtitle",
            label: "Subtitle (optional)",
            kind: FieldKind::Text {
                default: "Subtitle",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "child_copy",
            label: "Copy (Markdown)",
            kind: FieldKind::Textarea {
                rows: 5,
                default: "",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "links",
            label: "Links",
            kind: FieldKind::SubForm {
                template: &LINK_ITEM_FORM,
                min_items: 0,
                max_items: Some(4),
                summary_field_id: "label",
            },
            required: false,
            visible_when: None,
        },
    ],
};

pub static CARD_FORM: EditForm = EditForm {
    title: "dd-card",
    fields: &[
        FormField {
            id: "parent_type",
            label: "Layout",
            kind: FieldKind::Enum {
                options: &["-default", "-horizontal"],
                default: "-default",
            },
            required: true,
            visible_when: None,
        },
        SAL_STYLE_FIELD,
        SAL_DURATION_FIELD,
        SAL_DELAY_FIELD,
        FormField {
            id: "parent_width",
            label: "Width Classes",
            kind: FieldKind::Text {
                default: "dd-u-1-1 dd-u-md-12-24 dd-u-lg-8-24",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "items",
            label: "Items",
            kind: FieldKind::SubForm {
                template: &CARD_ITEM_FORM,
                min_items: 1,
                max_items: None,
                summary_field_id: "child_title",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub static FILMSTRIP_FORM: EditForm = EditForm {
    title: "dd-filmstrip",
    fields: &[
        FormField {
            id: "parent_type",
            label: "Direction",
            kind: FieldKind::Enum {
                options: &["-default", "-reverse"],
                default: "-default",
            },
            required: true,
            visible_when: None,
        },
        SAL_STYLE_FIELD,
        SAL_DURATION_FIELD,
        SAL_DELAY_FIELD,
        FormField {
            id: "items",
            label: "Items",
            kind: FieldKind::SubForm {
                template: &FILMSTRIP_ITEM_FORM,
                min_items: 1,
                max_items: None,
                summary_field_id: "child_title",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub static MILESTONES_FORM: EditForm = EditForm {
    title: "dd-milestones",
    fields: &[
        SAL_STYLE_FIELD,
        SAL_DURATION_FIELD,
        SAL_DELAY_FIELD,
        FormField {
            id: "parent_width",
            label: "Width Classes",
            kind: FieldKind::Text {
                default: "dd-u-1-1 dd-u-md-12-24",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "items",
            label: "Items",
            kind: FieldKind::SubForm {
                template: &MILESTONES_ITEM_FORM,
                min_items: 1,
                max_items: None,
                summary_field_id: "child_title",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub static SLIDER_FORM: EditForm = EditForm {
    title: "dd-slider",
    fields: &[
        FormField {
            id: "parent_title",
            label: "Slider Title",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "items",
            label: "Items",
            kind: FieldKind::SubForm {
                template: &SLIDER_ITEM_FORM,
                min_items: 1,
                max_items: None,
                summary_field_id: "child_title",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub static ACCORDION_FORM: EditForm = EditForm {
    title: "dd-accordion",
    fields: &[
        FormField {
            id: "parent_type",
            label: "Type",
            kind: FieldKind::Enum {
                options: &["-default", "-faq"],
                default: "-default",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "parent_class",
            label: "Variant",
            kind: FieldKind::Enum {
                options: &[
                    "-borderless",
                    "-compact",
                    "-primary",
                    "-secondary",
                    "-tertiary",
                ],
                default: "-primary",
            },
            required: true,
            visible_when: None,
        },
        SAL_STYLE_FIELD,
        SAL_DURATION_FIELD,
        SAL_DELAY_FIELD,
        FormField {
            id: "parent_group_name",
            label: "Group Name",
            kind: FieldKind::Text { default: "group1" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "items",
            label: "Items",
            kind: FieldKind::SubForm {
                template: &ACCORDION_ITEM_FORM,
                min_items: 1,
                max_items: None,
                summary_field_id: "child_title",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub static TABS_ITEM_FORM: EditForm = EditForm {
    title: "dd-tabs item",
    fields: &[
        FormField {
            id: "child_title",
            label: "Label",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_copy",
            label: "Panel (Markdown)",
            kind: FieldKind::Textarea {
                rows: 5,
                default: "",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub static TABS_FORM: EditForm = EditForm {
    title: "dd-tabs",
    fields: &[
        FormField {
            id: "parent_id",
            label: "Tabs ID",
            kind: FieldKind::Text { default: "tabs" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "parent_class",
            label: "Orientation",
            kind: FieldKind::Enum {
                options: TABS_ORIENTATION_OPTIONS,
                default: "-horizontal",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "aria_label",
            label: "ARIA label",
            kind: FieldKind::Text {
                default: "Content tabs",
            },
            required: false,
            visible_when: None,
        },
        SAL_STYLE_FIELD,
        SAL_DURATION_FIELD,
        SAL_DELAY_FIELD,
        FormField {
            id: "items",
            label: "Tabs",
            kind: FieldKind::SubForm {
                template: &TABS_ITEM_FORM,
                min_items: 1,
                max_items: None,
                summary_field_id: "child_title",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub static TIMELINE_ITEM_FORM: EditForm = EditForm {
    title: "dd-timeline item",
    fields: &[
        FormField {
            id: "child_year",
            label: "Year",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_datetime",
            label: "Datetime (optional)",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "child_title",
            label: "Title",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "heading_level",
            label: "Heading level",
            kind: FieldKind::Enum {
                options: HEADING_LEVEL_OPTIONS,
                default: "3",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_copy",
            label: "Copy (Markdown)",
            kind: FieldKind::Textarea {
                rows: 4,
                default: "",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "child_image_url",
            label: "Image URL (optional)",
            kind: FieldKind::Url { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "child_image_alt",
            label: "Image Alt (optional)",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
    ],
};

pub static TIMELINE_FORM: EditForm = EditForm {
    title: "dd-timeline",
    fields: &[
        FormField {
            id: "aria_label",
            label: "ARIA label",
            kind: FieldKind::Text {
                default: "Timeline",
            },
            required: false,
            visible_when: None,
        },
        SAL_STYLE_FIELD,
        SAL_DURATION_FIELD,
        SAL_DELAY_FIELD,
        FormField {
            id: "items",
            label: "Events",
            kind: FieldKind::SubForm {
                template: &TIMELINE_ITEM_FORM,
                min_items: 1,
                max_items: None,
                summary_field_id: "child_title",
            },
            required: true,
            visible_when: None,
        },
    ],
};

pub(super) const DATA_TABLE_ALIGN_OPTIONS: &[&str] = &["start", "center", "end"];
pub(super) const DATA_TABLE_DENSE_OPTIONS: &[&str] = &["comfortable", "dense"];
pub(super) const DATA_TABLE_CELL_TYPE_OPTIONS: &[&str] = &["text", "badge"];
pub(super) const DATA_TABLE_BADGE_OPTIONS: &[&str] = &["-critical", "-warning", "-info", "-pass"];

pub static DATA_TABLE_COLUMN_FORM: EditForm = EditForm {
    title: "dd-data-table column",
    fields: &[
        FormField {
            id: "label",
            label: "Label",
            kind: FieldKind::Text { default: "" },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "align",
            label: "Align",
            kind: FieldKind::Enum {
                options: DATA_TABLE_ALIGN_OPTIONS,
                default: "start",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "sortable",
            label: "Sortable",
            kind: FieldKind::Enum {
                options: BOOL_OPTIONS,
                default: "false",
            },
            required: false,
            visible_when: None,
        },
    ],
};

pub static DATA_TABLE_CELL_FORM: EditForm = EditForm {
    title: "dd-data-table cell",
    fields: &[
        FormField {
            id: "type",
            label: "Type",
            kind: FieldKind::Enum {
                options: DATA_TABLE_CELL_TYPE_OPTIONS,
                default: "text",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "text",
            label: "Text",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "badge",
            label: "Badge",
            kind: FieldKind::Enum {
                options: DATA_TABLE_BADGE_OPTIONS,
                default: "-info",
            },
            required: false,
            visible_when: Some(FieldPredicate::FieldEquals {
                other_id: "type",
                value: "badge",
            }),
        },
    ],
};

pub static DATA_TABLE_ROW_FORM: EditForm = EditForm {
    title: "dd-data-table row",
    fields: &[FormField {
        id: "cells",
        label: "Cells",
        kind: FieldKind::SubForm {
            template: &DATA_TABLE_CELL_FORM,
            min_items: 0,
            max_items: Some(5),
            summary_field_id: "text",
        },
        required: false,
        visible_when: None,
    }],
};

pub static DATA_TABLE_FORM: EditForm = EditForm {
    title: "dd-data-table",
    fields: &[
        FormField {
            id: "caption",
            label: "Caption",
            kind: FieldKind::Text {
                default: "Data table",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "dense",
            label: "Density",
            kind: FieldKind::Enum {
                options: DATA_TABLE_DENSE_OPTIONS,
                default: "comfortable",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "scroll_label",
            label: "Scroll label",
            kind: FieldKind::Text { default: "" },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "empty_message",
            label: "Empty message",
            kind: FieldKind::Text {
                default: "No data to display.",
            },
            required: false,
            visible_when: None,
        },
        FormField {
            id: "columns",
            label: "Columns",
            kind: FieldKind::SubForm {
                template: &DATA_TABLE_COLUMN_FORM,
                min_items: 1,
                max_items: Some(5),
                summary_field_id: "label",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "rows",
            label: "Rows",
            kind: FieldKind::SubForm {
                template: &DATA_TABLE_ROW_FORM,
                min_items: 0,
                max_items: None,
                summary_field_id: "label",
            },
            required: false,
            visible_when: None,
        },
    ],
};

pub static ALTERNATING_FORM: EditForm = EditForm {
    title: "dd-alternating",
    fields: &[
        FormField {
            id: "parent_type",
            label: "Alternation",
            kind: FieldKind::Enum {
                options: &["-default", "-reverse", "-no-alternate"],
                default: "-default",
            },
            required: true,
            visible_when: None,
        },
        FormField {
            id: "parent_class",
            label: "CSS Class",
            kind: FieldKind::Text {
                default: "-default",
            },
            required: true,
            visible_when: None,
        },
        SAL_STYLE_FIELD,
        SAL_DURATION_FIELD,
        SAL_DELAY_FIELD,
        FormField {
            id: "items",
            label: "Items",
            kind: FieldKind::SubForm {
                template: &ALTERNATING_ITEM_FORM,
                min_items: 1,
                max_items: None,
                summary_field_id: "child_title",
            },
            required: true,
            visible_when: None,
        },
    ],
};
