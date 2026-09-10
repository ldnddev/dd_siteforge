//! FormEdit textarea layout helpers.
use super::*;

/// One visual (wrapped) row of a textarea.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VisualRow {
    /// Character offset in `value` of the first character on this row.
    pub start: usize,
    /// Character count on this row (excludes a trailing logical newline).
    pub len: usize,
}

/// Layout used to paint, overlay the caret, and map clicks inside a textarea.
#[derive(Debug, Clone)]
pub(super) struct TextareaLayout {
    pub display: String,
    pub first_visible_row: usize,
    pub total_rows: usize,
    pub wrap_width: u16,
    pub has_scrollbar: bool,
    pub visible_rows: usize,
}

pub(super) fn focused_field_virtual_rows(state: &editform::EditFormState) -> (u16, u16) {
    let mut y: u16 = 0;
    for (idx, field) in state.form.fields.iter().enumerate() {
        if !state.field_visible(field) {
            continue;
        }
        let content_rows: u16 = match &field.kind {
            editform::FieldKind::Textarea { rows, .. } => {
                textarea_display_rows(
                    state.get(field.id),
                    (*rows).max(1),
                    None,
                    TEXTAREA_MAX_DISPLAY_ROWS,
                )
            }
            editform::FieldKind::SubForm { .. } => {
                let items_len = state
                    .sub_state
                    .get(field.id)
                    .map(|v| v.len())
                    .unwrap_or(0);
                (1 + items_len.max(1)) as u16
            }
            _ => 1,
        };
        let box_height = content_rows.saturating_add(2);
        let entry_height = 1u16.saturating_add(box_height).saturating_add(1);
        if idx == state.focused_field {
            return (y, y.saturating_add(1).saturating_add(box_height));
        }
        y = y.saturating_add(entry_height);
    }
    (0, 0)
}

pub(super) fn textarea_display_rows(
    value: &str,
    base_rows: u16,
    wrap_width: Option<u16>,
    max_rows: u16,
) -> u16 {
    let content_rows = textarea_visual_line_count(value, wrap_width).min(u16::MAX as usize) as u16;
    base_rows
        .max(content_rows.max(1))
        .min(max_rows.max(1))
}

pub(super) fn textarea_max_rows_for_window(content_height: u16) -> u16 {
    content_height
        .saturating_sub(3)
        .max(1)
        .min(TEXTAREA_MAX_DISPLAY_ROWS)
}

pub(super) fn textarea_visual_line_count(value: &str, wrap_width: Option<u16>) -> usize {
    textarea_visual_rows(value, wrap_width).len().max(1)
}

/// Wrap logical lines at `wrap_width` characters. `None` keeps hard newlines only.
pub(super) fn textarea_visual_rows(value: &str, wrap_width: Option<u16>) -> Vec<VisualRow> {
    let logical = input_lines_preserve(value);
    let width = wrap_width.map(|w| w.max(1) as usize);
    let mut rows = Vec::new();
    let mut offset = 0usize;

    for (i, line) in logical.iter().enumerate() {
        let line_len = line.chars().count();
        if let Some(w) = width {
            if line_len == 0 {
                rows.push(VisualRow {
                    start: offset,
                    len: 0,
                });
            } else {
                let mut col = 0;
                while col < line_len {
                    let take = (line_len - col).min(w);
                    rows.push(VisualRow {
                        start: offset + col,
                        len: take,
                    });
                    col += take;
                }
            }
        } else {
            rows.push(VisualRow {
                start: offset,
                len: line_len,
            });
        }
        offset += line_len;
        if i + 1 < logical.len() {
            offset += 1;
        }
    }
    if rows.is_empty() {
        rows.push(VisualRow { start: 0, len: 0 });
    }
    rows
}

fn visual_row_text(value: &str, row: VisualRow) -> String {
    value.chars().skip(row.start).take(row.len).collect()
}

/// `(visual_row, visual_col)` for `cursor_pos`. A wrap boundary sits at
/// column 0 of the next visual row.
pub(super) fn textarea_cursor_visual(
    value: &str,
    cursor_pos: usize,
    wrap_width: Option<u16>,
) -> (usize, usize) {
    let rows = textarea_visual_rows(value, wrap_width);
    let pos = cursor_pos.min(value.chars().count());
    let mut chosen = 0usize;
    for (i, row) in rows.iter().enumerate() {
        if pos >= row.start {
            chosen = i;
        }
    }
    let row = rows[chosen];
    let col = pos.saturating_sub(row.start).min(row.len);
    (chosen, col)
}

pub(super) fn textarea_line_home(
    value: &str,
    cursor_pos: usize,
    wrap_width: Option<u16>,
) -> usize {
    let rows = textarea_visual_rows(value, wrap_width);
    let (row, _) = textarea_cursor_visual(value, cursor_pos, wrap_width);
    rows.get(row).map(|r| r.start).unwrap_or(0)
}

pub(super) fn textarea_line_end(value: &str, cursor_pos: usize, wrap_width: Option<u16>) -> usize {
    let rows = textarea_visual_rows(value, wrap_width);
    let (row, _) = textarea_cursor_visual(value, cursor_pos, wrap_width);
    rows.get(row).map(|r| r.start + r.len).unwrap_or(0)
}

/// Wrap width used inside a bordered textarea box. Reserves one column for
/// a scrollbar when wrapping at full inner width overflows the window.
pub(super) fn textarea_wrap_width(value: &str, inner_w: u16, inner_h: u16) -> u16 {
    let visible = inner_h.max(1) as usize;
    let full = inner_w.max(1);
    let total_full = textarea_visual_line_count(value, Some(full));
    if total_full > visible && inner_w > 1 {
        inner_w.saturating_sub(1).max(1)
    } else {
        full
    }
}

pub(super) fn textarea_wrap_width_from_box(value: &str, box_rect: Rect) -> Option<u16> {
    if box_rect.width < 3 || box_rect.height < 3 {
        return None;
    }
    let inner_w = box_rect.width.saturating_sub(2);
    let inner_h = box_rect.height.saturating_sub(2);
    Some(textarea_wrap_width(value, inner_w, inner_h))
}

pub(super) fn textarea_layout(
    value: &str,
    cursor_pos: usize,
    focused: bool,
    inner_w: u16,
    inner_h: u16,
) -> TextareaLayout {
    let visible_rows = inner_h.max(1) as usize;
    let wrap_width = textarea_wrap_width(value, inner_w, inner_h);
    let has_scrollbar = wrap_width < inner_w.max(1);
    let (display, first_visible_row, total_rows) =
        render_textarea_display_window(value, cursor_pos, focused, visible_rows, Some(wrap_width));
    TextareaLayout {
        display,
        first_visible_row,
        total_rows,
        wrap_width,
        has_scrollbar,
        visible_rows,
    }
}

/// Map a click inside the bordered textarea `box_rect` to a caret offset.
/// Returns `None` when the click is on the border or scrollbar.
pub(super) fn textarea_cursor_from_click(
    value: &str,
    box_rect: Rect,
    cursor_pos: usize,
    focused: bool,
    click_x: u16,
    click_y: u16,
) -> Option<usize> {
    if box_rect.width < 3 || box_rect.height < 3 {
        return None;
    }
    let inner_x = box_rect.x.saturating_add(1);
    let inner_y = box_rect.y.saturating_add(1);
    let inner_w = box_rect.width.saturating_sub(2);
    let inner_h = box_rect.height.saturating_sub(2);
    let layout = textarea_layout(value, cursor_pos, focused, inner_w, inner_h);
    let text_rect = Rect {
        x: inner_x,
        y: inner_y,
        width: layout.wrap_width,
        height: inner_h,
    };
    if !contains(text_rect, click_x, click_y) {
        return None;
    }
    let rel_x = (click_x - text_rect.x) as usize;
    let rel_y = (click_y - text_rect.y) as usize;
    let visual_row = layout.first_visible_row.saturating_add(rel_y);
    let rows = textarea_visual_rows(value, Some(layout.wrap_width));
    if visual_row >= rows.len() {
        return Some(value.chars().count());
    }
    let row = rows[visual_row];
    Some(row.start + rel_x.min(row.len))
}

#[cfg(test)]
pub(super) fn render_textarea_display(
    value: &str,
    cursor_pos: usize,
    focused: bool,
    visible_rows: usize,
) -> String {
    render_textarea_display_window(value, cursor_pos, focused, visible_rows, None).0
}

/// Glyph shown at column `col` of `line`. Space if that cell is empty
/// (caret at end of the line, or the line is shorter than `col`).
fn overlay_glyph_at(line: &str, col: usize) -> char {
    line.chars().nth(col).filter(|c| *c != '\n').unwrap_or(' ')
}

/// Map a FormEdit text caret to a 1-cell overlay inside the bordered `box_rect`.
/// Single-line fields have no horizontal scroll, so a caret past the inner
/// width overlays the last visible glyph. Textareas wrap, so the caret sits
/// on the visual row/col of the wrapped layout.
pub(super) fn form_input_cursor_cell(
    kind: &editform::FieldKind,
    value: &str,
    cursor_pos: usize,
    box_rect: Rect,
) -> Option<(u16, u16, char)> {
    if box_rect.width < 3 || box_rect.height < 3 {
        return None;
    }
    let inner_x = box_rect.x.saturating_add(1);
    let inner_y = box_rect.y.saturating_add(1);
    let inner_w = box_rect.width.saturating_sub(2);
    let inner_h = box_rect.height.saturating_sub(2);
    if inner_w == 0 || inner_h == 0 {
        return None;
    }

    let pos = cursor_pos.min(value.chars().count());

    match kind {
        editform::FieldKind::Text { .. } | editform::FieldKind::Url { .. } => {
            let col = (pos as u16).min(inner_w.saturating_sub(1));
            let ch = overlay_glyph_at(value, col as usize);
            Some((inner_x.saturating_add(col), inner_y, ch))
        }
        editform::FieldKind::Textarea { .. } => {
            let layout = textarea_layout(value, pos, true, inner_w, inner_h);
            let (cursor_row, cursor_col) =
                textarea_cursor_visual(value, pos, Some(layout.wrap_width));
            if cursor_row < layout.first_visible_row {
                return None;
            }
            let row_in_view = (cursor_row - layout.first_visible_row) as u16;
            if row_in_view >= inner_h {
                return None;
            }
            let text_w = layout.wrap_width;
            if text_w == 0 {
                return None;
            }
            let col = (cursor_col as u16).min(text_w.saturating_sub(1));
            let rows = textarea_visual_rows(value, Some(layout.wrap_width));
            let line = rows
                .get(cursor_row)
                .map(|row| visual_row_text(value, *row))
                .unwrap_or_default();
            let ch = overlay_glyph_at(&line, col as usize);
            Some((
                inner_x.saturating_add(col),
                inner_y.saturating_add(row_in_view),
                ch,
            ))
        }
        _ => None,
    }
}

pub(super) fn render_textarea_display_window(
    value: &str,
    cursor_pos: usize,
    focused: bool,
    visible_rows: usize,
    wrap_width: Option<u16>,
) -> (String, usize, usize) {
    let visible_rows = visible_rows.max(1);
    let rows = textarea_visual_rows(value, wrap_width);
    let (cursor_row, _) = textarea_cursor_visual(value, cursor_pos, wrap_width);
    let cursor_row = cursor_row.min(rows.len().saturating_sub(1));
    let start = if focused {
        cursor_row.saturating_sub(visible_rows.saturating_sub(1))
    } else {
        0
    };
    let end = (start + visible_rows).min(rows.len());

    let mut display = Vec::with_capacity(visible_rows);
    for row in rows.iter().take(end).skip(start) {
        display.push(visual_row_text(value, *row));
    }
    while display.len() < visible_rows {
        display.push(String::new());
    }
    (display.join("\n"), start, rows.len())
}

pub(super) fn render_textarea_scrollbar(
    frame: &mut ratatui::Frame,
    area: Rect,
    first_visible_row: usize,
    visible_rows: usize,
    total_rows: usize,
    scrollbar_color: Color,
    background: Color,
) {
    paint_scrollbar(
        frame,
        area,
        first_visible_row,
        total_rows,
        visible_rows,
        scrollbar_color,
        scrollbar_color,
        background,
    );
}

pub(super) fn textarea_move_cursor_vertical(
    value: &str,
    cursor_pos: usize,
    row_delta: isize,
    wrap_width: Option<u16>,
) -> usize {
    let rows = textarea_visual_rows(value, wrap_width);
    let (current_row, current_col) = textarea_cursor_visual(value, cursor_pos, wrap_width);
    let target_row = current_row
        .saturating_add_signed(row_delta)
        .min(rows.len().saturating_sub(1));
    let row = rows[target_row];
    row.start + current_col.min(row.len)
}

impl App {
    /// Wrap width of the focused textarea from the last-painted box, if any.
    pub(super) fn focused_textarea_wrap_width(&self) -> Option<u16> {
        let Some(Modal::FormEdit { state, .. }) = &self.modal else {
            return None;
        };
        let field = state.form.fields.get(state.focused_field)?;
        if !matches!(field.kind, editform::FieldKind::Textarea { .. }) {
            return None;
        }
        let areas = self.modal_field_areas.borrow();
        let (_, box_rect) = areas
            .iter()
            .find(|(idx, _)| *idx == state.focused_field)?;
        textarea_wrap_width_from_box(state.get(field.id), *box_rect)
    }
}

/// Compute a new scroll offset that keeps the focused field in view given
/// a conservative estimate of the content window height. 16 rows covers the
/// common case of an 80% / 80% modal on a standard terminal.
pub(super) fn auto_scroll_for_focus(state: &editform::EditFormState, current_scroll: u16) -> u16 {
    const ESTIMATED_VISIBLE: u16 = 16;
    let (top, bottom) = focused_field_virtual_rows(state);
    if top < current_scroll {
        top
    } else if bottom > current_scroll.saturating_add(ESTIMATED_VISIBLE) {
        bottom.saturating_sub(ESTIMATED_VISIBLE)
    } else {
        current_scroll
    }
}
