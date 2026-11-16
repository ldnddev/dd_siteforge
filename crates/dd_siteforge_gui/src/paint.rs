//! Token colors for the shell. Values come from the resolved Omarchy theme.

use gpui::{App, Hsla, SharedString, px};
use gpui_component::{Theme, ThemeColor, ThemeMode};
use ldnddev_theme::{GuiTheme, Rgb, ThemeMode as SiteMode};

use crate::theme_load::{self, LiveTheme};

pub fn token(theme: &GuiTheme, name: &str) -> Hsla {
    hsla(theme_load::rgb(theme, name))
}

pub fn hsla(color: Rgb) -> Hsla {
    gpui::Rgba {
        r: f32::from(color.r) / 255.0,
        g: f32::from(color.g) / 255.0,
        b: f32::from(color.b) / 255.0,
        a: 1.0,
    }
    .into()
}

pub fn apply_component_theme(theme: &LiveTheme, cx: &mut App) {
    let mode = match theme.gui.mode {
        SiteMode::Light => ThemeMode::Light,
        SiteMode::Dark => ThemeMode::Dark,
    };
    let base = if theme.gui.mode == SiteMode::Light {
        *ThemeColor::light()
    } else {
        *ThemeColor::dark()
    };
    let mut colors = base;
    colors.background = hsla(theme_load::rgb(&theme.gui, "surface.base"));
    colors.foreground = hsla(theme_load::rgb(&theme.gui, "text.primary"));
    colors.accent = hsla(theme_load::rgb(&theme.gui, "accent.fill"));
    colors.accent_foreground = hsla(theme_load::rgb(&theme.gui, "text.on-accent"));
    colors.border = hsla(theme_load::rgb(&theme.gui, "border.input"));
    colors.caret = hsla(theme_load::rgb(&theme.gui, "caret"));
    colors.link = hsla(theme_load::rgb(&theme.gui, "text.accent"));
    let family = SharedString::from(theme.fonts.ui.clone());
    Theme::update(cx, |ui| {
        ui.mode = mode;
        ui.colors = colors;
        ui.font_family = family.clone();
        ui.mono_font_family = family;
        ui.font_size = px(theme.metrics.type_scale.sm);
        ui.radius = px(theme.metrics.radius_sm);
    });
}
