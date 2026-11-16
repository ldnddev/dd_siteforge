//! Omarchy theme load. Same `dd_siteforge_theme.yml` → `omarchy:` path as the TUI.
//! Colors are resolved here, not blended.

use std::path::PathBuf;
use std::process::Command;

use ldnddev_theme::{
    FONT_DEFAULT, FontStack, GuiTheme, Metrics, OmarchyChoice, OmarchyColors, OmarchyPaths,
    ParseMode, ThemeMode, ThemeSource, follow_stamp, load_colors, load_from_str, resolve_gui_theme,
};

/// Last-resort palette when no Omarchy theme can be read. The shipped
/// retro-82 colors, so the window still opens on a machine without Omarchy.
const RETRO_82: &str =
    include_str!("../../ldnddev_theme/tests/fixtures/omarchy/retro-82/colors.toml");

#[derive(Clone, Debug)]
pub struct LiveTheme {
    pub gui: GuiTheme,
    pub metrics: Metrics,
    pub fonts: FontStack,
    pub label: String,
    pub choice: Option<OmarchyChoice>,
    pub stamp: Option<String>,
    pub warning: Option<String>,
}

impl LiveTheme {
    pub fn load(paths: &OmarchyPaths, reduced_motion: bool) -> Self {
        let (choice, label, warning) = read_choice();
        let (colors, warning) = match &choice {
            Some(choice) => match load_colors(paths, choice) {
                Ok(colors) => (colors, warning),
                Err(err) => (
                    fallback_colors(),
                    Some(format!("omarchy theme failed ({err}); using retro-82")),
                ),
            },
            None => (
                fallback_colors(),
                warning
                    .or_else(|| Some("theme file has no omarchy: key; using retro-82".to_string())),
            ),
        };
        let family = fc_match_monospace();
        Self {
            gui: resolve_gui_theme(&colors),
            metrics: Metrics::standard().with_reduced_motion(reduced_motion),
            fonts: FontStack::from_family(&family),
            label,
            choice,
            stamp: follow_stamp(paths),
            warning,
        }
    }

    /// Re-read a followed theme and the fontconfig monospace family.
    pub fn refresh(&mut self, paths: &OmarchyPaths) -> bool {
        let family = fc_match_monospace();
        let font_changed = family != self.fonts.ui;
        if font_changed {
            self.fonts = FontStack::from_family(&family);
        }
        let Some(OmarchyChoice::Follow) = &self.choice else {
            return font_changed;
        };
        let stamp = follow_stamp(paths);
        if stamp == self.stamp && !font_changed {
            return false;
        }
        let reduced = self.metrics.reduced_motion;
        *self = Self::load(paths, reduced);
        true
    }

    pub fn mode_label(&self) -> &'static str {
        match self.gui.mode {
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
        }
    }
}

fn read_choice() -> (Option<OmarchyChoice>, String, Option<String>) {
    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("dd_siteforge_theme.yml"));
    }
    if let Some(home) = ldnddev_theme::default_config_home() {
        candidates.push(ldnddev_theme::global_theme_path(
            &home,
            "dd_siteforge_theme.yml",
        ));
    }
    for path in candidates {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        match load_from_str(&text, ParseMode::Lenient, ThemeSource::Local) {
            Ok(palette) => {
                return (palette.omarchy.clone(), path.display().to_string(), None);
            }
            Err(err) => {
                return (
                    None,
                    path.display().to_string(),
                    Some(format!("could not parse theme file: {err}")),
                );
            }
        }
    }
    (None, "built-in".to_string(), None)
}

fn fallback_colors() -> OmarchyColors {
    ldnddev_theme::colors_from_str(RETRO_82).unwrap_or_else(|_| {
        ldnddev_theme::colors_from_str("background = \"#05182e\"\nforeground = \"#f6dcac\"\n")
            .expect("minimal palette")
    })
}

/// `omarchy font current`: the first family `fc-match monospace` returns.
pub fn fc_match_monospace() -> String {
    let Ok(output) = Command::new("fc-match")
        .args(["monospace", "-f", "%{family}"])
        .output()
    else {
        return FONT_DEFAULT.to_string();
    };
    if !output.status.success() {
        return FONT_DEFAULT.to_string();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let first = text
        .split([',', '\n'])
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('"');
    if first.is_empty() {
        FONT_DEFAULT.to_string()
    } else {
        first.to_string()
    }
}

pub fn rgb(theme: &GuiTheme, name: &str) -> ldnddev_theme::Rgb {
    theme
        .rgb(name)
        .unwrap_or(ldnddev_theme::Rgb::new(255, 255, 255))
}

#[allow(dead_code)]
pub fn config_dir_hint() -> PathBuf {
    ldnddev_theme::default_config_home().unwrap_or_else(|| PathBuf::from(".config"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fc_match_returns_a_concrete_family() {
        let family = fc_match_monospace();
        assert!(!family.is_empty());
        assert_ne!(family, "monospace");
    }

    #[test]
    fn fallback_retro_82_resolves_without_omarchy_installed() {
        let colors = fallback_colors();
        let theme = resolve_gui_theme(&colors);
        let base = theme.token("surface.base").unwrap();
        assert_eq!(base.hex, "#020c17");
        assert_eq!(theme.primary_button.as_str(), "filled");
    }
}
