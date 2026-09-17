//! Details panel text, footer hints, and click-to-select.
use super::super::*;
use super::*;

fn is_footer_chrome(part: &str) -> bool {
    matches!(part, "F1:Help" | "F2:Theme" | "Esc:Close" | "Ctrl+Q:Quit")
}

const MOUSE_HINT: &str = "(mouse: click/scroll)";

impl App {
    pub(in crate::tui) fn footer_hint(&self, width: u16) -> String {
        if width == 0 {
            return String::new();
        }
        let overlay = self.overlay.is_some() || self.modal.is_some();
        let mut parts: Vec<&str> = vec!["F1:Help", "F2:Theme"];
        if overlay {
            parts.push("Esc:Close");
            parts.push("Ctrl+Q:Quit");
        } else {
            match self.selected_sidebar_section {
                SidebarSection::Pages => {
                    if width < 80 {
                        parts.push("Ctrl+Q:Quit");
                        parts.push("r:Rename");
                    } else {
                        parts.extend_from_slice(&[
                            "Shift+A:Add",
                            "Shift+X:Del",
                            "u:Undo-page",
                            "r:Rename",
                            "Shift+J/K:Move",
                            "Ctrl+Q:Quit",
                        ]);
                    }
                }
                SidebarSection::Regions => {
                    if width < 80 {
                        parts.push("Ctrl+Q:Quit");
                        parts.push("Enter:Edit");
                    } else {
                        parts.extend_from_slice(&[
                            "j/k:Site/Header/Footer",
                            "Enter:Edit",
                            "Ctrl+Q:Quit",
                        ]);
                    }
                }
                SidebarSection::Layouts => {
                    if width < 80 {
                        parts.push("Ctrl+Q:Quit");
                        parts.push("Enter:Edit");
                    } else if width < 110 {
                        parts.extend_from_slice(&[
                            "/:Insert",
                            "d:Del",
                            "y:Dup",
                            "u:Undo-tree",
                            "r:Col-id",
                            "J/K:Move",
                            "Ctrl+Q:Quit",
                        ]);
                    } else {
                        parts.extend_from_slice(&[
                            "/:Insert",
                            "Enter:Edit",
                            "d:Del",
                            "y:Dup",
                            "u:Undo-tree",
                            "r:Col-id",
                            "J/K:Move",
                            "p:Preview",
                            "Ctrl+Q:Quit",
                        ]);
                    }
                }
                SidebarSection::Details => {
                    if width < 80 {
                        parts.push("Ctrl+Q:Quit");
                        parts.push("j/k:Scroll");
                    } else {
                        parts.extend_from_slice(&["j/k:Scroll", "Enter:Edit", "Ctrl+Q:Quit"]);
                    }
                }
            }
        }
        let prefix = if self.dirty { "*  " } else { "" };
        let max = width as usize;
        // Drop trailing scoped tokens until chrome+actions fit. Never clip mid-key.
        // Mouse is appended only when that line already fits, so widening cannot hide a key.
        loop {
            let joined = format!("{prefix}{}", parts.join("  "));
            if joined.chars().count() <= max {
                break;
            }
            if let Some(idx) = parts.iter().rposition(|p| !is_footer_chrome(p)) {
                parts.remove(idx);
            } else if !parts.is_empty() {
                parts.pop();
            } else {
                return String::new();
            }
        }
        if !overlay && width >= 110 {
            let with_mouse = format!("{prefix}{}  {MOUSE_HINT}", parts.join("  "));
            if with_mouse.chars().count() <= max {
                parts.push(MOUSE_HINT);
            }
        }
        format!("{prefix}{}", parts.join("  "))
    }
    pub(in crate::tui) fn details_text(&self, detail_width: usize) -> DetailsView {
        match self.selected_region {
            SelectedRegion::Site => self.site_details_text(),
            SelectedRegion::Header => self.header_details_text(detail_width),
            SelectedRegion::Footer => self.footer_details_text(detail_width),
            SelectedRegion::Page => self.page_details_text(detail_width),
        }
    }
    pub(in crate::tui) fn site_details_text(&self) -> DetailsView {
        let mut view = DetailsView::new();
        let site_focus = matches!(self.blueprint_focus(), BlueprintFocus::Site);
        view.push_styled(
            "Site settings",
            if site_focus {
                BlueprintStyle::FocusFill
            } else {
                BlueprintStyle::Label
            },
        );
        view.push_plain("");
        view.push_plain(format!("name: {}", self.site.name));
        view.push_plain(format!("lang: {}", self.site.lang));
        view.push_plain(format!(
            "base_url: {}",
            self.site.base_url.as_deref().unwrap_or("")
        ));
        view.push_plain(format!(
            "export_dir: {}",
            self.site.export_dir.as_deref().unwrap_or("")
        ));
        paint_theme_color_line(&mut view, "primary_color", &self.site.theme.primary_color);
        paint_theme_color_line(
            &mut view,
            "secondary_color",
            &self.site.theme.secondary_color,
        );
        paint_theme_color_line(&mut view, "tertiary_color", &self.site.theme.tertiary_color);
        paint_theme_color_line(&mut view, "support_color", &self.site.theme.support_color);
        view
    }
    pub(in crate::tui) fn header_details_text(&self, detail_width: usize) -> DetailsView {
        let mut view = DetailsView::new();
        view.push_styled("Site header", BlueprintStyle::Label);
        view.push_plain("");
        let marker = if matches!(self.selected_region, SelectedRegion::Header) {
            "*"
        } else {
            " "
        };
        let decl = format!("{}[01] dd-header {}", marker, self.site.header.id);
        view.push_plain(decl);
        let decl_idx = view.lines.len() - 1;
        let map = header_ascii_map(
            &self.site.header,
            self.selected_header_section,
            self.selected_header_column,
            detail_width,
        );
        let (map_start, map_end, _) = view.extend_map(map);
        paint_header_region(
            &mut view,
            decl_idx,
            map_start,
            map_end,
            &self.site.header.sections,
            self.blueprint_focus(),
            true,
            &self.site.theme,
        );
        view.push_plain("");
        view.push_styled(
            format!(
                "Selected: {} | Insert mode: {}",
                self.header_selection_summary(),
                self.component_kind.label()
            ),
            BlueprintStyle::Label,
        );
        view
    }
    pub(in crate::tui) fn footer_details_text(&self, detail_width: usize) -> DetailsView {
        let mut view = DetailsView::new();
        view.push_styled("Site footer", BlueprintStyle::Label);
        view.push_plain("");
        let marker = if matches!(self.selected_region, SelectedRegion::Footer) {
            "*"
        } else {
            " "
        };
        view.push_plain(format!("{}[01] dd-footer {}", marker, self.site.footer.id));
        let decl_idx = view.lines.len() - 1;
        let map = footer_ascii_map(
            &self.site.footer,
            self.selected_header_section,
            self.selected_header_column,
            detail_width,
        );
        let (map_start, map_end, _) = view.extend_map(map);
        paint_header_region(
            &mut view,
            decl_idx,
            map_start,
            map_end,
            &self.site.footer.sections,
            self.blueprint_focus(),
            false,
            &self.site.theme,
        );
        view
    }
    pub(in crate::tui) fn page_details_text(&self, detail_width: usize) -> DetailsView {
        let page = self.current_page();
        if page.nodes.is_empty() {
            let mut view = DetailsView::new();
            view.push_plain("No nodes on this page.");
            return view;
        }
        let focus = self.blueprint_focus();
        let mut view = DetailsView::new();
        view.push_styled(
            format!("Page blueprint: {}", page.head.title),
            BlueprintStyle::Label,
        );
        view.push_plain("");
        for (idx, node) in page.nodes.iter().enumerate() {
            let marker = if idx == self.selected_node { "*" } else { " " };
            let node_focus = match focus {
                BlueprintFocus::Page { node, depth } if node == idx => Some(depth),
                _ => None,
            };
            match node {
                PageNode::Hero(v) => {
                    view.push_plain(format!("{marker}[{:02}] dd-hero", idx + 1));
                    let decl_idx = view.lines.len() - 1;
                    if node_focus.is_some() {
                        let style = if matches!(node_focus, Some(FocusDepth::Root)) {
                            BlueprintStyle::FocusFill
                        } else {
                            BlueprintStyle::Focus
                        };
                        view.paint_line(decl_idx, style);
                    }
                    let map = hero_ascii_map(v, detail_width);
                    let (map_start, map_end, _) = view.extend_map(map);
                    if node_focus.is_some() {
                        for i in map_start..map_end {
                            view.paint_line(i, BlueprintStyle::Focus);
                        }
                        if map_start + 1 < map_end {
                            view.paint_line(map_start + 1, BlueprintStyle::FocusFill);
                        }
                    }
                }
                PageNode::Section(v) => {
                    view.push_plain(format!("{marker}[{:02}] dd-section {}", idx + 1, v.id));
                    let decl_idx = view.lines.len() - 1;
                    if node_focus.is_some() {
                        let style = if matches!(node_focus, Some(FocusDepth::Root)) {
                            BlueprintStyle::FocusFill
                        } else {
                            BlueprintStyle::Focus
                        };
                        view.paint_line(decl_idx, style);
                    }
                    let map = section_ascii_map(
                        v,
                        if idx == self.selected_node {
                            self.selected_column
                        } else {
                            0
                        },
                        detail_width,
                    );
                    let (map_start, map_end, boxes) = view.extend_map(map);
                    paint_section_map(
                        &mut view,
                        map_start,
                        map_end,
                        &boxes,
                        v,
                        node_focus,
                        &self.site.theme,
                    );
                }
            }
            view.push_plain("");
        }
        view.push_styled(
            format!(
                "Selected: {} | Insert mode: {}",
                self.selection_summary(),
                self.component_kind.label()
            ),
            BlueprintStyle::Label,
        );
        view
    }
    pub(in crate::tui) fn details_max_scroll(&self) -> usize {
        let visible_rows = self.details_area.height.saturating_sub(2) as usize;
        if visible_rows == 0 {
            return 0;
        }
        let detail_width = self.details_area.width.saturating_sub(2) as usize;
        if detail_width == 0 {
            return 0;
        }
        let view = self.details_text(detail_width);
        let total_rows = view.lines.len().max(1);
        total_rows.saturating_sub(visible_rows)
    }
    pub(in crate::tui) fn scroll_details_by(&mut self, delta: isize) {
        let max_scroll = self.details_max_scroll() as isize;
        let next = self.details_scroll_row as isize + delta;
        self.details_scroll_row = next.clamp(0, max_scroll) as usize;
    }

    pub(in crate::tui) fn select_item_from_details_click(
        &mut self,
        text_line: usize,
        char_x: usize,
    ) {
        let detail_w = self.details_area.width.saturating_sub(2) as usize;
        if detail_w == 0 {
            return;
        }
        let generated = self.details_text(detail_w);
        if text_line >= generated.lines.len() {
            return;
        }
        // Prefer draw-time hits; regenerate only when the stored map is missing this line.
        let line_segs = self
            .details_hits
            .get(text_line)
            .or_else(|| generated.hits.get(text_line))
            .cloned()
            .unwrap_or_default();
        let lines: Vec<&str> = generated.lines.iter().map(|s| s.as_str()).collect();
        match self.selected_region {
            SelectedRegion::Site => return,
            SelectedRegion::Header | SelectedRegion::Footer => {
                self.select_header_from_details_lines(&lines, text_line);
                self.apply_header_details_hits(&line_segs, char_x);
            }
            SelectedRegion::Page => {
                self.select_page_from_details_lines(&lines, text_line, char_x, &line_segs);
            }
        }
        // Set tree row to the most specific (deepest) matching row for the selection level.
        // This makes tree highlight follow the clicked item, and double-click edit the right thing.
        let rows = self.build_tree_rows();
        if rows.is_empty() {
            return;
        }
        let clicked_line = lines[text_line];
        // For decl lines, use MAX for lower levels so only the decl row matches predicate (ancestors do but we pick specific).
        let mut tcol = self.selected_column;
        let mut tcomp = self.selected_component;
        let mut hsec = self.selected_header_section;
        let mut hcol = self.selected_header_column;
        let mut hcomp = self.selected_header_component;
        if clicked_line.contains('[')
            && (clicked_line.contains("dd-hero")
                || clicked_line.contains("dd-section")
                || clicked_line.contains("dd-header")
                || clicked_line.contains("dd-footer"))
        {
            tcol = usize::MAX;
            tcomp = usize::MAX;
            hcol = usize::MAX;
            hcomp = usize::MAX;
            // Header/footer have a Root above Section; MAX section so the tree lands on Root.
            if clicked_line.contains("dd-header") || clicked_line.contains("dd-footer") {
                hsec = usize::MAX;
            }
        } else if clicked_line.contains("column: ") || clicked_line.contains("item: ") {
            tcomp = usize::MAX;
            hcomp = usize::MAX;
        } else if clicked_line.contains("section: ") {
            hcol = usize::MAX;
            hcomp = usize::MAX;
        }
        let matches = |r: &TreeRow| -> bool {
            match r.kind {
                TreeRowKind::SiteRoot
                | TreeRowKind::HeaderRoot { .. }
                | TreeRowKind::FooterRoot => true,
                TreeRowKind::HeaderSection { section_idx }
                | TreeRowKind::FooterSection { section_idx } => section_idx == hsec,
                TreeRowKind::HeaderColumn {
                    section_idx,
                    column_idx,
                }
                | TreeRowKind::FooterColumn {
                    section_idx,
                    column_idx,
                } => section_idx == hsec && column_idx == hcol,
                TreeRowKind::HeaderComponent {
                    section_idx,
                    column_idx,
                    component_idx,
                }
                | TreeRowKind::FooterComponent {
                    section_idx,
                    column_idx,
                    component_idx,
                } => section_idx == hsec && column_idx == hcol && component_idx == hcomp,
                TreeRowKind::Hero { node_idx } | TreeRowKind::Section { node_idx } => {
                    node_idx == self.selected_node
                }
                TreeRowKind::Column {
                    node_idx,
                    column_idx,
                } => node_idx == self.selected_node && column_idx == tcol,
                TreeRowKind::Component {
                    node_idx,
                    column_idx,
                    component_idx,
                } => node_idx == self.selected_node && column_idx == tcol && component_idx == tcomp,
                _ => false,
            }
        };
        if let Some((i, _)) = rows.iter().enumerate().rev().find(|(_, r)| matches(r)) {
            self.selected_tree_row = i;
        }
    }

    pub(in crate::tui) fn select_page_from_details_lines(
        &mut self,
        lines: &[&str],
        up_to: usize,
        char_x: usize,
        line_segs: &[(usize, usize, usize, usize)],
    ) {
        let mut node_idx = None;
        let mut col_idx = 0usize;
        let mut comp_idx = 0usize;
        let mut cols_since = 0usize;
        let mut comps_since = 0usize;
        for (_i, &l) in lines.iter().enumerate().take(up_to + 1) {
            if let Some(br) = l.find('[') {
                if let Some(er) = l[br + 1..].find(']') {
                    let ns = &l[br + 1..br + 1 + er];
                    if let Ok(n) = ns.trim().parse::<usize>() {
                        if l.contains("dd-hero") || l.contains("dd-section") {
                            node_idx = Some(n.saturating_sub(1));
                            cols_since = 0;
                            comps_since = 0;
                            col_idx = 0;
                            comp_idx = 0;
                        }
                    }
                }
            }
            if node_idx.is_some() {
                let t = l.trim();
                if t.contains("item: ") || t.contains(" column: ") {
                    col_idx = cols_since;
                    cols_since += 1;
                    comps_since = 0;
                    comp_idx = 0;
                }
                if t.contains("dd-") && !t.contains("dd-section") && !t.contains("dd-hero") {
                    comp_idx = comps_since;
                    comps_since += 1;
                }
            }
        }
        if let Some(n) = node_idx {
            let page = self.current_page();
            if n < page.nodes.len() {
                self.selected_node = n;
                self.selected_column = col_idx;
                self.selected_component = comp_idx;
            }
        }
        // Stored (or generated) component segments win over string-contains when the row has hits.
        for &(x0, x1, c, cp) in line_segs {
            if char_x >= x0 && char_x < x1 {
                if let Some(n) = node_idx.or(Some(self.selected_node)) {
                    let page = self.current_page();
                    if n < page.nodes.len() {
                        self.selected_node = n;
                        self.selected_column = c;
                        self.selected_component = cp;
                    }
                }
                break;
            }
        }
    }

    fn apply_header_details_hits(
        &mut self,
        line_segs: &[(usize, usize, usize, usize)],
        char_x: usize,
    ) {
        for &(x0, x1, c, cp) in line_segs {
            if char_x >= x0 && char_x < x1 {
                self.selected_header_column = c;
                self.selected_header_component = cp;
                break;
            }
        }
    }

    pub(in crate::tui) fn select_header_from_details_lines(
        &mut self,
        lines: &[&str],
        up_to: usize,
    ) {
        let mut sec_idx = 0usize;
        let mut col_idx = 0usize;
        let mut comp_idx = 0usize;
        let mut secs = 0usize;
        let mut cols = 0usize;
        let mut comps = 0usize;
        for (_i, &l) in lines.iter().enumerate().take(up_to + 1) {
            let t = l.trim();
            if t.contains("section: ") {
                sec_idx = secs;
                secs += 1;
                cols = 0;
                comps = 0;
                col_idx = 0;
                comp_idx = 0;
            } else if t.contains("column: ") {
                col_idx = cols;
                cols += 1;
                comps = 0;
                comp_idx = 0;
            } else if t.contains("dd-") && !t.contains("section:") {
                comp_idx = comps;
                comps += 1;
            }
        }
        self.selected_header_section = sec_idx;
        self.selected_header_column = col_idx;
        self.selected_header_component = comp_idx;
    }
}

fn paint_theme_color_line(view: &mut DetailsView, label: &str, hex: &str) {
    let line = format!("{label}: {hex}");
    view.push_plain(line);
    let idx = view.lines.len() - 1;
    let label_len = label.chars().count();
    view.paint(idx, 0, label_len, BlueprintStyle::Label);
    if let Some(style) = brand_from_hex(hex) {
        let start = view.lines[idx].find(hex).unwrap_or(label_len + 2);
        view.paint(idx, start, start + hex.chars().count(), style);
    }
}

fn paint_section_map(
    view: &mut DetailsView,
    map_start: usize,
    map_end: usize,
    boxes: &[Vec<(usize, usize, usize)>],
    section: &crate::model::DdSection,
    node_focus: Option<FocusDepth>,
    theme: &crate::model::ThemeSettings,
) {
    let columns = section_columns_ref(section);
    match node_focus {
        Some(FocusDepth::Root) => {
            for i in map_start..map_end {
                view.paint_line(i, BlueprintStyle::Focus);
            }
            if map_start + 1 < map_end {
                view.paint_line(map_start + 1, BlueprintStyle::FocusFill);
            }
        }
        Some(FocusDepth::Column(col)) => {
            paint_column_boxes(view, map_start, boxes, col, BlueprintStyle::Focus);
        }
        Some(FocusDepth::Component { column, component }) => {
            paint_column_boxes(view, map_start, boxes, column, BlueprintStyle::Focus);
            let fills: Vec<(usize, usize, usize)> = view
                .hits
                .iter()
                .enumerate()
                .skip(map_start)
                .take(map_end.saturating_sub(map_start))
                .flat_map(|(offset, segs)| {
                    segs.iter().filter_map(move |&(x0, x1, c, cp)| {
                        (c == column && cp == component).then_some((offset, x0, x1))
                    })
                })
                .collect();
            for (offset, x0, x1) in fills {
                view.paint(offset, x0, x1, BlueprintStyle::FocusFill);
            }
        }
        None => {}
    }

    let mut brands = Vec::new();
    for offset in map_start..map_end {
        if let Some(segs) = view.hits.get(offset) {
            for &(x0, x1, c, cp) in segs {
                if let Some(style) = columns
                    .get(c)
                    .and_then(|col| col.components.get(cp))
                    .and_then(|component| component_token_style(component, theme))
                {
                    brands.push((offset, x0, x1, style));
                }
            }
        }
    }
    for (offset, x0, x1, style) in brands {
        view.paint(offset, x0, x1, style);
    }
}

fn paint_column_boxes(
    view: &mut DetailsView,
    map_start: usize,
    boxes: &[Vec<(usize, usize, usize)>],
    column: usize,
    style: BlueprintStyle,
) {
    for (i, segs) in boxes.iter().enumerate() {
        let line = map_start + i;
        for &(x0, x1, c) in segs {
            if c == column {
                view.paint(line, x0, x1, style);
            }
        }
    }
}

fn paint_header_region(
    view: &mut DetailsView,
    decl_idx: usize,
    map_start: usize,
    map_end: usize,
    sections: &[crate::model::DdSection],
    focus: BlueprintFocus,
    is_header: bool,
    theme: &crate::model::ThemeSettings,
) {
    let depth = match (focus, is_header) {
        (BlueprintFocus::Header { depth }, true) => Some(depth),
        (BlueprintFocus::Footer { depth }, false) => Some(depth),
        _ => None,
    };
    if let Some(d) = depth {
        let decl_style = if matches!(d, HeaderFocusDepth::Root) {
            BlueprintStyle::FocusFill
        } else {
            BlueprintStyle::Focus
        };
        view.paint_line(decl_idx, decl_style);
        if matches!(d, HeaderFocusDepth::Root) {
            for i in map_start..map_end {
                view.paint_line(i, BlueprintStyle::Focus);
            }
            if map_start + 1 < map_end {
                view.paint_line(map_start + 1, BlueprintStyle::FocusFill);
            }
        }
    }

    let mut sec = 0usize;
    let mut col = 0usize;
    let mut secs = 0usize;
    let mut cols = 0usize;
    let mut comps = 0usize;
    for i in map_start..map_end {
        let line = view.lines[i].clone();
        let t = line.trim();
        if t.contains("section: ") {
            sec = secs;
            secs += 1;
            cols = 0;
            comps = 0;
            col = 0;
            if let Some(style) = header_line_style(depth, HeaderFocusDepth::Section(sec)) {
                view.paint_line(i, style);
            }
        } else if t.contains("column: ") {
            col = cols;
            cols += 1;
            comps = 0;
            if let Some(style) = header_line_style(
                depth,
                HeaderFocusDepth::Column {
                    section: sec,
                    column: col,
                },
            ) {
                view.paint_line(i, style);
            }
        } else if t.contains("dd-") && !t.contains("section:") {
            let comp = comps;
            comps += 1;
            if let Some(style) = header_line_style(
                depth,
                HeaderFocusDepth::Component {
                    section: sec,
                    column: col,
                    component: comp,
                },
            ) {
                view.paint_line(i, style);
            }
            if let Some(component) = sections
                .get(sec)
                .and_then(|s| s.columns.get(col))
                .and_then(|c| c.components.get(comp))
            {
                if let Some(style) = component_token_style(component, theme) {
                    view.paint_line(i, style);
                }
            }
        }
    }
}

fn header_line_style(
    focus: Option<HeaderFocusDepth>,
    line: HeaderFocusDepth,
) -> Option<BlueprintStyle> {
    let focus = focus?;
    let exact = match (focus, line) {
        (HeaderFocusDepth::Root, HeaderFocusDepth::Root) => true,
        (HeaderFocusDepth::Section(a), HeaderFocusDepth::Section(b)) => a == b,
        (
            HeaderFocusDepth::Column {
                section: fs,
                column: fc,
            },
            HeaderFocusDepth::Column {
                section: ls,
                column: lc,
            },
        ) => fs == ls && fc == lc,
        (
            HeaderFocusDepth::Component {
                section: fs,
                column: fc,
                component: fp,
            },
            HeaderFocusDepth::Component {
                section: ls,
                column: lc,
                component: lp,
            },
        ) => fs == ls && fc == lc && fp == lp,
        _ => false,
    };
    if exact {
        return Some(BlueprintStyle::FocusFill);
    }
    let ancestor = match (focus, line) {
        (
            HeaderFocusDepth::Column { section: fs, .. }
            | HeaderFocusDepth::Component { section: fs, .. },
            HeaderFocusDepth::Section(ls),
        ) => fs == ls,
        (
            HeaderFocusDepth::Component {
                section: fs,
                column: fc,
                ..
            },
            HeaderFocusDepth::Column {
                section: ls,
                column: lc,
            },
        ) => fs == ls && fc == lc,
        _ => false,
    };
    if ancestor {
        Some(BlueprintStyle::Focus)
    } else {
        None
    }
}
