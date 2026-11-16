//! site.json load, autosave, and the Ctrl+S checkpoint.
//!
//! Every write goes through `dd_siteforge_core::storage`. GUI settings are not
//! part of the site model, so the bytes match a TUI save of the same `Site`.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use dd_siteforge_core::model::Site;
use dd_siteforge_core::storage::{
    ContentHash, SiteSave, checkpoint_overwriting, checkpoint_site, load_site_with_hash,
    save_site_checked,
};

pub const AUTOSAVE_AFTER: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub struct Document {
    pub site: Site,
    pub path: PathBuf,
    pub hash: Option<ContentHash>,
    pub dirty: bool,
    pub dirty_since: Option<Instant>,
    /// The file on disk is not the one we last read. Autosave will not write.
    pub conflict: bool,
    pub notice: Option<String>,
}

impl Document {
    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        let (site, hash) = load_site_with_hash(&path)?;
        Ok(Self {
            site,
            path,
            hash: Some(hash),
            dirty: false,
            dirty_since: None,
            conflict: false,
            notice: None,
        })
    }

    /// Phase 3's inspector calls this. The autosave tests cover it until then.
    #[allow(dead_code)]
    pub fn edit(&mut self, site: Site, now: Instant) {
        self.site = site;
        if !self.dirty {
            self.dirty_since = Some(now);
        }
        self.dirty = true;
    }

    pub fn autosave(&mut self, now: Instant) -> bool {
        if self.conflict || !self.dirty {
            return false;
        }
        let Some(since) = self.dirty_since else {
            self.dirty_since = Some(now);
            return false;
        };
        if now.duration_since(since) < AUTOSAVE_AFTER {
            return false;
        }
        match save_site_checked(&self.path, &self.site, self.hash) {
            Ok(SiteSave::Written { hash }) => {
                self.hash = Some(hash);
                self.dirty = false;
                self.dirty_since = None;
                self.notice = Some("Saved".to_string());
                true
            }
            Ok(SiteSave::Conflict { .. }) => {
                self.conflict = true;
                self.notice =
                    Some("Changed on disk. Not overwritten. Reload, or keep yours.".to_string());
                true
            }
            Err(err) => {
                self.notice = Some(format!("Autosave failed: {err}"));
                true
            }
        }
    }

    /// Ctrl+S. The first press while the file is in conflict keeps ours.
    pub fn checkpoint(&mut self) -> bool {
        if self.conflict {
            return self.keep_mine();
        }
        match checkpoint_site(&self.path, &self.site, self.hash) {
            Ok(SiteSave::Written { hash }) => {
                self.note_saved(hash, format!("Saved {}", self.path.display()));
                true
            }
            Ok(SiteSave::Conflict { .. }) => {
                self.conflict = true;
                self.notice =
                    Some("Changed on disk. Not overwritten. Reload, or keep yours.".to_string());
                true
            }
            Err(err) => {
                self.notice = Some(format!("Failed to save: {err}"));
                true
            }
        }
    }

    pub fn reload(&mut self) -> anyhow::Result<()> {
        let (site, hash) = load_site_with_hash(&self.path)?;
        self.site = site;
        self.hash = Some(hash);
        self.dirty = false;
        self.dirty_since = None;
        self.conflict = false;
        self.notice = Some(format!("Reloaded {}", self.path.display()));
        Ok(())
    }

    pub fn keep_mine(&mut self) -> bool {
        match checkpoint_overwriting(&self.path, &self.site) {
            Ok(hash) => {
                self.note_saved(hash, format!("Saved {}", self.path.display()));
                true
            }
            Err(err) => {
                self.notice = Some(format!("Failed to save: {err}"));
                true
            }
        }
    }

    fn note_saved(&mut self, hash: ContentHash, notice: String) {
        self.hash = Some(hash);
        self.dirty = false;
        self.dirty_since = None;
        self.conflict = false;
        self.notice = Some(notice);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dd_siteforge_core::storage::save_site;

    fn temp_site() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("site.json");
        save_site(&path, &Site::starter()).unwrap();
        (dir, path)
    }

    #[test]
    fn autosave_waits_two_seconds_then_matches_a_plain_save() {
        let (_dir, path) = temp_site();
        let mut doc = Document::load(path.clone()).unwrap();
        let started = Instant::now();
        let mut site = doc.site.clone();
        site.name = "From the GUI".to_string();
        doc.edit(site, started);
        assert!(!doc.autosave(started + Duration::from_millis(1_900)));
        assert!(doc.autosave(started + AUTOSAVE_AFTER));
        let gui_bytes = std::fs::read(&path).unwrap();

        let mut via_tui = Site::starter();
        via_tui.name = "From the GUI".to_string();
        let plain = _dir.path().join("plain.json");
        save_site(&plain, &via_tui).unwrap();
        assert_eq!(
            gui_bytes,
            std::fs::read(&plain).unwrap(),
            "GUI autosave bytes match the TUI writer for the same model"
        );
    }

    #[test]
    fn external_change_blocks_autosave_until_keep_or_reload() {
        let (_dir, path) = temp_site();
        let mut doc = Document::load(path.clone()).unwrap();
        std::fs::write(&path, b"{\"external\":true}").unwrap();
        let now = Instant::now();
        let mut site = doc.site.clone();
        site.name = "Mine".to_string();
        doc.edit(site, now);
        assert!(doc.autosave(now + AUTOSAVE_AFTER));
        assert!(doc.conflict);
        assert_eq!(std::fs::read(&path).unwrap(), b"{\"external\":true}");

        assert!(doc.keep_mine());
        assert!(!doc.conflict);
        let kept = std::fs::read(&path).unwrap();
        assert!(std::str::from_utf8(&kept).unwrap().contains("Mine"));
        let backup = dd_siteforge_core::storage::checkpoint_path(&path);
        assert_eq!(std::fs::read(&backup).unwrap(), kept);

        let mut other = Site::starter();
        other.name = "From disk".to_string();
        save_site(&path, &other).unwrap();
        let mut again = doc.site.clone();
        again.name = "Again".to_string();
        doc.edit(again, Instant::now());
        assert!(doc.autosave(Instant::now() + AUTOSAVE_AFTER));
        assert!(doc.conflict);
        doc.reload().unwrap();
        assert!(!doc.conflict);
        assert_eq!(doc.site.name, "From disk");
    }
}
