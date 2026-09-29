//! TUI Shift+B: run Lando/npx grunt in the background.
use super::*;

impl App {
    pub(in crate::tui) fn site_root(&self) -> std::path::PathBuf {
        self.path
            .as_ref()
            .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }

    pub(in crate::tui) fn start_asset_build(&mut self) {
        if self.build_rx.is_some() {
            self.push_toast(ToastLevel::Warning, "Build already running.");
            return;
        }
        let root = self.site_root();
        let cmd = crate::asset_build::asset_build_command(&root);
        self.push_toast(ToastLevel::Info, format!("Running `{}`…", cmd.display()));
        self.build_rx = Some(crate::asset_build::spawn_asset_build(root));
    }

    pub(in crate::tui) fn poll_asset_build(&mut self) -> bool {
        let Some(rx) = self.build_rx.as_ref() else {
            return false;
        };
        match rx.try_recv() {
            Ok(Ok(msg)) => {
                self.build_rx = None;
                if let Some(server) = self.preview_server.as_ref() {
                    server.bump_generation();
                }
                self.push_toast(ToastLevel::Success, msg);
                true
            }
            Ok(Err(msg)) => {
                self.build_rx = None;
                self.push_toast(ToastLevel::Error, msg);
                true
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => false,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.build_rx = None;
                self.push_toast(ToastLevel::Error, "Build thread stopped.");
                true
            }
        }
    }

    pub(in crate::tui) fn open_page_health(&mut self) {
        let items = crate::health::page_health(&self.site, self.selected_page);
        self.modal = Some(Modal::PageHealth {
            items,
            scroll_offset: 0,
        });
    }

    pub(in crate::tui) fn handle_page_health_event(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<ModalResult> {
        let (len, scroll) = match &self.modal {
            Some(Modal::PageHealth {
                items,
                scroll_offset,
            }) => (items.len(), *scroll_offset),
            _ => return Some(ModalResult::CloseCancel),
        };
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                self.modal = None;
                Some(ModalResult::CloseCancel)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(Modal::PageHealth { scroll_offset, .. }) = self.modal.as_mut() {
                    *scroll_offset = scroll_offset.saturating_sub(1);
                }
                Some(ModalResult::Continue)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(Modal::PageHealth { scroll_offset, .. }) = self.modal.as_mut() {
                    if scroll + 1 < len.max(1) {
                        *scroll_offset += 1;
                    }
                }
                Some(ModalResult::Continue)
            }
            KeyCode::PageUp => {
                if let Some(Modal::PageHealth { scroll_offset, .. }) = self.modal.as_mut() {
                    *scroll_offset = scroll_offset.saturating_sub(5);
                }
                Some(ModalResult::Continue)
            }
            KeyCode::PageDown => {
                if let Some(Modal::PageHealth { scroll_offset, .. }) = self.modal.as_mut() {
                    *scroll_offset = (scroll + 5).min(len.saturating_sub(1));
                }
                Some(ModalResult::Continue)
            }
            _ => Some(ModalResult::Continue),
        }
    }
}
