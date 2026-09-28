//! Bundled Handlebars templates + on-disk overlays.
//!
//! Load order (later wins): baked-in `templates/*.hbs`, then the crate
//! `templates/` directory when this binary was built from a tree that still
//! exists, then `<site>/templates/`, then `<site>/source/templates/`.
//! `init-templates` copies from that crate `templates/` dir when present so
//! a new site gets the files on disk, not a stale compile snapshot.
//! Export never writes these files. `init-templates --force` is the only overwrite.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use handlebars::Handlebars;
use serde_json::Value;

pub const BUNDLED: &[(&str, &str)] = &[
    ("_page", include_str!("../templates/_page.hbs")),
    ("_head", include_str!("../templates/_head.hbs")),
    ("_media", include_str!("../templates/_media.hbs")),
    ("dd-header", include_str!("../templates/dd-header.hbs")),
    ("dd-footer", include_str!("../templates/dd-footer.hbs")),
    ("dd-hero", include_str!("../templates/dd-hero.hbs")),
    ("dd-section", include_str!("../templates/dd-section.hbs")),
    (
        "dd-section-column",
        include_str!("../templates/dd-section-column.hbs"),
    ),
    (
        "dd-alternating",
        include_str!("../templates/dd-alternating.hbs"),
    ),
    ("dd-card", include_str!("../templates/dd-card.hbs")),
    ("dd-banner", include_str!("../templates/dd-banner.hbs")),
    ("dd-cta", include_str!("../templates/dd-cta.hbs")),
    (
        "dd-filmstrip",
        include_str!("../templates/dd-filmstrip.hbs"),
    ),
    (
        "dd-milestones",
        include_str!("../templates/dd-milestones.hbs"),
    ),
    ("dd-modal", include_str!("../templates/dd-modal.hbs")),
    ("dd-slider", include_str!("../templates/dd-slider.hbs")),
    (
        "dd-accordion",
        include_str!("../templates/dd-accordion.hbs"),
    ),
    (
        "dd-blockquote",
        include_str!("../templates/dd-blockquote.hbs"),
    ),
    ("dd-alert", include_str!("../templates/dd-alert.hbs")),
    ("dd-image", include_str!("../templates/dd-image.hbs")),
    (
        "dd-rich_text",
        include_str!("../templates/dd-rich_text.hbs"),
    ),
    (
        "dd-navigation",
        include_str!("../templates/dd-navigation.hbs"),
    ),
    (
        "dd-navigation-item",
        include_str!("../templates/dd-navigation-item.hbs"),
    ),
    (
        "dd-header-search",
        include_str!("../templates/dd-header-search.hbs"),
    ),
    (
        "dd-header-menu",
        include_str!("../templates/dd-header-menu.hbs"),
    ),
    ("dd-spacer", include_str!("../templates/dd-spacer.hbs")),
    ("dd-tabs", include_str!("../templates/dd-tabs.hbs")),
    ("dd-timeline", include_str!("../templates/dd-timeline.hbs")),
];

pub fn bundled(name: &str) -> Option<&'static str> {
    BUNDLED
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, src)| *src)
}

pub fn templates_dir(site_root: &Path) -> PathBuf {
    site_root.join("source").join("templates")
}

/// Crate `templates/` on disk when this binary was built from a checkout that
/// still exists (cargo run / cargo install --path). Release binaries fall back
/// to the baked-in copies.
pub fn live_crate_templates_dir() -> Option<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
    dir.is_dir().then_some(dir)
}

fn template_body(name: &str) -> anyhow::Result<String> {
    if let Some(dir) = live_crate_templates_dir() {
        let path = dir.join(format!("{name}.hbs"));
        if path.is_file() {
            return fs::read_to_string(&path)
                .with_context(|| format!("failed to read '{}'", path.display()));
        }
    }
    bundled(name)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("unknown template '{name}'"))
}

/// If a site overlay is still an exact copy of the baked-in template, prefer
/// the live crate file. Customized `source/templates` files are left alone.
fn promote_unmodified_seeds(sources: &mut HashMap<String, String>) -> anyhow::Result<()> {
    let Some(dir) = live_crate_templates_dir() else {
        return Ok(());
    };
    for (name, bundled_src) in BUNDLED {
        let Some(current) = sources.get(*name) else {
            continue;
        };
        if current != bundled_src {
            continue;
        }
        let path = dir.join(format!("{name}.hbs"));
        if !path.is_file() {
            continue;
        }
        let live = fs::read_to_string(&path)
            .with_context(|| format!("failed to read '{}'", path.display()))?;
        if live != *bundled_src {
            sources.insert((*name).to_string(), live);
        }
    }
    Ok(())
}

fn apply_template_dir(dir: &Path, sources: &mut HashMap<String, String>) -> anyhow::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)
        .with_context(|| format!("failed to read templates dir '{}'", dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("hbs") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read override '{}'", path.display()))?;
        sources.insert(name.to_string(), text);
    }
    Ok(())
}

pub struct Renderer {
    hbs: Handlebars<'static>,
}

impl Renderer {
    pub fn load(site_root: Option<&Path>) -> anyhow::Result<Self> {
        let mut sources: HashMap<String, String> = HashMap::new();
        for (name, src) in BUNDLED {
            sources.insert((*name).to_string(), (*src).to_string());
        }
        if let Some(root) = site_root {
            if let Some(dir) = live_crate_templates_dir() {
                apply_template_dir(&dir, &mut sources)?;
            }
            apply_template_dir(&root.join("templates"), &mut sources)?;
            apply_template_dir(&templates_dir(root), &mut sources)?;
            promote_unmodified_seeds(&mut sources)?;
        }

        let mut hbs = Handlebars::new();
        for (name, src) in &sources {
            hbs.register_template_string(name, src)
                .with_context(|| format!("failed to parse template '{name}.hbs'"))?;
        }
        Ok(Self { hbs })
    }

    #[cfg(test)]
    pub fn bundled_only() -> anyhow::Result<Self> {
        Self::load(None)
    }

    pub fn render(&self, name: &str, data: &Value) -> anyhow::Result<String> {
        self.hbs
            .render(name, data)
            .with_context(|| format!("failed to render template '{name}'"))
    }
}

pub struct SeedReport {
    pub written: Vec<String>,
    pub skipped: Vec<String>,
    /// Skipped files whose on-disk contents differ from the seed source.
    pub stale: Vec<String>,
    /// Directory the seed was copied from, when using the live crate tree.
    pub source: Option<PathBuf>,
}

/// Write templates into `<site_root>/source/templates/`.
/// Prefers the crate `templates/` directory on disk when it still exists.
/// Existing files are left alone unless `force` is set.
pub fn seed_templates(
    site_root: &Path,
    force: bool,
    only: Option<&str>,
) -> anyhow::Result<SeedReport> {
    if let Some(name) = only {
        if bundled(name).is_none() {
            return Err(anyhow!(
                "unknown template '{name}'. Known: {}",
                BUNDLED
                    .iter()
                    .map(|(n, _)| *n)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    let dir = templates_dir(site_root);
    fs::create_dir_all(&dir).with_context(|| format!("failed to create '{}'", dir.display()))?;
    let mut report = SeedReport {
        written: Vec::new(),
        skipped: Vec::new(),
        stale: Vec::new(),
        source: live_crate_templates_dir(),
    };
    for (name, _) in BUNDLED {
        if only.is_some_and(|n| n != *name) {
            continue;
        }
        let path = dir.join(format!("{name}.hbs"));
        let src = template_body(name)?;
        if path.exists() && !force {
            report.skipped.push((*name).to_string());
            if fs::read_to_string(&path).unwrap_or_default() != src {
                report.stale.push((*name).to_string());
            }
            continue;
        }
        fs::write(&path, src).with_context(|| format!("failed to write '{}'", path.display()))?;
        report.written.push((*name).to_string());
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp() -> std::path::PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("dd_tpl_{n}"))
    }

    #[test]
    fn bundled_covers_every_hbs_in_templates_dir() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
        let mut disk: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| {
                let path = e.ok()?.path();
                if path.extension().and_then(|e| e.to_str()) == Some("hbs") {
                    Some(path.file_stem()?.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .collect();
        disk.sort();
        let mut bundled: Vec<&str> = BUNDLED.iter().map(|(n, _)| *n).collect();
        bundled.sort();
        assert_eq!(
            bundled, disk,
            "every templates/*.hbs file must be in BUNDLED so cargo build embeds it"
        );
    }

    #[test]
    fn bundled_renderer_parses_every_template() {
        let r = Renderer::bundled_only().expect("bundled templates must parse");
        let html = r
            .render("_page", &json!({"lang":"en","head_html":"","header_html":"","footer_html":"","content":"hi"}))
            .unwrap();
        assert!(html.contains("<main>"));
        assert!(html.contains("hi"));
    }

    #[test]
    fn seed_skips_existing_unless_force() {
        let root = tmp();
        let first = seed_templates(&root, false, None).unwrap();
        assert!(!first.written.is_empty());
        assert!(first.skipped.is_empty());
        let path = templates_dir(&root).join("dd-hero.hbs");
        fs::write(&path, "OVERRIDE").unwrap();
        let second = seed_templates(&root, false, None).unwrap();
        assert!(second.written.is_empty());
        assert!(second.skipped.contains(&"dd-hero".to_string()));
        assert_eq!(fs::read_to_string(&path).unwrap(), "OVERRIDE");
        seed_templates(&root, true, Some("dd-hero")).unwrap();
        assert_ne!(fs::read_to_string(&path).unwrap(), "OVERRIDE");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn override_file_wins_and_broken_hbs_errors() {
        let root = tmp();
        seed_templates(&root, false, Some("dd-alert")).unwrap();
        let path = templates_dir(&root).join("dd-alert.hbs");
        fs::write(&path, "<div>ok {{parent_copy}}</div>").unwrap();
        let r = Renderer::load(Some(&root)).unwrap();
        let html = r
            .render(
                "dd-alert",
                &json!({
                    "parent_type": "-default",
                    "parent_class": "-default",
                    "sal": "fade",
                    "parent_title": "",
                    "has_title": false,
                    "parent_copy": "hello"
                }),
            )
            .unwrap();
        assert!(html.contains("hello"));
        fs::write(&path, "{{#if unterminated").unwrap();
        let err = match Renderer::load(Some(&root)) {
            Err(e) => e,
            Ok(_) => panic!("expected parse error"),
        };
        let msg = format!("{err:#}");
        assert!(msg.contains("dd-alert.hbs"), "{msg}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn site_templates_dir_overrides_bundled_and_source_templates_wins() {
        let root = tmp();
        let crate_dir = root.join("templates");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(
            crate_dir.join("dd-alert.hbs"),
            "<div>from-templates {{parent_copy}}</div>",
        )
        .unwrap();
        let r = Renderer::load(Some(&root)).unwrap();
        let data = json!({
            "parent_type": "-default",
            "parent_class": "-default",
            "sal": "fade",
            "parent_title": "",
            "has_title": false,
            "parent_copy": "x"
        });
        assert!(
            r.render("dd-alert", &data)
                .unwrap()
                .contains("from-templates"),
            "templates/ next to the site must load without a rebuild"
        );

        let overlay = templates_dir(&root);
        fs::create_dir_all(&overlay).unwrap();
        fs::write(
            overlay.join("dd-alert.hbs"),
            "<div>from-source {{parent_copy}}</div>",
        )
        .unwrap();
        let r = Renderer::load(Some(&root)).unwrap();
        assert!(
            r.render("dd-alert", &data).unwrap().contains("from-source"),
            "source/templates must win over templates/"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn seed_copies_live_crate_templates_when_present() {
        let Some(live) = live_crate_templates_dir() else {
            return;
        };
        let live_footer = live.join("dd-footer.hbs");
        if !live_footer.is_file() {
            return;
        }
        let expected = fs::read_to_string(&live_footer).unwrap();
        let root = tmp();
        seed_templates(&root, false, Some("dd-footer")).unwrap();
        let got = fs::read_to_string(templates_dir(&root).join("dd-footer.hbs")).unwrap();
        assert_eq!(
            got, expected,
            "init-templates must copy the on-disk crate template"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn unmodified_seed_yields_to_live_crate_edits() {
        let Some(live) = live_crate_templates_dir() else {
            return;
        };
        let live_footer = fs::read_to_string(live.join("dd-footer.hbs")).unwrap_or_default();
        let bundled_footer = bundled("dd-footer").unwrap();
        if live_footer == bundled_footer {
            return;
        }
        let root = tmp();
        let overlay = templates_dir(&root);
        fs::create_dir_all(&overlay).unwrap();
        fs::write(overlay.join("dd-footer.hbs"), bundled_footer).unwrap();
        let r = Renderer::load(Some(&root)).unwrap();
        let html = r
            .render(
                "dd-footer",
                &json!({
                    "custom": "",
                    "blurb": "",
                    "sections_html": "",
                    "has_socials": false,
                    "socials": [],
                    "copyright": "c"
                }),
            )
            .unwrap();
        assert!(
            html.contains("dd-scrolltop") || html == live_footer,
            "stock source/templates copy must not hide live crate dd-footer.hbs: {html}"
        );
        fs::remove_dir_all(&root).ok();
    }
}
