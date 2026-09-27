//! EditForm data model — the backbone of the unified component editor.
//!
//! Each editable component is described by a single `EditForm` literal
//! (a pure data value). One render function draws any form; one dispatch
//! applies field values back to the model. Adding a new component is a
//! data-only change — write an `EditForm`, wire its variant in the save
//! dispatch, and it gains the same editor UX as every other component.
//!
//! Future phases remain additive:
//!   - codegen from `components/dd-*.md` YAML → same `EditForm` values
//!   - runtime-loaded specs → `EditForm` parsed at app startup
//!   - user-authored components → `SectionComponent::Dynamic` variant
//! No further TUI logic is required for those phases; only data plumbing.

use std::collections::HashMap;

mod blocks;
mod collections;
mod layout;

pub use blocks::*;
pub use collections::*;
pub use layout::*;

/// Static description of an editable component's fields. One instance per
/// component type, stored as a `static` item and referenced by the editor.
#[derive(Debug, Clone, Copy)]
pub struct EditForm {
    pub title: &'static str,
    pub fields: &'static [FormField],
}

/// One editable field inside an `EditForm`.
#[derive(Debug, Clone, Copy)]
pub struct FormField {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: FieldKind,
    #[allow(dead_code)]
    pub required: bool,
    pub visible_when: Option<FieldPredicate>,
}

/// Shape of a field. The editor renders differently for each variant and
/// the save dispatch decodes values back into typed model fields.
#[derive(Debug, Clone, Copy)]
pub enum FieldKind {
    /// Single-line text input.
    Text { default: &'static str },
    /// Multi-line text area. `rows` is the rendered height in rows.
    Textarea { rows: u16, default: &'static str },
    /// Single-line text input validated as URL at save time.
    Url { default: &'static str },
    /// Cyclable enum. `options` carries the serde-wire strings (e.g. `-top-left`).
    Enum {
        options: &'static [&'static str],
        default: &'static str,
    },
    /// Three-in-one optional link: url + target + label, rendered as one
    /// logical field with three child inputs. Submits the triple together
    /// only when all three non-empty.
    #[allow(dead_code)]
    OptionalLinkTriple {
        url_id: &'static str,
        target_id: &'static str,
        label_id: &'static str,
    },
    /// Collection of sub-items. Each item follows `template`'s shape. The
    /// editor renders the collection as a summary list and drills into a
    /// nested FormEdit for individual-item editing. `summary_field_id` tells
    /// the parent renderer which field of the item to surface as the row
    /// summary (usually `child_title` or equivalent).
    SubForm {
        template: &'static EditForm,
        min_items: usize,
        /// `None` means no cap. Hero links use `Some(2)`; CTA / alternating /
        /// slider use `Some(4)`.
        max_items: Option<usize>,
        summary_field_id: &'static str,
    },
}

/// Predicate that gates whether a field is visible. The editor skips hidden
/// fields during Tab traversal and the renderer draws them dimmed or not at
/// all.
#[derive(Debug, Clone, Copy)]
pub enum FieldPredicate {
    FieldEquals {
        other_id: &'static str,
        value: &'static str,
    },
    FieldNotEquals {
        other_id: &'static str,
        value: &'static str,
    },
}

/// Live editor state for one form. Held inside `Modal::FormEdit`.
#[derive(Debug, Clone)]
pub struct EditFormState {
    pub form: &'static EditForm,
    pub values: HashMap<String, String>,
    /// For each `SubForm` field (keyed by its id), the live state of every
    /// item in the collection. Each item is itself an `EditFormState` whose
    /// `form` is the SubForm's item template.
    pub sub_state: HashMap<String, Vec<EditFormState>>,
    /// For each `SubForm` field, which item index is currently highlighted
    /// in the summary list (used by Up/Down/A/X/Enter when focus is on the
    /// SubForm field).
    pub selected_sub_item: HashMap<String, usize>,
    pub focused_field: usize,
    /// (row, col) cursor inside a `Textarea` field; only meaningful when
    /// `focused_field` points at a Textarea.
    pub textarea_cursor: (usize, usize),
}

impl EditFormState {
    /// Build a fresh state with every field initialised to its declared default.
    pub fn new(form: &'static EditForm) -> Self {
        let mut values = HashMap::new();
        let mut sub_state: HashMap<String, Vec<EditFormState>> = HashMap::new();
        let mut selected_sub_item: HashMap<String, usize> = HashMap::new();
        for field in form.fields {
            match &field.kind {
                FieldKind::Text { default } | FieldKind::Url { default } => {
                    values.insert(field.id.to_string(), default.to_string());
                }
                FieldKind::Textarea { default, .. } => {
                    values.insert(field.id.to_string(), default.to_string());
                }
                FieldKind::Enum { default, .. } => {
                    values.insert(field.id.to_string(), default.to_string());
                }
                FieldKind::OptionalLinkTriple {
                    url_id,
                    target_id,
                    label_id,
                } => {
                    values.insert(url_id.to_string(), String::new());
                    values.insert(target_id.to_string(), "_self".to_string());
                    values.insert(label_id.to_string(), String::new());
                }
                FieldKind::SubForm { .. } => {
                    sub_state.insert(field.id.to_string(), Vec::new());
                    selected_sub_item.insert(field.id.to_string(), 0);
                }
            }
        }
        Self {
            form,
            values,
            sub_state,
            selected_sub_item,
            focused_field: 0,
            textarea_cursor: (0, 0),
        }
    }

    /// Make an item-level state for adding a new item to the given SubForm.
    /// Returns None if the field isn't a SubForm.
    pub fn new_sub_item(&self, subform_field_id: &str) -> Option<EditFormState> {
        for field in self.form.fields {
            if field.id == subform_field_id {
                if let FieldKind::SubForm { template, .. } = &field.kind {
                    return Some(EditFormState::new(*template));
                }
            }
        }
        None
    }

    pub fn get(&self, id: &str) -> &str {
        self.values.get(id).map(String::as_str).unwrap_or("")
    }

    pub fn set(&mut self, id: &str, value: impl Into<String>) {
        self.values.insert(id.to_string(), value.into());
    }

    pub fn field_visible(&self, field: &FormField) -> bool {
        match &field.visible_when {
            None => true,
            Some(FieldPredicate::FieldEquals { other_id, value }) => self.get(other_id) == *value,
            Some(FieldPredicate::FieldNotEquals { other_id, value }) => {
                self.get(other_id) != *value
            }
        }
    }

    /// Indices of visible fields in tab order.
    pub fn visible_field_indices(&self) -> Vec<usize> {
        self.form
            .fields
            .iter()
            .enumerate()
            .filter_map(|(idx, field)| self.field_visible(field).then_some(idx))
            .collect()
    }

    /// Advance `focused_field` to the next visible field, wrapping.
    pub fn focus_next(&mut self) {
        let visible = self.visible_field_indices();
        if visible.is_empty() {
            return;
        }
        let current_pos = visible
            .iter()
            .position(|&i| i == self.focused_field)
            .unwrap_or(0);
        let next_pos = (current_pos + 1) % visible.len();
        self.focused_field = visible[next_pos];
        self.textarea_cursor = (0, 0);
    }

    /// Retreat `focused_field` to the previous visible field, wrapping.
    pub fn focus_prev(&mut self) {
        let visible = self.visible_field_indices();
        if visible.is_empty() {
            return;
        }
        let current_pos = visible
            .iter()
            .position(|&i| i == self.focused_field)
            .unwrap_or(0);
        let prev_pos = if current_pos == 0 {
            visible.len() - 1
        } else {
            current_pos - 1
        };
        self.focused_field = visible[prev_pos];
        self.textarea_cursor = (0, 0);
    }

    pub fn focused(&self) -> Option<&FormField> {
        self.form.fields.get(self.focused_field)
    }

    /// Cycle the focused enum field forward (`forward = true`) or backward.
    /// No-op when the focused field is not an enum.
    pub fn cycle_enum(&mut self, forward: bool) {
        let Some(field) = self.focused() else { return };
        let FieldKind::Enum { options, .. } = &field.kind else {
            return;
        };
        if options.is_empty() {
            return;
        }
        let current = self.get(field.id).to_string();
        let idx = options
            .iter()
            .position(|opt| *opt == current.as_str())
            .unwrap_or(0);
        let next = if forward {
            (idx + 1) % options.len()
        } else if idx == 0 {
            options.len() - 1
        } else {
            idx - 1
        };
        let new_value = options[next].to_string();
        let field_id = field.id;
        self.set(field_id, new_value);
    }
}

// Shared option lists reused by several forms.
pub(super) const SAL_OPTIONS: &[&str] = &[
    "no-animation",
    "fade",
    "slide-up",
    "slide-down",
    "slide-left",
    "slide-right",
    "zoom-in",
    "zoom-out",
    "flip-up",
    "flip-down",
    "flip-left",
    "flip-right",
];

/// CSS-valid `data-sal-duration` steps (200–2000 ms).
pub(super) const SAL_DURATION_OPTIONS: &[&str] = &[
    "200", "250", "300", "350", "400", "450", "500", "550", "600", "650", "700", "750", "800",
    "850", "900", "950", "1000", "1050", "1100", "1150", "1200", "1250", "1300", "1350", "1400",
    "1450", "1500", "1550", "1600", "1650", "1700", "1750", "1800", "1850", "1900", "1950", "2000",
];

/// CSS-valid `data-sal-delay` steps (0–1000 ms).
pub(super) const SAL_DELAY_OPTIONS: &[&str] = &[
    "0", "50", "100", "150", "200", "250", "300", "350", "400", "450", "500", "550", "600", "650",
    "700", "750", "800", "850", "900", "950", "1000",
];

/// Clone-style SAL field pack: style, duration, delay. Duration/delay hide when
/// style is `no-animation`. Compose these three into each SAL-capable form.
pub(super) static SAL_STYLE_FIELD: FormField = FormField {
    id: "sal",
    label: "Animation",
    kind: FieldKind::Enum {
        options: SAL_OPTIONS,
        default: "fade",
    },
    required: true,
    visible_when: None,
};
pub(super) static SAL_DURATION_FIELD: FormField = FormField {
    id: "sal_duration",
    label: "Duration (ms)",
    kind: FieldKind::Enum {
        options: SAL_DURATION_OPTIONS,
        default: "400",
    },
    required: false,
    visible_when: Some(FieldPredicate::FieldNotEquals {
        other_id: "sal",
        value: "no-animation",
    }),
};
pub(super) static SAL_DELAY_FIELD: FormField = FormField {
    id: "sal_delay",
    label: "Delay (ms)",
    kind: FieldKind::Enum {
        options: SAL_DELAY_OPTIONS,
        default: "0",
    },
    required: false,
    visible_when: Some(FieldPredicate::FieldNotEquals {
        other_id: "sal",
        value: "no-animation",
    }),
};

pub(super) const LINK_TARGET_OPTIONS: &[&str] = &["_self", "_blank"];

pub(super) const BUTTON_STYLE_OPTIONS: &[&str] = &["-primary", "-secondary", "-tertiary", "-ghost"];

pub(super) const MEDIA_KIND_OPTIONS: &[&str] = &["none", "image", "oembed", "local-video"];
pub(super) const BOOL_OPTIONS: &[&str] = &["false", "true"];

pub(super) static MEDIA_KIND_FIELD: FormField = FormField {
    id: "media_kind",
    label: "Media",
    kind: FieldKind::Enum {
        options: MEDIA_KIND_OPTIONS,
        default: "image",
    },
    required: true,
    visible_when: None,
};
pub(super) static MEDIA_IMAGE_URL_FIELD: FormField = FormField {
    id: "media_image_url",
    label: "Image URL",
    kind: FieldKind::Url { default: "" },
    required: false,
    visible_when: Some(FieldPredicate::FieldEquals {
        other_id: "media_kind",
        value: "image",
    }),
};
pub(super) static MEDIA_IMAGE_ALT_FIELD: FormField = FormField {
    id: "media_image_alt",
    label: "Image Alt",
    kind: FieldKind::Text { default: "" },
    required: false,
    visible_when: Some(FieldPredicate::FieldEquals {
        other_id: "media_kind",
        value: "image",
    }),
};
pub(super) static MEDIA_OEMBED_URL_FIELD: FormField = FormField {
    id: "media_oembed_url",
    label: "YouTube or Vimeo URL",
    kind: FieldKind::Url { default: "" },
    required: false,
    visible_when: Some(FieldPredicate::FieldEquals {
        other_id: "media_kind",
        value: "oembed",
    }),
};
pub(super) static MEDIA_LG_MP4_FIELD: FormField = FormField {
    id: "media_lg_mp4",
    label: "Large MP4",
    kind: FieldKind::Url { default: "" },
    required: false,
    visible_when: Some(FieldPredicate::FieldEquals {
        other_id: "media_kind",
        value: "local-video",
    }),
};
pub(super) static MEDIA_SM_MP4_FIELD: FormField = FormField {
    id: "media_sm_mp4",
    label: "Small MP4 (optional)",
    kind: FieldKind::Url { default: "" },
    required: false,
    visible_when: Some(FieldPredicate::FieldEquals {
        other_id: "media_kind",
        value: "local-video",
    }),
};
pub(super) static MEDIA_POSTER_FIELD: FormField = FormField {
    id: "media_poster",
    label: "Poster image",
    kind: FieldKind::Url { default: "" },
    required: false,
    visible_when: Some(FieldPredicate::FieldEquals {
        other_id: "media_kind",
        value: "local-video",
    }),
};
pub(super) static MEDIA_NAME_FIELD: FormField = FormField {
    id: "media_name",
    label: "Accessible name",
    kind: FieldKind::Text { default: "" },
    required: false,
    visible_when: Some(FieldPredicate::FieldEquals {
        other_id: "media_kind",
        value: "local-video",
    }),
};
pub(super) static MEDIA_LOOP_FIELD: FormField = FormField {
    id: "media_loop",
    label: "Loop",
    kind: FieldKind::Enum {
        options: BOOL_OPTIONS,
        default: "false",
    },
    required: false,
    visible_when: Some(FieldPredicate::FieldEquals {
        other_id: "media_kind",
        value: "local-video",
    }),
};
pub(super) static MEDIA_AUTOPLAY_FIELD: FormField = FormField {
    id: "media_autoplay",
    label: "Autoplay",
    kind: FieldKind::Enum {
        options: BOOL_OPTIONS,
        default: "false",
    },
    required: false,
    visible_when: Some(FieldPredicate::FieldEquals {
        other_id: "media_kind",
        value: "local-video",
    }),
};

pub(super) const ROBOTS_OPTIONS: &[&str] = &[
    "index, follow",
    "noindex, follow",
    "index, nofollow",
    "noindex, nofollow",
];

pub(super) const SCHEMA_OPTIONS: &[&str] = &[
    "WebPage",
    "Article",
    "AboutPage",
    "ContactPage",
    "CollectionPage",
    "Organization",
    "LocalBusiness",
    "Product",
    "Service",
];

pub(super) const HERO_CLASS_OPTIONS: &[&str] = &[
    "-contained",
    "-contained-md",
    "-contained-lg",
    "-contained-xl",
    "-contained-xxl",
    "-full-full",
    "-full-contained",
    "-full-contained-md",
    "-full-contained-lg",
    "-full-contained-xl",
    "-full-contained-xxl",
];

pub(super) const SECTION_CLASS_OPTIONS: &[&str] = &[
    "-full-full",
    "-full-contained",
    "-full-xxl",
    "-full-xl",
    "-full-lg",
    "-full-md",
    "-xxl",
    "-xl",
    "-lg",
    "-md",
];

pub(super) const ITEM_BOX_CLASS_OPTIONS: &[&str] = &["l-box", "ll-box"];

pub(super) const SECTION_BG_OPTIONS: &[&str] = &["none", "-bg-muted"];
pub(super) const SECTION_PADDING_OPTIONS: &[&str] = &["default", "-no-padding"];

pub(super) const HERO_OVERLAY_OPTIONS: &[&str] = &["none", "-overlay-light", "-overlay-dark"];
pub(super) const HERO_COPY_POSITION_OPTIONS: &[&str] = &["-left", "-center", "-right"];

pub(super) const SPACER_SIZE_OPTIONS: &[&str] = &["-sm", "-md", "-lg", "-xl", "-xxl", "-xxxl"];
pub(super) const TABS_ORIENTATION_OPTIONS: &[&str] = &["-horizontal", "-vertical"];
pub(super) const HEADING_LEVEL_OPTIONS: &[&str] = &["2", "3", "4", "5", "6"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sal_duration_and_delay_hide_when_no_animation() {
        let mut state = EditFormState::new(&IMAGE_FORM);
        let visible = |s: &EditFormState, id: &str| {
            s.form
                .fields
                .iter()
                .find(|f| f.id == id)
                .map(|f| s.field_visible(f))
                .unwrap_or(false)
        };
        assert!(visible(&state, "sal"));
        assert!(visible(&state, "sal_duration"));
        assert!(visible(&state, "sal_delay"));
        state.set("sal", "no-animation");
        assert!(visible(&state, "sal"));
        assert!(!visible(&state, "sal_duration"));
        assert!(!visible(&state, "sal_delay"));
    }

    #[test]
    fn media_fields_follow_kind() {
        let mut state = EditFormState::new(&BANNER_FORM);
        let visible = |s: &EditFormState, id: &str| {
            s.form
                .fields
                .iter()
                .find(|f| f.id == id)
                .map(|f| s.field_visible(f))
                .unwrap_or(false)
        };
        assert!(visible(&state, "media_kind"));
        assert!(visible(&state, "media_image_url"));
        assert!(!visible(&state, "media_oembed_url"));
        assert!(!visible(&state, "media_lg_mp4"));
        state.set("media_kind", "oembed");
        assert!(!visible(&state, "media_image_url"));
        assert!(visible(&state, "media_oembed_url"));
        state.set("media_kind", "local-video");
        assert!(visible(&state, "media_lg_mp4"));
        assert!(visible(&state, "media_name"));
        assert!(!visible(&state, "media_image_url"));
        state.set("media_kind", "none");
        assert!(!visible(&state, "media_image_url"));
        assert!(!visible(&state, "media_oembed_url"));
        assert!(!visible(&state, "media_lg_mp4"));
    }

    #[test]
    fn hero_form_visual_fields_follow_links() {
        let ids: Vec<&str> = HERO_FORM.fields.iter().map(|f| f.id).collect();
        let links = ids.iter().position(|id| *id == "links").expect("links");
        assert_eq!(
            &ids[links + 1..],
            &["overlay", "copy_position", "id", "aria_label"]
        );
    }
}
