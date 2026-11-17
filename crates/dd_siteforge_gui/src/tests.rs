//! Phase 2 view tests. Each case mounts the shell in GPUI's `TestAppContext`.

use std::path::PathBuf;

use dd_siteforge_core::model::{Page, PageTemplate, Site};
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
        settings_file: dir.join("gui.json"),
        wcag_bin: dir.join("dd_wcag"),
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
        shell.drain_background();
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
        let text = shell.visible_text();
        assert!(text.contains("Running `npx grunt build`…"), "{text}");
        shell.handle_key("ctrl-b");
        assert!(shell.visible_text().contains("Build already running."));
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
        shell.drain_background();
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

#[gpui::test]
fn issues_panel_lists_and_jumps_to_the_field(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    let mut site = Site::starter();
    let dd_siteforge_core::model::PageNode::Hero(hero) = &mut site.pages[0].nodes[0] else {
        panic!("starter hero");
    };
    hero.parent_title.clear();
    save_site(&path, &site).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        shell.handle_key("ctrl-e");
        let blocked = shell.visible_text();
        assert!(blocked.contains("hero is missing Title."), "{blocked}");
        assert!(blocked.contains("Go to field"), "{blocked}");
        assert!(shell.jump_to_issue(0));
        assert_eq!(shell.focused_field_id().as_deref(), Some("parent_title"));
        shell.handle_key("f4");
        let health = shell.visible_text();
        assert!(health.contains("Page health ·"), "{health}");
        shell.handle_key(keys::REDO_KEY);
        assert!(
            !shell.visible_text().contains("Building… (autosave)"),
            "Ctrl+R in the main window stays redo"
        );
        shell.handle_key("f5");
        assert!(
            shell.visible_text().contains("hero is missing Title."),
            "preview with errors opens the issues list"
        );
    });
}

#[gpui::test]
fn gui_export_from_the_shell_matches_core(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("site");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("site.json");
    let site = Site::starter();
    save_site(&path, &site).unwrap();
    for _ in 0..8 {
        let issues = dd_siteforge_core::validate::validate_site_with_root(&site, Some(&root));
        if issues.is_empty() {
            break;
        }
        for issue in issues {
            let Some(rel) = issue.message.split("expected at ").nth(1) else {
                continue;
            };
            let rel = rel.trim_end_matches(')').trim();
            let file = root.join(rel);
            if let Some(parent) = file.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(file, b"img").unwrap();
        }
    }
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        shell.handle_key("ctrl-e");
        assert!(shell.visible_text().contains("Export — output directory"));
        shell.handle_key("enter");
        for _ in 0..100 {
            shell.poll_jobs();
            if shell.visible_text().contains("Exported ") {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let text = shell.visible_text();
        assert!(text.contains("Exported "), "{text}");
        assert!(text.contains("Reveal"), "{text}");
        assert!(root.join("web").join("index.html").is_file());
    });
}

#[gpui::test]
fn palette_holds_init_and_f2_opens_appearance(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        let text = shell.visible_text();
        assert!(text.contains("title-actions:Export,Preview"), "{text}");
        assert!(!text.contains("Init site kit"), "{text}");
        shell.handle_key("ctrl-k");
        let text = shell.visible_text();
        assert!(text.contains("Command palette"), "{text}");
        assert!(text.contains("Init site kit"), "{text}");
        assert!(text.contains("Init templates"), "{text}");
        assert!(text.contains("Find in site"), "{text}");
        shell.handle_key("escape");
        assert!(shell.visible_text().contains("Command palette cancelled."));
        shell.handle_key("f2");
        let text = shell.visible_text();
        assert!(text.contains("Settings › Appearance"), "{text}");
        assert!(text.contains("Vim keys on"), "{text}");
        assert!(text.contains("Density Comfortable"), "{text}");
        assert!(!text.contains("Theme · "), "{text}");
        shell.set_density(crate::settings::Density::Compact);
        shell.set_vim_keys(false);
        let text = shell.visible_text();
        assert!(text.contains("Density Compact"), "{text}");
        assert!(text.contains("Vim keys off"), "{text}");
        shell.handle_key("escape");
        shell.handle_key("f1");
        let text = shell.visible_text();
        assert!(text.contains("Shortcut sheet"), "{text}");
        assert!(text.contains("Add page · Ctrl N · Ctrl N"), "{text}");
        assert!(text.contains("Refresh preview · F5 · F5"), "{text}");
    });
}

#[gpui::test]
fn vim_off_types_and_vim_on_moves_and_collapses(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        shell.set_vim_keys(true);
        shell.handle_key("j");
        shell.handle_key("j");
        let open = shell.visible_text();
        shell.handle_key("h");
        let collapsed = shell.visible_text();
        assert_ne!(open, collapsed, "h collapses the section");
        shell.handle_key("l");
        shell.set_vim_keys(false);
        shell.handle_key("j");
        let text = shell.visible_text();
        assert!(
            text.contains("No pages match “j”") || text.contains("j"),
            "{text}"
        );
    });
}

#[gpui::test]
fn find_in_site_jumps_across_pages(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    let mut site = Site::starter();
    site.pages.push(Page::from_template(
        "About the orchard",
        PageTemplate::Blank,
    ));
    save_site(&path, &site).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        shell.handle_key("ctrl-f");
        for ch in ["o", "r", "c", "h", "a", "r", "d"] {
            shell.handle_key(ch);
        }
        shell.handle_key("enter");
        let text = shell.visible_text();
        assert!(text.contains("Jumped to"), "{text}");
        assert!(text.contains("About the orchard"), "{text}");
        assert!(!text.contains("Find in site\n"), "{text}");
    });
}

#[gpui::test]
fn invalid_hex_does_not_call_wcag_or_write_site_json(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let before = std::fs::read(&path).unwrap();
    let log = dir.path().join("called");
    let script = dir.path().join("dd_wcag");
    std::fs::write(
        &script,
        format!("#!/bin/sh\ntouch '{}'\nexit 0\n", log.display()),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script, perms).unwrap();
    }
    let mut options = opts(dir.path(), Some(path.clone()));
    options.wcag_bin = script;
    let shell = mount(cx, options);
    shell.update(cx, |shell, _| {
        shell.select_sidebar_label("Site settings");
        shell.focus_inspector();
        assert!(shell.set_active_field("primary_color", "red"));
        shell.handle_key("ctrl-s");
        let text = shell.visible_text();
        assert!(text.contains("invalid primary_color"), "{text}");
        assert!(!log.exists(), "dd_wcag was spawned");
    });
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[gpui::test]
fn phase6_motion_focus_and_empty_states(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let shell = mount(cx, opts(dir.path(), None));
    shell.update(cx, |shell, _| {
        let text = shell.visible_text();
        assert!(text.contains("screen:welcome"), "{text}");
        assert!(text.contains("No recent sites"), "{text}");
        shell.handle_key("ctrl-k");
        let text = shell.visible_text();
        assert!(text.contains("motion:popover-open:90ms"), "{text}");
        assert!(text.contains("focus-ring:inset"), "{text}");
        for _ in 0..4 {
            shell.handle_key("z");
        }
        assert!(
            shell.visible_text().contains("No command selected."),
            "{}",
            shell.visible_text()
        );
        shell.note_pointer();
        assert!(!shell.visible_text().contains("focus-ring:"));
    });
}

#[gpui::test]
fn reduced_motion_compact_density_and_issues_drag_range(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    crate::settings::save_to(
        &dir.path().join("gui.json"),
        &crate::settings::GuiSettings {
            density: crate::settings::Density::Compact,
            vim_keys: true,
            reduced_motion: true,
        },
    )
    .unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        let text = shell.visible_text();
        assert!(text.contains("density:compact"), "{text}");
        shell.handle_key("ctrl-shift-n");
        let text = shell.visible_text();
        assert!(text.contains("motion:sheet-open:0ms"), "{text}");
        assert!(text.contains("New site"), "{text}");
        shell.handle_key("escape");
        shell.handle_key("ctrl-b");
        shell.handle_key("ctrl-k");
        let text = shell.visible_text();
        assert!(text.contains("reload"), "{text}");
        assert!(!text.contains("reload-spin"), "{text}");
        assert!(text.contains("motion:popover-open:0ms"), "{text}");
        shell.handle_key("escape");
        shell.handle_key("f3");
        shell.set_issues_height(12.0);
        assert!(
            shell.visible_text().contains("issues-height:160"),
            "{}",
            shell.visible_text()
        );
        shell.set_issues_height(5000.0);
        assert!(
            shell.visible_text().contains("issues-height:400"),
            "{}",
            shell.visible_text()
        );
        shell.set_page_filter("zzzz-no-such-page");
        assert!(shell.visible_text().contains("No pages match"));
        shell.handle_key("ctrl-f");
        shell.handle_key("z");
        shell.handle_key("z");
        shell.handle_key("q");
        assert!(shell.visible_text().contains("No matches."));
        shell.testing_fail_preview();
        assert!(shell.visible_text().contains("Preview failed · Show issue"));
    });
}

#[gpui::test]
fn light_themes_use_the_outlined_primary_and_theme_apply_is_instant(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        let latte =
            include_str!("../../ldnddev_theme/tests/fixtures/omarchy/catppuccin-latte/colors.toml");
        shell.testing_load_theme(latte);
        let text = shell.visible_text();
        assert!(text.contains("primary:outlined"), "{text}");
        assert!(text.contains("motion:theme-apply:0ms"), "{text}");
        let rose = include_str!("../../ldnddev_theme/tests/fixtures/omarchy/rose-pine/colors.toml");
        shell.testing_load_theme(rose);
        assert!(shell.visible_text().contains("primary:outlined"));
        let retro = include_str!("../../ldnddev_theme/tests/fixtures/omarchy/retro-82/colors.toml");
        shell.testing_load_theme(retro);
        assert!(shell.visible_text().contains("primary:filled"));
    });
}

#[test]
fn a_50_page_site_loads_under_150ms() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    let mut site = Site::starter();
    while site.pages.len() < 50 {
        let n = site.pages.len();
        let mut page = Page::from_template(&format!("Page {n}"), PageTemplate::Blank);
        page.slug = format!("page-{n}");
        page.id = format!("page-{n}");
        site.pages.push(page);
    }
    save_site(&path, &site).unwrap();
    let started = std::time::Instant::now();
    let doc = crate::document::Document::load(path).unwrap();
    let open_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(doc.site.pages.len(), 50);
    eprintln!("perf 50-page open {open_ms:.2} ms");
    assert!(
        open_ms < 150.0,
        "50-page open took {open_ms:.2} ms, limit 150"
    );
}

#[gpui::test]
fn typing_a_field_stays_inside_one_frame(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        shell.select_outline_label("dd-hero");
        shell.focus_inspector();
        assert!(shell.focus_field_id("parent_copy"));
        let mut worst = 0.0_f64;
        let mut value = String::new();
        for ch in "typing stays on the frame budget".chars() {
            value.push(ch);
            let started = std::time::Instant::now();
            assert!(shell.set_field_value("parent_copy", &value));
            let _ = shell.visible_text();
            worst = worst.max(started.elapsed().as_secs_f64() * 1000.0);
        }
        eprintln!("perf typing max {worst:.3} ms");
        assert!(
            worst < 16.0,
            "a keystroke took {worst:.3} ms, frame budget is 16"
        );
    });
}

#[gpui::test]
fn welcome_mount_is_measured(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let started = std::time::Instant::now();
    let shell = mount(cx, opts(dir.path(), None));
    let ms = started.elapsed().as_secs_f64() * 1000.0;
    shell.update(cx, |shell, _| {
        assert!(shell.visible_text().contains("screen:welcome"));
    });
    eprintln!("perf welcome mount {ms:.2} ms");
    assert!(ms < 2_000.0, "welcome mount took {ms:.2} ms");
}

#[test]
fn a_save_job_runs_off_the_caller_thread() {
    let caller = std::thread::current().id();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(std::thread::current().id());
    });
    let worker = rx.recv().unwrap();
    assert_ne!(caller, worker);
}

#[test]
fn followup_go_to_field_shows_for_the_selected_or_hovered_row_with_a_target() {
    assert!(crate::shell::go_to_field_visible(true, true, false));
    assert!(crate::shell::go_to_field_visible(true, false, true));
    assert!(!crate::shell::go_to_field_visible(true, false, false));
    assert!(!crate::shell::go_to_field_visible(false, true, true));
    assert!(!crate::shell::go_to_field_visible(false, false, true));
}

#[test]
fn followup_shrink_offer_keys_are_enter_two_and_escape() {
    use crate::shell::{ShrinkChoice, shrink_choice};
    assert_eq!(shrink_choice("enter"), Some(ShrinkChoice::AsIs));
    assert_eq!(shrink_choice("2"), Some(ShrinkChoice::Shrink));
    assert_eq!(shrink_choice("escape"), Some(ShrinkChoice::Cancel));
    assert_eq!(shrink_choice("s"), None);
    assert_eq!(shrink_choice("S"), None);
}

fn satisfy_local_images(root: &std::path::Path, site: &Site) {
    for _ in 0..8 {
        let issues = dd_siteforge_core::validate::validate_site_with_root(site, Some(root));
        if issues.is_empty() {
            return;
        }
        for issue in issues {
            let Some(rel) = issue.message.split("expected at ").nth(1) else {
                continue;
            };
            let rel = rel.trim_end_matches(')').trim();
            let file = root.join(rel);
            if let Some(parent) = file.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            if !file.exists() {
                std::fs::write(file, b"img").unwrap();
            }
        }
    }
}

fn raster_file(
    dir: &std::path::Path,
    name: &str,
    width: u32,
    height: u32,
    format: image::ImageFormat,
) -> PathBuf {
    let image = image::RgbImage::from_pixel(width, height, image::Rgb([9, 8, 7]));
    let dynamic = image::DynamicImage::ImageRgb8(image);
    let mut bytes = Vec::new();
    dynamic
        .write_to(&mut std::io::Cursor::new(&mut bytes), format)
        .unwrap();
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

#[gpui::test]
fn followup_insert_footer_counts_the_context_and_keeps_data_table_interactive(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        shell.handle_key("/");
        let text = shell.visible_text();
        assert!(
            text.contains("Enter inserts after the selection · 22 kinds"),
            "{text}"
        );
        assert!(!text.contains("24 kinds"), "{text}");
        let lines = shell.lines();
        let data = lines
            .iter()
            .position(|line| line == "dd-data-table")
            .expect("data table row");
        assert_eq!(lines[data - 1], "INTERACTIVE");
        shell.handle_key("escape");
        shell.select_sidebar_label("Header");
        shell.handle_key("/");
        let text = shell.visible_text();
        assert!(
            text.contains("Enter inserts after the selection · 23 kinds"),
            "{text}"
        );
        assert!(!text.contains("24 kinds"), "{text}");
        shell.handle_key("z");
        let text = shell.visible_text();
        assert!(
            text.contains("23 kinds"),
            "search keeps the context count, got {text}"
        );
        shell.handle_key("escape");
        shell.select_sidebar_label("Footer");
        shell.handle_key("/");
        assert!(
            shell.visible_text().contains("23 kinds"),
            "{}",
            shell.visible_text()
        );
    });
}

#[gpui::test]
fn followup_go_to_field_follows_the_hovered_row(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    let mut site = Site::starter();
    site.pages[0].head.title.clear();
    let dd_siteforge_core::model::PageNode::Hero(hero) = &mut site.pages[0].nodes[0] else {
        panic!("starter hero");
    };
    hero.parent_title.clear();
    save_site(&path, &site).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        shell.handle_key("f3");
        let selected = shell.visible_text();
        assert_eq!(
            selected.matches("Go to field").count(),
            1,
            "selected row only, got {selected}"
        );
        shell.hover_issue_row(Some(1));
        let hovered = shell.visible_text();
        assert_eq!(
            hovered.matches("Go to field").count(),
            2,
            "selected and hovered, got {hovered}"
        );
        shell.hover_issue_row(None);
        assert_eq!(shell.visible_text().matches("Go to field").count(), 1);
    });
}

#[gpui::test]
fn followup_image_shrink_keys_and_copy_errors(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let large = raster_file(dir.path(), "Wide.PNG", 2561, 8, image::ImageFormat::Png);
    let gif = raster_file(dir.path(), "anim.gif", 2561, 4, image::ImageFormat::Gif);
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        shell.select_outline_label("dd-hero");
        shell.focus_inspector();
        assert!(shell.focus_field_id("media_image_url"));
        shell.import_dropped(large.clone(), None);
        let offer = shell.visible_text();
        assert!(
            offer.contains("This image is 2561 px on the long edge."),
            "{offer}"
        );
        shell.handle_key("s");
        let after_s = shell.visible_text();
        assert!(
            after_s.contains("This image is 2561 px on the long edge."),
            "s is not Copy as-is, got {after_s}"
        );
        assert!(!after_s.contains("Picked:"), "{after_s}");
        shell.handle_key("escape");
        assert!(!shell.visible_text().contains("This image is 2561 px"));
        shell.focus_field_id("media_image_url");
        shell.import_dropped(gif, None);
        let copied = shell.visible_text();
        assert!(
            copied.contains("Picked: assets/images/anim.gif"),
            "gif copies as-is, got {copied}"
        );
        assert!(!copied.contains("This image is"), "{copied}");
        shell.focus_field_id("media_image_url");
        shell.import_dropped(dir.path().join("missing.png"), None);
        let missing = shell.visible_text();
        assert!(missing.contains("Copy failed:"), "{missing}");
        assert!(!missing.contains("Source folder not found:"), "{missing}");
    });
    let images = dir.path().join("source").join("images");
    if images.exists() {
        std::fs::remove_dir_all(&images).unwrap();
    }
    std::fs::create_dir_all(images.parent().unwrap()).unwrap();
    std::fs::write(&images, b"not-a-directory").unwrap();
    let small = raster_file(dir.path(), "small.png", 8, 8, image::ImageFormat::Png);
    shell.update(cx, |shell, _| {
        shell.focus_field_id("media_image_url");
        shell.import_dropped(small, Some(false));
        let failed = shell.visible_text();
        assert!(failed.contains("Copy failed:"), "{failed}");
        assert!(!failed.contains("Save failed:"), "{failed}");
    });
}

#[gpui::test]
fn followup_export_progress_is_the_toast_stack_card(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("site");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("site.json");
    let site = Site::starter();
    save_site(&path, &site).unwrap();
    satisfy_local_images(&root, &site);
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        shell.handle_key("ctrl-e");
        shell.handle_key("enter");
        let running = shell.visible_text();
        assert!(running.contains("Export run"), "{running}");
        assert!(running.contains("export-card:360"), "{running}");
        assert!(
            running.contains("Exporting ") && running.contains("page(s) to ./web/"),
            "{running}"
        );
        for _ in 0..100 {
            shell.poll_jobs();
            if shell.visible_text().contains("Exported ") {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let done = shell.visible_text();
        assert!(done.contains("Exported "), "{done}");
        assert!(done.contains("Reveal"), "{done}");
        assert!(!done.contains("Export run"), "card replaced, got {done}");
        assert!(!done.contains("export-card:360"), "{done}");
    });
}

#[gpui::test]
fn followup_preview_not_built_status_names_ctrl_p(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("site");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("site.json");
    let site = Site::starter();
    save_site(&path, &site).unwrap();
    satisfy_local_images(&root, &site);
    let shell = mount(cx, opts(dir.path(), Some(path)));
    shell.update(cx, |shell, _| {
        assert!(shell.visible_text().contains("Preview closed · Ctrl+P"));
        shell.handle_key("ctrl-p");
        let text = shell.visible_text();
        assert!(text.contains("Preview not built · Ctrl+P"), "{text}");
        assert_eq!(shell.preview_window_title(), "Preview \u{2014} My Site");
    });
}

#[gpui::test]
fn followup_issues_double_click_resets_height_without_saving_it(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("site.json");
    save_site(&path, &Site::starter()).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(path)));
    let settings = dir.path().join("gui.json");
    shell.update(cx, |shell, _| {
        shell.handle_key("f3");
        shell.set_issues_height(200.0);
        assert!(shell.visible_text().contains("issues-height:200"));
        let before = std::fs::read(&settings).unwrap();
        assert!(!String::from_utf8_lossy(&before).contains("issues"));
        shell.issues_handle_click(2);
        assert!(
            shell.visible_text().contains("issues-height:280"),
            "{}",
            shell.visible_text()
        );
        assert_eq!(std::fs::read(&settings).unwrap(), before);
        shell.set_issues_height(220.0);
        shell.issues_handle_click(1);
        assert!(shell.visible_text().contains("issues-height:220"));
        shell.set_issues_height(5000.0);
        assert!(shell.visible_text().contains("issues-height:400"));
    });
}
