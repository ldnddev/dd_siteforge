//! Golden site.json files and one starter export, captured before the core split.
//!
//! Load → save must reproduce each fixture byte for byte. Export of the
//! starter site must match the captured `web/` files. `sitemap.xml`
//! `<lastmod>` is the UTC date of the export, so that element is ignored.

use std::fs;
use std::path::{Path, PathBuf};

use dd_siteforge_core::export::export_site;
use dd_siteforge_core::model::Site;
use dd_siteforge_core::storage::{load_site, save_site};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn site_fixtures() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(fixtures().join("sites"))
        .expect("sites fixtures")
        .map(|entry| entry.expect("fixture entry").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    paths.sort();
    assert!(
        paths.len() >= 7,
        "expected the starter, all-kinds, deep-nest, header-footer, unicode, and both optional fixtures"
    );
    paths
}

#[test]
fn golden_sites_round_trip_byte_for_byte() {
    let scratch =
        std::env::temp_dir().join(format!("dd_siteforge_roundtrip_{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();
    for path in site_fixtures() {
        let original = fs::read(&path).unwrap();
        let site = load_site(&path).unwrap_or_else(|err| {
            panic!("load {}: {err:#}", path.display());
        });
        let saved = scratch.join(path.file_name().unwrap());
        save_site(&saved, &site).unwrap();
        let written = fs::read(&saved).unwrap();
        assert_eq!(
            original,
            written,
            "load/save changed {}",
            path.file_name().unwrap().to_string_lossy()
        );
    }
    let _ = fs::remove_dir_all(&scratch);
}

#[test]
fn starter_constructor_matches_golden_bytes() {
    let golden = fs::read(fixtures().join("sites/starter.json")).unwrap();
    let scratch = std::env::temp_dir().join(format!("dd_siteforge_starter_{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();
    let path = scratch.join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    assert_eq!(golden, fs::read(&path).unwrap());
    let _ = fs::remove_dir_all(&scratch);
}

#[test]
fn starter_export_matches_pre_split_output() {
    let expected = fixtures().join("export-starter");
    let actual = std::env::temp_dir().join(format!("dd_siteforge_export_{}", std::process::id()));
    let _ = fs::remove_dir_all(&actual);
    export_site(&Site::starter(), &actual, None).unwrap();
    assert_trees_equal(&expected, &actual);
    let _ = fs::remove_dir_all(&actual);
}

fn assert_trees_equal(expected: &Path, actual: &Path) {
    let mut expected_files = list_files(expected);
    let mut actual_files = list_files(actual);
    expected_files.sort();
    actual_files.sort();
    assert_eq!(
        expected_files, actual_files,
        "export file list changed\nexpected: {expected_files:?}\nactual: {actual_files:?}"
    );
    for rel in expected_files {
        let left = normalize(&rel, &fs::read(expected.join(&rel)).unwrap());
        let right = normalize(&rel, &fs::read(actual.join(&rel)).unwrap());
        assert_eq!(left, right, "export bytes changed for {rel}");
    }
}

fn list_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
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

/// Drop the sitemap `<lastmod>` date. Every other byte is compared.
fn normalize(rel: &str, bytes: &[u8]) -> Vec<u8> {
    if rel != "sitemap.xml" {
        return bytes.to_vec();
    }
    let text = String::from_utf8_lossy(bytes);
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("<lastmod>") && trimmed.contains("</lastmod>") {
            let indent = &line[..line.len() - trimmed.len()];
            let newline = if line.ends_with('\n') { "\n" } else { "" };
            out.push_str(indent);
            out.push_str("<lastmod>YYYY-MM-DD</lastmod>");
            out.push_str(newline);
        } else {
            out.push_str(line);
        }
    }
    out.into_bytes()
}
