//! Golden fixtures for the runtime slot-chain resolver.
//! `resolved/*.json` and `contrast/pairs.csv` are copied from the design
//! handoff. They are not shipped in the app.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use ldnddev_theme::{
    contrast_ratio, parse_hex_color, resolve_gui_theme, OmarchyColors, Rgb, ThemeMode,
};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/gui")
        .join(rel)
}

#[test]
fn resolver_matches_every_resolved_theme() {
    let dir = fixture("resolved");
    let mut files: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("missing {}: {err}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert_eq!(files.len(), 23, "expected one fixture per Omarchy theme");

    for path in files {
        let golden: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let slug = golden["theme"].as_str().unwrap();
        let mode = match golden["mode"].as_str().unwrap() {
            "dark" => ThemeMode::Dark,
            "light" => ThemeMode::Light,
            other => panic!("{slug}: unknown mode {other}"),
        };
        let mut slots = BTreeMap::new();
        for (key, value) in golden["slots"].as_object().unwrap() {
            let hex = value.as_str().unwrap();
            slots.insert(key.clone(), parse_hex_color(key, hex).unwrap());
        }
        let theme = resolve_gui_theme(&OmarchyColors::from_slots(mode, slots));
        assert_eq!(theme.mode, mode, "{slug}");
        assert_eq!(
            theme.primary_button.as_str(),
            golden["primary_button"].as_str().unwrap(),
            "{slug} primary button"
        );
        let expected_swaps = golden["on_surface_base"].as_object().unwrap();
        assert_eq!(theme.on_surface_base.len(), expected_swaps.len(), "{slug}");
        for (token, uses) in expected_swaps {
            assert_eq!(
                theme.on_surface_base.get(token).map(String::as_str),
                uses.as_str(),
                "{slug} rule b {token}"
            );
        }
        let tokens = golden["tokens"].as_object().unwrap();
        assert_eq!(theme.tokens.len(), tokens.len(), "{slug}");
        for token in &theme.tokens {
            let expected = &tokens[token.name];
            assert_eq!(
                token.hex,
                expected["hex"].as_str().unwrap(),
                "{slug} {} hex (slot {})",
                token.name,
                token.slot
            );
            assert_eq!(
                token.slot,
                expected["slot"].as_str().unwrap(),
                "{slug} {} slot",
                token.name
            );
        }
    }
}

#[test]
fn contrast_ratio_matches_pairs_csv_to_two_decimals() {
    let path = fixture("contrast/pairs.csv");
    let text = fs::read_to_string(&path).unwrap_or_else(|err| panic!("missing pairs.csv: {err}"));
    let mut rows = text.lines();
    let header = rows.next().unwrap();
    let columns = split_csv(header);
    let hex_at = columns.iter().position(|c| c == "resolved_hex").unwrap();
    let bg_at = columns.iter().position(|c| c == "bg_hex").unwrap();
    let ratio_at = columns.iter().position(|c| c == "ratio").unwrap();
    let mut checked = 0usize;
    for (line_no, line) in rows.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields = split_csv(line);
        let fg = parse_hex_color("fg", &fields[hex_at]).unwrap();
        let bg = parse_hex_color("bg", &fields[bg_at]).unwrap();
        let expected: f64 = fields[ratio_at].parse().unwrap();
        let got = contrast_ratio(fg, bg);
        let same = (got * 100.0).round() as i64 == (expected * 100.0).round() as i64;
        assert!(
            same,
            "line {}: {} on {} expected {expected} got {got}",
            line_no + 2,
            hex(fg),
            hex(bg)
        );
        checked += 1;
    }
    assert!(checked > 2000, "pairs.csv should cover the design report");
}

fn hex(color: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
}

fn split_csv(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    for ch in line.chars() {
        match ch {
            '"' if !quoted => quoted = true,
            '"' if quoted => quoted = false,
            ',' if !quoted => {
                out.push(std::mem::take(&mut field));
            }
            _ => field.push(ch),
        }
    }
    out.push(field);
    out
}
