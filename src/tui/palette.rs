//! Command palette (`:` / `Ctrl+K`).
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tui) enum PaletteCommand {
    Export,
    Preview,
    Validate,
    Save,
    Help,
    Theme,
    Find,
    Insert,
    AddPage,
    BuildAssets,
    PageHealth,
    Redo,
    Quit,
}

impl PaletteCommand {
    fn all() -> &'static [Self] {
        &[
            Self::Export,
            Self::Preview,
            Self::Validate,
            Self::Save,
            Self::Help,
            Self::Theme,
            Self::Find,
            Self::Insert,
            Self::AddPage,
            Self::BuildAssets,
            Self::PageHealth,
            Self::Redo,
            Self::Quit,
        ]
    }

    pub(in crate::tui) fn title(self) -> &'static str {
        match self {
            Self::Export => "Export HTML",
            Self::Preview => "Preview in browser",
            Self::Validate => "Validate site",
            Self::Save => "Save",
            Self::Help => "Help",
            Self::Theme => "Theme",
            Self::Find => "Find in site",
            Self::Insert => "Insert component",
            Self::AddPage => "Add page",
            Self::BuildAssets => "Build CSS/JS",
            Self::PageHealth => "Page health",
            Self::Redo => "Redo",
            Self::Quit => "Quit",
        }
    }

    fn haystack(self) -> &'static str {
        match self {
            Self::Export => "Export HTML Shift+E",
            Self::Preview => "Preview in browser Shift+P",
            Self::Validate => "Validate site F3",
            Self::Save => "Save s",
            Self::Help => "Help F1",
            Self::Theme => "Theme F2",
            Self::Find => "Find in site ? Ctrl+F",
            Self::Insert => "Insert component /",
            Self::AddPage => "Add page Shift+A",
            Self::BuildAssets => "Build CSS/JS grunt lando Shift+B npm",
            Self::PageHealth => "Page health F4 HEAD seo alt",
            Self::Redo => "Redo Ctrl+R",
            Self::Quit => "Quit Ctrl+Q",
        }
    }
}

impl App {
    pub(in crate::tui) fn open_palette(&mut self) {
        self.modal = Some(Modal::Palette {
            query: String::new(),
            selected: 0,
        });
    }

    pub(in crate::tui) fn palette_items(&self, query: &str) -> Vec<PaletteCommand> {
        let q = query.trim();
        let mut scored: Vec<(PaletteCommand, i32)> = PaletteCommand::all()
            .iter()
            .copied()
            .filter_map(|cmd| fuzzy_score(q, cmd.haystack()).map(|s| (cmd, s)))
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.title().cmp(b.0.title())));
        scored.into_iter().map(|(c, _)| c).collect()
    }

    pub(in crate::tui) fn run_palette_command(&mut self, cmd: PaletteCommand) {
        match cmd {
            PaletteCommand::Export => self.begin_export_flow(),
            PaletteCommand::Preview => self.begin_preview_flow(),
            PaletteCommand::Validate => self.open_validation_modal(),
            PaletteCommand::Save => self.begin_save_prompt(),
            PaletteCommand::Help => {
                self.overlay = Some(Overlay::Help { scroll: 0 });
            }
            PaletteCommand::Theme => {
                self.theme_editor = Some(ldnddev_theme::ThemeEditor::new(
                    super::theme::palette_from_theme(&self.theme),
                    super::theme::extra_theme_fields(),
                ));
                self.overlay = Some(Overlay::Theme { scroll: 0 });
            }
            PaletteCommand::Find => self.open_find(),
            PaletteCommand::Insert => self.open_component_picker(),
            PaletteCommand::AddPage => {
                self.begin_add_page();
            }
            PaletteCommand::BuildAssets => self.start_asset_build(),
            PaletteCommand::PageHealth => self.open_page_health(),
            PaletteCommand::Redo => self.redo_last(),
            PaletteCommand::Quit => self.request_quit(),
        }
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
