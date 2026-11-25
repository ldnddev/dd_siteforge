use crate::fields::{COLOR_FIELDS, ColorField};
use crate::palette::Palette;
use crate::rgb::parse_hex_input;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeSaveTarget {
    Global,
    Local,
}

impl ThemeSaveTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Local => "local",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Global => Self::Local,
            Self::Local => Self::Global,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeEditorRow {
    Header(&'static str),
    Color(usize),
}

pub fn theme_editor_rows(fields: &[ColorField]) -> Vec<ThemeEditorRow> {
    let mut rows = Vec::new();
    let mut last_group = "";
    for (i, field) in fields.iter().enumerate() {
        if field.group != last_group {
            rows.push(ThemeEditorRow::Header(field.group));
            last_group = field.group;
        }
        rows.push(ThemeEditorRow::Color(i));
    }
    rows
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorKey {
    Up,
    Down,
    Left,
    Right,
    Tab,
    Enter,
    Esc,
    Backspace,
    Delete,
    Home,
    End,
    Char(char),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorOutcome {
    None,
    PaletteChanged,
    RequestSave,
    Closed { reverted: bool },
    HexError(String),
}

#[derive(Clone, Debug)]
pub struct ThemeEditor {
    pub palette: Palette,
    snapshot: Palette,
    pub fields: Vec<ColorField>,
    pub selected: usize,
    pub scroll: usize,
    pub channel: usize,
    pub hex_draft: String,
    /// Character index into [`Self::hex_draft`] while the hex field is open.
    pub hex_cursor: usize,
    pub editing_hex: bool,
    pub save_target: ThemeSaveTarget,
}

impl ThemeEditor {
    pub fn new(mut palette: Palette, extras: &[ColorField]) -> Self {
        let mut fields = COLOR_FIELDS.to_vec();
        for extra in extras {
            if !fields.iter().any(|f| f.key == extra.key) {
                fields.push(*extra);
            }
        }
        palette.ensure_extras(extras);
        let hex_draft = palette
            .get(fields[0].key)
            .map(|c| c.to_hex())
            .unwrap_or_else(|| "#000000".to_string());
        Self {
            snapshot: palette.clone(),
            palette,
            fields,
            selected: 0,
            scroll: 0,
            channel: 0,
            hex_cursor: hex_draft.chars().count(),
            hex_draft,
            editing_hex: false,
            save_target: ThemeSaveTarget::Global,
        }
    }

    pub fn selected_key(&self) -> &'static str {
        self.fields[self.selected.min(self.fields.len() - 1)].key
    }

    pub fn revert(&mut self) {
        self.palette = self.snapshot.clone();
        self.sync_hex_from_selected();
        self.editing_hex = false;
    }

    pub fn reset_builtin(&mut self) {
        let quotes = self.palette.header_quotes.clone();
        let source = self.palette.source;
        self.palette = Palette::builtin();
        self.palette.header_quotes = quotes;
        self.palette.source = source;
        self.palette.ensure_extras(&self.fields);
        self.sync_hex_from_selected();
        self.editing_hex = false;
    }

    fn mark_override(&mut self, key: &str) {
        self.palette.overrides.insert(key.to_string());
    }

    pub fn select(&mut self, selected: usize) {
        let last = self.fields.len().saturating_sub(1);
        self.selected = selected.min(last);
        if self.selected < self.scroll {
            self.scroll = self.selected;
        }
        if self.selected > self.scroll + 12 {
            self.scroll = self.selected.saturating_sub(12);
        }
        if !self.editing_hex {
            self.sync_hex_from_selected();
        }
    }

    fn sync_hex_from_selected(&mut self) {
        self.hex_draft = self
            .palette
            .get(self.selected_key())
            .map(|c| c.to_hex())
            .unwrap_or_else(|| "#000000".to_string());
        self.hex_cursor = self.hex_draft.chars().count();
    }

    /// Place the caret in the hex field and open it for typing.
    pub fn place_hex_cursor(&mut self, col: usize) {
        self.editing_hex = true;
        let n = self.hex_draft.chars().count();
        self.hex_cursor = col.min(n);
    }

    pub fn handle(&mut self, key: EditorKey, shift: bool) -> EditorOutcome {
        if self.editing_hex {
            return self.handle_hex(key);
        }
        match key {
            EditorKey::Esc => {
                self.revert();
                EditorOutcome::Closed { reverted: true }
            }
            EditorKey::Char('y') | EditorKey::Char('Y') | EditorKey::Char('s') => {
                EditorOutcome::RequestSave
            }
            EditorKey::Char('R') => {
                self.reset_builtin();
                EditorOutcome::PaletteChanged
            }
            EditorKey::Tab => {
                self.save_target = self.save_target.toggled();
                EditorOutcome::None
            }
            EditorKey::Enter => {
                self.editing_hex = true;
                self.hex_cursor = self.hex_draft.chars().count();
                EditorOutcome::None
            }
            EditorKey::Down | EditorKey::Char('j') => {
                self.select(self.selected.saturating_add(1));
                EditorOutcome::None
            }
            EditorKey::Up | EditorKey::Char('k') => {
                self.select(self.selected.saturating_sub(1));
                EditorOutcome::None
            }
            EditorKey::Char('[') => {
                self.channel = self.channel.saturating_sub(1);
                EditorOutcome::None
            }
            EditorKey::Char(']') => {
                self.channel = (self.channel + 1).min(2);
                EditorOutcome::None
            }
            EditorKey::Char('+') | EditorKey::Char('=') => {
                let step = if shift { 1 } else { 8 };
                self.nudge(step)
            }
            EditorKey::Char('-') | EditorKey::Char('_') => {
                let step = if shift { -1 } else { -8 };
                self.nudge(step)
            }
            EditorKey::Left | EditorKey::Char('h') => self.nudge(-8),
            EditorKey::Right | EditorKey::Char('l') => self.nudge(8),
            EditorKey::Home | EditorKey::End | EditorKey::Delete => EditorOutcome::None,
            EditorKey::Char('G') => {
                self.select(self.fields.len().saturating_sub(1));
                EditorOutcome::None
            }
            _ => EditorOutcome::None,
        }
    }

    fn nudge(&mut self, delta: i16) -> EditorOutcome {
        let key = self.selected_key();
        let current = self
            .palette
            .get(key)
            .unwrap_or(crate::rgb::Rgb::new(0, 0, 0));
        let next = current.nudge_channel(self.channel, delta);
        self.palette.set(key, next);
        self.mark_override(key);
        self.hex_draft = next.to_hex();
        self.hex_cursor = self.hex_draft.chars().count();
        self.editing_hex = false;
        EditorOutcome::PaletteChanged
    }

    fn handle_hex(&mut self, key: EditorKey) -> EditorOutcome {
        match key {
            EditorKey::Esc => {
                self.editing_hex = false;
                self.sync_hex_from_selected();
                EditorOutcome::None
            }
            EditorKey::Enter => match parse_hex_input(&self.hex_draft) {
                Ok(rgb) => {
                    let key = self.selected_key();
                    self.palette.set(key, rgb);
                    self.mark_override(key);
                    self.hex_draft = rgb.to_hex();
                    self.hex_cursor = self.hex_draft.chars().count();
                    self.editing_hex = false;
                    EditorOutcome::PaletteChanged
                }
                Err(err) => EditorOutcome::HexError(err.to_string()),
            },
            EditorKey::Backspace => {
                hex_backspace(&mut self.hex_draft, &mut self.hex_cursor);
                EditorOutcome::None
            }
            EditorKey::Delete => {
                hex_delete(&mut self.hex_draft, &mut self.hex_cursor);
                EditorOutcome::None
            }
            EditorKey::Left => {
                self.hex_cursor = self.hex_cursor.saturating_sub(1);
                EditorOutcome::None
            }
            EditorKey::Right => {
                let n = self.hex_draft.chars().count();
                if self.hex_cursor < n {
                    self.hex_cursor += 1;
                } else {
                    self.hex_cursor = n;
                }
                EditorOutcome::None
            }
            EditorKey::Home | EditorKey::Up => {
                self.hex_cursor = 0;
                EditorOutcome::None
            }
            EditorKey::End | EditorKey::Down => {
                self.hex_cursor = self.hex_draft.chars().count();
                EditorOutcome::None
            }
            EditorKey::Char(c) if c.is_ascii_hexdigit() || c == '#' => {
                hex_insert(
                    &mut self.hex_draft,
                    &mut self.hex_cursor,
                    c.to_ascii_uppercase(),
                );
                EditorOutcome::None
            }
            _ => EditorOutcome::None,
        }
    }

    /// Insert a pasted hex block at the caret in one assignment. A value that
    /// contains `#` replaces the draft. Other digits fill until `#` plus six.
    pub fn paste_hex(&mut self, raw: &str) {
        let text: String = raw
            .chars()
            .filter(|c| c.is_ascii_hexdigit() || *c == '#')
            .map(|c| c.to_ascii_uppercase())
            .collect();
        if text.is_empty() {
            return;
        }
        if text.contains('#') {
            self.hex_draft = text.chars().take(7).collect();
            self.hex_cursor = self.hex_draft.chars().count();
            return;
        }
        for ch in text.chars() {
            hex_insert(&mut self.hex_draft, &mut self.hex_cursor, ch);
        }
    }
}

fn hex_clamp(text: &str, cursor: usize) -> usize {
    cursor.min(text.chars().count())
}

fn hex_byte(text: &str, cursor: usize) -> usize {
    match text.char_indices().nth(cursor) {
        Some((byte, _)) => byte,
        None => text.len(),
    }
}

fn hex_insert(text: &mut String, cursor: &mut usize, ch: char) {
    if text.chars().count() >= 7 {
        return;
    }
    let at = hex_clamp(text, *cursor);
    let byte = hex_byte(text, at);
    text.insert(byte, ch);
    *cursor = at + 1;
}

fn hex_backspace(text: &mut String, cursor: &mut usize) {
    let at = hex_clamp(text, *cursor);
    if at == 0 {
        *cursor = 0;
        return;
    }
    let start = hex_byte(text, at - 1);
    let end = hex_byte(text, at);
    text.replace_range(start..end, "");
    *cursor = at - 1;
}

fn hex_delete(text: &mut String, cursor: &mut usize) {
    let at = hex_clamp(text, *cursor);
    let n = text.chars().count();
    if at >= n {
        *cursor = n;
        return;
    }
    let start = hex_byte(text, at);
    let end = hex_byte(text, at + 1);
    text.replace_range(start..end, "");
    *cursor = at;
}
