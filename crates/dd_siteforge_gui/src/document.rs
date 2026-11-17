//! site.json load, autosave, and the Ctrl+S checkpoint.
//!
//! Every write goes through `dd_siteforge_core::storage`. GUI settings are not
//! part of the site model, so the bytes match a TUI save of the same `Site`.

use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use dd_siteforge_core::model::Site;
use dd_siteforge_core::storage::{
    ContentHash, SiteSave, checkpoint_overwriting, checkpoint_path, checkpoint_site,
    load_site_with_hash, save_site, save_site_checked,
};

pub const AUTOSAVE_AFTER: Duration = Duration::from_secs(2);
/// One persistent failure, then a pause, instead of a toast on every tick.
pub const AUTOSAVE_BACKOFF: Duration = Duration::from_secs(8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SavePhase {
    Saved,
    Unsaved,
    Saving,
    Failed,
}

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
    pub save_error: Option<String>,
    pub saving: bool,
    pub saved_at: SystemTime,
    pub retry_after: Option<Instant>,
    /// Edits since the document was opened. Preview uses this for the stale count.
    pub edits: u64,
}

impl Document {
    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        let (site, hash) = load_site_with_hash(&path)?;
        let saved_at = file_mtime(&path);
        Ok(Self {
            site,
            path,
            hash: Some(hash),
            dirty: false,
            dirty_since: None,
            conflict: false,
            notice: None,
            save_error: None,
            saving: false,
            saved_at,
            retry_after: None,
            edits: 0,
        })
    }

    pub fn phase(&self) -> SavePhase {
        if self.saving {
            SavePhase::Saving
        } else if self.save_error.is_some() {
            SavePhase::Failed
        } else if self.dirty {
            SavePhase::Unsaved
        } else {
            SavePhase::Saved
        }
    }

    /// Status-bar copy for the four designed states.
    pub fn indicator_text(&self) -> String {
        match self.phase() {
            SavePhase::Saved => format!("Saved {}", clock_label(self.saved_at)),
            SavePhase::Unsaved => "Unsaved changes".to_string(),
            SavePhase::Saving => "Saving…".to_string(),
            SavePhase::Failed => "Autosave failed · Retry".to_string(),
        }
    }

    pub fn mark_dirty(&mut self) {
        if !self.dirty {
            self.dirty_since = Some(Instant::now());
        }
        self.dirty = true;
        self.saving = false;
        self.edits = self.edits.saturating_add(1);
    }

    /// A bookkeeping write, such as remembering `export_dir`, is not an edit.
    pub fn mark_dirty_quiet(&mut self) {
        if !self.dirty {
            self.dirty_since = Some(Instant::now());
        }
        self.dirty = true;
        self.saving = false;
    }
    pub fn show_phase(&mut self, phase: SavePhase) {
        match phase {
            SavePhase::Saved => {
                self.dirty = false;
                self.dirty_since = None;
                self.saving = false;
                self.save_error = None;
                self.saved_at = SystemTime::now();
            }
            SavePhase::Unsaved => {
                self.dirty = true;
                self.dirty_since = Some(Instant::now());
                self.saving = false;
                self.save_error = None;
            }
            SavePhase::Saving => {
                self.saving = true;
                self.save_error = None;
            }
            SavePhase::Failed => {
                self.dirty = true;
                self.saving = false;
                self.save_error = Some("disk full".to_string());
            }
        }
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
        if let Some(after) = self.retry_after {
            if self.save_error.is_some() && now < after {
                return false;
            }
        }
        self.saving = true;
        match save_site_checked(&self.path, &self.site, self.hash) {
            Ok(SiteSave::Written { hash }) => {
                self.saving = false;
                self.hash = Some(hash);
                self.dirty = false;
                self.dirty_since = None;
                self.save_error = None;
                self.retry_after = None;
                self.saved_at = SystemTime::now();
                self.notice = Some("Saved".to_string());
                true
            }
            Ok(SiteSave::Conflict { .. }) => {
                self.saving = false;
                self.conflict = true;
                self.notice =
                    Some("Changed on disk. Not overwritten. Reload, or keep yours.".to_string());
                true
            }
            Err(err) => {
                self.saving = false;
                self.save_error = Some(err.to_string());
                self.retry_after = Some(now + AUTOSAVE_BACKOFF);
                self.notice = Some(format!("Autosave failed: {err}"));
                true
            }
        }
    }

    /// Banner Retry. Clears the backoff and writes now.
    pub fn retry_save(&mut self) -> bool {
        self.retry_after = None;
        self.dirty_since = Some(Instant::now() - AUTOSAVE_AFTER);
        if !self.dirty {
            self.dirty = true;
        }
        self.autosave(Instant::now())
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
        self.saving = false;
        self.save_error = None;
        self.retry_after = None;
        self.saved_at = SystemTime::now();
        self.notice = Some(notice);
    }

    /// Replace `site.json` with the checkpoint bytes, parsed and written through
    /// `save_site` so the atomic rename stays in core.
    pub fn restore_backup(&mut self) -> anyhow::Result<()> {
        let backup = checkpoint_path(&self.path);
        let text = std::fs::read_to_string(&backup)
            .map_err(|err| anyhow::anyhow!("failed to read site JSON: {err}"))?;
        let site: Site = serde_json::from_str(&text)
            .map_err(|err| anyhow::anyhow!("failed to parse site JSON: {err}"))?;
        save_site(&self.path, &site)?;
        let (site, hash) = load_site_with_hash(&self.path)?;
        self.site = site;
        self.note_saved(
            hash,
            format!("Restored {} from checkpoint.", self.path.display()),
        );
        Ok(())
    }
}

fn file_mtime(path: &std::path::Path) -> SystemTime {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .unwrap_or_else(|_| SystemTime::now())
}

pub fn clock_label(time: SystemTime) -> String {
    let local = chrono::DateTime::<chrono::Local>::from(time);
    let hour = local.format("%I").to_string();
    let hour = hour.trim_start_matches('0');
    let hour = if hour.is_empty() { "0" } else { hour };
    format!("{hour}:{} {}", local.format("%M"), local.format("%p"))
}

pub fn backup_stamp(time: SystemTime) -> String {
    let local = chrono::DateTime::<chrono::Local>::from(time);
    let day = local.format("%e").to_string();
    format!(
        "{} {}, {}",
        local.format("%b"),
        day.trim(),
        clock_label(time)
    )
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
