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
            editform::FieldKind::Textarea { rows, .. } => textarea_display_rows(
                state.get(field.id),
                (*rows).max(1),
                None,
                TEXTAREA_MAX_DISPLAY_ROWS,
            ),
            editform::FieldKind::SubForm { .. } => {
                let items_len = state.sub_state.get(field.id).map(|v| v.len()).unwrap_or(0);
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
    base_rows.max(content_rows.max(1)).min(max_rows.max(1))
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

/// Wrap logical lines at `wrap_width` characters, breaking on whitespace when
/// a word fits. Words longer than the width hard-break. `None` keeps hard newlines only.
pub(super) fn textarea_visual_rows(value: &str, wrap_width: Option<u16>) -> Vec<VisualRow> {
    let logical = input_lines_preserve(value);
    let width = wrap_width.map(|w| w.max(1) as usize);
    let mut rows = Vec::new();
    let mut offset = 0usize;

    for (i, line) in logical.iter().enumerate() {
        let line_len = line.chars().count();
        if let Some(w) = width {
            wrap_logical_line(line, w, offset, &mut rows);
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

/// Word-wrap one logical line into `rows`. Every scalar belongs to exactly one row.
fn wrap_logical_line(line: &str, width: usize, offset: usize, rows: &mut Vec<VisualRow>) {
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        rows.push(VisualRow {
            start: offset,
            len: 0,
        });
        return;
    }
    let mut start = 0usize;
    while start < chars.len() {
        let remaining = chars.len() - start;
        if remaining <= width {
            rows.push(VisualRow {
                start: offset + start,
                len: remaining,
            });
            break;
        }
        let window = &chars[start..start + width];
        if let Some(rel) = window.iter().rposition(|c| c.is_whitespace()) {
            let take = if rel == 0 { width } else { rel + 1 };
            rows.push(VisualRow {
                start: offset + start,
                len: take,
            });
            start += take;
        } else {
            rows.push(VisualRow {
                start: offset + start,
                len: width,
            });
            start += width;
        }
    }
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

pub(super) fn textarea_line_home(value: &str, cursor_pos: usize, wrap_width: Option<u16>) -> usize {
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

pub(super) fn textarea_clamp_vscroll(
    vscroll: usize,
    visible_rows: usize,
    total_rows: usize,
) -> usize {
    vscroll.min(total_rows.saturating_sub(visible_rows.max(1)))
}

/// Keep `cursor_row` inside the window. Does not pin it to the last line
/// when it is already visible.
pub(super) fn textarea_ensure_cursor_visible(
    vscroll: usize,
    cursor_row: usize,
    visible_rows: usize,
    total_rows: usize,
) -> usize {
    let visible_rows = visible_rows.max(1);
    let mut start = textarea_clamp_vscroll(vscroll, visible_rows, total_rows);
    if cursor_row < start {
        start = cursor_row;
    } else if cursor_row >= start.saturating_add(visible_rows) {
        start = cursor_row.saturating_add(1).saturating_sub(visible_rows);
    }
    textarea_clamp_vscroll(start, visible_rows, total_rows)
}

pub(super) fn textarea_layout(
    value: &str,
    cursor_pos: usize,
    focused: bool,
    inner_w: u16,
    inner_h: u16,
    vscroll: usize,
    follow_cursor: bool,
) -> TextareaLayout {
    let visible_rows = inner_h.max(1) as usize;
    let wrap_width = textarea_wrap_width(value, inner_w, inner_h);
    let has_scrollbar = wrap_width < inner_w.max(1);
    let (_, first_visible_row, total_rows) = render_textarea_display_window_at(
        value,
        cursor_pos,
        focused,
        visible_rows,
        Some(wrap_width),
        vscroll,
        follow_cursor,
    );
    TextareaLayout {
        first_visible_row,
        total_rows,
        wrap_width,
        has_scrollbar,
        visible_rows,
    }
}

pub(super) fn clamp_to_rect(rect: Rect, x: u16, y: u16) -> (u16, u16) {
    if rect.width == 0 || rect.height == 0 {
        return (rect.x, rect.y);
    }
    let max_x = rect.x.saturating_add(rect.width.saturating_sub(1));
    let max_y = rect.y.saturating_add(rect.height.saturating_sub(1));
    (x.clamp(rect.x, max_x), y.clamp(rect.y, max_y))
}

/// Horizontal origin so `cursor_pos` stays inside a single-line field of
/// `inner_w` columns. `cursor_pos` is a Unicode scalar index.
pub(super) fn single_line_hscroll(cursor_pos: usize, inner_w: u16) -> usize {
    let w = inner_w.max(1) as usize;
    cursor_pos.saturating_sub(w.saturating_sub(1))
}

/// Visible slice of a single-line field and the char offset it starts at.
pub(super) fn single_line_visible(value: &str, cursor_pos: usize, inner_w: u16) -> (usize, String) {
    let pos = cursor_pos.min(value.chars().count());
    let scroll = single_line_hscroll(pos, inner_w);
    let visible: String = value
        .chars()
        .skip(scroll)
        .take(inner_w.max(1) as usize)
        .collect();
    (scroll, visible)
}

/// Map a click inside a bordered single-line input `box_rect` to a caret
/// offset. Any click in the box (including the top/bottom border of the
/// 3-row field) maps `x` from the inner origin plus the caret-following
/// horizontal scroll. Past the last character clamps to the end. Returns
/// `None` when the box is too small to edit.
pub(super) fn text_cursor_from_click(
    value: &str,
    box_rect: Rect,
    cursor_pos: usize,
    click_x: u16,
    click_y: u16,
) -> Option<usize> {
    if box_rect.width < 3 || box_rect.height < 3 {
        return None;
    }
    if !contains(box_rect, click_x, click_y) {
        return None;
    }
    let inner_x = box_rect.x.saturating_add(1);
    let inner_w = box_rect.width.saturating_sub(2);
    let end = value.chars().count();
    let scroll = single_line_hscroll(cursor_pos.min(end), inner_w);
    if click_x <= inner_x {
        return Some(scroll);
    }
    let rel_x = (click_x - inner_x) as usize;
    Some(scroll.saturating_add(rel_x).min(end))
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
    vscroll: usize,
    follow_cursor: bool,
) -> Option<usize> {
    if box_rect.width < 3 || box_rect.height < 3 {
        return None;
    }
    let inner_x = box_rect.x.saturating_add(1);
    let inner_y = box_rect.y.saturating_add(1);
    let inner_w = box_rect.width.saturating_sub(2);
    let inner_h = box_rect.height.saturating_sub(2);
    let layout = textarea_layout(
        value,
        cursor_pos,
        focused,
        inner_w,
        inner_h,
        vscroll,
        follow_cursor,
    );
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
/// Single-line fields scroll horizontally so the caret stays in view.
/// Textareas wrap, so the caret sits on the visual row/col of the wrapped layout.
pub(super) fn form_input_cursor_cell(
    kind: &editform::FieldKind,
    value: &str,
    cursor_pos: usize,
    box_rect: Rect,
    textarea_vscroll: usize,
    textarea_follow: bool,
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
            let (scroll, visible) = single_line_visible(value, pos, inner_w);
            let col = (pos.saturating_sub(scroll) as u16).min(inner_w.saturating_sub(1));
            let ch = overlay_glyph_at(&visible, col as usize);
            Some((inner_x.saturating_add(col), inner_y, ch))
        }
        editform::FieldKind::Textarea { .. } => {
            let layout = textarea_layout(
                value,
                pos,
                true,
                inner_w,
                inner_h,
                textarea_vscroll,
                textarea_follow,
            );
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

#[cfg(test)]
pub(super) fn render_textarea_display_window(
    value: &str,
    cursor_pos: usize,
    focused: bool,
    visible_rows: usize,
    wrap_width: Option<u16>,
) -> (String, usize, usize) {
    render_textarea_display_window_at(
        value,
        cursor_pos,
        focused,
        visible_rows,
        wrap_width,
        0,
        focused,
    )
}

pub(super) fn render_textarea_display_window_at(
    value: &str,
    cursor_pos: usize,
    focused: bool,
    visible_rows: usize,
    wrap_width: Option<u16>,
    vscroll: usize,
    follow_cursor: bool,
) -> (String, usize, usize) {
    let visible_rows = visible_rows.max(1);
    let rows = textarea_visual_rows(value, wrap_width);
    let (cursor_row, _) = textarea_cursor_visual(value, cursor_pos, wrap_width);
    let cursor_row = cursor_row.min(rows.len().saturating_sub(1));
    let start = if !focused {
        0
    } else if follow_cursor {
        textarea_ensure_cursor_visible(vscroll, cursor_row, visible_rows, rows.len())
    } else {
        textarea_clamp_vscroll(vscroll, visible_rows, rows.len())
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

/// Caret position at the end of `s` (Unicode scalar count, matching visual layout).
pub(super) fn text_end(s: &str) -> usize {
    s.chars().count()
}

fn char_byte_index(s: &str, char_pos: usize) -> usize {
    s.char_indices()
        .nth(char_pos)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// Insert `insert` at `char_pos`. Returns the new string and caret (char index after the insert).
pub(super) fn insert_at_char(s: &str, char_pos: usize, insert: &str) -> (String, usize) {
    let pos = char_pos.min(text_end(s));
    let byte = char_byte_index(s, pos);
    let mut out = String::with_capacity(s.len() + insert.len());
    out.push_str(&s[..byte]);
    out.push_str(insert);
    out.push_str(&s[byte..]);
    (out, pos + insert.chars().count())
}

/// Delete the scalar before `char_pos`. Returns the new string and caret.
pub(super) fn delete_char_before(s: &str, char_pos: usize) -> (String, usize) {
    let pos = char_pos.min(text_end(s));
    if pos == 0 {
        return (s.to_string(), 0);
    }
    delete_char_range(s, pos - 1, pos)
}

/// Delete the scalar at `char_pos` (forward delete). Caret stays put.
pub(super) fn delete_char_after(s: &str, char_pos: usize) -> (String, usize) {
    let pos = char_pos.min(text_end(s));
    if pos >= text_end(s) {
        return (s.to_string(), pos);
    }
    delete_char_range(s, pos, pos + 1)
}

/// Delete scalars in `[from, to)`. Caret lands at `from`.
pub(super) fn delete_char_range(s: &str, from: usize, to: usize) -> (String, usize) {
    let len = text_end(s);
    let from = from.min(to).min(len);
    let to = to.max(from).min(len);
    if from == to {
        return (s.to_string(), from);
    }
    let start = char_byte_index(s, from);
    let end = char_byte_index(s, to);
    let mut out = String::with_capacity(s.len() - (end - start));
    out.push_str(&s[..start]);
    out.push_str(&s[end..]);
    (out, from)
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '\'')
}

/// Caret at the start of the word before `char_pos`.
pub(super) fn word_left(s: &str, char_pos: usize) -> usize {
    let chars: Vec<char> = s.chars().collect();
    let mut i = char_pos.min(chars.len());
    while i > 0 && !is_word_char(chars[i - 1]) {
        i -= 1;
    }
    while i > 0 && is_word_char(chars[i - 1]) {
        i -= 1;
    }
    i
}

/// Caret at the start of the next word after `char_pos`.
pub(super) fn word_right(s: &str, char_pos: usize) -> usize {
    let chars: Vec<char> = s.chars().collect();
    let mut i = char_pos.min(chars.len());
    while i < chars.len() && is_word_char(chars[i]) {
        i += 1;
    }
    while i < chars.len() && !is_word_char(chars[i]) {
        i += 1;
    }
    i
}

/// Highlight span between `anchor` and `caret`, or `None` when empty.
pub(super) fn selection_range(anchor: Option<usize>, caret: usize) -> Option<(usize, usize)> {
    let a = anchor?;
    let from = a.min(caret);
    let to = a.max(caret);
    if from == to { None } else { Some((from, to)) }
}

/// Word or whitespace run under `char_pos` (double-click). Past the last
/// character uses the last scalar.
pub(super) fn word_bounds_at(s: &str, char_pos: usize) -> (usize, usize) {
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return (0, 0);
    }
    let idx = char_pos.min(chars.len() - 1);
    let word = is_word_char(chars[idx]);
    let mut start = idx;
    let mut end = idx + 1;
    while start > 0 && is_word_char(chars[start - 1]) == word {
        start -= 1;
    }
    while end < chars.len() && is_word_char(chars[end]) == word {
        end += 1;
    }
    (start, end)
}

/// Split `text` (a slice of the field starting at `text_start` chars) into
/// normal / selected / normal spans.
pub(super) fn spans_with_selection(
    text: &str,
    text_start: usize,
    sel: Option<(usize, usize)>,
    normal: Style,
    selected: Style,
) -> Vec<ratatui::text::Span<'static>> {
    use ratatui::text::Span;
    let len = text.chars().count();
    let Some((from, to)) = sel else {
        return vec![Span::styled(text.to_string(), normal)];
    };
    if to <= text_start || from >= text_start + len {
        return vec![Span::styled(text.to_string(), normal)];
    }
    let rel_from = from.saturating_sub(text_start).min(len);
    let rel_to = to.saturating_sub(text_start).min(len);
    let mut spans = Vec::new();
    if rel_from > 0 {
        spans.push(Span::styled(
            text.chars().take(rel_from).collect::<String>(),
            normal,
        ));
    }
    if rel_to > rel_from {
        spans.push(Span::styled(
            text.chars()
                .skip(rel_from)
                .take(rel_to - rel_from)
                .collect::<String>(),
            selected,
        ));
    }
    if rel_to < len {
        spans.push(Span::styled(
            text.chars().skip(rel_to).collect::<String>(),
            normal,
        ));
    }
    if spans.is_empty() {
        spans.push(Span::styled(String::new(), normal));
    }
    spans
}

pub(super) fn begin_shift_selection(anchor: &mut Option<usize>, caret: usize) {
    if anchor.is_none() {
        *anchor = Some(caret);
    }
}

/// Normalize a clipboard dump: CRLF/CR become `\n`. Single-line fields flatten newlines to spaces.
pub(super) fn sanitize_paste(text: &str, multiline: bool) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    if multiline {
        normalized
    } else {
        normalized.replace('\n', " ")
    }
}

impl App {
    /// Map a pointer position to `(field_idx, caret)` inside a text/url/textarea box.
    /// When `clamp` is true, a point outside the box is mapped to the nearest cell
    /// (mouse-drag selection).
    pub(super) fn form_text_hit(&self, col: u16, row: u16, clamp: bool) -> Option<(usize, usize)> {
        let areas = self.modal_field_areas.borrow();
        let Some(Modal::FormEdit {
            state, cursor_pos, ..
        }) = &self.modal
        else {
            return None;
        };
        let map = |idx: usize, rect: Rect, x: u16, y: u16| -> Option<(usize, usize)> {
            let field = state.form.fields.get(idx)?;
            let value = state.get(field.id);
            let pos = match field.kind {
                editform::FieldKind::Textarea { .. } => {
                    let focused = idx == state.focused_field;
                    let vscroll = *self.form_textarea_vscroll.borrow();
                    let follow = focused && self.form_textarea_follow_cursor.get();
                    textarea_cursor_from_click(
                        value,
                        rect,
                        *cursor_pos,
                        focused,
                        x,
                        y,
                        vscroll,
                        follow,
                    )?
                }
                editform::FieldKind::Text { .. } | editform::FieldKind::Url { .. } => {
                    let caret = if idx == state.focused_field {
                        *cursor_pos
                    } else {
                        0
                    };
                    text_cursor_from_click(value, rect, caret, x, y)?
                }
                _ => return None,
            };
            Some((idx, pos))
        };
        if clamp {
            let (idx, rect) = areas.iter().find(|(i, _)| *i == state.focused_field)?;
            let (x, y) = clamp_to_rect(*rect, col, row);
            return map(*idx, *rect, x, y);
        }
        areas.iter().find_map(|(idx, rect)| {
            if !contains(*rect, col, row) {
                return None;
            }
            map(*idx, *rect, col, row)
        })
    }

    /// Scroll the focused textarea's independent view. Returns true when the
    /// field overflows (the gesture is consumed even at a scroll bound).
    pub(super) fn scroll_focused_textarea_view(&self, delta: i32) -> bool {
        let Some((visible, total)) = self.focused_textarea_view_size() else {
            return false;
        };
        if total <= visible {
            return false;
        }
        let current = *self.form_textarea_vscroll.borrow();
        let next = (current as i32 + delta).max(0) as usize;
        *self.form_textarea_vscroll.borrow_mut() = textarea_clamp_vscroll(next, visible, total);
        self.form_textarea_follow_cursor.set(false);
        true
    }

    fn focused_textarea_view_size(&self) -> Option<(usize, usize)> {
        let track = *self.textarea_scrollbar_track.borrow();
        if track.total > track.visible && track.visible > 0 {
            return Some((track.visible, track.total));
        }
        let Some(Modal::FormEdit {
            state, cursor_pos, ..
        }) = &self.modal
        else {
            return None;
        };
        let field = state.form.fields.get(state.focused_field)?;
        if !matches!(field.kind, editform::FieldKind::Textarea { .. }) {
            return None;
        }
        let areas = self.modal_field_areas.borrow();
        let (_, box_rect) = areas.iter().find(|(idx, _)| *idx == state.focused_field)?;
        if box_rect.width < 3 || box_rect.height < 3 {
            return None;
        }
        let inner_w = box_rect.width.saturating_sub(2);
        let inner_h = box_rect.height.saturating_sub(2);
        let layout = textarea_layout(
            state.get(field.id),
            *cursor_pos,
            true,
            inner_w,
            inner_h,
            *self.form_textarea_vscroll.borrow(),
            false,
        );
        Some((layout.visible_rows, layout.total_rows))
    }

    pub(super) fn pointer_over_focused_textarea(&self, col: u16, row: u16) -> bool {
        let track = *self.textarea_scrollbar_track.borrow();
        if contains(track.rect, col, row) {
            return true;
        }
        let Some(Modal::FormEdit { state, .. }) = &self.modal else {
            return false;
        };
        let areas = self.modal_field_areas.borrow();
        areas
            .iter()
            .any(|(idx, rect)| *idx == state.focused_field && contains(*rect, col, row))
    }

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
        let (_, box_rect) = areas.iter().find(|(idx, _)| *idx == state.focused_field)?;
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
