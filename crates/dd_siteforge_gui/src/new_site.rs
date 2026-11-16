//! New site sheet. Same writes as the TUI picker: folder, starter `site.json`,
//! kit files only, then templates. The site name stays "My Site" (J8).

use std::path::{Path, PathBuf};

use dd_siteforge_core::model::Site;
use dd_siteforge_core::scaffold::{self, SeedOpts};
use dd_siteforge_core::session;
use dd_siteforge_core::storage::save_site;
use dd_siteforge_core::templates;

#[derive(Debug)]
pub struct CreatedSite {
    pub path: PathBuf,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

/// `overlay` is the config dir the TUI would pass to `seed_scaffold`. `None`
/// uses `overlay_if_present()`. Kit paths are copied; `session.json` is not.
pub fn create_new_site(
    folder: &Path,
    project_name: &str,
    session_path: &Path,
    overlay: Option<&Path>,
) -> Result<CreatedSite, String> {
    if folder.as_os_str().is_empty() {
        return Err("Choose a folder.".to_string());
    }
    let root = site_root(folder);
    let site_path = root.join("site.json");
    if site_path.exists() {
        return Err("A site.json already exists here.".to_string());
    }
    std::fs::create_dir_all(&root).map_err(|err| err.to_string())?;

    let site = Site::starter();
    save_site(&site_path, &site)
        .map_err(|_| format!("could not write starter site to '{}'", site_path.display()))?;

    let slug = project_slug(&root, project_name);
    let overlay_buf = match overlay {
        Some(path) => Some(path.to_path_buf()),
        None => scaffold::overlay_if_present(),
    };
    let mut warnings = Vec::new();
    let mut notes = Vec::new();
    match scaffold::seed_scaffold(
        &root,
        SeedOpts {
            force: false,
            project_name: Some(slug.as_str()),
            overlay: overlay_buf.as_deref(),
        },
    ) {
        Ok(report) => {
            if !report.skipped.is_empty() {
                notes.push(format!("Kept {} existing files.", report.skipped.len()));
            }
        }
        Err(err) => warnings.push(err.to_string()),
    }
    match templates::seed_templates(&root, false, None) {
        Ok(report) => {
            if !report.stale.is_empty() {
                notes.push(format!("Kept {} existing files.", report.stale.len()));
            }
        }
        Err(err) => warnings.push(err.to_string()),
    }

    let mut session = session::load_from(session_path);
    session.record(&site_path, 0, 0, "page");
    if let Err(err) = session::save_to(session_path, &session) {
        warnings.push(format!("could not write session.json: {err}"));
    }

    Ok(CreatedSite {
        path: site_path,
        warnings,
        notes,
    })
}

pub fn project_slug(folder: &Path, project_name: &str) -> String {
    let raw = if project_name.trim().is_empty() {
        scaffold::dir_hint(folder)
    } else {
        project_name.trim().to_string()
    };
    scaffold::slugify_project_name(&raw)
}

fn site_root(folder: &Path) -> PathBuf {
    if folder
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "site.json")
    {
        folder.parent().unwrap_or(folder).to_path_buf()
    } else {
        folder.to_path_buf()
    }
}

pub fn remember(session_path: &Path, site_path: &Path, page: usize, region: &str) {
    let mut session = session::load_from(session_path);
    session.record(site_path, page, 0, region);
    let _ = session::save_to(session_path, &session);
}
