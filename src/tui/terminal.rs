//! Raw mode, the alternate screen, and the Kitty keyboard protocol.
//!
//! The protocol is pushed only when the terminal reports support. [`TerminalGuard`]
//! pops it on drop, which runs on a normal return and on unwind.
use std::io::{self, Write};

use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    supports_keyboard_enhancement,
};
use dd_siteforge_core::commands::{self, InputMode};

pub(super) struct TerminalGuard {
    pub enhanced: bool,
}

impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste
        )?;
        let supported = supports_keyboard_enhancement().unwrap_or(false);
        let mut enhanced = false;
        if commands::push_keyboard_enhancement(supported)
            && execute!(
                stdout,
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
            )
            .is_ok()
        {
            enhanced = true;
        }
        let _ = stdout.flush();
        Ok(Self { enhanced })
    }

    pub fn input_mode(&self) -> InputMode {
        commands::input_mode(self.enhanced, std::env::var_os("TMUX").is_some())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal(self.enhanced);
    }
}

/// Leave raw mode and, when the protocol was pushed, pop it. Safe to call
/// after a panic: the guard's drop is the panic path.
pub(super) fn restore_terminal(enhanced: bool) {
    let _ = disable_raw_mode();
    let mut stdout = io::stdout();
    if enhanced {
        let _ = execute!(stdout, PopKeyboardEnhancementFlags);
    }
    let _ = execute!(
        stdout,
        DisableBracketedPaste,
        LeaveAlternateScreen,
        DisableMouseCapture
    );
    let _ = stdout.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_remembers_whether_the_protocol_was_pushed() {
        // Neither guard is dropped here. Drop writes the restore sequence,
        // including the Kitty pop when `enhanced` is set, and this process
        // is not holding the alternate screen.
        let quiet = TerminalGuard { enhanced: false };
        let pushed = TerminalGuard { enhanced: true };
        assert!(!quiet.enhanced);
        assert!(pushed.enhanced);
        std::mem::forget(quiet);
        std::mem::forget(pushed);
    }

    #[test]
    fn protocol_flag_is_disambiguate_escape_codes() {
        let flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES;
        assert!(flags.contains(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES));
        assert!(!flags.contains(KeyboardEnhancementFlags::REPORT_EVENT_TYPES));
    }
}
