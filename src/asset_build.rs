//! Resolve the CSS/JS build command: Lando when `.lando.yml` is present.
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetBuildCommand {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub via_lando: bool,
}

impl AssetBuildCommand {
    pub fn display(&self) -> String {
        let args = self.args.join(" ");
        if args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, args)
        }
    }
}

/// Prefer `lando grunt build` when the site has `.lando.yml`.
/// Otherwise `npx grunt build` on the host.
///
/// When Lando is configured, a missing or stopped Lando is a hard error:
/// do not silently fall back to host Node.
pub fn asset_build_command(root: &Path) -> AssetBuildCommand {
    let cwd = root.to_path_buf();
    if root.join(".lando.yml").is_file() {
        AssetBuildCommand {
            program: "lando".into(),
            args: vec!["grunt".into(), "build".into()],
            cwd,
            via_lando: true,
        }
    } else {
        AssetBuildCommand {
            program: "npx".into(),
            args: vec!["grunt".into(), "build".into()],
            cwd,
            via_lando: false,
        }
    }
}

pub fn spawn_asset_build(root: PathBuf) -> std::sync::mpsc::Receiver<Result<String, String>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let cmd = asset_build_command(&root);
        let result = std::process::Command::new(&cmd.program)
            .args(&cmd.args)
            .current_dir(&cmd.cwd)
            .output();
        let _ = tx.send(match result {
            Ok(out) if out.status.success() => Ok(format!("{} finished.", cmd.display())),
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                Err(format_build_failure(&cmd, &stderr, &stdout))
            }
            Err(e) => {
                if cmd.via_lando {
                    Err(format!(
                        "Could not run `{}` ({e}). Start Lando with `lando start` in the site folder.",
                        cmd.display()
                    ))
                } else {
                    Err(format!("Could not run `{}`: {e}", cmd.display()))
                }
            }
        });
    });
    rx
}

fn format_build_failure(cmd: &AssetBuildCommand, stderr: &str, stdout: &str) -> String {
    let detail = last_useful_line(stderr).or_else(|| last_useful_line(stdout));
    match detail {
        Some(line) => format!("{} failed: {line}", cmd.display()),
        None => format!("{} failed.", cmd.display()),
    }
}

fn last_useful_line(text: &str) -> Option<String> {
    let line = text.lines().rev().map(str::trim).find(|l| !l.is_empty())?;
    let chars: Vec<char> = line.chars().collect();
    if chars.len() > 180 {
        Some(chars[..180].iter().collect())
    } else {
        Some(line.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lando_yml_selects_lando_grunt() {
        let dir = std::env::temp_dir().join(format!(
            "dd_build_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".lando.yml"), "name: test\n").unwrap();
        let cmd = asset_build_command(&dir);
        assert!(cmd.via_lando);
        assert_eq!(cmd.program, "lando");
        assert_eq!(cmd.args, ["grunt", "build"]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_lando_yml_uses_npx() {
        let dir = std::env::temp_dir().join(format!(
            "dd_build_npx_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let cmd = asset_build_command(&dir);
        assert!(!cmd.via_lando);
        assert_eq!(cmd.program, "npx");
        assert_eq!(cmd.display(), "npx grunt build");
        std::fs::remove_dir_all(&dir).ok();
    }
}
