//! Pins YAML values produced by the crate's current parser.
//!
//! Theme files go through [`ldnddev_theme::load_from_str`]. Config templates
//! are not deserialized by the app (scaffold stamps them as text); their
//! document values are pinned here so a parser swap cannot change them.

use ldnddev_theme::{
    load_from_str, parse_hex_color, OmarchyChoice, Palette, ParseMode, ThemeSource,
};
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_repo(rel: &str) -> String {
    let path = repo_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn load(yaml: &str, mode: ParseMode) -> Result<Palette, String> {
    load_from_str(yaml, mode, ThemeSource::Local).map_err(|err| format!("{err:#}"))
}

fn shipped_quotes() -> Vec<String> {
    [
        "Drafts are just commits that lost their nerve.",
        "Saved. Probably. Hopefully. Definitely. (It saved.).",
        "This post is live, which means it's officially out of your hands.",
        "Scheduled for later \u{2014} future you can deal with the typos.",
        "Deleted. We won't talk about it again. (We both saw it.)",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn representative_quotes() -> Vec<String> {
    vec![
        "Quoted ampersand & parentheses (kept).".to_string(),
        "Scheduled later \u{2014} em dash stays.".to_string(),
        "Single quoted #010203 is text, not a comment.".to_string(),
        // Newlines are control characters. The loader strips them after YAML parse.
        "multilinequote".to_string(),
    ]
}

fn representative_colors() -> Vec<(&'static str, &'static str)> {
    vec![
        ("base_background", "#010203"),
        ("body_background", "#000000"),
        ("modal_background", "#808080"),
        ("selected_background", "#112233"),
        ("text_primary", "#445566"),
        ("text_secondary", "#778899"),
        ("text_labels", "#ABCDEF"),
        ("text_active_focus", "#010101"),
        ("modal_labels", "#020202"),
        ("modal_text", "#030303"),
        ("modal_header", "#28292A"),
        ("text_disabled", "#2B2C2D"),
        ("text_inverse", "#2E2F30"),
        ("border_default", "#040404"),
        ("border_active", "#050505"),
        ("scrollbar", "#060606"),
        ("scrollbar_hover", "#070707"),
        ("selection", "#313233"),
        ("input_border_default", "#080808"),
        ("input_border_focus", "#090909"),
        ("input_text_default", "#0A0B0C"),
        ("input_text_focus", "#0D0E0F"),
        ("cursor", "#101112"),
        ("success", "#131415"),
        ("warning", "#161718"),
        ("error", "#191A1B"),
        ("info", "#1C1D1E"),
        ("folders", "#1F2021"),
        ("files", "#222324"),
        ("links", "#252627"),
        ("custom_extra", "#343536"),
    ]
}

fn palette_with_colors(colors: &[(&str, &str)], quotes: Vec<String>) -> Palette {
    let mut expected = Palette::builtin();
    for (key, hex) in colors {
        expected.set(key, parse_hex_color(key, hex).expect("hex"));
    }
    expected.header_quotes = quotes;
    expected.version = 1;
    expected.source = ThemeSource::Local;
    expected
}

fn assert_yaml_inventory() {
    let mut found = Vec::new();
    collect_yaml(&repo_root(), &repo_root(), &mut found);
    found.sort();
    let mut expected = vec![
        ".ddev/config.yaml".to_string(),
        ".github/workflows/ci.yml".to_string(),
        ".github/workflows/pages.yml".to_string(),
        ".github/workflows/release.yml".to_string(),
        ".lando.yml".to_string(),
        "crates/ldnddev_theme/tests/fixtures/themes/representative.yml".to_string(),
        "dd_siteforge_theme.yml".to_string(),
    ];
    expected.sort();
    assert_eq!(
        found, expected,
        "a YAML file was added or removed; pin its parsed values in this test"
    );
}

fn collect_yaml(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|err| panic!("read {}: {err}", dir.display()));
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name == "target" || name == ".git" {
            continue;
        }
        if path.is_dir() {
            collect_yaml(root, &path, out);
            continue;
        }
        if name.ends_with(".yml") || name.ends_with(".yaml") {
            let rel = path
                .strip_prefix(root)
                .expect("under repo")
                .to_string_lossy()
                .replace('\\', "/");
            out.push(rel);
        }
    }
}

#[test]
fn repo_yaml_files_are_accounted_for() {
    assert_yaml_inventory();
}

#[test]
fn shipped_theme_matches_builtin_plus_link_and_quotes() {
    let raw = read_repo("dd_siteforge_theme.yml");
    let lenient = load(&raw, ParseMode::Lenient).expect("lenient shipped theme");
    let strict = load(&raw, ParseMode::Strict).expect("strict shipped theme");
    let mut expected = Palette::builtin();
    expected.header_quotes = shipped_quotes();
    expected.version = 1;
    expected.source = ThemeSource::Local;
    expected.omarchy = Some(OmarchyChoice::Theme("retro-82".into()));
    assert_eq!(lenient, expected);
    assert_eq!(strict, expected);
    assert!(lenient.overrides.is_empty());
}

#[test]
fn representative_theme_pins_every_color_and_quote() {
    let raw = read_repo("crates/ldnddev_theme/tests/fixtures/themes/representative.yml");
    let lenient = load(&raw, ParseMode::Lenient).expect("lenient representative");
    let strict = load(&raw, ParseMode::Strict).expect("strict representative");
    let colors = representative_colors();
    let expected = palette_with_colors(
        &colors.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        representative_quotes(),
    );
    assert_eq!(lenient, expected);
    assert_eq!(strict, expected);
    assert!(lenient.omarchy.is_none());
    assert!(lenient.overrides.is_empty());
}

#[test]
fn quoted_and_unquoted_scalars_match() {
    let plain = load("version: 1\nomarchy: retro-82\n", ParseMode::Lenient).unwrap();
    let double = load("version: 1\nomarchy: \"retro-82\"\n", ParseMode::Lenient).unwrap();
    let single = load("version: 1\nomarchy: 'retro-82'\n", ParseMode::Lenient).unwrap();
    assert_eq!(plain, double);
    assert_eq!(plain, single);
    assert_eq!(plain.omarchy, Some(OmarchyChoice::Theme("retro-82".into())));

    let int = load("version: 1\n", ParseMode::Lenient).unwrap();
    let quoted = load("version: \"1\"\n", ParseMode::Lenient).unwrap();
    let quoted_single = load("version: '1'\n", ParseMode::Lenient).unwrap();
    assert_eq!(int.version, 1);
    assert_eq!(int, quoted);
    assert_eq!(int, quoted_single);

    let leading_zero = load("version: 01\n", ParseMode::Lenient).unwrap();
    assert_eq!(leading_zero.version, 1);
    assert_eq!(
        leading_zero.colors().get("base_background"),
        int.colors().get("base_background")
    );
}

#[test]
fn quoted_hex_that_looks_numeric_is_that_color() {
    let yaml = "\
version: 1
colors:
  base_background: \"#010203\"
  body_background: \"#000000\"
  modal_background: '#808080'
";
    let palette = load(yaml, ParseMode::Lenient).expect("quoted hex");
    assert_eq!(palette.get("base_background").unwrap().to_hex(), "#010203");
    assert_eq!(palette.get("body_background").unwrap().to_hex(), "#000000");
    assert_eq!(palette.get("modal_background").unwrap().to_hex(), "#808080");
    assert_eq!(
        palette.get("text_primary").unwrap().to_hex(),
        Palette::builtin().get("text_primary").unwrap().to_hex()
    );
}

#[test]
fn unquoted_numeric_hex_keeps_source_spelling_then_fails_hex() {
    // String fields keep the scalar's spelling, including a leading zero and `0x`.
    for raw in ["010203", "000000", "0x010203"] {
        let yaml = format!("version: 1\ncolors:\n  base_background: {raw}\n");
        let err = load(&yaml, ParseMode::Lenient).expect_err(raw);
        assert!(err.contains("must start with #"), "{raw}: {err}");
        assert!(
            !err.contains("Failed to parse theme YAML"),
            "{raw} was not kept as a string: {err}"
        );
    }
    let quoted = load(
        "version: 1\ncolors:\n  base_background: \"010203\"\n",
        ParseMode::Lenient,
    )
    .expect_err("quoted digits without hash");
    assert!(quoted.contains("must start with #"), "{quoted}");
}

#[test]
fn unquoted_hash_is_an_empty_color() {
    // `#010203` is a comment, so the scalar is null. A string field becomes empty.
    let err = load(
        "version: 1\ncolors:\n  base_background: #010203\n",
        ParseMode::Lenient,
    )
    .expect_err("comment");
    assert!(
        err.contains("Missing color value for theme key `base_background`"),
        "{err}"
    );
    let bare = load(
        "version: 1\ncolors:\n  base_background:\n",
        ParseMode::Lenient,
    )
    .expect_err("empty scalar");
    assert!(
        bare.contains("Missing color value for theme key `base_background`"),
        "{bare}"
    );
}

#[test]
fn multiline_scalars_keep_newlines_until_the_loader_strips_controls() {
    let raw: serde_json::Value = serde_yaml::from_str(
        "header_quotes:\n  - |\n    keep\n    newline\n  - >\n    fold\n    these\n  - |-\n    multiline\n    quote\n",
    )
    .expect("raw multiline");
    assert_eq!(
        raw,
        serde_json::json!({
            "header_quotes": ["keep\nnewline\n", "fold these\n", "multiline\nquote"]
        })
    );

    let palette = load(
        "version: 1\nheader_quotes:\n  - |\n    keep\n    newline\n  - >\n    fold\n    these\n  - |-\n    multiline\n    quote\n",
        ParseMode::Lenient,
    )
    .expect("filtered multiline");
    assert_eq!(
        palette.header_quotes,
        vec![
            "keepnewline".to_string(),
            "fold these".to_string(),
            "multilinequote".to_string(),
        ]
    );
}

#[test]
fn empty_and_missing_keys() {
    let missing_version = load(
        "colors:\n  base_background: \"#010203\"\n",
        ParseMode::Lenient,
    )
    .expect_err("missing version");
    assert!(
        missing_version.contains("Missing required theme key `version`"),
        "{missing_version}"
    );

    // Null, `~`, and an empty scalar all become a missing Option before version parsing.
    for yaml in ["version:\n", "version: null\n", "version: ~\n", ""] {
        let err = load(yaml, ParseMode::Lenient).expect_err(yaml);
        assert!(
            err.contains("Missing required theme key `version`"),
            "{yaml:?}: {err}"
        );
    }
    let blank = load("version: \"\"\n", ParseMode::Lenient).expect_err("blank version");
    assert!(
        blank.contains("Missing value for theme key `version`"),
        "{blank}"
    );

    let no_colors = load("version: 1\n", ParseMode::Lenient).unwrap();
    assert_eq!(no_colors, {
        let mut expected = Palette::builtin();
        expected.version = 1;
        expected.source = ThemeSource::Local;
        expected
    });
    let empty_map = load("version: 1\ncolors: {}\n", ParseMode::Lenient).unwrap();
    assert_eq!(empty_map.colors(), no_colors.colors());
    let null_colors = load("version: 1\ncolors:\n", ParseMode::Lenient).unwrap();
    assert_eq!(null_colors.colors(), no_colors.colors());
    let empty_quotes = load("version: 1\nheader_quotes: []\n", ParseMode::Lenient).unwrap();
    assert!(empty_quotes.header_quotes.is_empty());

    let empty_color = load(
        "version: 1\ncolors:\n  base_background: \"\"\n",
        ParseMode::Lenient,
    )
    .expect_err("empty color");
    assert!(
        empty_color.contains("Missing color value for theme key `base_background`"),
        "{empty_color}"
    );
    let blank_color = load(
        "version: 1\ncolors:\n  base_background: \"   \"\n",
        ParseMode::Lenient,
    )
    .expect_err("blank color");
    assert!(
        blank_color.contains("Missing color value for theme key `base_background`"),
        "{blank_color}"
    );
}

#[test]
fn version_type_errors_and_unsupported_schema() {
    let float = load("version: 1.5\n", ParseMode::Lenient).expect_err("float");
    assert!(float.contains("must be an integer"), "{float}");
    let negative = load("version: -1\n", ParseMode::Lenient).expect_err("negative");
    assert!(negative.contains("must be an integer"), "{negative}");
    let word_yes = load("version: yes\n", ParseMode::Lenient).expect_err("yes");
    assert!(word_yes.contains("invalid digit"), "{word_yes}");
    let boolean = load("version: true\n", ParseMode::Lenient).expect_err("true");
    assert!(boolean.contains("must be an integer"), "{boolean}");
    assert!(!boolean.contains("invalid digit"), "{boolean}");
    let hex_number = load("version: 0x10\n", ParseMode::Lenient).expect_err("hex");
    assert!(
        hex_number.contains("Unsupported theme schema version `16`"),
        "{hex_number}"
    );
    let word = load("version: \"nope\"\n", ParseMode::Lenient).expect_err("word");
    assert!(word.contains("must be an integer"), "{word}");
    let too_new = load("version: 2\n", ParseMode::Lenient).expect_err("2");
    assert!(
        too_new.contains("Unsupported theme schema version `2`"),
        "{too_new}"
    );
    let too_new_quoted = load("version: \"2\"\n", ParseMode::Lenient).expect_err("\"2\"");
    assert!(
        too_new_quoted.contains("Unsupported theme schema version `2`"),
        "{too_new_quoted}"
    );
}

#[test]
fn bool_like_tokens_keep_their_spelling_on_string_fields() {
    for token in ["yes", "no", "on", "off", "true", "false"] {
        let palette = load(
            &format!("version: 1\nomarchy: {token}\n"),
            ParseMode::Lenient,
        )
        .unwrap_or_else(|err| panic!("{token}: {err}"));
        assert_eq!(
            palette.omarchy,
            Some(OmarchyChoice::Theme(token.to_string())),
            "{token}"
        );
    }
}

#[test]
fn untyped_scalars_match_the_current_parser() {
    let value: serde_json::Value = serde_yaml::from_str(
        "version: 01\nflag: true\nword: yes\nhex: 0x010203\nn: 123\nrate: 1.5\nempty:\n",
    )
    .expect("typed scalars");
    assert_eq!(
        value,
        serde_json::json!({
            "version": "01",
            "flag": true,
            "word": "yes",
            "hex": 66051,
            "n": 123,
            "rate": 1.5,
            "empty": null
        })
    );
}

#[test]
fn anchors_and_aliases_are_rejected() {
    for yaml in [
        "version: &v 1\ncolors:\n  base_background: \"#010203\"\n",
        "version: 1\ncolors:\n  base_background: &c \"#010203\"\n  body_background: *c\n",
        "version: 1\nheader_quotes:\n  - &q \"hello\"\n  - *q\n",
    ] {
        let err = load(yaml, ParseMode::Lenient).expect_err(yaml);
        assert!(err.contains("anchors and aliases are not allowed"), "{err}");
    }
}

#[test]
fn malformed_yaml_fails_parse() {
    for yaml in [
        "version: [\n",
        ":\n",
        "version: 1\ncolors: [\n",
        "version: 1\n\tcolors: {}\n",
    ] {
        let err = load(yaml, ParseMode::Strict).expect_err(yaml);
        assert!(
            err.contains("Failed to parse theme YAML"),
            "{yaml:?}: {err}"
        );
    }
}

#[test]
fn config_yaml_documents_match_pinned_values() {
    let ddev: serde_json::Value =
        serde_yaml::from_str(&read_repo(".ddev/config.yaml")).expect("ddev config");
    assert_eq!(
        ddev,
        serde_json::json!({
            "name": "dd-siteforge",
            "type": "php",
            "docroot": "web",
            "php_version": "8.2",
            "webserver_type": "apache-fpm",
            "nodejs_version": "20",
            "disable_settings_management": true,
            "omit_containers": ["db"]
        })
    );

    let lando: serde_json::Value =
        serde_yaml::from_str(&read_repo(".lando.yml")).expect("lando config");
    assert_eq!(
        lando,
        serde_json::json!({
            "name": "dd-siteforge",
            "recipe": "lamp",
            "proxy": {
                "appserver": ["dd-siteforge.lndo.site"]
            },
            "config": {
                "php": "8.2",
                "via": "apache",
                "webroot": "web"
            },
            "services": {
                "appserver": {
                    "ports": ["8888:80"]
                },
                "node": {
                    "type": "node:20",
                    "build": ["npm install"]
                }
            },
            "tooling": {
                "npm": { "service": "node" },
                "node": { "service": "node" },
                "grunt": { "service": "node" }
            }
        })
    );
}
