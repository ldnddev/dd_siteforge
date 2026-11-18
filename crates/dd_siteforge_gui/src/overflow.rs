//! How sidebar titles and Inspector text fit their rows.
//!
//! Spec 02 §2.2 keeps a tree row at `size.control.<d>.row` and truncates the
//! title with "…". Spec 04 keeps a text input at `size.control.<d>.input` and
//! is silent on a long single-line value; the ticket keeps that fixed height
//! and truncates the value the same way. Spec 04 field rows wrap the label
//! and never truncate it, so the row grows. Hover-scroll of a truncated
//! sidebar page title is Jared's addition: it is not in either spec.

use crate::density::{Metrics, metrics};
use crate::settings::Density;

/// Advance used to decide truncation and wrapping without a font rasterizer.
///
/// Spec 02 only notes that mono labels run about 17% wider than proportional
/// ones. `typography.ui.body` is 13 px. 8 px per character is a conservative
/// proportional advance so a string that only barely fits is still treated as
/// truncated (tooltip and hover-scroll stay available).
pub const CHAR_PX: f32 = 8.0;

/// Tree row padding. Spec 02 §2.3 indent plus an 8 px inset, and `space.3`
/// (12 px) at the row end for the badge.
pub const ROW_PAD_LEFT: f32 = 8.0;
pub const ROW_PAD_RIGHT: f32 = 12.0;

/// Badge gap and horizontal padding. Spec 02 §2.2: gap `space.1`, pad-x `space.1`.
pub const BADGE_GAP: f32 = 4.0;
pub const BADGE_PAD_X: f32 = 8.0;

/// Text input pad-x. Spec 04 §2.1 `space.2`. The 1 px border sits inside the box.
pub const VALUE_PAD_X: f32 = 8.0;
pub const VALUE_BORDER: f32 = 1.0;

/// Hover-scroll timing. The delay is Jared's "about 500 ms". The pause and
/// speed are not in the ticket or the specs. The pause uses spec 01
/// `motion.deliberate` (400 ms). Speed is 60 px/s so a long title stays readable.
pub const HOVER_SCROLL_DELAY_MS: u64 = 500;
pub const HOVER_SCROLL_PAUSE_MS: u64 = 400;
pub const HOVER_SCROLL_SPEED_PX_PER_SEC: f32 = 60.0;

const ELLIPSIS: char = '…';

#[derive(Clone, Debug, PartialEq)]
pub struct Clip {
    pub shown: String,
    pub tooltip: Option<String>,
    pub truncated: bool,
    pub travel_px: f32,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldValueLayout {
    pub clip: Clip,
    /// Inspector values never hover-scroll.
    pub scrolls: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LabelLayout {
    /// The label text, unchanged. Never an ellipsis.
    pub text: String,
    pub lines: u32,
    pub truncated: bool,
    /// Label lines, the label gap, and one fixed text-input height.
    pub row_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollPhase {
    /// Ellipsis at rest: not hovering, still inside the delay, reduced motion,
    /// or the title is not truncated.
    Rest,
    ToEnd {
        offset_px: f32,
    },
    PauseEnd {
        offset_px: f32,
    },
    ToStart {
        offset_px: f32,
    },
    PauseStart {
        offset_px: f32,
    },
}

pub fn text_width(text: &str) -> f32 {
    text.chars().count() as f32 * CHAR_PX
}

pub fn label_line_px() -> f32 {
    ldnddev_theme::TypeScale::SIZES.line_height(ldnddev_theme::TypeScale::SIZES.sm)
}

pub fn sidebar_title_available(metrics: Metrics, depth: u32, badges: &[&str]) -> f32 {
    let mut width =
        metrics.sidebar_width - ROW_PAD_LEFT - metrics.tree_indent * depth as f32 - ROW_PAD_RIGHT;
    for badge in badges {
        width -= BADGE_GAP + BADGE_PAD_X + text_width(badge);
    }
    width.max(0.0)
}

pub fn sidebar_row_layout(label: &str, depth: u32, badges: &[&str], metrics: Metrics) -> Clip {
    clip_line(
        label,
        sidebar_title_available(metrics, depth, badges),
        metrics.row,
    )
}

pub fn value_available(metrics: Metrics) -> f32 {
    (metrics.inspector_width - metrics.pad_x * 2.0 - VALUE_PAD_X * 2.0 - VALUE_BORDER * 2.0)
        .max(0.0)
}

pub fn field_value_layout(value: &str, metrics: Metrics) -> FieldValueLayout {
    let text = collapse_lines(value);
    FieldValueLayout {
        clip: clip_line(&text, value_available(metrics), metrics.input),
        scrolls: false,
    }
}

pub fn label_available(metrics: Metrics) -> f32 {
    (metrics.inspector_width - metrics.pad_x * 2.0).max(0.0)
}

pub fn field_label_layout(label: &str, required: bool, metrics: Metrics) -> LabelLayout {
    let measured = if required {
        format!("{label} *")
    } else {
        label.to_string()
    };
    let lines = wrap_line_count(&measured, label_available(metrics));
    LabelLayout {
        text: label.to_string(),
        lines,
        truncated: false,
        row_height: lines as f32 * label_line_px() + metrics.label_gap + metrics.input,
    }
}

/// Hover-scroll is only for a sidebar page title that is actually cut off,
/// and only when reduced motion is off.
pub fn sidebar_scrolls(is_page: bool, truncated: bool, reduced_motion: bool) -> bool {
    is_page && truncated && !reduced_motion
}

pub fn travel_ms(travel_px: f32) -> u64 {
    if travel_px <= 0.5 {
        return 0;
    }
    ((travel_px / HOVER_SCROLL_SPEED_PX_PER_SEC) * 1000.0).ceil() as u64
}

pub fn cycle_ms(travel_px: f32) -> u64 {
    let travel = travel_ms(travel_px);
    if travel == 0 {
        return 0;
    }
    travel
        .saturating_mul(2)
        .saturating_add(HOVER_SCROLL_PAUSE_MS.saturating_mul(2))
}

pub fn scroll_phase(
    hovering: bool,
    elapsed_ms: u64,
    travel_px: f32,
    reduced_motion: bool,
) -> ScrollPhase {
    if !hovering || reduced_motion || travel_px <= 0.5 || elapsed_ms < HOVER_SCROLL_DELAY_MS {
        return ScrollPhase::Rest;
    }
    let cycle = cycle_ms(travel_px).max(1);
    let elapsed = (elapsed_ms - HOVER_SCROLL_DELAY_MS) % cycle;
    place_in_cycle(elapsed, travel_px)
}

pub fn offset_at_delta(delta: f32, travel_px: f32) -> f32 {
    let cycle = cycle_ms(travel_px);
    if cycle == 0 {
        return 0.0;
    }
    let elapsed = (delta.clamp(0.0, 0.999_999) * cycle as f32) as u64;
    match place_in_cycle(elapsed, travel_px) {
        ScrollPhase::Rest => 0.0,
        ScrollPhase::ToEnd { offset_px }
        | ScrollPhase::PauseEnd { offset_px }
        | ScrollPhase::ToStart { offset_px }
        | ScrollPhase::PauseStart { offset_px } => offset_px,
    }
}

fn clip_line(text: &str, available_px: f32, height: f32) -> Clip {
    let width = text_width(text);
    if text.is_empty() || width <= available_px + 0.01 {
        return Clip {
            shown: text.to_string(),
            tooltip: None,
            truncated: false,
            travel_px: 0.0,
            height,
        };
    }
    Clip {
        shown: ellipsize(text, available_px),
        tooltip: Some(text.to_string()),
        truncated: true,
        travel_px: (width - available_px).max(0.0),
        height,
    }
}

fn ellipsize(text: &str, available_px: f32) -> String {
    let budget = available_px - CHAR_PX;
    if budget <= 0.0 {
        return ELLIPSIS.to_string();
    }
    let count = (budget / CHAR_PX).floor() as usize;
    let mut shown: String = text.chars().take(count).collect();
    shown.push(ELLIPSIS);
    shown
}

fn collapse_lines(value: &str) -> String {
    value.replace("\r\n", " ").replace(['\n', '\r'], " ")
}

fn wrap_line_count(text: &str, available_px: f32) -> u32 {
    if text.is_empty() || available_px <= 0.0 {
        return 1;
    }
    let max_chars = (available_px / CHAR_PX).floor().max(1.0) as usize;
    let mut lines = 1u32;
    let mut used = 0usize;
    for (index, word) in text.split(' ').enumerate() {
        let word_chars = word.chars().count();
        if index == 0 {
            used = word_chars;
            continue;
        }
        let needed = used + 1 + word_chars;
        if needed <= max_chars {
            used = needed;
        } else {
            lines += 1;
            used = word_chars;
        }
    }
    lines
}

fn place_in_cycle(elapsed_in_cycle_ms: u64, travel_px: f32) -> ScrollPhase {
    let travel = travel_ms(travel_px).max(1);
    let pause = HOVER_SCROLL_PAUSE_MS;
    if elapsed_in_cycle_ms < travel {
        let offset = travel_px * (elapsed_in_cycle_ms as f32) / (travel as f32);
        ScrollPhase::ToEnd { offset_px: offset }
    } else if elapsed_in_cycle_ms < travel + pause {
        ScrollPhase::PauseEnd {
            offset_px: travel_px,
        }
    } else if elapsed_in_cycle_ms < travel + pause + travel {
        let into = elapsed_in_cycle_ms - travel - pause;
        let offset = travel_px * (1.0 - (into as f32) / (travel as f32));
        ScrollPhase::ToStart { offset_px: offset }
    } else {
        ScrollPhase::PauseStart { offset_px: 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LONG_TITLE: &str = "Building Better Workflows: From Why Tools to Custom Frameworks";
    const LONG_VALUE: &str = "A description that is much longer than the inspector field and used to overlap the Canonical URL row.";
    const LONG_LABEL: &str = "OG Image alt (uses hero alt, then site default)";

    fn comfortable() -> Metrics {
        metrics(Density::Comfortable)
    }

    #[test]
    fn long_sidebar_title_truncates_on_one_fixed_height_row() {
        let metrics = comfortable();
        let clip = sidebar_row_layout(LONG_TITLE, 0, &[], metrics);
        assert_eq!(clip.height, metrics.row, "tree row stays 28 px");
        assert!(clip.truncated, "the ticket title is wider than the sidebar");
        assert!(clip.shown.ends_with('…'), "shown = {}", clip.shown);
        assert!(clip.shown.chars().count() < LONG_TITLE.chars().count());
        assert!(
            LONG_TITLE.starts_with(clip.shown.trim_end_matches('…')),
            "shown = {}",
            clip.shown
        );
        assert_eq!(clip.tooltip.as_deref(), Some(LONG_TITLE));
        assert!(clip.travel_px > 0.0);
        assert!(text_width(&clip.shown) <= sidebar_title_available(metrics, 0, &[]));
        assert!(sidebar_scrolls(true, clip.truncated, false));
    }

    #[test]
    fn short_sidebar_title_is_not_truncated_and_does_not_scroll() {
        let metrics = comfortable();
        let clip = sidebar_row_layout("Home", 0, &[], metrics);
        assert_eq!(clip.height, metrics.row);
        assert!(!clip.truncated);
        assert_eq!(clip.shown, "Home");
        assert_eq!(clip.tooltip, None);
        assert_eq!(clip.travel_px, 0.0);
        assert!(!sidebar_scrolls(true, clip.truncated, false));
        assert_eq!(
            scroll_phase(true, 10_000, clip.travel_px, false),
            ScrollPhase::Rest
        );
        assert!(
            !sidebar_scrolls(false, true, false),
            "folders and region rows do not hover-scroll"
        );
    }

    #[test]
    fn long_inspector_value_truncates_and_does_not_scroll() {
        let metrics = comfortable();
        let layout = field_value_layout(LONG_VALUE, metrics);
        assert_eq!(layout.clip.height, metrics.input, "text input stays 28 px");
        assert!(layout.clip.truncated);
        assert!(layout.clip.shown.ends_with('…'), "{}", layout.clip.shown);
        assert!(!layout.clip.shown.contains('\n'));
        assert_eq!(layout.clip.tooltip.as_deref(), Some(LONG_VALUE));
        assert!(text_width(&layout.clip.shown) <= value_available(metrics));
        assert!(
            !layout.scrolls,
            "inspector values get a tooltip, not a marquee"
        );

        let collapsed = field_value_layout("line one\nline two", metrics);
        assert_eq!(collapsed.clip.shown, "line one line two");
        assert!(!collapsed.clip.truncated);
        assert_eq!(collapsed.clip.tooltip, None);
    }

    #[test]
    fn long_inspector_label_wraps_and_grows_the_row() {
        let layout = comfortable();
        let long = field_label_layout(LONG_LABEL, false, layout);
        let short = field_label_layout("Title", true, layout);
        assert!(
            long.lines >= 2,
            "the OG alt label must wrap, lines = {}",
            long.lines
        );
        assert_eq!(long.text, LONG_LABEL);
        assert!(!long.text.contains('…'));
        assert!(!long.truncated, "labels are never truncated");
        assert!(
            long.row_height > short.row_height,
            "wrapping label grows the field row: {} vs {}",
            long.row_height,
            short.row_height
        );
        assert_eq!(short.lines, 1);
        assert!(!short.truncated);
        assert_eq!(short.text, "Title");

        let compact = metrics(Density::Compact);
        let compact_title = sidebar_row_layout(LONG_TITLE, 0, &[], compact);
        let compact_value = field_value_layout(LONG_VALUE, compact);
        assert_eq!(compact_title.height, compact.row);
        assert_eq!(compact_value.clip.height, compact.input);
        assert_eq!(compact.row, 24.0);
        assert_eq!(compact.input, 24.0);
    }

    #[test]
    fn hover_scroll_waits_pauses_at_each_end_and_snaps_back() {
        let clip = sidebar_row_layout(LONG_TITLE, 0, &[], comfortable());
        let travel = clip.travel_px;
        assert!(travel > 1.0);
        let move_ms = travel_ms(travel);
        assert!(move_ms > 0);

        assert_eq!(
            scroll_phase(true, HOVER_SCROLL_DELAY_MS - 1, travel, false),
            ScrollPhase::Rest,
            "still the ellipsis during the hover delay"
        );
        match scroll_phase(true, HOVER_SCROLL_DELAY_MS, travel, false) {
            ScrollPhase::ToEnd { offset_px } => assert!(offset_px < 1.0, "{offset_px}"),
            other => panic!("expected the scroll to start, got {other:?}"),
        }
        match scroll_phase(true, HOVER_SCROLL_DELAY_MS + move_ms, travel, false) {
            ScrollPhase::PauseEnd { offset_px } => {
                assert!((offset_px - travel).abs() < 0.01, "{offset_px} vs {travel}")
            }
            other => panic!("expected a pause on the full title, got {other:?}"),
        }
        let mid_return = HOVER_SCROLL_DELAY_MS + move_ms + HOVER_SCROLL_PAUSE_MS + move_ms / 2;
        match scroll_phase(true, mid_return, travel, false) {
            ScrollPhase::ToStart { offset_px } => {
                assert!(offset_px > 0.0 && offset_px < travel, "{offset_px}")
            }
            other => panic!("expected the scroll back, got {other:?}"),
        }
        let parked = HOVER_SCROLL_DELAY_MS + move_ms + HOVER_SCROLL_PAUSE_MS + move_ms;
        match scroll_phase(true, parked, travel, false) {
            ScrollPhase::PauseStart { offset_px } => assert!(offset_px.abs() < 0.01, "{offset_px}"),
            other => panic!("expected a pause at the start, got {other:?}"),
        }
        assert_eq!(
            scroll_phase(false, mid_return, travel, false),
            ScrollPhase::Rest,
            "leaving the row snaps back to the ellipsis"
        );

        let cycle = cycle_ms(travel) as f32;
        assert!(cycle > 0.0);
        assert!(offset_at_delta(0.0, travel).abs() < 0.01);
        let pause_end_delta = (move_ms as f32 + HOVER_SCROLL_PAUSE_MS as f32 / 2.0) / cycle;
        let parked_offset = offset_at_delta(pause_end_delta, travel);
        assert!(
            (parked_offset - travel).abs() < 0.01,
            "{parked_offset} vs {travel}"
        );
    }

    #[test]
    fn reduced_motion_skips_the_scroll_and_keeps_the_tooltip() {
        let clip = sidebar_row_layout(LONG_TITLE, 0, &[], comfortable());
        assert!(clip.truncated);
        assert_eq!(clip.tooltip.as_deref(), Some(LONG_TITLE));
        assert!(!sidebar_scrolls(true, true, true));
        assert_eq!(
            scroll_phase(true, 10_000, clip.travel_px, true),
            ScrollPhase::Rest
        );
    }
}
