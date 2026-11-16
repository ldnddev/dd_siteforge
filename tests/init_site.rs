//! CLI `init-site` against a sandbox config dir.
//!
//! The house overlay lives in `$XDG_CONFIG_HOME/ldnddev/dd_siteforge/` next to
//! `session.json`. A new site must pick up kit customizations only, and the
//! destination folder does not have to exist yet.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn tmp(prefix: &str) -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let p = std::env::temp_dir().join(format!("dd_init_{prefix}_{n}"));
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn init_site_creates_missing_folder_and_skips_config_files() {
    let base = tmp("xdg");
    let config = base.join("ldnddev").join("dd_siteforge");
    fs::create_dir_all(config.join("source/js")).unwrap();
    fs::write(
        config.join("session.json"),
        "{\"recents\":[{\"path\":\"/secret/client\"}]}",
    )
    .unwrap();
    fs::write(config.join("notes.txt"), "junk").unwrap();
    fs::write(config.join("Gruntfile.js"), "/* house */\n").unwrap();
    fs::write(config.join("source/js/extra.js"), "console.log('x');\n").unwrap();

    let site_json = base.join("brand-new").join("site.json");
    assert!(!site_json.parent().unwrap().exists());

    let output = Command::new(env!("CARGO_BIN_EXE_dd_siteforge"))
        .args([
            "init-site",
            site_json.to_str().unwrap(),
            "--name",
            "acme-site",
        ])
        .env("XDG_CONFIG_HOME", &base)
        .output()
        .expect("spawn init-site");
    assert!(
        output.status.success(),
        "init-site failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let root = site_json.parent().unwrap();
    assert!(site_json.is_file(), "site.json was not created");
    assert_eq!(
        fs::read_to_string(root.join("Gruntfile.js")).unwrap(),
        "/* house */\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("source/js/extra.js")).unwrap(),
        "console.log('x');\n"
    );
    let pkg = fs::read_to_string(root.join("package.json")).unwrap();
    assert!(
        pkg.contains("\"name\": \"acme-site\""),
        "package.json was not stamped:\n{pkg}"
    );
    assert!(root.join(".lando.yml").is_file());
    assert!(root.join(".ddev/config.yaml").is_file());
    assert!(!root.join("session.json").exists());
    assert!(!root.join("notes.txt").exists());
    let site = fs::read_to_string(&site_json).unwrap();
    assert!(!site.contains("/secret/client"), "{site}");

    fs::remove_dir_all(&base).ok();
}
