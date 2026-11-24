//! GUI-only preferences. They never go in `site.json`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuiSettings {
    #[serde(default)]
    pub density: Density,
    /// On by default, matching the TUI. The palette that reads this is later.
    #[serde(default = "default_true")]
    pub vim_keys: bool,
    #[serde(default)]
    pub reduced_motion: bool,
}

fn default_true() -> bool {
    true
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self {
            density: Density::Comfortable,
            vim_keys: true,
            reduced_motion: false,
        }
    }
}

pub fn settings_path() -> PathBuf {
    dd_siteforge_core::scaffold::config_scaffold_dir().join("gui.json")
}

pub fn load_from(path: &Path) -> GuiSettings {
    let Ok(bytes) = fs::read(path) else {
        return GuiSettings::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

pub fn save_to(path: &Path, settings: &GuiSettings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(settings).unwrap_or_else(|_| b"{}".to_vec());
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_outside_site_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("gui.json");
        let settings = GuiSettings {
            density: Density::Compact,
            vim_keys: false,
            reduced_motion: true,
        };
        save_to(&path, &settings).unwrap();
        let loaded = load_from(&path);
        assert_eq!(loaded, settings);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("compact"));
        assert!(!text.contains("schema_version"));
    }
}
