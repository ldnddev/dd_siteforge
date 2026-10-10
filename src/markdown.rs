//! Markdown for copy fields.
//!
//! Attribute blocks use pulldown-cmark's heading syntax, `{.class #id key=value}`.
//! On an image or link the block follows that element immediately, with at most
//! one space: `![alt](url){.dd-img}`. A block of only bare words, such as
//! `{note}`, stays as text.

use pulldown_cmark::{CowStr, Event, LinkType, Options, Parser, Tag, TagEnd, html};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Attrs {
    id: Option<String>,
    classes: Vec<String>,
    /// Valueless attributes are stored as `None` and rendered as `name=""`.
    attrs: Vec<(String, Option<String>)>,
}

pub(crate) fn to_html(input: &str) -> String {
    let events = parse(input);
    let rewritten = apply_inline_attrs(&events);
    let mut out = String::new();
    html::push_html(&mut out, rewritten.into_iter());
    out
}

pub(crate) fn to_plain(input: &str) -> String {
    let events = parse(input);
    let events = strip_inline_attr_text(&events);
    let mut out = String::new();
    for event in events {
        match event {
            Event::Text(t) | Event::Code(t) => out.push_str(&t),
            Event::SoftBreak | Event::HardBreak => out.push(' '),
            Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::Item
                | TagEnd::CodeBlock
                | TagEnd::Table
                | TagEnd::TableHead
                | TagEnd::TableRow
                | TagEnd::BlockQuote(_),
            ) => out.push(' '),
            _ => {}
        }
    }
    collapse_ws(&out)
}

/// Drop a trailing `{.class #id key=value}` block from a heading title.
pub(crate) fn without_trailing_attrs(text: &str) -> &str {
    let bytes = text.as_bytes();
    if bytes.last() != Some(&b'}') {
        return text;
    }
    let mut ix = bytes.len() - 1;
    while ix > 0 {
        let b = bytes[ix - 1];
        if matches!(b, b'{' | b'}' | b'<' | b'>' | b'\\' | b'\n' | b'\r') {
            break;
        }
        ix -= 1;
    }
    if ix == 0 || bytes[ix - 1] != b'{' {
        return text;
    }
    let inside = &text[ix..bytes.len() - 1];
    if parse_attr_block(inside).is_none() {
        return text;
    }
    text[..ix - 1].trim_end()
}

fn parse(input: &str) -> Vec<Event<'_>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    Parser::new_ext(input, options).collect()
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn apply_inline_attrs<'a>(events: &[Event<'a>]) -> Vec<Event<'a>> {
    let mut out = Vec::with_capacity(events.len());
    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                let end = matching_end(events, i);
                if let Some((attrs, next, rest)) = consume_following_attrs(events, end + 1) {
                    let alt = inline_plain(&events[i + 1..end]);
                    out.push(Event::InlineHtml(CowStr::from(render_img(
                        dest_url, title, &alt, &attrs,
                    ))));
                    if let Some(rest) = rest {
                        out.push(Event::Text(rest));
                    }
                    i = next;
                } else {
                    out.extend(events[i..=end].iter().cloned());
                    i = end + 1;
                }
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                ..
            }) => {
                let end = matching_end(events, i);
                let link_type = *link_type;
                let dest_url = dest_url.clone();
                let title = title.clone();
                let inner = apply_inline_attrs(&events[i + 1..end]);
                if let Some((attrs, next, rest)) = consume_following_attrs(events, end + 1) {
                    out.push(Event::InlineHtml(CowStr::from(render_link_open(
                        link_type, &dest_url, &title, &attrs,
                    ))));
                    out.extend(inner);
                    out.push(Event::InlineHtml("</a>".into()));
                    if let Some(rest) = rest {
                        out.push(Event::Text(rest));
                    }
                    i = next;
                } else {
                    out.push(events[i].clone());
                    out.extend(inner);
                    out.push(events[end].clone());
                    i = end + 1;
                }
            }
            _ => {
                out.push(events[i].clone());
                i += 1;
            }
        }
    }
    out
}

fn strip_inline_attr_text<'a>(events: &[Event<'a>]) -> Vec<Event<'a>> {
    let mut out = Vec::with_capacity(events.len());
    let mut i = 0;
    while i < events.len() {
        let is_inline_end = matches!(events[i], Event::End(TagEnd::Image | TagEnd::Link));
        out.push(events[i].clone());
        i += 1;
        if is_inline_end {
            if let Some((_, next, rest)) = consume_following_attrs(events, i) {
                if let Some(rest) = rest {
                    out.push(Event::Text(rest));
                }
                i = next;
            }
        }
    }
    out
}

fn matching_end(events: &[Event<'_>], start: usize) -> usize {
    let mut depth = 0usize;
    for (offset, event) in events[start..].iter().enumerate() {
        match event {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return start + offset;
                }
            }
            _ => {}
        }
    }
    start
}

/// Attribute block immediately after an image or link.
///
/// Returns the parsed attributes, the index after the consumed text event, and
/// any characters that followed the closing `}`.
fn consume_following_attrs<'a>(
    events: &[Event<'a>],
    idx: usize,
) -> Option<(Attrs, usize, Option<CowStr<'a>>)> {
    if idx >= events.len() {
        return None;
    }
    let mut idx = idx;
    let mut skipped_space = false;
    if let Event::Text(text) = &events[idx] {
        if text.as_ref() == " " || text.as_ref() == "\t" {
            idx += 1;
            skipped_space = true;
        }
    }
    let Event::Text(text) = events.get(idx)? else {
        return None;
    };
    let body = if skipped_space {
        text.as_ref()
    } else if let Some(rest) = text.strip_prefix(' ').or_else(|| text.strip_prefix('\t')) {
        rest
    } else {
        text.as_ref()
    };
    if !body.starts_with('{') {
        return None;
    }
    let close = body.find('}')?;
    let inside = &body[1..close];
    if inside
        .bytes()
        .any(|b| matches!(b, b'{' | b'<' | b'>' | b'\\' | b'\n' | b'\r'))
    {
        return None;
    }
    let attrs = parse_attr_block(inside)?;
    let rest = &body[close + 1..];
    let rest = if rest.is_empty() {
        None
    } else {
        Some(CowStr::from(rest.to_string()))
    };
    Some((attrs, idx + 1, rest))
}

fn parse_attr_block(inside: &str) -> Option<Attrs> {
    let mut id = None;
    let mut classes = Vec::new();
    let mut attrs = Vec::new();
    for token in inside.split_ascii_whitespace() {
        let bytes = token.as_bytes();
        if bytes.first() == Some(&b'#') && is_token_value(&token[1..]) {
            id = Some(token[1..].to_string());
        } else if bytes.first() == Some(&b'.') && is_token_value(&token[1..]) {
            classes.push(token[1..].to_string());
        } else if let Some((key, raw_value)) = token.split_once('=') {
            if !is_attr_name(key) {
                continue;
            }
            let value = unquote(raw_value);
            if key.eq_ignore_ascii_case("class") {
                classes.extend(
                    value
                        .split_ascii_whitespace()
                        .filter(|part| is_token_value(part))
                        .map(str::to_string),
                );
            } else if key.eq_ignore_ascii_case("id") {
                if is_token_value(&value) {
                    id = Some(value);
                }
            } else {
                attrs.push((key.to_string(), Some(value)));
            }
        } else if is_attr_name(token) {
            attrs.push((token.to_string(), None));
        }
    }
    // `{note}` is prose. Require a class, an id, or key=value before consuming.
    let meaningful =
        id.is_some() || !classes.is_empty() || attrs.iter().any(|(_, value)| value.is_some());
    if !meaningful {
        return None;
    }
    Some(Attrs { id, classes, attrs })
}

fn is_attr_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == ':' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '-' | '.'))
}

fn is_token_value(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|c| matches!(c, '"' | '\'' | '<' | '>' | '&' | '='))
}

fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let quote = bytes[0];
        if (quote == b'"' || quote == b'\'') && bytes[bytes.len() - 1] == quote {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

fn inline_plain(events: &[Event<'_>]) -> String {
    let mut out = String::new();
    for event in events {
        match event {
            Event::Text(text) | Event::Code(text) => out.push_str(text),
            Event::SoftBreak | Event::HardBreak => out.push(' '),
            _ => {}
        }
    }
    out
}

fn render_img(dest_url: &str, title: &str, alt: &str, attrs: &Attrs) -> String {
    let mut html = String::new();
    html.push_str("<img src=\"");
    push_href(&mut html, dest_url);
    html.push_str("\" alt=\"");
    push_html(&mut html, alt);
    html.push('"');
    if !title.is_empty() {
        html.push_str(" title=\"");
        push_html(&mut html, title);
        html.push('"');
    }
    push_attrs(&mut html, attrs);
    html.push_str(" />");
    html
}

fn render_link_open(link_type: LinkType, dest_url: &str, title: &str, attrs: &Attrs) -> String {
    let mut html = String::new();
    html.push_str("<a href=\"");
    if link_type == LinkType::Email {
        html.push_str("mailto:");
    }
    push_href(&mut html, dest_url);
    html.push('"');
    if !title.is_empty() {
        html.push_str(" title=\"");
        push_html(&mut html, title);
        html.push('"');
    }
    push_attrs(&mut html, attrs);
    html.push('>');
    html
}

fn push_attrs(html: &mut String, attrs: &Attrs) {
    if let Some(id) = &attrs.id {
        html.push_str(" id=\"");
        push_html(html, id);
        html.push('"');
    }
    if !attrs.classes.is_empty() {
        html.push_str(" class=\"");
        for (index, class) in attrs.classes.iter().enumerate() {
            if index > 0 {
                html.push(' ');
            }
            push_html(html, class);
        }
        html.push('"');
    }
    for (name, value) in &attrs.attrs {
        html.push(' ');
        html.push_str(name);
        html.push_str("=\"");
        if let Some(value) = value {
            push_html(html, value);
        }
        html.push('"');
    }
}

fn push_html(out: &mut String, value: &str) {
    pulldown_cmark_escape::escape_html(&mut *out, value).expect("string write");
}

fn push_href(out: &mut String, value: &str) {
    pulldown_cmark_escape::escape_href(&mut *out, value).expect("string write");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_attribute_block_sets_class() {
        let html = to_html("![Sample estimate](/assets/images/fee.png){.dd-img}");
        assert!(
            html.contains(
                r#"<img src="/assets/images/fee.png" alt="Sample estimate" class="dd-img" />"#
            ),
            "{html}"
        );
        assert!(!html.contains("{.dd-img}"), "{html}");
    }

    #[test]
    fn image_attribute_block_keeps_title_id_and_custom_attr() {
        let html = to_html(r#"![Alt](/a.png "Fee"){#fee .dd-img .wide width="640"}"#);
        assert!(
            html.contains(
                r#"<img src="/a.png" alt="Alt" title="Fee" id="fee" class="dd-img wide" width="640" />"#
            ),
            "{html}"
        );
    }

    #[test]
    fn image_without_attribute_block_is_unchanged() {
        let html = to_html("![Alt](/a.png)");
        assert!(html.contains(r#"<img src="/a.png" alt="Alt" />"#), "{html}");
        assert!(!html.contains("class="), "{html}");
    }

    #[test]
    fn spaced_attribute_block_is_accepted() {
        let html = to_html("![Alt](/a.png) {.dd-img}");
        assert!(html.contains(r#"class="dd-img""#), "{html}");
        assert!(!html.contains("{.dd-img}"), "{html}");
    }

    #[test]
    fn bare_braces_stay_text() {
        let html = to_html("![Alt](/a.png){note}");
        assert!(html.contains(r#"<img src="/a.png" alt="Alt" />"#), "{html}");
        assert!(html.contains("{note}"), "{html}");
        assert!(!html.contains("class="), "{html}");
    }

    #[test]
    fn attribute_block_on_the_next_line_stays_text() {
        let html = to_html("![Alt](/a.png)\n{.dd-img}");
        assert!(!html.contains("class="), "{html}");
        assert!(html.contains("{.dd-img}"), "{html}");
    }

    #[test]
    fn link_attribute_block_wraps_formatted_label() {
        let html = to_html("[**Go**](/go){.dd-button}");
        assert!(
            html.contains(r#"<a href="/go" class="dd-button"><strong>Go</strong></a>"#),
            "{html}"
        );
    }

    #[test]
    fn nested_image_and_link_each_take_a_class() {
        let html = to_html("[![Alt](/a.png){.dd-img}](/go){.dd-button}");
        assert!(
            html.contains(
                r#"<a href="/go" class="dd-button"><img src="/a.png" alt="Alt" class="dd-img" /></a>"#
            ),
            "{html}"
        );
    }

    #[test]
    fn heading_attribute_block_sets_class() {
        let html = to_html("# Fee {.dd-title}");
        assert!(html.contains(r#"<h1 class="dd-title">Fee</h1>"#), "{html}");
    }

    #[test]
    fn quoted_alt_is_escaped() {
        let html = to_html(r#"![say "hi" & more](/a.png){.dd-img}"#);
        assert!(
            html.contains(r#"alt="say &quot;hi&quot; &amp; more""#),
            "{html}"
        );
    }

    #[test]
    fn code_block_does_not_interpret_attribute_syntax() {
        let html = to_html("```\n![Alt](/a.png){.dd-img}\n```");
        assert!(html.contains("{.dd-img}"), "{html}");
        assert!(!html.contains("<img"), "{html}");
    }

    #[test]
    fn plain_text_drops_the_attribute_block_and_keeps_alt() {
        let plain = to_plain("See ![Fee table](/a.png){.dd-img} now");
        assert_eq!(plain, "See Fee table now");
    }

    #[test]
    fn trailing_heading_attrs_are_removed_only_when_they_are_attributes() {
        assert_eq!(without_trailing_attrs("Fee {.dd-title}"), "Fee");
        assert_eq!(without_trailing_attrs("Use {braces}"), "Use {braces}");
        assert_eq!(without_trailing_attrs("Plain"), "Plain");
    }
}
