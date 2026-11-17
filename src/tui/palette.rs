//! Command palette (`:` / `Ctrl+K`). Rows come from the core chord table.
use super::*;
use dd_siteforge_core::commands::Command;

impl App {
    pub(in crate::tui) fn open_palette(&mut self) {
        self.modal = Some(Modal::Palette {
            query: String::new(),
            selected: 0,
        });
    }

    pub(in crate::tui) fn palette_items(&self, query: &str) -> Vec<Command> {
        let q = query.trim();
        let mut scored: Vec<(Command, i32, &str)> = dd_siteforge_core::commands::registry()
            .iter()
            .filter(|spec| spec.tui_palette)
            .filter_map(|spec| {
                let hay = format!("{} {} {}", spec.title, spec.detail, spec.kbd);
                fuzzy_score(q, &hay).map(|score| (spec.command, score, spec.title))
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.2.cmp(b.2)));
        scored.into_iter().map(|(cmd, _, _)| cmd).collect()
    }

    pub(in crate::tui) fn run_palette_command(&mut self, cmd: Command) {
        let _ = self.run_command_key(cmd);
    }

    pub(in crate::tui) fn handle_palette_event(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<ModalResult> {
        let (query, selected) = if let Some(Modal::Palette { query, selected }) = self.modal.take()
        {
            (query, selected)
        } else {
            return Some(ModalResult::CloseCancel);
        };
        let items = self.palette_items(&query);
        match key.code {
            KeyCode::Esc => {
                self.push_toast(ToastLevel::Info, "Command palette cancelled.");
                Some(ModalResult::CloseCancel)
            }
            KeyCode::Up => {
                self.modal = Some(Modal::Palette {
                    query,
                    selected: selected.saturating_sub(1),
                });
                Some(ModalResult::Continue)
            }
            KeyCode::Down => {
                let max = items.len().saturating_sub(1);
                self.modal = Some(Modal::Palette {
                    query,
                    selected: if items.is_empty() {
                        0
                    } else {
                        (selected + 1).min(max)
                    },
                });
                Some(ModalResult::Continue)
            }
            KeyCode::Backspace => {
                let mut query = query;
                query.pop();
                self.modal = Some(Modal::Palette { query, selected: 0 });
                Some(ModalResult::Continue)
            }
            KeyCode::Enter => {
                if let Some(cmd) = items.get(selected).copied() {
                    self.run_palette_command(cmd);
                    Some(ModalResult::CloseSuccess)
                } else {
                    self.push_toast(ToastLevel::Warning, "No command selected.");
                    self.modal = Some(Modal::Palette { query, selected });
                    Some(ModalResult::Continue)
                }
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let mut query = query;
                query.push(c);
                self.modal = Some(Modal::Palette { query, selected: 0 });
                Some(ModalResult::Continue)
            }
            _ => {
                self.modal = Some(Modal::Palette { query, selected });
                Some(ModalResult::Continue)
            }
        }
    }
}
