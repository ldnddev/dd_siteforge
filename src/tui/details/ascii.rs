//! ASCII blueprints for hero, section, header, and footer.
use super::super::*;
use super::*;
use crate::tui::cursor;
use crate::tui::editform::{EditFormState, FieldKind};

pub(in crate::tui) fn section_ascii_map(
    section: &crate::model::DdSection,
    selected_column: usize,
    panel_width: usize,
) -> AsciiMap {
    let inner_width = panel_width.saturating_sub(4).max(12);
    let columns = section_columns_ref(section);
    if columns.is_empty() {
        return AsciiMap::from_lines(vec!["(no columns)".to_string()], vec![vec![]], vec![vec![]]);
    }
    let active = selected_column.min(columns.len().saturating_sub(1));
    let gap = 1usize;
    let ratios: Vec<(usize, usize)> = columns
        .iter()
        .map(|col| layout_dd_u_ratio(&col.width_class))
        .collect();
    let packed = pack_fraction_rows(&ratios, inner_width, gap);

    let mut column_data: Vec<(Vec<String>, Vec<Option<usize>>)> =
        vec![(Vec::new(), Vec::new()); columns.len()];
    for row in &packed {
        for &(col_idx, outer_width) in row {
            let col = &columns[col_idx];
            let inner = outer_width.saturating_sub(4).max(1);
            let marker = if col_idx == active { "*" } else { "-" };
            let (num, den) = ratios[col_idx];
            let item_border = format!("+{}+", "-".repeat(inner + 2));
            let mut box_lines = vec![
                item_border.clone(),
                format!(
                    "| {} |",
                    fit_ascii_cell(&format!("{marker} item: {}", col.id), inner)
                ),
                format!(
                    "| {} |",
                    fit_ascii_cell(&format!("width: {} ({num}/{den})", col.width_class), inner)
                ),
            ];
            let mut box_comps: Vec<Option<usize>> = vec![None, None, None];
            if col.components.is_empty() {
                box_lines.push(format!("| {} |", fit_ascii_cell("(empty)", inner)));
                box_comps.push(None);
            } else {
                for (comp_i, component) in col.components.iter().enumerate() {
                    for line in component_blueprint_lines(component, inner) {
                        box_lines.push(format!("| {} |", fit_ascii_cell(&line, inner)));
                        box_comps.push(Some(comp_i));
                    }
                }
            }
            box_lines.push(item_border);
            box_comps.push(None);
            column_data[col_idx] = (box_lines, box_comps);
        }
    }

    let mut inner_composed_lines: Vec<String> = vec![];
    let mut inner_line_segments: Vec<Vec<(usize, usize, usize, usize)>> = vec![];
    let mut inner_line_boxes: Vec<Vec<(usize, usize, usize)>> = vec![];

    let section_state = cursor::section_to_form_state(section);
    inner_composed_lines.push(fit_ascii_cell("SECTION", inner_width));
    inner_line_segments.push(vec![]);
    inner_line_boxes.push(vec![]);
    for line in blueprint_form_lines(&section_state, &["columns"], inner_width) {
        inner_composed_lines.push(fit_ascii_cell(&line, inner_width));
        inner_line_segments.push(vec![]);
        inner_line_boxes.push(vec![]);
    }
    inner_composed_lines.push(fit_ascii_cell("items:", inner_width));
    inner_line_segments.push(vec![]);
    inner_line_boxes.push(vec![]);

    for (row_idx, row) in packed.iter().enumerate() {
        if row_idx > 0 {
            inner_composed_lines.push("".to_string());
            inner_line_segments.push(vec![]);
            inner_line_boxes.push(vec![]);
        }
        let max_height = row
            .iter()
            .map(|(idx, _)| column_data[*idx].0.len())
            .max()
            .unwrap_or(0);
        for line_idx in 0..max_height {
            let mut composed = String::new();
            let mut segs: Vec<(usize, usize, usize, usize)> = vec![];
            let mut boxes: Vec<(usize, usize, usize)> = vec![];
            let mut cur_x = 0usize;
            for (pos, &(col_idx, outer_width)) in row.iter().enumerate() {
                if pos > 0 {
                    composed.push_str(" ");
                    cur_x += 1;
                }
                let (box_lines, box_comps) = &column_data[col_idx];
                let part = box_lines
                    .get(line_idx)
                    .cloned()
                    .unwrap_or_else(|| " ".repeat(outer_width));
                let part_start = cur_x;
                composed.push_str(&part);
                cur_x += part.chars().count();
                boxes.push((part_start, cur_x, col_idx));
                if let Some(cp) = box_comps.get(line_idx).copied().flatten() {
                    segs.push((part_start, cur_x, col_idx, cp));
                }
            }
            let fitted = fit_ascii_cell(&composed, inner_width);
            inner_composed_lines.push(fitted);
            inner_line_segments.push(segs);
            inner_line_boxes.push(boxes);
        }
    }

    let border = format!("+{}+", "-".repeat(inner_width + 2));
    let mut out = Vec::new();
    let mut out_hits: Vec<Vec<(usize, usize, usize, usize)>> = vec![];
    let mut out_boxes: Vec<Vec<(usize, usize, usize)>> = vec![];
    out.push(border.clone());
    out_hits.push(vec![]);
    out_boxes.push(vec![]);
    for (i, line) in inner_composed_lines.into_iter().enumerate() {
        let final_line = format!("| {} |", line);
        // adjust the inner segs x by +2 for the leading "| "
        let adjusted: Vec<(usize, usize, usize, usize)> = inner_line_segments[i]
            .iter()
            .map(|(x0, x1, c, cp)| (x0 + 2, x1 + 2, *c, *cp))
            .collect();
        let adjusted_boxes: Vec<(usize, usize, usize)> = inner_line_boxes[i]
            .iter()
            .map(|(x0, x1, c)| (x0 + 2, x1 + 2, *c))
            .collect();
        out.push(final_line);
        out_hits.push(adjusted);
        out_boxes.push(adjusted_boxes);
    }
    out.push(border);
    out_hits.push(vec![]);
    out_boxes.push(vec![]);
    AsciiMap::from_lines(out, out_hits, out_boxes)
}

pub(in crate::tui) fn header_ascii_map(
    header: &crate::model::DdHeader,
    selected_section: usize,
    selected_column: usize,
    panel_width: usize,
) -> AsciiMap {
    let inner_width = panel_width.saturating_sub(4).max(12);
    let mut lines = vec![fit_ascii_cell("HEADER", inner_width)];
    let mut line_hits: Vec<Vec<(usize, usize, usize, usize)>> = vec![vec![]];
    let state = cursor::header_root_to_form_state(header);
    for line in blueprint_form_lines(&state, &[], inner_width) {
        lines.push(fit_ascii_cell(&line, inner_width));
        line_hits.push(vec![]);
    }
    if header.alert.is_some() {
        lines.push(fit_ascii_cell("alert: yes", inner_width));
        line_hits.push(vec![]);
    }
    lines.push(fit_ascii_cell("sections:", inner_width));
    line_hits.push(vec![]);

    if header.sections.is_empty() {
        lines.push(fit_ascii_cell(
            "(no sections - press '/' to add)",
            inner_width,
        ));
        line_hits.push(vec![]);
    } else {
        let active_section = selected_section.min(header.sections.len().saturating_sub(1));
        for (s_idx, section) in header.sections.iter().enumerate() {
            let col = if s_idx == active_section {
                selected_column
            } else {
                0
            };
            append_nested_section_map(&mut lines, &mut line_hits, section, col, inner_width);
        }
    }

    wrap_ascii_block(lines, line_hits, inner_width)
}

pub(in crate::tui) fn footer_ascii_map(
    footer: &crate::model::DdFooter,
    selected_section: usize,
    selected_column: usize,
    panel_width: usize,
) -> AsciiMap {
    let inner_width = panel_width.saturating_sub(4).max(12);
    let mut lines = vec![fit_ascii_cell("FOOTER", inner_width)];
    let mut line_hits: Vec<Vec<(usize, usize, usize, usize)>> = vec![vec![]];
    let state = cursor::footer_to_form_state(footer);
    for line in blueprint_form_lines(&state, &[], inner_width) {
        lines.push(fit_ascii_cell(&line, inner_width));
        line_hits.push(vec![]);
    }
    lines.push(fit_ascii_cell("sections:", inner_width));
    line_hits.push(vec![]);
    if footer.sections.is_empty() {
        lines.push(fit_ascii_cell(
            "(no sections - press '/' to add)",
            inner_width,
        ));
        line_hits.push(vec![]);
    } else {
        let active_section = selected_section.min(footer.sections.len().saturating_sub(1));
        for (s_idx, section) in footer.sections.iter().enumerate() {
            let col = if s_idx == active_section {
                selected_column
            } else {
                0
            };
            append_nested_section_map(&mut lines, &mut line_hits, section, col, inner_width);
        }
    }
    wrap_ascii_block(lines, line_hits, inner_width)
}

fn append_nested_section_map(
    lines: &mut Vec<String>,
    line_hits: &mut Vec<Vec<(usize, usize, usize, usize)>>,
    section: &crate::model::DdSection,
    selected_column: usize,
    inner_width: usize,
) {
    let map = section_ascii_map(section, selected_column, inner_width);
    for (i, line) in map.lines.into_iter().enumerate() {
        lines.push(fit_ascii_cell(&line, inner_width));
        line_hits.push(map.hits.get(i).cloned().unwrap_or_default());
    }
}

fn wrap_ascii_block(
    lines: Vec<String>,
    line_hits: Vec<Vec<(usize, usize, usize, usize)>>,
    inner_width: usize,
) -> AsciiMap {
    let border = format!("+{}+", "-".repeat(inner_width + 2));
    let mut out = Vec::new();
    let mut out_hits: Vec<Vec<(usize, usize, usize, usize)>> = vec![];
    out.push(border.clone());
    out_hits.push(vec![]);
    for (i, line) in lines.into_iter().enumerate() {
        out.push(format!("| {} |", line));
        let adjusted = line_hits
            .get(i)
            .map(|segs| {
                segs.iter()
                    .map(|(x0, x1, c, cp)| (x0 + 2, x1 + 2, *c, *cp))
                    .collect()
            })
            .unwrap_or_default();
        out_hits.push(adjusted);
    }
    out.push(border);
    out_hits.push(vec![]);
    let boxes = vec![vec![]; out.len()];
    AsciiMap::from_lines(out, out_hits, boxes)
}

pub(in crate::tui) fn card_items_ascii_lines(
    card: &crate::model::DdCard,
    container_inner_width: usize,
) -> Vec<String> {
    if card.items.is_empty() {
        return vec![fit_ascii_cell("(empty)", container_inner_width)];
    }

    let ratio = layout_dd_u_ratio(&card.parent_width);
    let ratios = vec![ratio; card.items.len()];
    let packed = pack_fraction_rows(&ratios, container_inner_width, 1);
    let state = cursor::card_to_form_state(card);
    let items = state.sub_state.get("items").cloned().unwrap_or_default();

    let mut child_boxes: Vec<Vec<String>> = vec![Vec::new(); card.items.len()];
    for row in &packed {
        for &(idx, outer_width) in row {
            let inner = outer_width.saturating_sub(4).max(1);
            let border = format!("+{}+", "-".repeat(inner + 2));
            let mut lines = vec![
                border.clone(),
                format!(
                    "| {} |",
                    fit_ascii_cell(&format!("card {}:", idx + 1), inner)
                ),
            ];
            if let Some(item) = items.get(idx) {
                for line in blueprint_content_lines(item, inner.saturating_sub(2)) {
                    lines.push(format!("| {} |", fit_ascii_cell(&line, inner)));
                }
            }
            lines.push(border);
            child_boxes[idx] = lines;
        }
    }

    let mut lines = Vec::new();
    for (row_idx, row) in packed.iter().enumerate() {
        if row_idx > 0 {
            lines.push(String::new());
        }
        let row_height = row
            .iter()
            .map(|(idx, _)| child_boxes[*idx].len())
            .max()
            .unwrap_or(0);
        for line_idx in 0..row_height {
            let mut composed = String::new();
            for (pos, &(idx, outer_width)) in row.iter().enumerate() {
                if pos > 0 {
                    composed.push_str(" ");
                }
                let part = child_boxes[idx]
                    .get(line_idx)
                    .cloned()
                    .unwrap_or_else(|| " ".repeat(outer_width));
                composed.push_str(&part);
            }
            lines.push(composed);
        }
    }
    lines
}

fn component_blueprint_lines(
    component: &crate::model::SectionComponent,
    inner: usize,
) -> Vec<String> {
    if let crate::model::SectionComponent::Spacer(sp) = component {
        let size = match sp.size {
            crate::model::SpacerSize::Sm => "-sm",
            crate::model::SpacerSize::Md => "-md",
            crate::model::SpacerSize::Lg => "-lg",
            crate::model::SpacerSize::Xl => "-xl",
            crate::model::SpacerSize::Xxl => "-xxl",
            crate::model::SpacerSize::Xxxl => "-xxxl",
        };
        let mut lines = vec![format!("- dd-spacer {size}")];
        if sp.divider {
            let rule_w = inner.saturating_sub(2).max(3);
            lines.push(format!("  {}", "-".repeat(rule_w)));
        }
        return lines;
    }
    if let crate::model::SectionComponent::Card(card) = component {
        let mut lines = vec!["- dd-card".to_string()];
        for line in card_items_ascii_lines(card, inner) {
            lines.push(line);
        }
        return lines;
    }
    let Some(state) = cursor::component_to_form_state(component) else {
        return vec![format!("- {}", component_label(component))];
    };
    let mut lines = vec![format!("- {}", state.form.title)];
    for line in blueprint_content_lines(&state, inner.saturating_sub(2)) {
        lines.push(format!("  {line}"));
    }
    lines
}

/// Fields that identify the placed component on the page. Editor-only
/// chrome (SAL, ids, CSS, media URLs, targets) stays out of the blueprint.
fn content_field_label(field_id: &str) -> Option<&'static str> {
    Some(match field_id {
        "parent_title" | "child_title" | "section_title" | "text" => "title",
        "heading_level" => "level",
        "parent_subtitle" | "child_subtitle" => "subtitle",
        "parent_copy" | "child_copy" => "copy",
        "parent_name" => "name",
        "parent_role" => "role",
        "child_year" => "year",
        "child_percentage" => "percent",
        "child_link_label" => "link",
        "banner" => "banner",
        "cta_label" => "cta",
        "blurb" => "blurb",
        "copyright" => "copyright",
        "caption" => "caption",
        _ => return None,
    })
}

fn blueprint_form_lines(state: &EditFormState, skip_ids: &[&str], width: usize) -> Vec<String> {
    blueprint_content_lines_skipping(state, skip_ids, width)
}

fn blueprint_content_lines(state: &EditFormState, width: usize) -> Vec<String> {
    blueprint_content_lines_skipping(state, &[], width)
}

fn blueprint_content_lines_skipping(
    state: &EditFormState,
    skip_ids: &[&str],
    width: usize,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut socials: Vec<&str> = Vec::new();
    for field in state.form.fields {
        if skip_ids.contains(&field.id) {
            continue;
        }
        if !state.field_visible(field) {
            continue;
        }
        match field.kind {
            FieldKind::SubForm { .. } if field.id == "links" => {
                let labels = link_labels(state, field.id);
                if !labels.is_empty() {
                    let joined = labels.join(", ");
                    let max = width.saturating_sub(7);
                    lines.push(format!("links: {}", truncate_ascii(&joined, max.max(4))));
                }
            }
            FieldKind::SubForm { .. } if field.id == "columns" => {
                let labels = column_labels(state, field.id);
                if !labels.is_empty() {
                    let joined = labels.join(", ");
                    let max = width.saturating_sub(9);
                    lines.push(format!("columns: {}", truncate_ascii(&joined, max.max(4))));
                }
            }
            FieldKind::SubForm { .. } if field.id == "rows" => {
                let headers = row_headers(state, field.id);
                for (i, header) in headers.iter().enumerate() {
                    let max = width.saturating_sub(8);
                    lines.push(format!(
                        "row {}: {}",
                        i + 1,
                        truncate_ascii(header, max.max(4))
                    ));
                }
            }
            FieldKind::SubForm { .. } => {
                let items = state
                    .sub_state
                    .get(field.id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                for (i, item) in items.iter().enumerate() {
                    let nested = blueprint_content_lines(item, width.saturating_sub(2));
                    if nested.is_empty() {
                        continue;
                    }
                    lines.push(format!("[{}]", i + 1));
                    for line in nested {
                        lines.push(format!("  {line}"));
                    }
                }
            }
            _ if field.id == "cta_label" => {
                if state.get("cta_url").trim().is_empty() {
                    continue;
                }
                push_content_line(&mut lines, field.id, state.get(field.id), width);
            }
            _ if matches!(field.id, "social_linkedin" | "social_x" | "social_github") => {
                if !state.get(field.id).trim().is_empty() {
                    socials.push(match field.id {
                        "social_linkedin" => "LinkedIn",
                        "social_x" => "X",
                        "social_github" => "GitHub",
                        _ => field.id,
                    });
                }
            }
            _ => {
                push_content_line(&mut lines, field.id, state.get(field.id), width);
            }
        }
    }
    if !socials.is_empty() {
        lines.push(format!("social: {}", socials.join(", ")));
    }
    lines
}

fn link_labels(state: &EditFormState, field_id: &str) -> Vec<String> {
    state
        .sub_state
        .get(field_id)
        .into_iter()
        .flatten()
        .map(|item| item.get("label").trim())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn column_labels(state: &EditFormState, field_id: &str) -> Vec<String> {
    link_labels(state, field_id)
}

fn row_headers(state: &EditFormState, field_id: &str) -> Vec<String> {
    state
        .sub_state
        .get(field_id)
        .into_iter()
        .flatten()
        .map(|row| {
            row.sub_state
                .get("cells")
                .and_then(|cells| cells.first())
                .map(|cell| cell.get("text").trim().to_string())
                .filter(|s| !s.is_empty())
                .or_else(|| {
                    let label = row.get("label").trim();
                    if label.is_empty() {
                        None
                    } else {
                        Some(label.to_string())
                    }
                })
                .unwrap_or_else(|| "row".to_string())
        })
        .collect()
}

fn push_content_line(lines: &mut Vec<String>, field_id: &str, value: &str, width: usize) {
    let Some(label) = content_field_label(field_id) else {
        return;
    };
    let text = value.lines().next().unwrap_or("").trim();
    if text.is_empty() {
        return;
    }
    let max = width.saturating_sub(label.len().saturating_add(2));
    lines.push(format!("{}: {}", label, truncate_ascii(text, max.max(4))));
}

/// Blueprint uses the largest authored breakpoint (xxl → base) so a class like
/// `dd-u-1-1 dd-u-md-12-24` lays out as half-width even in a narrow TUI pane.
pub(in crate::tui) fn layout_dd_u_ratio(width_class: &str) -> (usize, usize) {
    resolve_dd_u_ratio_layout(width_class).unwrap_or((1, 1))
}

pub(in crate::tui) fn resolve_dd_u_ratio_layout(width_class: &str) -> Option<(usize, usize)> {
    let mut found = [None; 6];
    for token in width_class.split_whitespace() {
        if let Some((bp, ratio)) = parse_dd_u_token_ratio(token) {
            found[bp.index()] = Some(ratio);
        }
    }
    found.into_iter().rev().flatten().next()
}

fn pack_fraction_rows(
    ratios: &[(usize, usize)],
    inner_width: usize,
    gap: usize,
) -> Vec<Vec<(usize, usize)>> {
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut num: u64 = 0;
    let mut den: u64 = 1;
    for (i, &(n, d)) in ratios.iter().enumerate() {
        let n = n.max(1) as u64;
        let d = d.max(1) as u64;
        if current.is_empty() {
            current.push(i);
            num = n;
            den = d;
            continue;
        }
        let sum_n = num * d + n * den;
        let sum_d = den * d;
        if sum_n * 64 > sum_d * 65 {
            groups.push(std::mem::take(&mut current));
            current.push(i);
            num = n;
            den = d;
        } else {
            current.push(i);
            num = sum_n;
            den = sum_d;
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }

    groups
        .into_iter()
        .map(|idxs| allocate_row_outer_widths(&idxs, ratios, inner_width, gap))
        .collect()
}

fn allocate_row_outer_widths(
    idxs: &[usize],
    ratios: &[(usize, usize)],
    inner_width: usize,
    gap: usize,
) -> Vec<(usize, usize)> {
    let n = idxs.len().max(1);
    let gaps = gap * n.saturating_sub(1);
    let avail = inner_width.saturating_sub(gaps).max(n);
    let weights: Vec<f64> = idxs
        .iter()
        .map(|&i| {
            let (a, b) = ratios[i];
            a.max(1) as f64 / b.max(1) as f64
        })
        .collect();
    let sum: f64 = weights.iter().sum::<f64>().max(1e-9);
    let mut raw: Vec<usize> = weights
        .iter()
        .map(|w| ((avail as f64) * (*w / sum)).floor() as usize)
        .collect();
    let used: usize = raw.iter().sum();
    if avail > used {
        if let Some(last) = raw.last_mut() {
            *last += avail - used;
        }
    }
    for w in &mut raw {
        if *w == 0 {
            *w = 1;
        }
    }
    idxs.iter().copied().zip(raw).collect()
}

#[allow(dead_code)]
pub(in crate::tui) fn section_item_ascii_inner_width(
    width_class: &str,
    section_inner_width: usize,
) -> usize {
    let (num, den) = layout_dd_u_ratio(width_class);
    let packed = pack_fraction_rows(&[(num, den)], section_inner_width, 1);
    packed
        .first()
        .and_then(|row| row.first())
        .map(|&(_, outer)| outer.saturating_sub(4).max(1))
        .unwrap_or(1)
}

#[allow(dead_code)]
pub(in crate::tui) fn resolve_dd_u_ratio_for_panel(
    width_class: &str,
    panel_chars: usize,
) -> Option<(usize, usize)> {
    let current_bp = breakpoint_for_panel_chars(panel_chars);
    let mut found = [None; 6];
    for token in width_class.split_whitespace() {
        if let Some((bp, ratio)) = parse_dd_u_token_ratio(token) {
            found[bp.index()] = Some(ratio);
        }
    }
    let idx = current_bp.index();
    for i in (0..=idx).rev() {
        if let Some(ratio) = found[i] {
            return Some(ratio);
        }
    }
    found.iter().skip(idx + 1).copied().flatten().next()
}

pub(in crate::tui) fn parse_dd_u_token_ratio(
    token: &str,
) -> Option<(ResponsiveBp, (usize, usize))> {
    let value = token.strip_prefix("dd-u-")?;
    let parts = value.split('-').collect::<Vec<_>>();
    let (bp, num_raw, den_raw) = match parts.as_slice() {
        [num, den] => (ResponsiveBp::Base, *num, *den),
        [bp, num, den] => (
            match *bp {
                "sm" => ResponsiveBp::Sm,
                "md" => ResponsiveBp::Md,
                "lg" => ResponsiveBp::Lg,
                "xl" => ResponsiveBp::Xl,
                "xxl" => ResponsiveBp::Xxl,
                _ => return None,
            },
            *num,
            *den,
        ),
        _ => return None,
    };
    let num = num_raw.parse::<usize>().ok()?;
    let den = den_raw.parse::<usize>().ok()?;
    if den == 0 || num == 0 {
        return None;
    }
    Some((bp, (num.min(den), den)))
}

#[allow(dead_code)]
pub(in crate::tui) fn breakpoint_for_panel_chars(panel_chars: usize) -> ResponsiveBp {
    if panel_chars >= 180 {
        ResponsiveBp::Xxl
    } else if panel_chars >= 150 {
        ResponsiveBp::Xl
    } else if panel_chars >= 120 {
        ResponsiveBp::Lg
    } else if panel_chars >= 90 {
        ResponsiveBp::Md
    } else if panel_chars >= 60 {
        ResponsiveBp::Sm
    } else {
        ResponsiveBp::Base
    }
}

pub(in crate::tui) fn hero_ascii_map(hero: &crate::model::DdHero, panel_width: usize) -> AsciiMap {
    let inner_width = panel_width.saturating_sub(4).max(8);
    let border = format!("+{}+", "-".repeat(inner_width + 2));
    let state = cursor::hero_to_form_state(hero);
    let mut lines = vec![fit_ascii_cell("HERO", inner_width)];
    for line in blueprint_form_lines(&state, &[], inner_width) {
        lines.push(fit_ascii_cell(&line, inner_width));
    }
    let mut out = Vec::new();
    out.push(border.clone());
    for line in lines {
        out.push(format!("| {} |", line));
    }
    out.push(border);
    let hits = vec![vec![]; out.len()];
    let boxes = out
        .iter()
        .map(|l| vec![(0, l.chars().count(), 0)])
        .collect();
    AsciiMap::from_lines(out, hits, boxes)
}

pub(in crate::tui) fn fit_ascii_cell(value: &str, width: usize) -> String {
    let shortened = truncate_ascii(value, width);
    format!("{shortened:<width$}")
}

pub(in crate::tui) fn truncate_ascii(value: &str, max_chars: usize) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= max_chars {
        return value.to_string();
    }
    if max_chars <= 3 {
        return chars.into_iter().take(max_chars).collect();
    }
    let mut out = chars.into_iter().take(max_chars - 3).collect::<String>();
    out.push_str("...");
    out
}

#[derive(Clone, Copy)]
pub(in crate::tui) enum ResponsiveBp {
    Base,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
}

impl ResponsiveBp {
    pub(in crate::tui) fn index(self) -> usize {
        match self {
            ResponsiveBp::Base => 0,
            ResponsiveBp::Sm => 1,
            ResponsiveBp::Md => 2,
            ResponsiveBp::Lg => 3,
            ResponsiveBp::Xl => 4,
            ResponsiveBp::Xxl => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        ButtonStyle, CardLinkTarget, CtaClass, DdCta, DdHeadline, DdSection, DdSpacer,
        HeadingLevel, SalAnimation, SectionClass, SectionColumn, SectionComponent,
        SectionItemBoxClass, SpacerSize,
    };

    fn test_section(columns: Vec<SectionColumn>) -> DdSection {
        DdSection {
            id: "section-1".to_string(),
            section_title: Some("Ready to publish?".to_string()),
            section_class: Some(SectionClass::FullContained),
            item_box_class: Some(SectionItemBoxClass::LBox),
            bg: None,
            padding: None,
            custom_css: None,
            aria_label: None,
            sal: SalAnimation::NoAnimation,
            sal_duration: None,
            sal_delay: None,
            columns,
        }
    }

    fn empty_col(id: &str, width: &str) -> SectionColumn {
        SectionColumn {
            id: id.to_string(),
            width_class: width.to_string(),
            components: Vec::new(),
        }
    }

    #[test]
    fn layout_ratio_prefers_largest_breakpoint() {
        assert_eq!(layout_dd_u_ratio("dd-u-12-24"), (12, 24));
        assert_eq!(layout_dd_u_ratio("dd-u-1-1 dd-u-md-12-24"), (12, 24));
        assert_eq!(
            layout_dd_u_ratio("dd-u-1-1 dd-u-md-12-24 dd-u-lg-8-24"),
            (8, 24)
        );
        assert_eq!(layout_dd_u_ratio("dd-u-1-1"), (1, 1));
    }

    #[test]
    fn half_width_columns_share_one_blueprint_row() {
        let section = test_section(vec![
            empty_col("left", "dd-u-12-24"),
            empty_col("right", "dd-u-12-24"),
        ]);
        let map = section_ascii_map(&section, 0, 80);
        let paired = map
            .lines
            .iter()
            .filter(|l| l.contains("item: left") && l.contains("item: right"))
            .count();
        assert!(
            paired > 0,
            "12/24 columns should sit on one row, got:\n{}",
            map.lines.join("\n")
        );
    }

    #[test]
    fn md_half_layout_used_even_in_narrow_panel() {
        let section = test_section(vec![
            empty_col("a", "dd-u-1-1 dd-u-md-12-24"),
            empty_col("b", "dd-u-1-1 dd-u-md-12-24"),
        ]);
        let map = section_ascii_map(&section, 0, 50);
        let paired = map
            .lines
            .iter()
            .filter(|l| l.contains("item: a") && l.contains("item: b"))
            .count();
        assert!(
            paired > 0,
            "md 12/24 should pack side by side in a 50-col pane, got:\n{}",
            map.lines.join("\n")
        );
    }

    #[test]
    fn section_blueprint_lists_title_not_editor_chrome() {
        let section = test_section(vec![empty_col("column-1", "dd-u-1-1")]);
        let map = section_ascii_map(&section, 0, 80);
        let blob = map.lines.join("\n");
        assert!(blob.contains("title: Ready to publish?"), "{blob}");
        assert!(!blob.contains("ARIA label"), "{blob}");
        assert!(!blob.contains("Background"), "{blob}");
        assert!(!blob.contains("Padding"), "{blob}");
        assert!(!blob.contains("Animation"), "{blob}");
    }

    #[test]
    fn headline_blueprint_lists_title_and_level() {
        let section = test_section(vec![SectionColumn {
            id: "column-1".to_string(),
            width_class: "dd-u-1-1".to_string(),
            components: vec![SectionComponent::Headline(DdHeadline {
                text: "Services".to_string(),
                heading_level: HeadingLevel::H3,
                custom_css: Some("-center".to_string()),
                sal: SalAnimation::Fade,
                sal_duration: Some(500),
                sal_delay: None,
            })],
        }]);
        let blob = section_ascii_map(&section, 0, 80).lines.join("\n");
        assert!(blob.contains("- dd-headline"), "{blob}");
        assert!(blob.contains("title: Services"), "{blob}");
        assert!(blob.contains("level: h3"), "{blob}");
        assert!(!blob.contains("-center"), "{blob}");
        assert!(!blob.contains("Animation"), "{blob}");
        assert!(!blob.contains("fade"), "{blob}");
    }

    #[test]
    fn spacer_blueprint_label_and_divider_line() {
        let without = test_section(vec![SectionColumn {
            id: "column-1".to_string(),
            width_class: "dd-u-1-1".to_string(),
            components: vec![SectionComponent::Spacer(DdSpacer {
                size: SpacerSize::Lg,
                divider: false,
            })],
        }]);
        let blob = section_ascii_map(&without, 0, 80).lines.join("\n");
        assert!(blob.contains("- dd-spacer -lg"), "{blob}");
        assert!(!blob.contains("divider"), "{blob}");

        let with_div = test_section(vec![SectionColumn {
            id: "column-1".to_string(),
            width_class: "dd-u-1-1".to_string(),
            components: vec![SectionComponent::Spacer(DdSpacer {
                size: SpacerSize::Md,
                divider: true,
            })],
        }]);
        let map = section_ascii_map(&with_div, 0, 80);
        let blob = map.lines.join("\n");
        assert!(blob.contains("- dd-spacer -md"), "{blob}");
        let has_rule = map
            .lines
            .iter()
            .any(|l| !l.trim_start().starts_with('+') && l.contains("--------"));
        assert!(
            has_rule,
            "divider spacer should draw a horizontal rule, got:\n{blob}"
        );
    }

    #[test]
    fn hero_blueprint_lists_title_subtitle_copy_and_link_labels() {
        let site = crate::model::Site::starter();
        let crate::model::PageNode::Hero(hero) = &site.pages[0].nodes[0] else {
            panic!("starter hero");
        };
        let map = hero_ascii_map(hero, 80);
        let blob = map.lines.join("\n");
        assert!(blob.contains("title:"), "{blob}");
        assert!(blob.contains("subtitle:"), "{blob}");
        assert!(blob.contains("copy:"), "{blob}");
        assert!(blob.contains("links:"), "{blob}");
        assert!(blob.contains("Get Started"), "{blob}");
        assert!(!blob.contains("Animation"), "{blob}");
        assert!(!blob.contains("overlay"), "{blob}");
        assert!(!blob.contains("Hero ID"), "{blob}");
        assert!(
            !blob.contains("media_kind") && !blob.contains("Media"),
            "{blob}"
        );
    }

    #[test]
    fn cta_blueprint_lists_component_fields() {
        let cta = DdCta {
            parent_class: CtaClass::TopLeft,
            parent_image_url: "/a.jpg".to_string(),
            parent_image_alt: "alt".to_string(),
            sal: SalAnimation::Fade,
            sal_duration: None,
            sal_delay: None,
            parent_title: "Get in touch".to_string(),
            parent_subtitle: "Subtitle".to_string(),
            parent_copy: "Copy".to_string(),
            links: vec![crate::model::DdLink {
                url: "/go".to_string(),
                label: "Go".to_string(),
                target: CardLinkTarget::SelfTarget,
                style: ButtonStyle::Primary,
            }],
            parent_link_url: None,
            parent_link_target: None,
            parent_link_label: None,
        };
        let section = test_section(vec![SectionColumn {
            id: "column-1".to_string(),
            width_class: "dd-u-1-1".to_string(),
            components: vec![SectionComponent::Cta(cta)],
        }]);
        let map = section_ascii_map(&section, 0, 80);
        let blob = map.lines.join("\n");
        assert!(blob.contains("dd-cta"), "{blob}");
        assert!(blob.contains("title: Get in touch"), "{blob}");
        assert!(blob.contains("subtitle: Subtitle"), "{blob}");
        assert!(blob.contains("copy: Copy"), "{blob}");
        assert!(blob.contains("links: Go"), "{blob}");
        assert!(!blob.contains("Image URL"), "{blob}");
        assert!(!blob.contains("Animation"), "{blob}");
    }
}
