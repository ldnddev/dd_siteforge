//! Ephemeral toast notifications.
use super::super::*;

impl App {
    pub(in crate::tui) fn push_toast(&mut self, level: ToastLevel, message: impl Into<String>) {
        self.toasts.push(Toast {
            level,
            message: message.into(),
            shown_at: std::time::Instant::now(),
        });
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    pub(in crate::tui) fn prune_toasts(&mut self) -> bool {
        let before = self.toasts.len();
        let now = std::time::Instant::now();
        self.toasts
            .retain(|t| now.duration_since(t.shown_at) < TOAST_TTL);
        self.toasts.len() != before
    }

    pub(in crate::tui) fn render_toasts(&self, frame: &mut ratatui::Frame, area: Rect) {
        let lines: Vec<crate::tui::toast_chrome::ToastLine> = self
            .toasts
            .iter()
            .map(|toast| crate::tui::toast_chrome::ToastLine {
                kind: match toast.level {
                    ToastLevel::Success => crate::tui::toast_chrome::ToastKind::Success,
                    ToastLevel::Info => crate::tui::toast_chrome::ToastKind::Info,
                    ToastLevel::Warning => crate::tui::toast_chrome::ToastKind::Warning,
                    ToastLevel::Error => crate::tui::toast_chrome::ToastKind::Error,
                },
                message: toast.message.as_str(),
            })
            .collect();
        let _ = crate::tui::toast_chrome::render_toast_stack(
            frame,
            area,
            &lines,
            crate::tui::toast_chrome::ToastPaint {
                modal_background: self.theme.modal_background,
                text: self.theme.text_primary,
                success: self.theme.success,
                info: self.theme.info,
                warning: self.theme.warning,
                error: self.theme.error,
            },
            0,
        );
    }
}
