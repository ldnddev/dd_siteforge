//! Recent sites and last selection, stored under the user config dir.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const MAX_RECENTS: usize = 10;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    #[serde(default)]
    pub recents: Vec<RecentSite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentSite {
    pub path: PathBuf,
    #[serde(default)]
    pub page: usize,
    #[serde(default)]
    pub tree_row: usize,
    /// `page`, `header`, `footer`, or `site`. Default `page` for older files.
    #[serde(default = "default_region")]
    pub region: String,
}

fn default_region() -> String {
    "page".to_string()
}

pub fn session_file() -> PathBuf {
    crate::scaffold::config_scaffold_dir().join("session.json")
}

pub fn load() -> Session {
    load_from(&session_file())
}

pub fn load_from(path: &Path) -> Session {
    let Ok(bytes) = std::fs::read(path) else {
        return Session::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

pub fn save(session: &Session) -> std::io::Result<()> {
    save_to(&session_file(), session)
}

pub fn save_to(path: &Path, session: &Session) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(session).unwrap_or_else(|_| b"{}".to_vec());
    std::fs::write(path, json)
}

impl Session {
    pub fn existing_recents(&self) -> Vec<RecentSite> {
        self.recents
            .iter()
            .filter(|r| r.path.is_file())
            .cloned()
            .collect()
    }

    pub fn selection_for(&self, path: &Path) -> (usize, usize, String) {
        let canon = canonicalize_or(path);
        self.recents
            .iter()
            .find(|r| canonicalize_or(&r.path) == canon)
            .map(|r| (r.page, r.tree_row, r.region.clone()))
            .unwrap_or_else(|| (0, 0, default_region()))
    }

    pub fn record(&mut self, path: &Path, page: usize, tree_row: usize, region: &str) {
        let path = canonicalize_or(path);
        self.recents.retain(|r| canonicalize_or(&r.path) != path);
        self.recents.insert(
            0,
            RecentSite {
                path,
                page,
                tree_row,
                region: region.to_string(),
            },
        );
        if self.recents.len() > MAX_RECENTS {
            self.recents.truncate(MAX_RECENTS);
        }
    }
}

fn canonicalize_or(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_moves_path_to_front_and_caps() {
        let dir = std::env::temp_dir().join(format!(
            "dd_session_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.json");
        let b = dir.join("b.json");
        std::fs::write(&a, "{}").unwrap();
        std::fs::write(&b, "{}").unwrap();
        let mut s = Session::default();
        s.record(&a, 1, 2, "header");
        s.record(&b, 0, 0, "page");
        s.record(&a, 3, 4, "footer");
        assert_eq!(s.recents.len(), 2);
        assert_eq!(s.recents[0].page, 3);
        assert_eq!(s.recents[0].tree_row, 4);
        assert_eq!(s.recents[0].region, "footer");
        let file = dir.join("session.json");
        save_to(&file, &s).unwrap();
        let loaded = load_from(&file);
        assert_eq!(loaded.selection_for(&a), (3, 4, "footer".into()));
        std::fs::remove_dir_all(&dir).ok();
    }
}
