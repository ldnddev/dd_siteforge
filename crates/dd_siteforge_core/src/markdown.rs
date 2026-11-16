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
    let rewritten = scrub_events(rewritten);
    let mut out = String::new();
    html::push_html(&mut out, rewritten.into_iter());
    sanitize_html(&out)
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
            } else if attr_kept(key, Some(&value)) {
                attrs.push((key.to_string(), Some(value)));
            }
        } else if is_attr_name(token) && attr_kept(token, None) {
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
    push_href(&mut html, &safe_img_src(dest_url));
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
    push_href(&mut html, &safe_link_href(dest_url));
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

fn safe_link_href(url: &str) -> String {
    let safe = crate::model::safe_href(url);
    if safe.is_empty() {
        "#".to_string()
    } else {
        safe
    }
}

fn safe_img_src(url: &str) -> String {
    crate::model::safe_href(url)
}

fn scrub_events<'a>(events: Vec<Event<'a>>) -> Vec<Event<'a>> {
    events.into_iter().map(scrub_event).collect()
}

fn scrub_event<'a>(event: Event<'a>) -> Event<'a> {
    match event {
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: CowStr::from(safe_link_href(&dest_url)),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: CowStr::from(safe_img_src(&dest_url)),
            title,
            id,
        }),
        Event::Start(Tag::Heading {
            level,
            id,
            classes,
            attrs,
        }) => Event::Start(Tag::Heading {
            level,
            id,
            classes,
            attrs: attrs
                .into_iter()
                .filter(|(name, value)| attr_kept(name, value.as_deref()))
                .collect(),
        }),
        other => other,
    }
}

/// Extra attributes from `{.class #id key=value}` and heading blocks.
/// Event-handler names and scriptable URLs are not styling.
fn attr_kept(name: &str, value: Option<&str>) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower.starts_with("on") {
        return false;
    }
    if lower == "style" {
        let Some(value) = value else {
            return false;
        };
        return style_value_kept(value);
    }
    let allowed = matches!(
        lower.as_str(),
        "class"
            | "id"
            | "href"
            | "src"
            | "alt"
            | "title"
            | "width"
            | "height"
            | "loading"
            | "decoding"
            | "sizes"
            | "srcset"
            | "referrerpolicy"
            | "crossorigin"
            | "role"
            | "tabindex"
            | "align"
            | "colspan"
            | "rowspan"
            | "span"
            | "start"
            | "target"
            | "type"
            | "disabled"
            | "checked"
            | "allow"
            | "allowfullscreen"
            | "frameborder"
            | "sandbox"
            | "name"
            | "scrolling"
            | "rel"
            | "media"
            | "integrity"
            | "async"
            | "defer"
            | "nomodule"
    ) || lower.starts_with("aria-")
        || lower.starts_with("data-");
    if !allowed || !is_attr_name(&lower) {
        return false;
    }
    if lower == "target" {
        return matches!(value, Some("_self" | "_blank" | "_parent" | "_top"));
    }
    if matches!(
        lower.as_str(),
        "src" | "href" | "srcset" | "poster" | "action" | "formaction"
    ) {
        let Some(value) = value else {
            return false;
        };
        return !crate::model::is_scriptable_url(value)
            && !crate::model::is_scriptable_url(value.trim_start_matches('/'));
    }
    true
}

/// Layout CSS stays. Scriptable URLs and CSS expressions do not.
fn style_value_kept(value: &str) -> bool {
    let compact: String = value
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect();
    !compact.contains("javascript:")
        && !compact.contains("vbscript:")
        && !compact.contains("expression(")
        && !compact.contains("@import")
        && !compact.contains("-moz-binding")
        && !compact.contains("behavior:")
}

fn is_allowed_tag(name: &str) -> bool {
    matches!(
        name,
        "p" | "br"
            | "hr"
            | "strong"
            | "em"
            | "del"
            | "s"
            | "code"
            | "pre"
            | "blockquote"
            | "ul"
            | "ol"
            | "li"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "a"
            | "img"
            | "div"
            | "span"
            | "table"
            | "thead"
            | "tbody"
            | "tfoot"
            | "tr"
            | "th"
            | "td"
            | "figure"
            | "figcaption"
            | "picture"
            | "source"
            | "sup"
            | "sub"
            | "details"
            | "summary"
            | "caption"
            | "small"
            | "mark"
            | "abbr"
            | "cite"
            | "q"
            | "kbd"
            | "samp"
            | "var"
            | "wbr"
            | "b"
            | "i"
            | "u"
            | "input"
            | "col"
            | "colgroup"
            | "iframe"
            | "script"
            | "style"
            | "link"
            | "noscript"
    )
}

/// Keep CommonMark tags and author HTML in copy fields (`img`, `div`,
/// `iframe`, `script`, `style`). Drop other tags, event-handler attributes,
/// and scriptable URLs. `script` and `style` bodies are copied verbatim so a
/// comparison such as `i < 10` is not treated as a tag.
fn sanitize_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        if after.starts_with("<!--") {
            if let Some(end) = after.find("-->") {
                rest = &after[end + 3..];
                continue;
            }
            push_html(&mut out, after);
            break;
        }
        let Some(rel_end) = after.find('>') else {
            push_html(&mut out, after);
            break;
        };
        let tag = &after[..=rel_end];
        let (name, closing, self_closing) = tag_flags(tag);
        if matches!(name.as_str(), "script" | "style") && !closing && !self_closing {
            if let Some(end) = find_closing_tag(after, &name) {
                out.push_str(&sanitize_raw_container(&after[..end], &name));
                rest = &after[end..];
                continue;
            }
            out.push_str(&rewrite_tag(tag));
            out.push_str(&after[rel_end + 1..]);
            break;
        }
        out.push_str(&rewrite_tag(tag));
        rest = &after[rel_end + 1..];
    }
    out.push_str(rest);
    out
}

fn tag_flags(tag: &str) -> (String, bool, bool) {
    let inner = &tag[1..tag.len() - 1];
    let trimmed = inner.trim();
    let self_closing = trimmed.ends_with('/');
    let closing = trimmed.starts_with('/');
    let body = trimmed.trim_start_matches('/').trim_end_matches('/').trim();
    let name = body
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    (name, closing, self_closing)
}

/// Rewrite the opening tag, then copy the element body unchanged.
fn sanitize_raw_container(block: &str, name: &str) -> String {
    let Some(rel_end) = block.find('>') else {
        let mut escaped = String::new();
        push_html(&mut escaped, block);
        return escaped;
    };
    let open = rewrite_tag(&block[..=rel_end]);
    let rest = &block[rel_end + 1..];
    let close_at = rest.to_ascii_lowercase().rfind(&format!("</{name}"));
    let Some(close_at) = close_at else {
        return format!("{open}{rest}");
    };
    let body = &rest[..close_at];
    format!("{open}{body}</{name}>")
}

fn find_closing_tag(haystack: &str, name: &str) -> Option<usize> {
    let lower = haystack.to_ascii_lowercase();
    let needle = format!("</{name}");
    let mut from = 0;
    while let Some(rel) = lower[from..].find(&needle) {
        let abs = from + rel;
        let mut tail = abs + needle.len();
        while lower
            .as_bytes()
            .get(tail)
            .is_some_and(|b| b.is_ascii_whitespace())
        {
            tail += 1;
        }
        if lower.as_bytes().get(tail) == Some(&b'>') {
            return Some(tail + 1);
        }
        from = abs + needle.len();
    }
    None
}

fn rewrite_tag(tag: &str) -> String {
    let (name_lower, closing, self_closing) = tag_flags(tag);
    let inner = &tag[1..tag.len() - 1];
    let trimmed = inner.trim();
    let body = trimmed.trim_start_matches('/').trim_end_matches('/').trim();
    let name = body.split_whitespace().next().unwrap_or("");
    if name_lower.is_empty() || !is_allowed_tag(&name_lower) {
        let mut escaped = String::new();
        push_html(&mut escaped, tag);
        return escaped;
    }
    if closing {
        return format!("</{name_lower}>");
    }
    let attr_src = body[name.len()..].trim().trim_end_matches('/').trim();
    let attrs = parse_attrs(attr_src);
    let embedded_quotes = attrs.iter().any(|(_, value)| {
        value
            .as_ref()
            .is_some_and(|value| value.contains('"') || value.contains('\''))
    });
    if !embedded_quotes
        && attrs
            .iter()
            .all(|(attr, value)| attr_kept(attr, value.as_deref()))
    {
        return tag.to_string();
    }
    let mut rewritten = format!("<{name_lower}");
    for (attr, value) in attrs {
        if !attr_kept(&attr, value.as_deref()) {
            continue;
        }
        let attr_name = attr.to_ascii_lowercase();
        rewritten.push(' ');
        rewritten.push_str(&attr_name);
        if let Some(value) = value {
            rewritten.push_str("=\"");
            push_html(&mut rewritten, &value);
            rewritten.push('"');
        }
    }
    if self_closing {
        rewritten.push_str(" />");
    } else {
        rewritten.push('>');
    }
    rewritten
}

fn parse_attrs(input: &str) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let mut rest = input.trim();
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        let name_len = rest
            .find(|c: char| c.is_whitespace() || c == '=' || c == '/' || c == '>')
            .unwrap_or(rest.len());
        if name_len == 0 {
            break;
        }
        let name = rest[..name_len].to_string();
        rest = rest[name_len..].trim_start();
        if let Some(stripped) = rest.strip_prefix('=') {
            rest = stripped.trim_start();
            let (value, next) = take_attr_value(rest);
            out.push((name, Some(value)));
            rest = next;
        } else {
            out.push((name, None));
        }
    }
    out
}

fn take_attr_value(input: &str) -> (String, &str) {
    let Some(first) = input.chars().next() else {
        return (String::new(), input);
    };
    if first == '"' || first == '\'' {
        if let Some(end) = find_closing_quote(input, first) {
            return (input[1..end].to_string(), &input[end + 1..]);
        }
        return (input[1..].to_string(), "");
    }
    let end = input.find(char::is_whitespace).unwrap_or(input.len());
    (input[..end].to_string(), &input[end..])
}

/// The closing quote is the one followed by another attribute, `>`, or `/`.
/// A prefill query such as `entry={"Audit_Type":"wcag_audit"}` contains quotes
/// that are not the end of the attribute.
fn find_closing_quote(input: &str, quote: char) -> Option<usize> {
    let bytes = input.as_bytes();
    let q = quote as u8;
    if bytes.first() != Some(&q) {
        return None;
    }
    let mut index = 1;
    let mut last = None;
    while index < bytes.len() {
        if bytes[index] == q {
            last = Some(index);
            if quote_closes_attr(input, index + 1) {
                return Some(index);
            }
        }
        index += 1;
    }
    last
}

fn quote_closes_attr(input: &str, after: usize) -> bool {
    let rest = input[after..].trim_start();
    if rest.is_empty() || rest.starts_with('>') || rest.starts_with('/') {
        return true;
    }
    let name_len = rest
        .find(|c: char| c.is_whitespace() || c == '=' || c == '/' || c == '>')
        .unwrap_or(rest.len());
    if name_len == 0 || !is_attr_name(&rest[..name_len]) {
        return false;
    }
    let after_name = rest[name_len..].trim_start();
    after_name.is_empty()
        || after_name.starts_with('=')
        || after_name.starts_with('>')
        || after_name.starts_with('/')
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

    #[test]
    fn scriptable_markdown_urls_and_event_handlers_are_removed() {
        let link = to_html("[click](javascript:alert(1))");
        assert!(!link.to_ascii_lowercase().contains("javascript:"), "{link}");
        assert!(link.contains("href=\"#\""), "{link}");

        let mixed = to_html("[Go](/go){.dd-button onclick=alert(1)}");
        assert!(mixed.contains("class=\"dd-button\""), "{mixed}");
        assert!(!mixed.to_ascii_lowercase().contains("onclick"), "{mixed}");

        let heading = to_html("# Title {onclick=alert(1)}");
        assert!(
            !heading.to_ascii_lowercase().contains("onclick"),
            "{heading}"
        );

        let raw = to_html("<script>alert(1)</script>\n\n<div class=\"note\">ok</div>");
        assert!(raw.contains("<script>alert(1)</script>"), "{raw}");
        assert!(raw.contains("class=\"note\""), "{raw}");

        let image = to_html("<img src=x onerror=alert(1)>");
        assert!(!image.to_ascii_lowercase().contains("onerror"), "{image}");
        assert!(image.contains("<img"), "{image}");
    }

    #[test]
    fn author_iframe_script_and_style_pass_through() {
        let iframe = to_html(
            r#"<iframe src="https://www.cognitoforms.com/f/nKwDEJz_5k-922Y1-zu0fg/4" allow="payment" style="border:0;width:100%;" height="1122"></iframe>"#,
        );
        assert!(
            iframe.contains(r#"src="https://www.cognitoforms.com/f/nKwDEJz_5k-922Y1-zu0fg/4""#),
            "{iframe}"
        );
        assert!(iframe.contains(r#"allow="payment""#), "{iframe}");
        assert!(
            iframe.contains(r#"style="border:0;width:100%;""#),
            "{iframe}"
        );
        assert!(iframe.contains(r#"height="1122""#), "{iframe}");
        assert!(!iframe.contains("&lt;iframe"), "{iframe}");

        let cognito = to_html(
            "<script src=\"https://www.cognitoforms.com/f/seamless.js\" data-key=\"nKwDEJz_5k-922Y1-zu0fg\" data-form=\"1\"></script>\n<script>\n  (function () { if (i < 10) return; window.Cognito.on('afterSubmit', function () {}); })();\n</script>",
        );
        assert!(
            cognito.contains(
                r#"<script src="https://www.cognitoforms.com/f/seamless.js" data-key="nKwDEJz_5k-922Y1-zu0fg" data-form="1"></script>"#
            ),
            "{cognito}"
        );
        assert!(cognito.contains("if (i < 10) return;"), "{cognito}");
        assert!(cognito.contains("window.Cognito.on"), "{cognito}");

        let json_ld =
            to_html(r#"<script type="application/ld+json">{"@type":"BreadcrumbList"}</script>"#);
        assert!(
            json_ld.contains(
                r#"<script type="application/ld+json">{"@type":"BreadcrumbList"}</script>"#
            ),
            "{json_ld}"
        );

        let css = to_html("<style>\n.embed { width: 100%; }\n</style>");
        assert!(css.contains("<style>"), "{css}");
        assert!(css.contains(".embed { width: 100%; }"), "{css}");
        assert!(css.contains("</style>"), "{css}");
    }

    #[test]
    fn script_src_keeps_a_quoted_prefill_query() {
        let html = to_html(
            r#"<script src="https://www.cognitoforms.com/f/seamless.js?entry={"Audit_Type":"wcag_audit"}" data-key="nKwDEJz_5k-922Y1-zu0fg" data-form="2"></script>"#,
        );
        assert!(
            html.contains(
                r#"src="https://www.cognitoforms.com/f/seamless.js?entry={&quot;Audit_Type&quot;:&quot;wcag_audit&quot;}""#
            ),
            "{html}"
        );
        assert!(
            html.contains(r#"data-key="nKwDEJz_5k-922Y1-zu0fg""#),
            "{html}"
        );
        assert!(html.contains(r#"data-form="2""#), "{html}");
        assert!(!html.contains("entry={\""), "{html}");

        // Saved author copy is missing the quote that closes the field name.
        // The exporter still has to keep that string; it must not stop at `{`.
        let saved = to_html(
            r#"<script src="https://www.cognitoforms.com/f/seamless.js?entry={"Audit_Type:"wcag_audit"}" data-key="nKwDEJz_5k-922Y1-zu0fg" data-form="2"></script>"#,
        );
        assert!(
            saved.contains(
                r#"src="https://www.cognitoforms.com/f/seamless.js?entry={&quot;Audit_Type:&quot;wcag_audit&quot;}""#
            ),
            "{saved}"
        );
        assert!(
            saved.contains(r#"data-key="nKwDEJz_5k-922Y1-zu0fg""#),
            "{saved}"
        );
        assert!(saved.contains(r#"data-form="2""#), "{saved}");
    }

    #[test]
    fn scriptable_embed_urls_and_css_are_removed() {
        let iframe = to_html(r#"<iframe src="javascript:alert(1)" allow="payment"></iframe>"#);
        assert!(
            !iframe.to_ascii_lowercase().contains("javascript:"),
            "{iframe}"
        );
        assert!(iframe.contains("<iframe"), "{iframe}");
        assert!(iframe.contains(r#"allow="payment""#), "{iframe}");

        let script = to_html(r#"<script src="javascript:alert(1)">if (i < 10) return;</script>"#);
        assert!(
            !script.to_ascii_lowercase().contains("javascript:"),
            "{script}"
        );
        assert!(script.contains("if (i < 10) return;"), "{script}");

        let styled = to_html(r#"<div style="background:url(javascript:alert(1))">x</div>"#);
        assert!(
            !styled.to_ascii_lowercase().contains("javascript:"),
            "{styled}"
        );
        assert!(styled.contains("<div>"), "{styled}");

        let aligned = to_html(r#"<td style="text-align: center">x</td>"#);
        assert!(aligned.contains("text-align: center"), "{aligned}");
    }
}
