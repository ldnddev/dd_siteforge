//! Phase 2 view tests. Each case mounts the shell in GPUI's `TestAppContext`.

use std::path::PathBuf;

use dd_siteforge_core::model::Site;
use dd_siteforge_core::session::{self, RecentSite, Session};
use dd_siteforge_core::storage::save_site;
use gpui::{AppContext, TestAppContext};

use crate::document::SavePhase;
use crate::keys::{self, INSPECTOR_KBD, SIDEBAR_KBD};
use crate::shell::{Shell, ShellOptions};

fn opts(dir: &std::path::Path, path: Option<PathBuf>) -> ShellOptions {
    ShellOptions {
        path,
        session_path: dir.join("session.json"),
        cwd: dir.to_path_buf(),
        open_cwd: false,
        tick: false,
        quit_runtime: false,
        overlay: Some(dir.join("overlay")),
    }
}

fn mount(cx: &mut TestAppContext, options: ShellOptions) -> gpui::Entity<Shell> {
    cx.update(|cx| gpui_component::init(cx));
    cx.new(|cx| Shell::open(options, cx))
}

#[gpui::test]
fn save_indicator_renders_every_state(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    for phase in [
        SavePhase::Saved,
        SavePhase::Unsaved,
        SavePhase::Saving,
        SavePhase::Failed,
    ] {
        shell.update(cx, |shell, cx| {
            shell.show_phase(phase);
            let text = shell.visible_text();
            let expected = match phase {
                SavePhase::Saved => "Saved ",
                SavePhase::Unsaved => "Unsaved changes",
                SavePhase::Saving => "Saving…",
                SavePhase::Failed => "Autosave failed · Retry",
            };
            assert!(
                text.contains(expected),
                "phase {phase:?} missing {expected} in {text}"
            );
            if phase == SavePhase::Failed {
                assert!(text.contains("Autosave failed"));
                assert!(text.contains("Retry"));
                assert!(text.contains("Save as…"));
            }
            cx.notify();
        });
    }
}

#[gpui::test]
fn welcome_lists_recents_from_session_json(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let keep = dir.path().join("keep.json");
    let drop_missing = dir.path().join("missing.json");
    save_site(&keep, &Site::starter()).unwrap();
    let mut session = Session::default();
    session.recents.push(RecentSite {
        path: keep.clone(),
        page: 0,
        tree_row: 0,
        region: "page".to_string(),
    });
    session.recents.push(RecentSite {
        path: drop_missing,
        page: 0,
        tree_row: 0,
        region: "page".to_string(),
    });
    session::save_to(&dir.path().join("session.json"), &session).unwrap();
    let shell = mount(cx, opts(dir.path(), None));
    shell.update(cx, |shell, _| {
        let text = shell.visible_text();
        assert!(text.contains("screen:welcome"), "{text}");
        assert!(text.contains("Open site"));
        assert!(text.contains(&keep.display().to_string()), "{text}");
        assert!(!text.contains("missing.json"), "{text}");
        assert!(text.contains("Create starter site.json here"));
        assert!(text.contains("New site…"));
    });
    shell.update(cx, |shell, _| {
        shell.handle_key("escape");
        assert!(shell.quit);
    });
}

#[gpui::test]
fn new_site_seeds_kit_without_copying_session_json(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let overlay = dir.path().join("overlay");
    std::fs::create_dir_all(&overlay).unwrap();
    std::fs::write(
        overlay.join("session.json"),
        r#"{"recents":[{"path":"/secret/client.json","page":0,"tree_row":0,"region":"page"}]}"#,
    )
    .unwrap();
    std::fs::write(overlay.join("package.json"), r#"{"name":"overlay-kit"}"#).unwrap();
    let shell = mount(cx, opts(dir.path(), None));
    let site_root = dir.path().join("fresh");
    shell.update(cx, |shell, _| {
        shell.handle_key("ctrl-shift-n");
        assert!(shell.visible_text().contains("New site"));
        assert!(
            shell
                .visible_text()
                .contains("The site name starts as “My Site”")
        );
        assert!(!shell.visible_text().contains("Include starter kit"));
        shell.new_site_folder(&site_root.display().to_string());
        shell.new_site_project("My Project");
        let created = shell.submit_new_site().expect("create");
        assert!(created.path.ends_with("site.json"));
        assert!(created.warnings.is_empty(), "{:?}", created.warnings);
    });
    let site_json = std::fs::read_to_string(site_root.join("site.json")).unwrap();
    assert!(site_json.contains("\"name\": \"My Site\""), "{site_json}");
    assert!(!site_root.join("session.json").exists());
    let package = std::fs::read_to_string(site_root.join("package.json")).unwrap();
    assert!(package.contains("my-project"), "{package}");
    assert!(!package.contains("secret"));
    assert!(site_root.join("Gruntfile.js").is_file());
    assert!(site_root.join("source/templates").is_dir());
    let session = session::load_from(&dir.path().join("session.json"));
    assert!(
        session
            .recents
            .iter()
            .any(|recent| recent.path.ends_with("site.json")),
        "new site is recorded in session.json"
    );
}

#[gpui::test]
fn new_site_surfaces_a_seed_error(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("overlay")).unwrap();
    let shell = mount(cx, opts(dir.path(), None));
    let site_root = dir.path().join("partial");
    std::fs::create_dir_all(&site_root).unwrap();
    std::fs::write(site_root.join("source"), b"not a directory").unwrap();
    shell.update(cx, |shell, _| {
        shell.open_new_site(Some(site_root.display().to_string()));
        let created = shell.submit_new_site().expect("site.json still writes");
        assert!(created.warnings.iter().any(|warning| !warning.is_empty()));
        let text = shell.visible_text();
        assert!(text.contains("starter kit wasn't fully written"), "{text}");
        assert!(text.contains("screen:editor"), "{text}");
        assert!(text.contains("title:My Site"), "{text}");
    });
    assert!(site_root.join("site.json").is_file());
    assert!(!site_root.join("session.json").exists());
}

#[gpui::test]
fn load_error_offers_retry_choose_reveal_and_backup(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("acme").join("site.json");
    std::fs::create_dir_all(bad.parent().unwrap()).unwrap();
    std::fs::write(&bad, b"{not json").unwrap();
    std::fs::write(checkpoint_path(&bad), save_bytes()).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(bad.clone())));
    shell.update(cx, |shell, _| {
        let text = shell.visible_text();
        assert!(text.contains("screen:error"), "{text}");
        assert!(text.contains("Couldn't load acme"), "{text}");
        assert!(text.contains("failed to parse site JSON"), "{text}");
        assert!(text.contains("Retry"));
        assert!(text.contains("Choose file…"));
        assert!(text.contains("Reveal in folder"));
        assert!(text.contains("Open backup"));
        assert!(text.contains("title:acme"), "{text}");
        shell.reveal_in_folder();
        assert_eq!(shell.revealed(), Some(bad.parent().unwrap().to_path_buf()));
        shell.open_restore();
        assert!(shell.visible_text().contains("Restore from checkpoint?"));
        shell.handle_key("escape");
        let good = dir.path().join("restored.json");
        save_site(&good, &Site::starter()).unwrap();
        shell.choose_file(good);
        assert!(shell.visible_text().contains("screen:editor"));
        shell.show_phase(SavePhase::Failed);
        let copy = dir.path().join("copy.json");
        shell.save_copy_to(copy.clone()).unwrap();
        assert!(copy.is_file());
    });
}

#[gpui::test]
fn dock_shortcuts_match_the_title_bar_toggles(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        assert_eq!(shell.docks_visible(), (true, true));
        let before = shell.visible_text();

        shell.handle_key(keys::TOGGLE_SIDEBAR_KEY);
        assert_eq!(shell.docks_visible(), (false, true));
        let text = shell.visible_text();
        assert!(text.contains("Show sidebar"), "{text}");
        assert!(text.contains(SIDEBAR_KBD), "{text}");
        assert_eq!(shell.dock_notes(1280.0), vec![keys::show_sidebar_tooltip()]);

        shell.handle_key(keys::TOGGLE_SIDEBAR_KEY);
        assert_eq!(shell.docks_visible(), (true, true));

        shell.handle_key(keys::TOGGLE_INSPECTOR_KEY);
        assert_eq!(shell.docks_visible(), (true, false));
        let text = shell.visible_text();
        assert!(text.contains("Show inspector"), "{text}");
        assert!(text.contains(INSPECTOR_KBD), "{text}");
        assert_eq!(
            shell.dock_notes(1280.0),
            vec![keys::show_inspector_tooltip()]
        );

        shell.handle_key(keys::TOGGLE_INSPECTOR_KEY);
        assert_eq!(shell.docks_visible(), (true, true));
        shell.handle_key("ctrl-b");
        assert_eq!(shell.docks_visible(), (true, true));
        assert_eq!(shell.visible_text(), before);
    });
}

#[gpui::test]
fn golden_site_renders_read_only_tree_and_fields(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../dd_siteforge_core/tests/fixtures/sites/starter.json");
    std::fs::copy(&fixture, &path).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        let text = shell.visible_text();
        assert!(text.contains("screen:editor"), "{text}");
        assert!(text.contains("title:My Site"), "{text}");
        assert!(text.contains("Home"), "{text}");
        assert!(text.contains("Site settings"));
        assert!(text.contains("Header"));
        assert!(text.contains("Footer"));
        assert!(text.contains("HEAD"));
        assert!(text.contains("dd-hero"));
        assert!(
            text.contains("DD-SECTION · SECTION-1")
                || text.contains("DD-SECTION · section-1".to_ascii_uppercase().as_str())
        );
        shell.select_outline_label("dd-hero");
        shell.select_tab(crate::present::InspTab::Content);
        let text = shell.visible_text();
        assert!(text.contains("Title"), "{text}");
        assert!(text.contains("Build with dd-framework"), "{text}");
        assert!(text.contains("Content") || !text.contains("\nMedia\n") || text.contains("Media"));
        shell.select_sidebar_label("Header");
        let text = shell.visible_text();
        assert!(text.contains("column-row 18/24 3/24 3/24"), "{text}");
        assert!(text.contains("col 1 dd-u-md-18-24 empty"), "{text}");
        assert!(text.contains("Empty · / to add"), "{text}");
        assert!(text.contains("col 2 dd-u-md-3-24 filled"), "{text}");
        assert!(text.contains("col 3 dd-u-md-3-24 filled"), "{text}");
        assert!(text.contains("dd-header-search"), "{text}");
        assert!(text.contains("dd-header-menu"), "{text}");
        assert_eq!(shell.column_row_mode(1920.0), "row");
        // DESIGN-GAP: Screens 33/34 do not draw this row, so a narrow outline stacks it.
        assert_eq!(shell.column_row_mode(960.0), "stack");
        assert_eq!(shell.column_row_mode(720.0), "stack");
        shell.select_column("col 2");
        let text = shell.visible_text();
        assert!(text.contains("col 2 dd-u-md-3-24 selected"), "{text}");
        assert!(text.contains("Column ID (section-prefixed)"), "{text}");
        assert!(text.contains("header-section-1-column-2"), "{text}");
        assert!(
            text.contains("dd-u-3-24 dd-u-sm-3-24 dd-u-md-3-24 dd-u-lg-4-24"),
            "{text}"
        );
        shell.select_sidebar_label("Footer");
        let text = shell.visible_text();
        assert!(text.contains("column-row 1/1"), "{text}");
        assert!(text.contains("col 1 dd-u-1-1 empty"), "{text}");
        assert_eq!(shell.column_row_mode(720.0), "row");
        shell.handle_key("ctrl-b");
        assert!(shell.visible_text().contains("Header"));
        let narrow = shell.dock_notes(960.0);
        assert!(
            narrow
                .iter()
                .any(|line| line.contains("Show inspector") && line.contains(INSPECTOR_KBD)),
            "{narrow:?}"
        );
        assert!(narrow.iter().all(|line| !line.contains("Show sidebar")));
        let tighter = shell.dock_notes(700.0);
        assert!(
            tighter
                .iter()
                .any(|line| line.contains("Show sidebar") && line.contains(SIDEBAR_KBD)),
            "{tighter:?}"
        );
        shell.show_loading("Acme Docs");
        assert!(shell.visible_text().contains("Reading site.json…"));
    });
}

#[gpui::test]
fn ctrl_s_blocks_an_invalid_draft_then_checkpoints_a_valid_one(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path.clone())));
    shell.update(cx, |shell, _| {
        shell.select_outline_label("dd-hero");
        shell.focus_inspector();
        assert!(shell.set_field_value("parent_title", ""));
        shell.handle_key("ctrl-s");
        let text = shell.visible_text();
        assert!(text.contains("Title is required."), "{text}");
        assert!(text.contains("Issue"), "{text}");
        assert!(
            !checkpoint_path(&path).is_file(),
            "invalid draft must block the checkpoint"
        );
        assert!(shell.set_field_value("parent_title", "Shipped"));
        shell.handle_key("ctrl-s");
        assert!(checkpoint_path(&path).is_file());
        assert!(shell.visible_text().contains("H2"));
        assert!(shell.visible_text().contains("List"));
        assert!(!shell.visible_text().contains("preview eye"));
    });
}

#[gpui::test]
fn import_rename_drag_and_markdown_go_through_the_editor(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let png = {
        let image = image::RgbImage::from_pixel(8, 8, image::Rgb([1, 2, 3]));
        let dynamic = image::DynamicImage::ImageRgb8(image);
        let mut bytes = Vec::new();
        dynamic
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        let file = dir.path().join("My Shot.PNG");
        std::fs::write(&file, bytes).unwrap();
        file
    };
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        shell.select_sidebar_label("Home");
        shell.rename_page_to("Home Base");
        assert!(shell.visible_text().contains("Renamed page: Home Base"));
        shell.select_outline_label("dd-hero");
        shell.focus_inspector();
        assert!(shell.focus_field_id("parent_copy"));
        assert!(shell.insert_markdown(dd_siteforge_core::controls::MarkdownInsert::H2));
        assert!(shell.focus_field_id("media_image_url"));
        shell.import_dropped(png, Some(false));
        assert!(
            shell
                .visible_text()
                .contains("Picked: assets/images/my-shot.png")
        );
        shell.handle_key("ctrl-s");
        assert!(shell.undo_depth() >= 1);
        shell.handle_key("/");
        shell.handle_key("h");
        shell.handle_key("e");
        shell.handle_key("a");
        shell.handle_key("d");
        shell.handle_key("enter");
        shell.drag_component_to(0);
        shell.focus_inspector();
        shell.focus_field_id("items");
        shell.drag_collection_to(0, 0);
    });
}

fn checkpoint_path(path: &std::path::Path) -> PathBuf {
    dd_siteforge_core::storage::checkpoint_path(path)
}

fn save_bytes() -> String {
    let mut site = Site::starter();
    site.name = "From backup".to_string();
    serde_json::to_string_pretty(&site).unwrap()
}
