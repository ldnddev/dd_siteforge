//! Clipboard and bracketed-paste text.
//!
//! A paste is one string. Callers insert that string in one edit. They do not
//! walk it as typed characters.

/// Clipboard or bracketed-paste text.
///
/// Newlines stay when `multiline` is set. A carriage return becomes a newline
/// in that case, and a single space otherwise. A tab stays on a multiline
/// field and becomes a space on one line. Other controls, including Escape,
/// are dropped so the paste cannot run as keystrokes.
pub fn sanitize_paste(raw: &str, multiline: bool) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push(if multiline { '\n' } else { ' ' });
            }
            '\n' => out.push(if multiline { '\n' } else { ' ' }),
            '\t' => out.push(if multiline { '\t' } else { ' ' }),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}

/// Hex field paste: keep `#` and hex digits, and write letters in uppercase.
pub fn sanitize_hex_paste(raw: &str) -> String {
    sanitize_paste(raw, false)
        .chars()
        .filter(|c| c.is_ascii_hexdigit() || *c == '#')
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line_collapses_newlines_and_drops_escape() {
        assert_eq!(sanitize_paste("a\r\nb\u{1b}c\td", false), "a bc d");
    }

    #[test]
    fn multiline_keeps_newline_and_tab() {
        assert_eq!(sanitize_paste("a\r\nb\u{1b}\tc", true), "a\nb\tc");
    }

    #[test]
    fn hex_paste_keeps_hash_and_digits() {
        assert_eq!(sanitize_hex_paste(" #aa\nzz12 "), "#AA12");
    }
}
