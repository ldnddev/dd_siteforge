//! In-process build, export, preview, and init. The GUI calls the same core
//! functions as the CLI and TUI. Wording matches those frontends.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use dd_siteforge_core::asset_build::AssetBuildCommand;
use dd_siteforge_core::export::{ExportReport, export_site, join_inside};
use dd_siteforge_core::model::Site;
use dd_siteforge_core::scaffold::{self, SeedReport as KitReport};
use dd_siteforge_core::templates::{self, SeedReport as TemplateReport};

use crate::document::clock_label;

#[derive(Debug)]
pub struct RanExport {
    pub report: ExportReport,
    pub output: PathBuf,
    pub normalized: String,
    pub display: String,
}

pub fn normalize_relative_path(raw: &str) -> String {
    let mut s = raw.trim();
    while let Some(rest) = s.strip_prefix("./") {
        s = rest.trim_start_matches('/');
    }
    s.trim_end_matches('/').to_string()
}

pub fn display_relative_path(normalized: &str, out: &Path) -> String {
    if normalized.is_empty() {
        out.display().to_string()
    } else {
        format!("./{normalized}/")
    }
}

/// Export `site` into `root`/`rel`, the same call the CLI makes once the path
/// is resolved. Empty and outside paths use the TUI sentences.
pub fn run_export(site: &Site, root: &Path, rel: &str) -> Result<RanExport, String> {
    let trimmed = rel.trim();
    if trimmed.is_empty() {
        return Err("Export path required.".to_string());
    }
    let Some(output) = join_inside(root, trimmed) else {
        return Err("Export path must stay inside the site directory.".to_string());
    };
    let normalized = normalize_relative_path(trimmed);
    match export_site(site, &output, Some(root)) {
        Ok(report) => {
            let display = display_relative_path(&normalized, &output);
            Ok(RanExport {
                report,
                output,
                normalized,
                display,
            })
        }
        Err(err) => Err(format!("Export failed: {err}")),
    }
}

pub fn export_toast(report: &ExportReport, display: &str) -> String {
    if report.wrote_404 {
        format!(
            "Exported {} page(s) to {} (wrote 404.html)",
            report.pages, display
        )
    } else {
        format!("Exported {} page(s) to {}", report.pages, display)
    }
}

pub fn exporting_line(pages: usize, display: &str) -> String {
    format!("Exporting {pages} page(s) to {display}…")
}

pub fn build_running(cmd: &AssetBuildCommand) -> String {
    format!("Running `{}`…", cmd.display())
}

pub const BUILD_IDLE: &str = "Build CSS/JS";
pub const BUILD_ALREADY: &str = "Build already running.";
pub const BUILD_STOPPED: &str = "Build thread stopped.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewView {
    Closed,
    NotBuilt,
    Building,
    Live,
    Stale { edits: u64 },
    Error,
}

pub fn preview_status(view: PreviewView, live_at: SystemTime) -> String {
    match view {
        PreviewView::Closed => "Preview closed · Ctrl+P".to_string(),
        PreviewView::NotBuilt => "Preview not built".to_string(),
        PreviewView::Building => "Building… (autosave)".to_string(),
        PreviewView::Live => format!("Preview live {}", clock_label(live_at)),
        PreviewView::Stale { edits } => format!("{edits} edits since last render"),
        PreviewView::Error => "Preview failed · Show issue".to_string(),
    }
}

pub fn kit_notes(report: &KitReport) -> Vec<String> {
    let mut notes = Vec::new();
    if !report.written.is_empty() {
        notes.push(format!("Wrote {} scaffold file(s)", report.written.len()));
    }
    if !report.skipped.is_empty() {
        notes.push(format!(
            "Skipped {} existing (use --force to overwrite)",
            report.skipped.len()
        ));
    }
    notes
}

pub fn template_notes(root: &Path, report: &TemplateReport) -> Vec<String> {
    let mut notes = Vec::new();
    let dest = templates::templates_dir(root);
    if !report.written.is_empty() {
        notes.push(format!(
            "Wrote to {}: {}",
            dest.display(),
            report.written.join(", ")
        ));
    }
    if !report.skipped.is_empty() {
        notes.push(format!(
            "Skipped existing (use --force to overwrite): {}",
            report.skipped.join(", ")
        ));
    }
    if !report.stale.is_empty() {
        notes.push(format!(
            "Out of date: {} (run init-templates --force to refresh)",
            report.stale.join(", ")
        ));
    }
    notes
}

pub fn seed_kit(root: &Path, project: &str) -> Result<Vec<String>, String> {
    let overlay = scaffold::overlay_if_present();
    let report = scaffold::seed_scaffold(
        root,
        scaffold::SeedOpts {
            force: false,
            project_name: Some(project),
            overlay: overlay.as_deref(),
        },
    )
    .map_err(|err| err.to_string())?;
    Ok(kit_notes(&report))
}

pub fn seed_template_files(root: &Path) -> Result<Vec<String>, String> {
    let report = templates::seed_templates(root, false, None).map_err(|err| err.to_string())?;
    Ok(template_notes(root, &report))
}

#[derive(Clone, Debug)]
pub struct PreviewChrome {
    pub url: String,
    pub status: String,
    pub follow: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dd_siteforge_core::storage::save_site;

    fn list_files(root: &Path) -> Vec<String> {
        let mut out = Vec::new();
        walk(root, root, &mut out);
        out.sort();
        out
    }

    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push(rel);
            }
        }
    }

    fn normalize(rel: &str, bytes: &[u8]) -> Vec<u8> {
        if rel != "sitemap.xml" {
            return bytes.to_vec();
        }
        let text = String::from_utf8_lossy(bytes);
        let mut out = String::new();
        let mut rest = text.as_ref();
        while let Some(start) = rest.find("<lastmod>") {
            out.push_str(&rest[..start]);
            out.push_str("<lastmod>");
            rest = &rest[start + "<lastmod>".len()..];
            if let Some(end) = rest.find("</lastmod>") {
                out.push_str("DATE");
                rest = &rest[end..];
            }
        }
        out.push_str(rest);
        out.into_bytes()
    }

    #[test]
    fn gui_export_matches_the_cli_tree() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("site");
        std::fs::create_dir_all(&root).unwrap();
        let site = Site::starter();
        save_site(root.join("site.json"), &site).unwrap();
        let ran = run_export(&site, &root, "./web/").unwrap();
        let cli = root.join("web-cli");
        let cli_report = export_site(&site, &cli, Some(&root)).unwrap();
        assert_eq!(ran.report.pages, cli_report.pages);
        assert_eq!(ran.report.wrote_404, cli_report.wrote_404);
        assert_eq!(ran.display, "./web/");
        let gui_files = list_files(&ran.output);
        let cli_files = list_files(&cli);
        assert_eq!(gui_files, cli_files);
        for rel in &gui_files {
            let left = normalize(rel, &std::fs::read(ran.output.join(rel)).unwrap());
            let right = normalize(rel, &std::fs::read(cli.join(rel)).unwrap());
            assert_eq!(left, right, "{rel}");
        }
        assert!(export_toast(&ran.report, &ran.display).starts_with("Exported "));
        assert_eq!(
            exporting_line(site.pages.len(), "./web/"),
            format!("Exporting {} page(s) to ./web/…", site.pages.len())
        );
    }

    #[test]
    fn export_path_errors_match_the_tui() {
        let dir = tempfile::tempdir().unwrap();
        let site = Site::starter();
        assert_eq!(
            run_export(&site, dir.path(), "  ").unwrap_err(),
            "Export path required."
        );
        assert_eq!(
            run_export(&site, dir.path(), "../outside").unwrap_err(),
            "Export path must stay inside the site directory."
        );
    }

    #[test]
    fn preview_status_uses_the_designed_sentences() {
        assert_eq!(
            preview_status(PreviewView::Closed, SystemTime::UNIX_EPOCH),
            "Preview closed · Ctrl+P"
        );
        assert_eq!(
            preview_status(PreviewView::Building, SystemTime::UNIX_EPOCH),
            "Building… (autosave)"
        );
        assert_eq!(
            preview_status(PreviewView::Stale { edits: 2 }, SystemTime::UNIX_EPOCH),
            "2 edits since last render"
        );
        assert_eq!(
            preview_status(PreviewView::Error, SystemTime::UNIX_EPOCH),
            "Preview failed · Show issue"
        );
    }
}
