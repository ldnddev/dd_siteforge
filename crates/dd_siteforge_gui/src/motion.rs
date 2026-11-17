//! Motion tokens for every transition in design spec 06 §7.
//!
//! Durations and easings are spec 01 §3.8. Reduced motion, the desktop
//! setting stored in `gui.json`, makes every duration instant. Spinners keep
//! the static glyph, the export bar holds a 40% segment, and the insert
//! highlight is skipped.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Easing {
    Standard,
    Enter,
    Exit,
    Linear,
}

impl Easing {
    /// Cubic-bezier control points from `core.motion.easing.*`.
    pub fn bezier(self) -> (f32, f32, f32, f32) {
        match self {
            Self::Standard => (0.2, 0.0, 0.0, 1.0),
            Self::Enter => (0.0, 0.0, 0.2, 1.0),
            Self::Exit => (0.4, 0.0, 1.0, 1.0),
            Self::Linear => (0.0, 0.0, 1.0, 1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurationToken {
    Instant,
    Fast,
    Base,
    Moderate,
    Slow,
    Deliberate,
}

impl DurationToken {
    pub fn ms(self) -> u64 {
        match self {
            Self::Instant => 0,
            Self::Fast => 90,
            Self::Base => 140,
            Self::Moderate => 200,
            Self::Slow => 280,
            Self::Deliberate => 400,
        }
    }
}

/// What reduced motion does besides zeroing the duration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReducedRule {
    Instant,
    /// Static `reload` glyph plus the running text. No turn.
    StaticGlyph,
    /// A still segment covering 40% of the track.
    StillSegment,
    /// The insert highlight does not play.
    Skip,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    pub id: &'static str,
    pub duration: DurationToken,
    pub easing: Option<Easing>,
    pub reduced: ReducedRule,
    /// Slide in pixels for enter transitions. Zero when the motion is not a slide.
    pub slide_px: f32,
}

/// The spec 06 §7 table, in the same order.
pub fn table() -> &'static [Transition] {
    &TABLE
}

const TABLE: &[Transition] = &[
    Transition {
        id: "hover",
        duration: DurationToken::Fast,
        easing: Some(Easing::Standard),
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "focus-ring",
        duration: DurationToken::Instant,
        easing: None,
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "selection",
        duration: DurationToken::Instant,
        easing: None,
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "disclosure",
        duration: DurationToken::Base,
        easing: Some(Easing::Standard),
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "panel-collapse",
        duration: DurationToken::Base,
        easing: Some(Easing::Standard),
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "panel-resize",
        duration: DurationToken::Instant,
        easing: None,
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "popover-open",
        duration: DurationToken::Fast,
        easing: Some(Easing::Enter),
        reduced: ReducedRule::Instant,
        slide_px: 4.0,
    },
    Transition {
        id: "popover-close",
        duration: DurationToken::Fast,
        easing: Some(Easing::Exit),
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "sheet-open",
        duration: DurationToken::Moderate,
        easing: Some(Easing::Enter),
        reduced: ReducedRule::Instant,
        slide_px: 8.0,
    },
    Transition {
        id: "sheet-close",
        duration: DurationToken::Base,
        easing: Some(Easing::Exit),
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "toast-in",
        duration: DurationToken::Moderate,
        easing: Some(Easing::Enter),
        reduced: ReducedRule::Instant,
        slide_px: 8.0,
    },
    Transition {
        id: "toast-out",
        duration: DurationToken::Base,
        easing: Some(Easing::Exit),
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "reorder",
        duration: DurationToken::Moderate,
        easing: Some(Easing::Standard),
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "inspector-drill",
        duration: DurationToken::Base,
        easing: Some(Easing::Standard),
        reduced: ReducedRule::Instant,
        slide_px: 16.0,
    },
    Transition {
        id: "insert-highlight",
        duration: DurationToken::Deliberate,
        easing: Some(Easing::Exit),
        reduced: ReducedRule::Skip,
        slide_px: 0.0,
    },
    Transition {
        id: "status-crossfade",
        duration: DurationToken::Base,
        easing: Some(Easing::Standard),
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "spinner",
        duration: DurationToken::Slow,
        easing: Some(Easing::Linear),
        reduced: ReducedRule::StaticGlyph,
        slide_px: 0.0,
    },
    Transition {
        id: "indeterminate-bar",
        duration: DurationToken::Deliberate,
        easing: Some(Easing::Standard),
        reduced: ReducedRule::StillSegment,
        slide_px: 0.0,
    },
    Transition {
        id: "skeleton",
        duration: DurationToken::Deliberate,
        easing: Some(Easing::Standard),
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "theme-apply",
        duration: DurationToken::Instant,
        easing: None,
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "density-switch",
        duration: DurationToken::Instant,
        easing: None,
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "vim-toggle",
        duration: DurationToken::Instant,
        easing: None,
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
    Transition {
        id: "window-narrow",
        duration: DurationToken::Base,
        easing: Some(Easing::Standard),
        reduced: ReducedRule::Instant,
        slide_px: 0.0,
    },
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    pub id: &'static str,
    pub duration_ms: u64,
    pub easing: Option<Easing>,
    pub slide_px: f32,
    pub reduced: bool,
    /// Spinner draws the static glyph instead of turning.
    pub static_glyph: bool,
    /// `Some` when the export bar is a still fraction of the track.
    pub still_bar: Option<f32>,
    /// Insert highlight is not shown.
    pub skip: bool,
}

pub fn spec(id: &str) -> &'static Transition {
    table()
        .iter()
        .find(|row| row.id == id)
        .unwrap_or_else(|| panic!("unknown motion transition {id}"))
}

pub fn plan(id: &str, reduced: bool) -> Plan {
    let row = spec(id);
    let duration_ms = if reduced { 0 } else { row.duration.ms() };
    // A full spinner turn is four quarter-turns of `slow` (1.12 s).
    let duration_ms = if !reduced && row.id == "spinner" {
        row.duration.ms() * 4
    } else if !reduced && row.id == "indeterminate-bar" {
        // Three `deliberate` passes (1.2 s).
        DurationToken::Deliberate.ms() * 3
    } else {
        duration_ms
    };
    Plan {
        id: row.id,
        duration_ms,
        easing: if reduced { None } else { row.easing },
        slide_px: if reduced { 0.0 } else { row.slide_px },
        reduced,
        static_glyph: reduced && matches!(row.reduced, ReducedRule::StaticGlyph),
        still_bar: if reduced && matches!(row.reduced, ReducedRule::StillSegment) {
            Some(0.4)
        } else {
            None
        },
        skip: reduced && matches!(row.reduced, ReducedRule::Skip),
    }
}

/// Palette running row. Reduced motion keeps the static glyph.
pub fn running_glyph(reduced: bool) -> &'static str {
    if plan("spinner", reduced).static_glyph {
        "reload"
    } else {
        "reload-spin"
    }
}

/// Export progress track. Reduced motion is a still 40% segment.
pub fn export_bar(reduced: bool) -> &'static str {
    if plan("indeterminate-bar", reduced).still_bar == Some(0.4) {
        "bar-40"
    } else {
        "indeterminate-bar"
    }
}

pub fn ease(easing: Easing, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let (x1, y1, x2, y2) = easing.bezier();
    if easing == Easing::Linear {
        return t;
    }
    sample_bezier(x1, y1, x2, y2, t)
}

fn sample_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let mut t = x;
    for _ in 0..8 {
        let dx = bezier_coord(t, x1, x2) - x;
        let slope = bezier_slope(t, x1, x2);
        if slope.abs() < 1.0e-4 {
            break;
        }
        t = (t - dx / slope).clamp(0.0, 1.0);
    }
    bezier_coord(t, y1, y2)
}

fn bezier_coord(t: f32, a1: f32, a2: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * t * a1 + 3.0 * u * t * t * a2 + t * t * t
}

fn bezier_slope(t: f32, a1: f32, a2: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * a1 + 6.0 * u * t * (a2 - a1) + 3.0 * t * t * (1.0 - a2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_spec_transition_has_its_duration_and_easing() {
        let expect = [
            ("hover", 90, Some(Easing::Standard)),
            ("focus-ring", 0, None),
            ("selection", 0, None),
            ("disclosure", 140, Some(Easing::Standard)),
            ("panel-collapse", 140, Some(Easing::Standard)),
            ("panel-resize", 0, None),
            ("popover-open", 90, Some(Easing::Enter)),
            ("popover-close", 90, Some(Easing::Exit)),
            ("sheet-open", 200, Some(Easing::Enter)),
            ("sheet-close", 140, Some(Easing::Exit)),
            ("toast-in", 200, Some(Easing::Enter)),
            ("toast-out", 140, Some(Easing::Exit)),
            ("reorder", 200, Some(Easing::Standard)),
            ("inspector-drill", 140, Some(Easing::Standard)),
            ("insert-highlight", 400, Some(Easing::Exit)),
            ("status-crossfade", 140, Some(Easing::Standard)),
            ("spinner", 1120, Some(Easing::Linear)),
            ("indeterminate-bar", 1200, Some(Easing::Standard)),
            ("skeleton", 400, Some(Easing::Standard)),
            ("theme-apply", 0, None),
            ("density-switch", 0, None),
            ("vim-toggle", 0, None),
            ("window-narrow", 140, Some(Easing::Standard)),
        ];
        assert_eq!(table().len(), expect.len());
        for (id, ms, easing) in expect {
            let plan = plan(id, false);
            assert_eq!(plan.duration_ms, ms, "{id}");
            assert_eq!(plan.easing, easing, "{id}");
            assert!(!plan.skip, "{id}");
        }
        assert_eq!(spec("popover-open").slide_px, 4.0);
        assert_eq!(spec("sheet-open").slide_px, 8.0);
        assert_eq!(spec("toast-in").slide_px, 8.0);
        assert_eq!(spec("inspector-drill").slide_px, 16.0);
    }

    #[test]
    fn reduced_motion_is_instant_and_stops_the_spinner_bar_and_highlight() {
        for row in table() {
            let plan = plan(row.id, true);
            assert_eq!(plan.duration_ms, 0, "{}", row.id);
            assert!(plan.easing.is_none(), "{}", row.id);
            assert_eq!(plan.slide_px, 0.0, "{}", row.id);
        }
        assert_eq!(running_glyph(false), "reload-spin");
        assert_eq!(running_glyph(true), "reload");
        assert!(plan("spinner", true).static_glyph);
        assert_eq!(export_bar(false), "indeterminate-bar");
        assert_eq!(export_bar(true), "bar-40");
        assert_eq!(plan("indeterminate-bar", true).still_bar, Some(0.4));
        assert!(plan("insert-highlight", true).skip);
        assert!(!plan("insert-highlight", false).skip);
        // Theme apply stays instant even with motion on. No color tween.
        assert_eq!(plan("theme-apply", false).duration_ms, 0);
    }

    #[test]
    fn easings_match_the_token_control_points_and_stay_in_range() {
        assert_eq!(Easing::Standard.bezier(), (0.2, 0.0, 0.0, 1.0));
        assert_eq!(Easing::Enter.bezier(), (0.0, 0.0, 0.2, 1.0));
        assert_eq!(Easing::Exit.bezier(), (0.4, 0.0, 1.0, 1.0));
        assert_eq!(Easing::Linear.bezier(), (0.0, 0.0, 1.0, 1.0));
        for easing in [
            Easing::Standard,
            Easing::Enter,
            Easing::Exit,
            Easing::Linear,
        ] {
            assert!((ease(easing, 0.0) - 0.0).abs() < 0.02, "{easing:?}");
            assert!((ease(easing, 1.0) - 1.0).abs() < 0.02, "{easing:?}");
        }
        assert!((ease(Easing::Linear, 0.25) - 0.25).abs() < 0.001);
    }

    #[test]
    fn durations_match_the_theme_metrics() {
        let metrics = ldnddev_theme::Metrics::standard();
        assert_eq!(DurationToken::Instant.ms(), metrics.motion_instant_ms);
        assert_eq!(DurationToken::Fast.ms(), metrics.motion_fast_ms);
        assert_eq!(DurationToken::Base.ms(), metrics.motion_base_ms);
        assert_eq!(DurationToken::Moderate.ms(), metrics.motion_moderate_ms);
        assert_eq!(DurationToken::Slow.ms(), metrics.motion_slow_ms);
        assert_eq!(DurationToken::Deliberate.ms(), metrics.motion_deliberate_ms);
        let reduced = metrics.with_reduced_motion(true);
        assert_eq!(reduced.motion_fast_ms, 0);
        assert!(reduced.reduced_motion);
    }
}
