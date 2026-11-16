//! Semantic GUI colors resolved from an Omarchy palette.
//!
//! The slot chains and fallback rules (a), (b), and (c) are the ones in
//! `build_tokens.py`. `resolved/<slug>.json` is a golden fixture for this
//! function, not a file the app ships. Contrast checks call
//! [`crate::contrast_ratio`].

use std::collections::{BTreeMap, HashMap};

use crate::omarchy::{contrast_ratio, mix, OmarchyColors, ThemeMode};
use crate::rgb::Rgb;

pub const FONT_DEFAULT: &str = "JetBrainsMono Nerd Font";

/// `<fc-match monospace>` is prepended at runtime. These follow it.
pub const FONT_FALLBACKS: &[&str] = &[FONT_DEFAULT, "JetBrains Mono", "monospace"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontStack {
    pub ui: String,
    pub mono: String,
    pub fallbacks: Vec<String>,
}

impl FontStack {
    /// `family` is the concrete name from `fc-match`, or [`FONT_DEFAULT`].
    pub fn from_family(family: &str) -> Self {
        let family = family.trim();
        let chosen = if family.is_empty() {
            FONT_DEFAULT.to_string()
        } else {
            family.to_string()
        };
        let fallbacks = FONT_FALLBACKS
            .iter()
            .copied()
            .filter(|name| *name != chosen)
            .map(str::to_string)
            .collect();
        Self {
            ui: chosen.clone(),
            mono: chosen,
            fallbacks,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeScale {
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub xxxl: f32,
}

impl TypeScale {
    pub const SIZES: Self = Self {
        xxs: 11.0,
        xs: 12.0,
        sm: 13.0,
        md: 14.0,
        lg: 16.0,
        xl: 20.0,
        xxl: 24.0,
        xxxl: 32.0,
    };

    pub const fn line_height(self, size: f32) -> f32 {
        if size <= 12.0 {
            16.0
        } else if size <= 13.0 {
            18.0
        } else if size <= 14.0 {
            20.0
        } else if size <= 16.0 {
            22.0
        } else if size <= 20.0 {
            28.0
        } else if size <= 24.0 {
            32.0
        } else {
            44.0
        }
    }
}

/// Non-color tokens from the design spec. Durations are milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub type_scale: TypeScale,
    pub space_px: f32,
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
    pub border_hairline: f32,
    pub border_focus: f32,
    pub focus_offset: f32,
    pub motion_instant_ms: u64,
    pub motion_fast_ms: u64,
    pub motion_base_ms: u64,
    pub motion_moderate_ms: u64,
    pub motion_slow_ms: u64,
    pub motion_deliberate_ms: u64,
    pub reduced_motion: bool,
}

impl Metrics {
    pub fn standard() -> Self {
        Self {
            type_scale: TypeScale::SIZES,
            space_px: 1.0,
            radius_sm: 4.0,
            radius_md: 6.0,
            radius_lg: 8.0,
            border_hairline: 1.0,
            border_focus: 2.0,
            focus_offset: 1.0,
            motion_instant_ms: 0,
            motion_fast_ms: 90,
            motion_base_ms: 140,
            motion_moderate_ms: 200,
            motion_slow_ms: 280,
            motion_deliberate_ms: 400,
            reduced_motion: false,
        }
    }

    pub fn with_reduced_motion(mut self, reduced: bool) -> Self {
        self.reduced_motion = reduced;
        if reduced {
            self.motion_fast_ms = 0;
            self.motion_base_ms = 0;
            self.motion_moderate_ms = 0;
            self.motion_slow_ms = 0;
            self.motion_deliberate_ms = 0;
        }
        self
    }

    pub fn duration_ms(self, ms: u64) -> u64 {
        if self.reduced_motion {
            0
        } else {
            ms
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimaryButton {
    Filled,
    Outlined,
}

impl PrimaryButton {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Filled => "filled",
            Self::Outlined => "outlined",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiToken {
    pub name: &'static str,
    pub hex: String,
    pub slot: String,
    pub rgb: Rgb,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiTheme {
    pub mode: ThemeMode,
    pub tokens: Vec<GuiToken>,
    /// Rule (a). Outlined when `text.on-accent` is under 4.5:1 on `accent.fill`,
    /// or when no slot can form a hover layer without lowering that contrast.
    pub primary_button: PrimaryButton,
    /// Rule (b). Icon token → the `status.*-text` token to draw on `surface.base`.
    pub on_surface_base: BTreeMap<String, String>,
}

impl GuiTheme {
    pub fn token(&self, name: &str) -> Option<&GuiToken> {
        self.tokens.iter().find(|token| token.name == name)
    }

    pub fn rgb(&self, name: &str) -> Option<Rgb> {
        self.token(name).map(|token| token.rgb)
    }
}

/// Resolve every semantic color from a derived Omarchy palette.
pub fn resolve_gui_theme(colors: &OmarchyColors) -> GuiTheme {
    let mut vals: HashMap<String, Rgb> = HashMap::new();
    for (key, rgb) in colors.slots() {
        vals.insert(key.to_string(), rgb);
    }
    let mode = colors.mode;
    let mut out: HashMap<&'static str, Rgb> = HashMap::new();
    let mut picked: HashMap<&'static str, String> = HashMap::new();

    for spec in SPECS {
        if let Some((slot, cents, base)) = spec.overlay {
            let (slot, cents) = match slot {
                "@away" => away_alpha(&vals, &out, cents),
                "@card" => {
                    let raised = rgb_of(&out, base);
                    let canvas = rgb_of(&out, "surface.base");
                    let paint = vals
                        .get("foreground")
                        .copied()
                        .unwrap_or(Rgb::new(255, 255, 255));
                    ("foreground", card_alpha(paint, raised, canvas))
                }
                other => (other, cents),
            };
            let paint = vals
                .get(slot)
                .copied()
                .or_else(|| vals.get("foreground").copied())
                .unwrap_or(Rgb::new(255, 255, 255));
            let base_rgb = rgb_of(&out, base);
            out.insert(spec.name, mix(base_rgb, paint, f64::from(cents) / 100.0));
            picked.insert(
                spec.name,
                format!("{slot} @ {}% over {base}", percent_label(cents)),
            );
            continue;
        }

        let chain = full_chain(spec, mode, &vals);
        if spec.step {
            resolve_selection(&vals, &chain, &mut out, &mut picked, spec.name);
            continue;
        }

        let Some(first) = chain.first().copied() else {
            let fallback = vals
                .get("foreground")
                .copied()
                .unwrap_or(Rgb::new(255, 255, 255));
            out.insert(spec.name, fallback);
            picked.insert(spec.name, "foreground".to_string());
            continue;
        };

        let floor = spec.role.floor();
        if floor.is_none() || spec.on.is_empty() {
            out.insert(spec.name, vals[first]);
            picked.insert(spec.name, first.to_string());
            continue;
        }
        let floor = floor.unwrap_or(0.0);
        let surfaces = resolved_surfaces(&out, spec.on);
        let mut chosen = None;
        for candidate in &chain {
            if passes(&vals, candidate, &surfaces, floor) {
                chosen = Some(*candidate);
                break;
            }
        }
        if chosen.is_none() && is_rule_b(spec.name) && spec.on.contains(&"surface.base") {
            let rest: Vec<Rgb> = spec
                .on
                .iter()
                .filter(|name| **name != "surface.base")
                .filter_map(|name| out.get(*name).copied())
                .collect();
            for candidate in &chain {
                if passes(&vals, candidate, &rest, floor) {
                    chosen = Some(*candidate);
                    break;
                }
            }
        }
        let chosen = chosen.unwrap_or_else(|| worst_slot(&vals, &chain, &surfaces));
        out.insert(spec.name, vals[chosen]);
        picked.insert(spec.name, chosen.to_string());
    }

    let tokens = SPECS
        .iter()
        .map(|spec| {
            let rgb = out[spec.name];
            GuiToken {
                name: spec.name,
                hex: hex(rgb),
                slot: picked[spec.name].clone(),
                rgb,
            }
        })
        .collect::<Vec<_>>();

    let on_accent = out["text.on-accent"];
    let fill = out["accent.fill"];
    // Rule (a), including rose-pine: accent.fill fell through to foreground, so
    // the away-slot search finds no hover/pressed layer (0%). catppuccin-latte
    // is outlined because the label contrast itself is under 4.5:1.
    let plan = accent_plan(&vals, on_accent, fill);
    let hover_layer = plan.hover_cents > 0 && plan.pressed_cents > 0;
    let primary_button = if meets(contrast_ratio(on_accent, fill), 4.5) && hover_layer {
        PrimaryButton::Filled
    } else {
        PrimaryButton::Outlined
    };

    let base = out["surface.base"];
    let mut on_surface_base = BTreeMap::new();
    for (icon, text) in RULE_B {
        if !meets(contrast_ratio(out[icon], base), 3.0) {
            on_surface_base.insert((*icon).to_string(), (*text).to_string());
        }
    }

    GuiTheme {
        mode,
        tokens,
        primary_button,
        on_surface_base,
    }
}

fn resolve_selection(
    vals: &HashMap<String, Rgb>,
    chain: &[&str],
    out: &mut HashMap<&'static str, Rgb>,
    picked: &mut HashMap<&'static str, String>,
    name: &'static str,
) {
    let hover = rgb_of(out, "state.hover");
    let panel = rgb_of(out, "surface.panel");
    let text = ["selection_foreground", "foreground", "bright_foreground"];
    if let Some(first) = chain.first() {
        if let Some(color) = vals.get(*first) {
            if step_ok(vals, hover, panel, *color, &text) {
                out.insert(name, *color);
                picked.insert(name, (*first).to_string());
                return;
            }
        }
    }
    let fallback = vals
        .get("accent")
        .copied()
        .or_else(|| vals.get("foreground").copied())
        .unwrap_or(Rgb::new(255, 255, 255));
    let mut chosen = None;
    for cents in 14..=40 {
        let flat = mix(panel, fallback, f64::from(cents) / 100.0);
        if step_ok(vals, hover, panel, flat, &text) {
            chosen = Some((cents, flat));
            break;
        }
    }
    let (cents, flat) = chosen.unwrap_or_else(|| {
        let cents = 40;
        let flat = mix(panel, fallback, f64::from(cents) / 100.0);
        (cents, flat)
    });
    out.insert(name, flat);
    picked.insert(
        name,
        format!("accent @ {}% over surface.panel", percent_label(cents)),
    );
}

fn step_ok(
    vals: &HashMap<String, Rgb>,
    hover: Rgb,
    panel: Rgb,
    candidate: Rgb,
    text: &[&str],
) -> bool {
    let d_h = ciede2000(candidate, hover);
    let d_p = ciede2000(candidate, panel);
    let d_hp = ciede2000(hover, panel);
    let readable = text.iter().any(|name| {
        vals.get(*name)
            .is_some_and(|rgb| meets(contrast_ratio(*rgb, candidate), 4.5))
    });
    d_h >= 8.0 && d_p > d_hp && readable
}

fn card_alpha(paint: Rgb, raised: Rgb, canvas: Rgb) -> i32 {
    let mut best = 12;
    for cents in 12..=30 {
        let flat = mix(raised, paint, f64::from(cents) / 100.0);
        best = cents;
        if ciede2000(flat, raised) >= 10.0 && ciede2000(flat, canvas) >= 10.0 {
            break;
        }
    }
    best
}

fn away_alpha(
    vals: &HashMap<String, Rgb>,
    out: &HashMap<&'static str, Rgb>,
    nominal_cents: i32,
) -> (&'static str, i32) {
    let plan = accent_plan(
        vals,
        rgb_of(out, "text.on-accent"),
        rgb_of(out, "accent.fill"),
    );
    let cents = if nominal_cents < 20 {
        plan.hover_cents
    } else {
        plan.pressed_cents
    };
    (plan.slot, cents)
}

#[derive(Clone, Copy)]
struct AccentPlan {
    slot: &'static str,
    hover_cents: i32,
    pressed_cents: i32,
    floor: bool,
    score: f64,
}

fn accent_plan(vals: &HashMap<String, Rgb>, label: Rgb, fill: Rgb) -> AccentPlan {
    let dark_label = lum(label) < lum(fill);
    let r0 = contrast_ratio(label, fill);
    let away: &[&str] = if dark_label {
        &[
            "foreground",
            "bright_foreground",
            "light_foreground",
            "background",
            "lighter_background",
        ]
    } else {
        &[
            "darker_background",
            "dark_background",
            "background",
            "foreground",
            "dark_foreground",
        ]
    };
    let toward: &[&str] = if dark_label {
        &["darker_background", "dark_background", "background"]
    } else {
        &["foreground", "bright_foreground", "light_foreground"]
    };

    let mut cands: Vec<AccentPlan> = Vec::new();
    for name in away {
        let Some(rgb) = vals.get(*name).copied() else {
            continue;
        };
        if lum(rgb) != lum(fill) && ((lum(rgb) > lum(fill)) == dark_label) {
            if let Some(plan) = plan_slot(name, rgb, label, fill, r0, true) {
                cands.push(plan);
            }
        }
    }
    if cands.is_empty() && meets(r0, 7.0) {
        for name in toward {
            let Some(rgb) = vals.get(*name).copied() else {
                continue;
            };
            if let Some(plan) = plan_slot(name, rgb, label, fill, r0, false) {
                cands.push(plan);
            }
        }
    }
    if let Some(plan) = cands.iter().find(|plan| plan.floor) {
        return *plan;
    }
    if cands.is_empty() {
        return AccentPlan {
            slot: "foreground",
            hover_cents: 0,
            pressed_cents: 0,
            floor: false,
            score: 0.0,
        };
    }
    let mut best = &cands[0];
    for plan in &cands[1..] {
        if plan.score > best.score {
            best = plan;
        }
    }
    *best
}

fn plan_slot(
    name: &'static str,
    slot: Rgb,
    label: Rgb,
    fill: Rgb,
    r0: f64,
    away: bool,
) -> Option<AccentPlan> {
    let ok = |ratio: f64| {
        if away {
            ratio >= r0 && (meets(ratio, 4.5) || !meets(r0, 4.5))
        } else {
            meets(ratio, 7.0)
        }
    };
    let mut hover = None;
    for cents in 14..=30 {
        let (ratio, de) = layer_eval(slot, cents, fill, label);
        if !ok(ratio) {
            break;
        }
        hover = Some((cents, de));
        if de >= 6.0 {
            break;
        }
    }
    let (hover_cents, hover_de) = hover?;
    let mut pressed = None;
    let press_lo = 24.max(hover_cents + 1);
    for cents in press_lo..=40 {
        let (ratio, de) = layer_eval(slot, cents, fill, label);
        if !ok(ratio) {
            break;
        }
        pressed = Some((cents, de));
        if de >= 10.0 {
            break;
        }
    }
    let (pressed_cents, pressed_de) = pressed?;
    Some(AccentPlan {
        slot: name,
        hover_cents,
        pressed_cents,
        floor: hover_de >= 6.0 && pressed_de >= 10.0,
        score: hover_de + pressed_de,
    })
}

fn layer_eval(slot: Rgb, cents: i32, fill: Rgb, label: Rgb) -> (f64, f64) {
    let flat = mix(fill, slot, f64::from(cents) / 100.0);
    (contrast_ratio(label, flat), ciede2000(fill, flat))
}

fn passes(vals: &HashMap<String, Rgb>, slot: &str, surfaces: &[Rgb], floor: f64) -> bool {
    let Some(rgb) = vals.get(slot) else {
        return false;
    };
    surfaces
        .iter()
        .all(|surface| meets(contrast_ratio(*rgb, *surface), floor))
}

/// `pairs.csv` matches [`contrast_ratio`] to 2 decimals. The design harness
/// accepted 2.9989 as 3.0 on miasma, so the floor check uses that rounding.
fn meets(ratio: f64, floor: f64) -> bool {
    (ratio * 100.0).round() / 100.0 >= floor
}

fn worst_slot<'a>(vals: &HashMap<String, Rgb>, chain: &[&'a str], surfaces: &[Rgb]) -> &'a str {
    let mut best_name = chain[0];
    let mut best_score = f64::MIN;
    for name in chain {
        let Some(rgb) = vals.get(*name) else {
            continue;
        };
        let score = if surfaces.is_empty() {
            0.0
        } else {
            surfaces
                .iter()
                .map(|surface| contrast_ratio(*rgb, *surface))
                .fold(f64::MAX, f64::min)
        };
        if score > best_score {
            best_score = score;
            best_name = name;
        }
    }
    best_name
}

fn resolved_surfaces(out: &HashMap<&'static str, Rgb>, names: &[&str]) -> Vec<Rgb> {
    names
        .iter()
        .filter_map(|name| out.get(*name).copied())
        .collect()
}

fn rgb_of(out: &HashMap<&'static str, Rgb>, name: &str) -> Rgb {
    out.get(name).copied().unwrap_or(Rgb::new(0, 0, 0))
}

fn full_chain<'a>(spec: &Spec, mode: ThemeMode, vals: &'a HashMap<String, Rgb>) -> Vec<&'a str> {
    let slots = match (mode, spec.light) {
        (ThemeMode::Light, Some(light)) => light,
        _ => spec.slots,
    };
    let mut chain: Vec<&str> = slots
        .iter()
        .copied()
        .filter(|slot| vals.contains_key(*slot))
        .collect();
    let add_foreground = spec.role.floor().is_some()
        && !is_rule_b(spec.name)
        && spec.name != "text.on-accent"
        && !chain.contains(&"foreground")
        && vals.contains_key("foreground");
    if add_foreground {
        chain.push("foreground");
    }
    chain
}

fn is_rule_b(name: &str) -> bool {
    RULE_B.iter().any(|(icon, _)| *icon == name)
}

/// `build_tokens.py` labels an alpha with `int(a * 100)`, which truncates.
/// 0.29 is the float just under 0.29, so that label is 28.
fn percent_label(cents: i32) -> i32 {
    (f64::from(cents) / 100.0 * 100.0) as i32
}

fn hex(rgb: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b)
}

fn lum(rgb: Rgb) -> f64 {
    fn channel(v: u8) -> f64 {
        let x = f64::from(v) / 255.0;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(rgb.r) + 0.7152 * channel(rgb.g) + 0.0722 * channel(rgb.b)
}

fn lab(rgb: Rgb) -> (f64, f64, f64) {
    fn channel(v: u8) -> f64 {
        let x = f64::from(v) / 255.0;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    }
    let r = channel(rgb.r);
    let g = channel(rgb.g);
    let b = channel(rgb.b);
    let x = (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / 0.95047;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = (0.0193339 * r + 0.1191920 * g + 0.9503041 * b) / 1.08883;
    let f = |t: f64| {
        if t > 216.0 / 24389.0 {
            t.cbrt()
        } else {
            (24389.0 / 27.0 * t + 16.0) / 116.0
        }
    };
    let fx = f(x);
    let fy = f(y);
    let fz = f(z);
    (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
}

/// CIEDE2000. Same formulation `build_tokens.py` uses for selection, cards, and accent layers.
pub fn ciede2000(a: Rgb, b: Rgb) -> f64 {
    let (l1, a1, b1) = lab(a);
    let (l2, a2, b2) = lab(b);
    let c1 = a1.hypot(b1);
    let c2 = a2.hypot(b2);
    let cb = (c1 + c2) / 2.0;
    let cb7 = cb.powi(7);
    let g = 0.5 * (1.0 - (cb7 / (cb7 + 25.0_f64.powi(7))).sqrt());
    let a1p = (1.0 + g) * a1;
    let a2p = (1.0 + g) * a2;
    let c1p = a1p.hypot(b1);
    let c2p = a2p.hypot(b2);
    let h1p = b1.atan2(a1p).to_degrees().rem_euclid(360.0);
    let h2p = b2.atan2(a2p).to_degrees().rem_euclid(360.0);
    let dlp = l2 - l1;
    let dcp = c2p - c1p;
    let dhp = if c1p * c2p == 0.0 {
        0.0
    } else if (h2p - h1p).abs() <= 180.0 {
        h2p - h1p
    } else if h2p - h1p > 180.0 {
        h2p - h1p - 360.0
    } else {
        h2p - h1p + 360.0
    };
    let dhp_term = 2.0 * (c1p * c2p).sqrt() * (dhp / 2.0).to_radians().sin();
    let lbp = (l1 + l2) / 2.0;
    let cbp = (c1p + c2p) / 2.0;
    let hbp = if c1p * c2p == 0.0 {
        h1p + h2p
    } else if (h1p - h2p).abs() <= 180.0 {
        (h1p + h2p) / 2.0
    } else if h1p + h2p < 360.0 {
        (h1p + h2p + 360.0) / 2.0
    } else {
        (h1p + h2p - 360.0) / 2.0
    };
    let t = 1.0 - 0.17 * (hbp - 30.0).to_radians().cos()
        + 0.24 * (2.0 * hbp).to_radians().cos()
        + 0.32 * (3.0 * hbp + 6.0).to_radians().cos()
        - 0.20 * (4.0 * hbp - 63.0).to_radians().cos();
    let dth = 30.0 * (-((hbp - 275.0) / 25.0).powi(2)).exp();
    let cbp7 = cbp.powi(7);
    let rc = 2.0 * (cbp7 / (cbp7 + 25.0_f64.powi(7))).sqrt();
    let sl = 1.0 + 0.015 * (lbp - 50.0).powi(2) / (20.0 + (lbp - 50.0).powi(2)).sqrt();
    let sc = 1.0 + 0.045 * cbp;
    let sh = 1.0 + 0.015 * cbp * t;
    let rt = -(2.0 * dth).to_radians().sin() * rc;
    ((dlp / sl).powi(2)
        + (dcp / sc).powi(2)
        + (dhp_term / sh).powi(2)
        + rt * (dcp / sc) * (dhp_term / sh))
        .sqrt()
}

const RULE_B: &[(&str, &str)] = &[
    ("status.success", "status.success-text"),
    ("status.warning", "status.warning-text"),
    ("status.error", "status.error-text"),
    ("status.info", "status.info-text"),
    ("preview.state.live", "status.success-text"),
    ("preview.state.building", "status.info-text"),
    ("preview.state.stale", "status.warning-text"),
    ("preview.state.error", "status.error-text"),
];

#[derive(Clone, Copy)]
struct Spec {
    name: &'static str,
    slots: &'static [&'static str],
    light: Option<&'static [&'static str]>,
    role: Role,
    on: &'static [&'static str],
    /// `Some((slot, nominal cents, base token))`. `@away` and `@card` are special.
    overlay: Option<(&'static str, i32, &'static str)>,
    step: bool,
}

#[derive(Clone, Copy)]
enum Role {
    Text,
    Ui,
    Disabled,
    Decor,
    Surface,
}

impl Role {
    fn floor(self) -> Option<f64> {
        match self {
            Self::Text => Some(4.5),
            Self::Ui => Some(3.0),
            Self::Disabled | Self::Decor | Self::Surface => None,
        }
    }
}

const fn chain(
    name: &'static str,
    slots: &'static [&'static str],
    role: Role,
    on: &'static [&'static str],
) -> Spec {
    Spec {
        name,
        slots,
        light: None,
        role,
        on,
        overlay: None,
        step: false,
    }
}

const fn chain_mode(
    name: &'static str,
    dark: &'static [&'static str],
    light: &'static [&'static str],
) -> Spec {
    Spec {
        name,
        slots: dark,
        light: Some(light),
        role: Role::Surface,
        on: &[],
        overlay: None,
        step: false,
    }
}

const fn paint(name: &'static str, slot: &'static str, cents: i32, base: &'static str) -> Spec {
    Spec {
        name,
        slots: &[],
        light: None,
        role: Role::Surface,
        on: &[],
        overlay: Some((slot, cents, base)),
        step: false,
    }
}

const SPECS: &[Spec] = &[
    chain_mode("surface.base", &["darker_background"], &["dark_background"]),
    chain("surface.panel", &["background"], Role::Surface, &[]),
    chain(
        "surface.raised",
        &["lighter_background"],
        Role::Surface,
        &[],
    ),
    chain_mode("surface.overlay", &["dark_background"], &["background"]),
    chain_mode("surface.input", &["darker_background"], &["background"]),
    paint("state.hover", "foreground", 6, "surface.panel"),
    paint("state.pressed", "foreground", 10, "surface.panel"),
    Spec {
        name: "selection.bg",
        slots: &["selection"],
        light: None,
        role: Role::Surface,
        on: &[],
        overlay: None,
        step: true,
    },
    chain(
        "text.primary",
        &["foreground", "bright_foreground", "light_foreground"],
        Role::Text,
        &[
            "surface.base",
            "surface.panel",
            "surface.raised",
            "surface.overlay",
            "surface.input",
            "state.hover",
            "state.pressed",
        ],
    ),
    chain(
        "text.secondary",
        &["light_foreground", "foreground", "bright_foreground"],
        Role::Text,
        &[
            "surface.base",
            "surface.panel",
            "surface.raised",
            "surface.overlay",
            "surface.input",
            "state.hover",
        ],
    ),
    chain(
        "text.placeholder",
        &["dark_foreground", "light_foreground", "foreground"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "text.disabled",
        &["muted"],
        Role::Disabled,
        &["surface.panel", "surface.input"],
    ),
    chain(
        "text.accent",
        &["accent", "bright_blue", "blue", "foreground"],
        Role::Text,
        &["surface.panel", "surface.overlay", "surface.base"],
    ),
    chain(
        "text.selected",
        &["selection_foreground", "foreground", "bright_foreground"],
        Role::Text,
        &["selection.bg"],
    ),
    chain(
        "accent.fill",
        &["accent", "bright_blue", "blue"],
        Role::Ui,
        &["surface.panel", "surface.base", "surface.overlay"],
    ),
    chain(
        "text.on-accent",
        &[
            "background",
            "darker_background",
            "foreground",
            "bright_foreground",
        ],
        Role::Text,
        &["accent.fill"],
    ),
    paint("state.on-accent-hover", "@away", 14, "accent.fill"),
    paint("state.on-accent-pressed", "@away", 24, "accent.fill"),
    chain(
        "focus.ring",
        &["accent", "bright_blue", "blue", "foreground"],
        Role::Ui,
        &[
            "surface.base",
            "surface.panel",
            "surface.raised",
            "surface.overlay",
            "surface.input",
            "selection.bg",
        ],
    ),
    chain(
        "border.input",
        &["muted", "dark_foreground", "light_foreground", "foreground"],
        Role::Ui,
        &["surface.panel", "surface.overlay", "surface.input"],
    ),
    chain(
        "border.subtle",
        &["lighter_background", "selection", "muted"],
        Role::Decor,
        &["surface.panel", "surface.base"],
    ),
    paint("border.card", "@card", 12, "surface.raised"),
    chain(
        "border.active",
        &["active_border_color", "accent"],
        Role::Ui,
        &["surface.panel", "surface.base"],
    ),
    chain(
        "caret",
        &["cursor", "bright_foreground", "foreground"],
        Role::Ui,
        &["surface.input", "surface.panel"],
    ),
    chain(
        "scrollbar.thumb",
        &["muted", "dark_foreground", "light_foreground"],
        Role::Ui,
        &["surface.panel", "surface.base"],
    ),
    chain(
        "scrollbar.thumb-hover",
        &["accent", "bright_blue"],
        Role::Ui,
        &["surface.panel", "surface.base"],
    ),
    chain(
        "status.success",
        &["green", "bright_green", "cyan", "bright_cyan"],
        Role::Ui,
        &["surface.panel", "surface.overlay", "surface.base"],
    ),
    chain(
        "status.success-text",
        &[
            "green",
            "bright_green",
            "cyan",
            "bright_cyan",
            "light_foreground",
            "foreground",
        ],
        Role::Text,
        &["surface.panel", "surface.overlay", "surface.base"],
    ),
    chain(
        "status.warning",
        &["yellow", "bright_yellow", "orange", "brown"],
        Role::Ui,
        &["surface.panel", "surface.overlay", "surface.base"],
    ),
    chain(
        "status.warning-text",
        &[
            "yellow",
            "bright_yellow",
            "orange",
            "brown",
            "light_foreground",
            "foreground",
        ],
        Role::Text,
        &["surface.panel", "surface.overlay", "surface.base"],
    ),
    chain(
        "status.error",
        &["red", "bright_red", "orange", "magenta"],
        Role::Ui,
        &["surface.panel", "surface.overlay", "surface.base"],
    ),
    chain(
        "status.error-text",
        &[
            "red",
            "bright_red",
            "orange",
            "magenta",
            "light_foreground",
            "foreground",
        ],
        Role::Text,
        &["surface.panel", "surface.overlay", "surface.base"],
    ),
    chain(
        "status.info",
        &["cyan", "bright_cyan", "blue", "bright_blue", "accent"],
        Role::Ui,
        &["surface.panel", "surface.overlay", "surface.base"],
    ),
    chain(
        "status.info-text",
        &[
            "cyan",
            "bright_cyan",
            "blue",
            "bright_blue",
            "accent",
            "light_foreground",
            "foreground",
        ],
        Role::Text,
        &["surface.panel", "surface.overlay", "surface.base"],
    ),
    paint("status.success-subtle", "green", 14, "surface.panel"),
    paint("status.warning-subtle", "yellow", 14, "surface.panel"),
    paint("status.error-subtle", "red", 14, "surface.panel"),
    paint("status.info-subtle", "cyan", 14, "surface.panel"),
    chain(
        "tree.folder",
        &["blue", "bright_blue", "accent", "light_foreground"],
        Role::Text,
        &["surface.base", "surface.overlay"],
    ),
    chain(
        "tree.file",
        &["yellow", "bright_yellow", "foreground", "light_foreground"],
        Role::Text,
        &["surface.base", "surface.overlay"],
    ),
    chain_mode(
        "preview.frame",
        &["darker_background"],
        &["dark_background"],
    ),
    chain_mode("preview.toolbar", &["dark_background"], &["background"]),
    chain(
        "preview.viewport-border",
        &["muted", "dark_foreground", "light_foreground"],
        Role::Ui,
        &["preview.frame"],
    ),
    chain(
        "preview.state.live",
        &["green", "bright_green", "cyan"],
        Role::Ui,
        &["preview.toolbar", "surface.base"],
    ),
    chain(
        "preview.state.building",
        &["accent", "bright_blue", "blue"],
        Role::Ui,
        &["preview.toolbar", "surface.base"],
    ),
    chain(
        "preview.state.stale",
        &["yellow", "bright_yellow", "orange", "brown"],
        Role::Ui,
        &["preview.toolbar", "surface.base"],
    ),
    chain(
        "preview.state.error",
        &["red", "bright_red", "orange"],
        Role::Ui,
        &["preview.toolbar", "surface.base"],
    ),
    paint("preview.scrim", "darker_background", 72, "surface.panel"),
    chain(
        "md.heading",
        &[
            "accent",
            "bright_blue",
            "blue",
            "foreground",
            "light_foreground",
        ],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "md.strong",
        &["bright_foreground", "foreground"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "md.link",
        &["cyan", "bright_cyan", "blue", "accent", "light_foreground"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "md.code",
        &["orange", "yellow", "bright_yellow", "light_foreground"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "md.quote",
        &["light_foreground", "foreground"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "md.marker",
        &["dark_foreground", "light_foreground", "foreground"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "syntax.keyword",
        &["magenta", "bright_magenta", "blue", "light_foreground"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "syntax.string",
        &["green", "bright_green", "cyan", "light_foreground"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "syntax.number",
        &["orange", "yellow", "bright_yellow", "light_foreground"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "syntax.attribute",
        &["yellow", "bright_yellow", "orange", "light_foreground"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "syntax.tag",
        &["blue", "bright_blue", "accent", "light_foreground"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "syntax.comment",
        &["dark_foreground", "light_foreground", "muted"],
        Role::Text,
        &["surface.input"],
    ),
    chain(
        "syntax.punctuation",
        &["light_foreground", "foreground"],
        Role::Text,
        &["surface.input"],
    ),
];
