//! Styled Details blueprint: selection echo + brand/semantic chips.
use super::super::*;
use ratatui::text::{Line, Span, Text};

/// Geometry from an ASCII map, before Details chrome (decl lines, captions).
pub(in crate::tui) struct AsciiMap {
    pub lines: Vec<String>,
    pub hits: Vec<Vec<(usize, usize, usize, usize)>>,
    /// Column-box cells `(x0, x1, col)` on each line. Empty for header/footer.
    pub boxes: Vec<Vec<(usize, usize, usize)>>,
}

impl AsciiMap {
    pub(in crate::tui) fn from_lines(
        lines: Vec<String>,
        hits: Vec<Vec<(usize, usize, usize, usize)>>,
        boxes: Vec<Vec<(usize, usize, usize)>>,
    ) -> Self {
        let n = lines.len();
        let mut hits = hits;
        let mut boxes = boxes;
        hits.resize(n, vec![]);
        boxes.resize(n, vec![]);
        Self { lines, hits, boxes }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::tui) enum BlueprintStyle {
    Default,
    Label,
    Focus,
    FocusFill,
    Brand(Color),
    Success,
    Warning,
    Error,
    Info,
}

impl BlueprintStyle {
    fn priority(self) -> u8 {
        match self {
            BlueprintStyle::Default => 0,
            BlueprintStyle::Label => 1,
            BlueprintStyle::Focus => 2,
            BlueprintStyle::Brand(_)
            | BlueprintStyle::Success
            | BlueprintStyle::Warning
            | BlueprintStyle::Error
            | BlueprintStyle::Info => 3,
            BlueprintStyle::FocusFill => 4,
        }
    }

    fn to_style(self, theme: &AppTheme) -> Style {
        match self {
            BlueprintStyle::Default => Style::default().fg(theme.text_primary),
            BlueprintStyle::Label => Style::default().fg(theme.text_labels),
            BlueprintStyle::Focus => Style::default().fg(theme.text_active_focus),
            BlueprintStyle::FocusFill => Style::default()
                .fg(theme.text_active_focus)
                .bg(theme.selected_background)
                .add_modifier(Modifier::BOLD),
            BlueprintStyle::Brand(c) => Style::default().fg(c),
            BlueprintStyle::Success => Style::default().fg(theme.success),
            BlueprintStyle::Warning => Style::default().fg(theme.warning),
            BlueprintStyle::Error => Style::default().fg(theme.error),
            BlueprintStyle::Info => Style::default().fg(theme.info),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::tui) enum FocusDepth {
    Root,
    Column(usize),
    Component { column: usize, component: usize },
}

#[derive(Clone, Copy, Debug)]
pub(in crate::tui) enum HeaderFocusDepth {
    Root,
    Section(usize),
    Column { section: usize, column: usize },
    Component { section: usize, column: usize, component: usize },
}

#[derive(Clone, Copy, Debug)]
pub(in crate::tui) enum BlueprintFocus {
    None,
    Site,
    Page { node: usize, depth: FocusDepth },
    Header { depth: HeaderFocusDepth },
    Footer { depth: HeaderFocusDepth },
}

pub(in crate::tui) struct DetailsView {
    pub lines: Vec<String>,
    pub hits: Vec<Vec<(usize, usize, usize, usize)>>,
    pub styles: Vec<Vec<(usize, usize, BlueprintStyle)>>,
}

impl DetailsView {
    pub(in crate::tui) fn new() -> Self {
        Self {
            lines: Vec::new(),
            hits: Vec::new(),
            styles: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(in crate::tui) fn as_str(&self) -> String {
        self.lines.join("\n")
    }

    pub(in crate::tui) fn push_plain(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
        self.hits.push(vec![]);
        self.styles.push(vec![]);
    }

    pub(in crate::tui) fn push_styled(&mut self, line: impl Into<String>, style: BlueprintStyle) {
        let line = line.into();
        let n = line.chars().count();
        self.lines.push(line);
        self.hits.push(vec![]);
        self.styles.push(if n == 0 {
            vec![]
        } else {
            vec![(0, n, style)]
        });
    }

    /// Append an ASCII map and return the line range plus its column-box segments.
    pub(in crate::tui) fn extend_map(
        &mut self,
        map: AsciiMap,
    ) -> (usize, usize, Vec<Vec<(usize, usize, usize)>>) {
        let start = self.lines.len();
        let boxes = map.boxes;
        self.lines.extend(map.lines);
        self.hits.extend(map.hits);
        let n = self.lines.len() - start;
        self.styles.resize(self.lines.len(), vec![]);
        self.hits.resize(self.lines.len(), vec![]);
        let mut boxes = boxes;
        boxes.resize(n, vec![]);
        (start, self.lines.len(), boxes)
    }

    pub(in crate::tui) fn paint(&mut self, line: usize, x0: usize, x1: usize, style: BlueprintStyle) {
        if line >= self.lines.len() {
            return;
        }
        let max = self.lines[line].chars().count();
        let x0 = x0.min(max);
        let x1 = x1.min(max);
        if x0 >= x1 {
            return;
        }
        self.styles[line].push((x0, x1, style));
    }

    pub(in crate::tui) fn paint_line(&mut self, line: usize, style: BlueprintStyle) {
        let n = self.lines.get(line).map(|l| l.chars().count()).unwrap_or(0);
        self.paint(line, 0, n, style);
    }

    pub(in crate::tui) fn to_text(&self, theme: &AppTheme) -> Text<'static> {
        let lines = self
            .lines
            .iter()
            .enumerate()
            .map(|(i, raw)| {
                let overlays = self.styles.get(i).map(|s| s.as_slice()).unwrap_or(&[]);
                Line::from(spans_for_line(raw, overlays, theme))
            })
            .collect::<Vec<_>>();
        Text::from(lines)
    }
}

fn spans_for_line(
    raw: &str,
    overlays: &[(usize, usize, BlueprintStyle)],
    theme: &AppTheme,
) -> Vec<Span<'static>> {
    let chars: Vec<char> = raw.chars().collect();
    if chars.is_empty() {
        return vec![Span::raw("")];
    }
    let mut kind = vec![BlueprintStyle::Default; chars.len()];
    let mut ranked = overlays.to_vec();
    ranked.sort_by_key(|(_, _, k)| k.priority());
    for &(x0, x1, k) in &ranked {
        let x0 = x0.min(chars.len());
        let x1 = x1.min(chars.len());
        for slot in kind.iter_mut().take(x1).skip(x0) {
            if k.priority() >= slot.priority() {
                *slot = k;
            }
        }
    }
    let mut spans = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let k = kind[start];
        let mut end = start + 1;
        while end < chars.len() && kind[end] == k {
            end += 1;
        }
        let s: String = chars[start..end].iter().collect();
        spans.push(Span::styled(s, k.to_style(theme)));
        start = end;
    }
    spans
}

pub(in crate::tui) fn brand_from_hex(hex: &str) -> Option<BlueprintStyle> {
    parse_hex_color(hex).ok().map(BlueprintStyle::Brand)
}

pub(in crate::tui) fn component_token_style(
    component: &crate::model::SectionComponent,
    theme: &crate::model::ThemeSettings,
) -> Option<BlueprintStyle> {
    use crate::model::{AccordionClass, AlertType, SectionComponent};
    match component {
        SectionComponent::Accordion(v) => match v.parent_class {
            AccordionClass::Primary => brand_from_hex(&theme.primary_color),
            AccordionClass::Secondary => brand_from_hex(&theme.secondary_color),
            AccordionClass::Tertiary => brand_from_hex(&theme.tertiary_color),
            AccordionClass::Borderless | AccordionClass::Compact => None,
        },
        SectionComponent::Alternating(v) => match v.parent_class.as_str() {
            "-primary" => brand_from_hex(&theme.primary_color),
            "-secondary" => brand_from_hex(&theme.secondary_color),
            "-tertiary" => brand_from_hex(&theme.tertiary_color),
            _ => None,
        },
        SectionComponent::Alert(v) => match v.parent_type {
            AlertType::Info => Some(BlueprintStyle::Info),
            AlertType::Warning => Some(BlueprintStyle::Warning),
            AlertType::Error => Some(BlueprintStyle::Error),
            AlertType::Success => Some(BlueprintStyle::Success),
            AlertType::Default => None,
        },
        _ => None,
    }
}

impl App {
    pub(in crate::tui) fn blueprint_focus(&self) -> BlueprintFocus {
        let rows = self.build_tree_rows();
        let Some(row) = rows.get(self.selected_tree_row) else {
            return BlueprintFocus::None;
        };
        match row.kind {
            TreeRowKind::SiteRoot => BlueprintFocus::Site,
            TreeRowKind::PageHead => BlueprintFocus::None,
            TreeRowKind::Hero { node_idx } | TreeRowKind::Section { node_idx } => {
                BlueprintFocus::Page {
                    node: node_idx,
                    depth: FocusDepth::Root,
                }
            }
            TreeRowKind::Column {
                node_idx,
                column_idx,
            } => BlueprintFocus::Page {
                node: node_idx,
                depth: FocusDepth::Column(column_idx),
            },
            TreeRowKind::Component {
                node_idx,
                column_idx,
                component_idx,
            }
            | TreeRowKind::AccordionItem {
                node_idx,
                column_idx,
                component_idx,
                ..
            }
            | TreeRowKind::AlternatingItem {
                node_idx,
                column_idx,
                component_idx,
                ..
            }
            | TreeRowKind::CardItem {
                node_idx,
                column_idx,
                component_idx,
                ..
            }
            | TreeRowKind::FilmstripItem {
                node_idx,
                column_idx,
                component_idx,
                ..
            }
            | TreeRowKind::MilestonesItem {
                node_idx,
                column_idx,
                component_idx,
                ..
            }
            | TreeRowKind::SliderItem {
                node_idx,
                column_idx,
                component_idx,
                ..
            } => BlueprintFocus::Page {
                node: node_idx,
                depth: FocusDepth::Component {
                    column: column_idx,
                    component: component_idx,
                },
            },
            TreeRowKind::HeaderRoot => BlueprintFocus::Header {
                depth: HeaderFocusDepth::Root,
            },
            TreeRowKind::HeaderSection { section_idx } => BlueprintFocus::Header {
                depth: HeaderFocusDepth::Section(section_idx),
            },
            TreeRowKind::HeaderColumn {
                section_idx,
                column_idx,
            } => BlueprintFocus::Header {
                depth: HeaderFocusDepth::Column {
                    section: section_idx,
                    column: column_idx,
                },
            },
            TreeRowKind::HeaderComponent {
                section_idx,
                column_idx,
                component_idx,
            } => BlueprintFocus::Header {
                depth: HeaderFocusDepth::Component {
                    section: section_idx,
                    column: column_idx,
                    component: component_idx,
                },
            },
            TreeRowKind::FooterRoot => BlueprintFocus::Footer {
                depth: HeaderFocusDepth::Root,
            },
            TreeRowKind::FooterSection { section_idx } => BlueprintFocus::Footer {
                depth: HeaderFocusDepth::Section(section_idx),
            },
            TreeRowKind::FooterColumn {
                section_idx,
                column_idx,
            } => BlueprintFocus::Footer {
                depth: HeaderFocusDepth::Column {
                    section: section_idx,
                    column: column_idx,
                },
            },
            TreeRowKind::FooterComponent {
                section_idx,
                column_idx,
                component_idx,
            } => BlueprintFocus::Footer {
                depth: HeaderFocusDepth::Component {
                    section: section_idx,
                    column: column_idx,
                    component: component_idx,
                },
            },
        }
    }
}
