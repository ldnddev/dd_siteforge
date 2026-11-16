//! Preview opens in the default browser.
//!
//! The Phase 1 spike did not show a clean `wry`/`tao` helper on this machine
//! (no Hyprland session at scale 1.67, and no display to judge tiling, IME, or
//! fractional scale). The helper is not linked. `open_in_browser` is the path
//! the window uses.

use std::io;
use std::process::{Command, Stdio};

/// Chosen preview engine. `"browser"` is the default-browser fallback.
pub const PREVIEW_ENGINE: &str = "browser";

pub fn open_in_browser(target: &str) -> io::Result<()> {
    let mut cmd = Command::new(browser_command());
    cmd.arg(target);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

fn browser_command() -> &'static str {
    if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "xdg-open"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spike_keeps_the_browser_fallback() {
        assert_eq!(PREVIEW_ENGINE, "browser");
        assert_eq!(browser_command(), "xdg-open");
    }
}
