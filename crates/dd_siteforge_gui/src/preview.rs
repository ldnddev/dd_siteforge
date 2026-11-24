//! Preview opens in the default browser.
//!
//! The Phase 1 spike did not get a clean `wry`/`tao` helper (libEGL DRI3, and
//! no Hyprland session). Phase 4 keeps that engine. A GPUI companion window
//! holds Follow editor and Ctrl+R. The browser tab stays in sync through the
//! preview server's reload and follow endpoints.

use std::io;
use std::process::{Command, Stdio};

/// Chosen preview engine. `"browser"` is the default-browser fallback.
#[cfg(test)]
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
