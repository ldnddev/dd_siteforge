//! Brand colors go through the `dd_wcag` CLI. The GUI does no contrast math.
//!
//! `require_css_hex` runs first. Invalid hex never spawns `dd_wcag` and never
//! writes `site.json`. A valid hex then runs
//! `dd_wcag palette --primary … --secondary … --tertiary … --support … --json`.
//! Exit 0 is pass, 1 is blocking, 2 is bad input. A missing binary is the
//! "dd_wcag missing" state.

use std::path::{Path, PathBuf};
use std::process::Command;

use dd_siteforge_core::cursor::require_css_hex;
use dd_siteforge_core::model::Site;
#[cfg(test)]
use dd_siteforge_core::storage::{load_site, save_site};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrandColors {
    pub primary: String,
    pub secondary: String,
    pub tertiary: String,
    pub support: String,
}

impl BrandColors {
    pub fn from_site(site: &Site) -> Self {
        Self {
            primary: site.theme.primary_color.clone(),
            secondary: site.theme.secondary_color.clone(),
            tertiary: site.theme.tertiary_color.clone(),
            support: site.theme.support_color.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckRow {
    pub label: String,
    pub ratio: f64,
    pub threshold: f64,
    pub passes: bool,
    pub blocking: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaletteReport {
    pub pass: bool,
    pub blocking_failures: usize,
    pub advisory_warnings: usize,
    pub checks: Vec<CheckRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BrandView {
    Invalid { message: String },
    Missing,
    Pass(PaletteReport),
    Blocking(PaletteReport),
    BadInput { message: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Gate {
    pub allow_write: bool,
    pub spawned: bool,
    pub colors: BrandColors,
    pub view: BrandView,
}

#[derive(Deserialize)]
struct RawReport {
    pass: bool,
    blocking_failures: usize,
    advisory_warnings: usize,
    #[serde(default)]
    checks: Vec<RawCheck>,
}

#[derive(Deserialize)]
struct RawCheck {
    label: String,
    ratio: f64,
    threshold: f64,
    passes: bool,
    blocking: bool,
}

/// Screen the four colors, then run `bin` only when every hex is valid.
pub fn gate(colors: &BrandColors, bin: &Path) -> Gate {
    let screened = match screen(colors) {
        Ok(colors) => colors,
        Err(message) => {
            return Gate {
                allow_write: false,
                spawned: false,
                colors: colors.clone(),
                view: BrandView::Invalid { message },
            };
        }
    };
    match run(bin, &screened) {
        Run::Missing => Gate {
            allow_write: true,
            spawned: false,
            colors: screened,
            view: BrandView::Missing,
        },
        Run::BadInput { message } => Gate {
            allow_write: false,
            spawned: true,
            colors: screened,
            view: BrandView::BadInput { message },
        },
        Run::Report(report) => {
            let view = if report.pass {
                BrandView::Pass(report)
            } else {
                BrandView::Blocking(report)
            };
            Gate {
                allow_write: true,
                spawned: true,
                colors: screened,
                view,
            }
        }
    }
}

/// Write the screened colors when the gate allows it. Invalid hex leaves the file alone.
#[cfg(test)]
pub fn commit_brand(path: &Path, colors: &BrandColors, bin: &Path) -> Gate {
    let before = std::fs::read(path).ok();
    let result = gate(colors, bin);
    if !result.allow_write {
        return result;
    }
    let Ok(mut site) = load_site(path) else {
        return result;
    };
    site.theme.primary_color = result.colors.primary.clone();
    site.theme.secondary_color = result.colors.secondary.clone();
    site.theme.tertiary_color = result.colors.tertiary.clone();
    site.theme.support_color = result.colors.support.clone();
    if save_site(path, &site).is_err() {
        if let Some(bytes) = before {
            let _ = std::fs::write(path, bytes);
        }
    }
    result
}

pub fn screen(colors: &BrandColors) -> Result<BrandColors, String> {
    Ok(BrandColors {
        primary: one(&colors.primary, "primary_color")?,
        secondary: one(&colors.secondary, "secondary_color")?,
        tertiary: one(&colors.tertiary, "tertiary_color")?,
        support: one(&colors.support, "support_color")?,
    })
}

fn one(raw: &str, field: &str) -> Result<String, String> {
    require_css_hex(raw, field).map_err(|err| err.to_string())
}

enum Run {
    Missing,
    BadInput { message: String },
    Report(PaletteReport),
}

fn run(bin: &Path, colors: &BrandColors) -> Run {
    if !bin.as_os_str().is_empty() && bin.is_absolute() && !bin.exists() {
        return Run::Missing;
    }
    let output = match Command::new(bin).args(palette_args(colors)).output() {
        Ok(output) => output,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Run::Missing,
        Err(err) => {
            return Run::BadInput {
                message: err.to_string(),
            };
        }
    };
    let code = output.status.code().unwrap_or(2);
    if code == 2 {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Run::BadInput {
            message: if stderr.is_empty() {
                "dd_wcag rejected the colors.".to_string()
            } else {
                stderr
            },
        };
    }
    match parse_report(&output.stdout) {
        Some(report) => Run::Report(report),
        None if code == 0 => Run::Report(PaletteReport {
            pass: true,
            blocking_failures: 0,
            advisory_warnings: 0,
            checks: Vec::new(),
        }),
        None => Run::BadInput {
            message: "dd_wcag did not return palette JSON.".to_string(),
        },
    }
}

pub fn palette_args(colors: &BrandColors) -> Vec<String> {
    vec![
        "palette".to_string(),
        "--primary".to_string(),
        colors.primary.clone(),
        "--secondary".to_string(),
        colors.secondary.clone(),
        "--tertiary".to_string(),
        colors.tertiary.clone(),
        "--support".to_string(),
        colors.support.clone(),
        "--json".to_string(),
    ]
}

/// `dd_wcag --primary … --secondary … --tertiary … --support … --generate`
pub fn generate_args(colors: &BrandColors) -> Vec<String> {
    vec![
        "--primary".to_string(),
        colors.primary.clone(),
        "--secondary".to_string(),
        colors.secondary.clone(),
        "--tertiary".to_string(),
        colors.tertiary.clone(),
        "--support".to_string(),
        colors.support.clone(),
        "--generate".to_string(),
    ]
}

pub fn spawn_generate(bin: &Path, colors: &BrandColors) -> std::io::Result<()> {
    Command::new(bin)
        .args(generate_args(colors))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
}

fn parse_report(stdout: &[u8]) -> Option<PaletteReport> {
    let raw: RawReport = serde_json::from_slice(stdout).ok()?;
    Some(PaletteReport {
        pass: raw.pass,
        blocking_failures: raw.blocking_failures,
        advisory_warnings: raw.advisory_warnings,
        checks: raw
            .checks
            .into_iter()
            .map(|check| CheckRow {
                label: check.label,
                ratio: check.ratio,
                threshold: check.threshold,
                passes: check.passes,
                blocking: check.blocking,
            })
            .collect(),
    })
}

pub fn view_lines(view: &BrandView) -> Vec<String> {
    match view {
        BrandView::Invalid { message } => vec![message.clone()],
        BrandView::Missing => vec!["dd_wcag missing".to_string()],
        BrandView::Pass(report) => report_lines("pass", report),
        BrandView::Blocking(report) => report_lines("blocking", report),
        BrandView::BadInput { message } => vec![message.clone()],
    }
}

fn report_lines(kind: &str, report: &PaletteReport) -> Vec<String> {
    let mut lines = vec![
        format!("pass:{}", report.pass),
        format!("blocking_failures:{}", report.blocking_failures),
        format!("advisory_warnings:{}", report.advisory_warnings),
        format!("brand:{kind}"),
    ];
    for check in &report.checks {
        lines.push(format!(
            "check:{} {} {} {} {}",
            check.label, check.ratio, check.threshold, check.passes, check.blocking
        ));
    }
    lines.push("Fix in dd_wcag".to_string());
    lines
}

pub fn default_bin() -> PathBuf {
    PathBuf::from("dd_wcag")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> BrandColors {
        BrandColors {
            primary: "  #112233 ".to_string(),
            secondary: "#445566".to_string(),
            tertiary: "#778899".to_string(),
            support: "#aabbcc".to_string(),
        }
    }

    fn write_script(dir: &Path, body: &str) -> PathBuf {
        let path = dir.join("dd_wcag");
        std::fs::write(&path, body).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms).unwrap();
        }
        path
    }

    fn site_file(dir: &Path) -> PathBuf {
        let path = dir.join("site.json");
        save_site(&path, &Site::starter()).unwrap();
        path
    }

    #[test]
    fn exit_0_parses_pass_and_checks() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("argv");
        let script = write_script(
            dir.path(),
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" > '{}'\nprintf '%s\\n' '{{\"pass\":true,\"blocking_failures\":0,\"advisory_warnings\":1,\"checks\":[{{\"label\":\"primary on white\",\"ratio\":7.1,\"threshold\":4.5,\"passes\":true,\"blocking\":false}}]}}'\nexit 0\n",
                log.display()
            ),
        );
        let path = site_file(dir.path());
        let gate = commit_brand(&path, &valid(), &script);
        assert!(gate.spawned);
        assert!(gate.allow_write);
        let BrandView::Pass(report) = gate.view else {
            panic!("{:?}", gate.view);
        };
        assert!(report.pass);
        assert_eq!(report.blocking_failures, 0);
        assert_eq!(report.advisory_warnings, 1);
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.checks[0].label, "primary on white");
        let argv = std::fs::read_to_string(log).unwrap();
        assert!(argv.contains("palette"), "{argv}");
        assert!(argv.contains("--primary"), "{argv}");
        assert!(argv.contains("--secondary"), "{argv}");
        assert!(argv.contains("--tertiary"), "{argv}");
        assert!(argv.contains("--support"), "{argv}");
        assert!(argv.contains("--json"), "{argv}");
        assert!(argv.contains("#112233"), "{argv}");
        let saved = load_site(&path).unwrap();
        assert_eq!(saved.theme.primary_color, "#112233");
    }

    #[test]
    fn exit_1_is_blocking_and_exit_2_does_not_write() {
        let dir = tempfile::tempdir().unwrap();
        let blocking = write_script(
            dir.path(),
            "#!/bin/sh\nprintf '%s\\n' '{\"pass\":false,\"blocking_failures\":2,\"advisory_warnings\":0,\"checks\":[{\"label\":\"support on primary\",\"ratio\":1.2,\"threshold\":4.5,\"passes\":false,\"blocking\":true}]}'\nexit 1\n",
        );
        let path = site_file(dir.path());
        let gate = commit_brand(&path, &valid(), &blocking);
        assert!(gate.spawned);
        let BrandView::Blocking(report) = &gate.view else {
            panic!("{:?}", gate.view);
        };
        assert!(!report.pass);
        assert_eq!(report.blocking_failures, 2);
        assert!(report.checks.iter().any(|check| check.blocking));
        assert_eq!(load_site(&path).unwrap().theme.primary_color, "#112233");

        let bad = write_script(
            dir.path(),
            "#!/bin/sh\nprintf '%s\\n' 'error: bad input' >&2\nexit 2\n",
        );
        let before = std::fs::read(&path).unwrap();
        let gate = commit_brand(
            &path,
            &BrandColors {
                primary: "#000000".to_string(),
                secondary: "#111111".to_string(),
                tertiary: "#222222".to_string(),
                support: "#333333".to_string(),
            },
            &bad,
        );
        assert!(gate.spawned);
        assert!(!gate.allow_write);
        assert!(matches!(gate.view, BrandView::BadInput { .. }));
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn missing_binary_is_the_missing_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = site_file(dir.path());
        let missing = dir.path().join("no-such-dd-wcag");
        let gate = commit_brand(&path, &valid(), &missing);
        assert!(!gate.spawned);
        assert!(gate.allow_write);
        assert_eq!(gate.view, BrandView::Missing);
        assert_eq!(load_site(&path).unwrap().theme.primary_color, "#112233");
    }

    #[test]
    fn invalid_hex_never_calls_dd_wcag_or_writes_site_json() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("called");
        let script = write_script(
            dir.path(),
            &format!("#!/bin/sh\ntouch '{}'\nexit 0\n", log.display()),
        );
        let path = site_file(dir.path());
        let before = std::fs::read(&path).unwrap();
        let mut colors = valid();
        colors.primary = "red".to_string();
        let gate = commit_brand(&path, &colors, &script);
        assert!(!gate.spawned);
        assert!(!gate.allow_write);
        assert!(!log.exists(), "dd_wcag was spawned");
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let BrandView::Invalid { message } = gate.view else {
            panic!("{:?}", gate.view);
        };
        assert!(message.contains("invalid primary_color"), "{message}");
    }

    #[test]
    fn generate_args_include_the_four_colors() {
        let args = generate_args(&valid());
        assert_eq!(
            args,
            vec![
                "--primary",
                "  #112233 ",
                "--secondary",
                "#445566",
                "--tertiary",
                "#778899",
                "--support",
                "#aabbcc",
                "--generate",
            ]
        );
    }
}
