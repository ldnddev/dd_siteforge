//! Frame layout: header, sidebar, details, footer, overlays.
use super::*;
use ratatui::text::{Line, Span};

/// Sidebar block title. Compact when pane width (incl. borders) is under 22:
/// 25% of an 80-col frame is 20, which still fits `[1]` / `[2]` / `[3]`.
pub(super) fn sidebar_pane_title(
    section: SidebarSection,
    width: u16,
    page_idx: usize,
    page_count: usize,
) -> String {
    let compact = width < 22;
    match section {
        SidebarSection::Regions => {
            if compact {
                "[1]".to_string()
            } else {
                "[1] Regions".to_string()
            }
        }
        SidebarSection::Pages => {
            if page_count == 0 {
                if compact {
                    "[2]".to_string()
                } else {
                    "[2] Pages".to_string()
                }
            } else if compact {
                format!("[2] {page_idx}/{page_count}")
            } else {
                format!("[2] Pages {page_idx}/{page_count}")
            }
        }
        SidebarSection::Layouts => {
            if compact {
                "[3]".to_string()
            } else {
                "[3] Layout".to_string()
            }
        }
        SidebarSection::Details => String::new(),
    }
}

/// Right half of the frame, between the header and the footer.
fn settings_sheet_area(frame_area: Rect) -> Rect {
    let y = frame_area.y.saturating_add(3);
    let height = frame_area.height.saturating_sub(4);
    let width = (frame_area.width / 2).max(1).min(frame_area.width);
    Rect {
        x: frame_area.x + frame_area.width.saturating_sub(width),
        y,
        width,
        height,
    }
}

fn settings_top(title: &str, width: usize, edge: Style, title_style: Style) -> Line<'static> {
    let width = width.max(2);
    let inner = width - 2;
    let mut label = format!(" {title} ");
    if label.chars().count() > inner {
        label = label.chars().take(inner).collect();
    }
    let pad = inner - label.chars().count();
    Line::from(vec![
        Span::styled("┌", edge),
        Span::styled(label, title_style),
        Span::styled(format!("{}┐", "─".repeat(pad)), edge),
    ])
}

fn settings_bottom(width: usize, edge: Style) -> Line<'static> {
    let width = width.max(2);
    Line::from(Span::styled(format!("└{}┘", "─".repeat(width - 2)), edge))
}

fn settings_row(body: &str, width: usize, edge: Style, fill: Style) -> Line<'static> {
    let width = width.max(2);
    let inner = width - 2;
    let mut text: String = body.chars().take(inner).collect();
    let pad = inner - text.chars().count();
    text.push_str(&" ".repeat(pad));
    Line::from(vec![
        Span::styled("│", edge),
        Span::styled(text, fill),
        Span::styled("│", edge),
    ])
}

impl App {
    pub(super) fn draw(&mut self, frame: &mut ratatui::Frame) {
        self.prune_toasts();

        // Full-screen app shell base layer (base_background + text_primary).
        frame.render_widget(Block::default().style(self.theme.app_shell), frame.area());

        let page_label = {
            let page = self.current_page();
            if page.head.title.trim().is_empty() {
                page.slug.clone()
            } else {
                page.head.title.clone()
            }
        };
        let page_idx = self.page_tree_ordinal(self.selected_page);
        let details_title = format!("Details — {:02}: {}", page_idx, page_label);
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // HEADER (fixed, per LDNDDEV standard)
                Constraint::Min(0),    // Main content area
                Constraint::Length(1), // FOOTER (fixed, decluttered keys only)
            ])
            .split(frame.area());
        let main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(if self.details_open {
                [Constraint::Percentage(25), Constraint::Percentage(75)]
            } else {
                [Constraint::Percentage(100), Constraint::Length(0)]
            })
            .split(root[1]);

        // === NEW 3-line header (bordered, title=project, content=header_copy) ===
        let header_block = Block::default()
            .title("dd_siteforge")
            .borders(Borders::ALL)
            .border_style(self.theme.active_border)
            .style(self.theme.app_shell)
            .title_style(
                Style::default()
                    .fg(self.theme.text_labels)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(header_block.clone(), root[0]);

        if root[0].height >= 3 {
            let inner = header_block.inner(root[0]);
            let quote = Paragraph::new(self.header_copy.as_str()).style(
                Style::default()
                    .fg(self.theme.text_primary)
                    .bg(self.theme.base_background),
            );
            frame.render_widget(quote, inner);
        }
        // (no mouse/keyboard capture for header — purely decorative)

        // Regions section (Site, Header, Footer)
        let regions_items: Vec<ListItem> = vec!["  Site", "  Header", "  Footer"]
            .iter()
            .enumerate()
            .map(|(idx, label)| {
                let is_selected = match self.selected_region {
                    SelectedRegion::Site => idx == 0,
                    SelectedRegion::Header => idx == 1,
                    SelectedRegion::Footer => idx == 2,
                    SelectedRegion::Page => false,
                };
                let style = if is_selected {
                    Style::default()
                        .fg(self.theme.text_active_focus)
                        .bg(self.theme.selected_background)
                } else {
                    Style::default().fg(self.theme.text_primary)
                };
                ListItem::new(*label).style(style)
            })
            .collect();

        // Length = 2 border rows + visible items. Pages cap shrinks so Layout keeps Min(1).
        let page_rows = self.build_pages_panel_rows();
        let visible_page_count = page_rows.len();
        let regions_height = if self.regions_open {
            2 + regions_items.len()
        } else {
            0
        };
        let remaining = (main[0].height as usize).saturating_sub(regions_height);
        let pages_cap = remaining.saturating_sub(1 + 2).clamp(1, 10);
        let pages_height = 2 + visible_page_count.clamp(1, pages_cap);

        let sidebar = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(regions_height as u16),
                Constraint::Length(pages_height as u16),
                Constraint::Min(1),
            ])
            .split(main[0]);

        // Determine border colors based on active section
        let regions_border = if self.selected_sidebar_section == SidebarSection::Regions {
            self.theme.border_active
        } else {
            self.theme.border
        };
        let pages_border = if self.selected_sidebar_section == SidebarSection::Pages {
            self.theme.border_active
        } else {
            self.theme.border
        };
        let layouts_border = if self.selected_sidebar_section == SidebarSection::Layouts {
            self.theme.border_active
        } else {
            self.theme.border
        };
        let details_border = if self.selected_sidebar_section == SidebarSection::Details {
            self.theme.border_active
        } else {
            self.theme.border
        };

        let regions_list = List::new(regions_items)
            .block(
                Block::default()
                    .title(sidebar_pane_title(
                        SidebarSection::Regions,
                        sidebar[0].width,
                        0,
                        0,
                    ))
                    .borders(Borders::ALL)
                    .style(
                        Style::default()
                            .fg(self.theme.text_primary)
                            .bg(self.theme.body_background),
                    )
                    .border_style(Style::default().fg(regions_border))
                    .title_style(
                        Style::default()
                            .fg(
                                if self.selected_sidebar_section == SidebarSection::Regions {
                                    self.theme.text_active_focus
                                } else {
                                    self.theme.text_labels
                                },
                            )
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .style(
                Style::default()
                    .fg(self.theme.text_primary)
                    .bg(self.theme.body_background),
            )
            .highlight_style(
                Style::default()
                    .fg(self.theme.text_active_focus)
                    .bg(self.theme.selected_background)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        let mut regions_state = ListState::default();
        let regions_selected = match self.selected_region {
            SelectedRegion::Site => Some(0),
            SelectedRegion::Header => Some(1),
            SelectedRegion::Footer => Some(2),
            SelectedRegion::Page => None,
        };
        regions_state.select(regions_selected);
        if self.regions_open {
            frame.render_stateful_widget(regions_list, sidebar[0], &mut regions_state);
            self.regions_area = sidebar[0];
        } else {
            self.regions_area = Rect::default();
        }

        // Pages section (nested by slug; parents expand/collapse)
        let page_items: Vec<ListItem> = page_rows
            .iter()
            .map(|row| {
                let label = self.page_tree_label(row);
                let style = if row.page_idx == self.selected_page {
                    Style::default()
                        .fg(self.theme.text_active_focus)
                        .bg(self.theme.selected_background)
                } else {
                    Style::default().fg(self.theme.text_primary)
                };
                ListItem::new(label).style(style)
            })
            .collect();
        let pages_title = sidebar_pane_title(
            SidebarSection::Pages,
            sidebar[1].width,
            self.page_tree_ordinal(self.selected_page),
            self.site.pages.len(),
        );
        let pages_list = List::new(page_items)
            .block(
                Block::default()
                    .title(pages_title)
                    .borders(Borders::ALL)
                    .style(
                        Style::default()
                            .fg(self.theme.text_primary)
                            .bg(self.theme.body_background),
                    )
                    .border_style(Style::default().fg(pages_border))
                    .title_style(
                        Style::default()
                            .fg(if self.selected_sidebar_section == SidebarSection::Pages {
                                self.theme.text_active_focus
                            } else {
                                self.theme.text_labels
                            })
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .style(
                Style::default()
                    .fg(self.theme.text_primary)
                    .bg(self.theme.body_background),
            )
            .highlight_style(
                Style::default()
                    .fg(self.theme.text_active_focus)
                    .bg(self.theme.selected_background)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        if let Some(vis) = page_rows
            .iter()
            .position(|r| r.page_idx == self.selected_page)
        {
            self.pages_list_state.select(Some(vis));
        } else {
            self.pages_list_state.select(None);
        }
        frame.render_stateful_widget(pages_list, sidebar[1], &mut self.pages_list_state);
        self.pages_area = sidebar[1];

        // Layouts section (component tree)
        let tree_rows = self.build_tree_rows();
        let layout_items: Vec<ListItem> = tree_rows
            .iter()
            .enumerate()
            .map(|(idx, row)| {
                let label = self.tree_row_label(row);
                let style = if idx == self.selected_tree_row {
                    Style::default()
                        .fg(self.theme.text_active_focus)
                        .bg(self.theme.selected_background)
                } else {
                    Style::default().fg(self.theme.text_primary)
                };
                ListItem::new(label).style(style)
            })
            .collect();
        let layouts_list = List::new(layout_items)
            .block(
                Block::default()
                    .title(sidebar_pane_title(
                        SidebarSection::Layouts,
                        sidebar[2].width,
                        0,
                        0,
                    ))
                    .borders(Borders::ALL)
                    .style(
                        Style::default()
                            .fg(self.theme.text_primary)
                            .bg(self.theme.body_background),
                    )
                    .border_style(Style::default().fg(layouts_border))
                    .title_style(
                        Style::default()
                            .fg(
                                if self.selected_sidebar_section == SidebarSection::Layouts {
                                    self.theme.text_active_focus
                                } else {
                                    self.theme.text_labels
                                },
                            )
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .style(
                Style::default()
                    .fg(self.theme.text_primary)
                    .bg(self.theme.body_background),
            )
            .highlight_style(
                Style::default()
                    .fg(self.theme.text_active_focus)
                    .bg(self.theme.selected_background)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        if !tree_rows.is_empty() {
            self.layout_list_state
                .select(Some(self.selected_tree_row.min(tree_rows.len() - 1)));
        } else {
            self.layout_list_state.select(None);
        }
        frame.render_stateful_widget(layouts_list, sidebar[2], &mut self.layout_list_state);
        self.list_area = sidebar[2];

        // Scrollbar on the right edge of the Layout list when the tree
        // overflows the inner height. Track lives on the last column inside
        // the border; thumb uses ListState offset as the first visible row.
        self.layout_scrollbar_track = ScrollbarTrack::default();
        let layout_inner_h = sidebar[2].height.saturating_sub(2) as usize;
        if tree_rows.len() > layout_inner_h && sidebar[2].width >= 3 && sidebar[2].height >= 4 {
            let track = Rect {
                x: sidebar[2].x + sidebar[2].width.saturating_sub(2),
                y: sidebar[2].y + 1,
                width: 1,
                height: sidebar[2].height.saturating_sub(2),
            };
            self.layout_scrollbar_track = ScrollbarTrack {
                rect: track,
                total: tree_rows.len(),
                visible: layout_inner_h,
            };
            paint_scrollbar(
                frame,
                track,
                self.layout_list_state.offset(),
                tree_rows.len(),
                layout_inner_h,
                self.theme.scrollbar,
                self.theme.scrollbar_hover,
                self.theme.body_background,
            );
        }

        self.details_area = if self.details_open {
            main[1]
        } else {
            Rect::default()
        };
        if !self.details_open {
            self.details_scrollbar_track = ScrollbarTrack::default();
        } else {
            let details_width = main[1].width.saturating_sub(2) as usize;
            let details_view = self.details_text(details_width);
            self.details_hits = details_view.hits.clone();
            let details_total_rows = details_view.lines.len().max(1);
            let details_visible_rows = main[1].height.saturating_sub(2) as usize;
            let details_max_scroll = details_total_rows.saturating_sub(details_visible_rows);
            self.details_scroll_row = self.details_scroll_row.min(details_max_scroll);

            let details = Paragraph::new(details_view.to_text(&self.theme))
                .style(
                    Style::default()
                        .fg(self.theme.text_primary)
                        .bg(self.theme.body_background),
                )
                .block(
                    Block::default()
                        .title(details_title)
                        .borders(Borders::ALL)
                        .style(
                            Style::default()
                                .fg(self.theme.text_primary)
                                .bg(self.theme.body_background),
                        )
                        .border_style(Style::default().fg(details_border))
                        .title_style(
                            Style::default()
                                .fg(
                                    if self.selected_sidebar_section == SidebarSection::Details {
                                        self.theme.text_active_focus
                                    } else {
                                        self.theme.text_labels
                                    },
                                )
                                .add_modifier(Modifier::BOLD),
                        ),
                )
                .scroll((self.details_scroll_row.min(u16::MAX as usize) as u16, 0))
                .wrap(Wrap { trim: true });
            frame.render_widget(details, main[1]);

            // Scrollbar on the right edge of the Details panel — only painted
            // when the content exceeds the visible window. Track lives on the
            // last column inside the border; thumb height is proportional to
            // visible/total rows.
            self.details_scrollbar_track = ScrollbarTrack::default();
            if details_total_rows > details_visible_rows
                && main[1].width >= 3
                && main[1].height >= 4
            {
                let track = Rect {
                    x: main[1].x + main[1].width.saturating_sub(2),
                    y: main[1].y + 1,
                    width: 1,
                    height: main[1].height.saturating_sub(2),
                };
                self.details_scrollbar_track = ScrollbarTrack {
                    rect: track,
                    total: details_total_rows,
                    visible: details_visible_rows,
                };
                paint_scrollbar(
                    frame,
                    track,
                    self.details_scroll_row,
                    details_total_rows,
                    details_visible_rows,
                    self.theme.scrollbar,
                    self.theme.scrollbar_hover,
                    self.theme.body_background,
                );
            }
        }

        let footer_text = self.footer_hint(root[2].width);
        let footer = Paragraph::new(footer_text).style(self.theme.app_shell);
        frame.render_widget(footer, root[2]);

        // Render unified modal if open (handles all modal types)
        self.render_modal(frame);

        // Settings sits above FormEdit. The sheet is the right half of the
        // body: a section list on the left, one page on the right.
        if self.overlay.is_some() {
            self.paint_settings(frame);
        } else {
            self.help_scrollbar_track = ScrollbarTrack::default();
            self.theme_scrollbar_track = ScrollbarTrack::default();
            self.settings_nav_area = Rect::default();
        }

        // Toasts paint last so they float above everything except the
        // active-input cursor overlay.
        self.render_toasts(frame, frame.area());

        let cursor_overlay = self.set_cursor_for_active_input(frame);
        if let Some((x, y, ch)) = cursor_overlay {
            let cursor_cell = Paragraph::new(ch.to_string()).style(
                Style::default()
                    .fg(self.theme.modal_background)
                    .bg(self.theme.cursor),
            );
            frame.render_widget(
                cursor_cell,
                Rect {
                    x,
                    y,
                    width: 1,
                    height: 1,
                },
            );
        }
    }

    fn paint_settings(&mut self, frame: &mut ratatui::Frame) {
        let area = settings_sheet_area(frame.area());
        if area.width == 0 || area.height == 0 {
            return;
        }
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(" Settings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_active))
            .title_style(
                Style::default()
                    .fg(self.theme.modal_header)
                    .add_modifier(Modifier::BOLD),
            )
            .style(
                Style::default()
                    .bg(self.theme.modal_background)
                    .fg(self.theme.modal_text),
            );
        let inner = block.inner(area);
        frame.render_widget(block, area);
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        if self.settings_nav as usize >= super::settings::NAV.len() {
            self.settings_nav = 0;
        }
        let nav_w = {
            let spare = inner.width.saturating_sub(8);
            if spare == 0 {
                1.min(inner.width)
            } else {
                14.min(spare).max(1)
            }
        };
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(nav_w), Constraint::Min(1)])
            .split(inner);
        let nav = cols[0];
        let page = cols[1];
        self.settings_nav_area = nav;
        self.paint_settings_nav(frame, nav);
        let section = super::settings::NAV
            .get(self.settings_nav as usize)
            .copied()
            .unwrap_or("Themes");
        let title_h = u16::from(page.height > 0);
        if title_h > 0 {
            let shown: String = section.chars().take(page.width as usize).collect();
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    shown,
                    Style::default()
                        .fg(self.theme.modal_header)
                        .add_modifier(Modifier::BOLD),
                )))
                .style(
                    Style::default()
                        .bg(self.theme.modal_background)
                        .fg(self.theme.modal_header),
                ),
                Rect {
                    x: page.x,
                    y: page.y,
                    width: page.width,
                    height: title_h,
                },
            );
        }
        let content = Rect {
            x: page.x,
            y: page.y.saturating_add(title_h),
            width: page.width,
            height: page.height.saturating_sub(title_h),
        };
        if self.settings_nav == super::settings::HELP {
            self.paint_settings_help(frame, content);
            return;
        }
        if self.settings_nav == super::settings::WINDOW {
            self.paint_settings_window(frame, content);
            return;
        }
        self.paint_settings_themes(frame, content);
    }

    fn paint_settings_nav(&self, frame: &mut ratatui::Frame, nav: Rect) {
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(self.theme.border))
            .style(
                Style::default()
                    .bg(self.theme.body_background)
                    .fg(self.theme.modal_text),
            );
        let inner = block.inner(nav);
        frame.render_widget(block, nav);
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        let selected_style = Style::default()
            .fg(self.theme.text_active_focus)
            .bg(self.theme.selected_background);
        let idle = Style::default()
            .fg(self.theme.modal_text)
            .bg(self.theme.body_background);
        let width = inner.width as usize;
        let mut lines = Vec::new();
        for (index, label) in super::settings::NAV.iter().enumerate() {
            let on = self.settings_nav == index as u8;
            let marker = if on { ">" } else { " " };
            let text: String = format!("{marker} {label}").chars().take(width).collect();
            let style = if on { selected_style } else { idle };
            lines.push(Line::from(Span::styled(text, style)));
        }
        frame.render_widget(
            Paragraph::new(lines).style(
                Style::default()
                    .bg(self.theme.body_background)
                    .fg(self.theme.modal_text),
            ),
            inner,
        );
    }

    fn paint_settings_help(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        self.theme_scrollbar_track = ScrollbarTrack::default();
        if area.width == 0 || area.height == 0 {
            self.help_scrollbar_track = ScrollbarTrack::default();
            self.overlay_scroll_max = 0;
            return;
        }
        let body_w = area.width.saturating_sub(2);
        let body = Rect {
            width: body_w,
            ..area
        };
        let help = build_help_text(&self.theme, body_w as usize, self.in_tmux);
        let total = help.lines.len();
        self.paint_scrolled_lines(frame, body, help.lines, None, self.theme.scrollbar);
        if total > area.height as usize && area.height > 0 {
            let track = Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y,
                width: 1,
                height: area.height,
            };
            let scroll = match self.overlay {
                Some(Overlay::Theme { scroll }) => scroll as usize,
                None => 0,
            };
            self.help_scrollbar_track = ScrollbarTrack {
                rect: track,
                total,
                visible: area.height as usize,
            };
            paint_scrollbar(
                frame,
                track,
                scroll,
                total,
                area.height as usize,
                self.theme.scrollbar,
                self.theme.scrollbar_hover,
                self.theme.modal_background,
            );
        } else {
            self.help_scrollbar_track = ScrollbarTrack::default();
        }
    }

    fn paint_settings_window(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let selected_style = Style::default()
            .fg(self.theme.text_active_focus)
            .bg(self.theme.selected_background);
        let idle = Style::default().fg(self.theme.modal_text);
        let mut lines = vec![Line::from(Span::styled(
            "Tab section   j/k move   Enter   Esc close",
            Style::default().fg(self.theme.text_secondary),
        ))];
        let mut selected_line = None;
        for (index, row) in super::settings::window_items().iter().enumerate() {
            if index == self.settings_row {
                selected_line = Some(lines.len());
            }
            let value = super::settings::value(self, row.id);
            let marker = if index == self.settings_row { ">" } else { " " };
            let style = if index == self.settings_row {
                selected_style
            } else {
                idle
            };
            lines.push(Line::from(Span::styled(
                format!("{marker} {}  {value}", row.title),
                style,
            )));
        }
        self.help_scrollbar_track = ScrollbarTrack::default();
        self.paint_scrolled_lines(frame, area, lines, selected_line, self.theme.modal_text);
    }

    fn paint_settings_themes(&mut self, frame: &mut ratatui::Frame, content: Rect) {
        use ldnddev_theme::{PickerRow, ThemeEditorRow, theme_editor_rows};
        let editor = self.theme_editor.clone();
        let picker = self.theme_picker.clone();
        let box_w = (content.width as usize).max(2);
        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut selected_line = None;
        let rule = Style::default().fg(self.theme.border);
        let header = Style::default()
            .fg(self.theme.modal_header)
            .add_modifier(Modifier::BOLD);
        let hint = if editor.is_some() {
            let target = editor
                .as_ref()
                .map(|editor| editor.save_target.label())
                .unwrap_or("");
            let back = if picker.is_some() {
                "Esc back"
            } else {
                "Esc revert"
            };
            format!("Tab section   j/k move   t target   y save {target}   R reset   {back}")
        } else {
            "Tab section   j/k move   Enter apply   e colors   y save   t target   Esc revert"
                .to_string()
        };
        lines.push(Line::from(Span::styled(
            hint,
            Style::default().fg(self.theme.text_secondary),
        )));
        let selected_style = Style::default()
            .fg(self.theme.text_active_focus)
            .bg(self.theme.selected_background);
        let idle = Style::default().fg(self.theme.modal_text);
        if let Some(editor) = &editor {
            lines.push(Line::from(if editor.editing_hex {
                format!("Hex: {}█", editor.hex_draft)
            } else {
                format!("Hex: {}   Enter edit", editor.hex_draft)
            }));
            let mut open = false;
            for row in theme_editor_rows(&editor.fields) {
                match row {
                    ThemeEditorRow::Header(name) => {
                        if open {
                            lines.push(settings_bottom(box_w, rule));
                        }
                        lines.push(Line::from(""));
                        lines.push(settings_top(name, box_w, rule, header));
                        open = true;
                    }
                    ThemeEditorRow::Color(idx) => {
                        if idx == editor.selected {
                            selected_line = Some(lines.len());
                        }
                        let field = editor.fields[idx];
                        let hex = editor
                            .palette
                            .get(field.key)
                            .map(|color| color.to_hex())
                            .unwrap_or_else(|| "#000000".into());
                        let cursor = if idx == editor.selected { ">" } else { " " };
                        let style = if idx == editor.selected {
                            selected_style
                        } else {
                            idle
                        };
                        lines.push(settings_row(
                            &format!("{cursor} {:<22} {hex}", field.key),
                            box_w,
                            rule,
                            style,
                        ));
                    }
                }
            }
            if open {
                lines.push(settings_bottom(box_w, rule));
            }
        } else if let Some(picker) = &picker {
            lines.push(Line::from(""));
            lines.push(settings_top("Themes", box_w, rule, header));
            for (idx, row) in picker.rows.iter().enumerate() {
                if idx == picker.selected {
                    selected_line = Some(lines.len());
                }
                let (label, detail) = match row {
                    PickerRow::Follow { name } => ("Follow desktop".to_string(), name.clone()),
                    PickerRow::Theme(theme) => (theme.name.clone(), theme.mode.label().to_string()),
                    PickerRow::Custom => ("Custom".to_string(), "built-in".to_string()),
                };
                let cursor = if idx == picker.selected { ">" } else { " " };
                let style = if idx == picker.selected {
                    selected_style
                } else {
                    idle
                };
                lines.push(settings_row(
                    &format!("{cursor} {label:<22} {detail}"),
                    box_w,
                    rule,
                    style,
                ));
            }
            lines.push(settings_bottom(box_w, rule));
        }
        self.help_scrollbar_track = ScrollbarTrack::default();
        self.paint_scrolled_lines(frame, content, lines, selected_line, self.theme.modal_text);
    }

    fn paint_scrolled_lines(
        &mut self,
        frame: &mut ratatui::Frame,
        area: Rect,
        lines: Vec<Line<'static>>,
        selected_line: Option<usize>,
        _thumb: ratatui::style::Color,
    ) {
        if area.width == 0 || area.height == 0 {
            self.overlay_scroll_max = 0;
            return;
        }
        let view_h = area.height as usize;
        let max_scroll = lines.len().saturating_sub(view_h) as u16;
        self.overlay_scroll_max = max_scroll;
        if self.settings_follow {
            if let Some(sel) = selected_line {
                let top = match self.overlay {
                    Some(Overlay::Theme { scroll }) => scroll as usize,
                    None => 0,
                };
                if sel < top {
                    if let Some(Overlay::Theme { scroll }) = &mut self.overlay {
                        *scroll = sel as u16;
                    }
                } else if sel >= top + view_h {
                    if let Some(Overlay::Theme { scroll }) = &mut self.overlay {
                        *scroll = (sel + 1 - view_h) as u16;
                    }
                }
            }
            self.settings_follow = false;
        }
        if let Some(Overlay::Theme { scroll }) = &mut self.overlay {
            if *scroll > max_scroll {
                *scroll = max_scroll;
            }
        }
        let scroll = match self.overlay {
            Some(Overlay::Theme { scroll }) => scroll,
            None => 0,
        };
        frame.render_widget(
            Paragraph::new(lines)
                .style(
                    Style::default()
                        .fg(self.theme.modal_text)
                        .bg(self.theme.modal_background),
                )
                .scroll((scroll, 0)),
            area,
        );
    }

    pub(super) fn active_input_cursor_cell(&self) -> Option<(u16, u16, char)> {
        // Help/Theme paint after FormEdit; a caret overlay would punch through.
        if self.overlay.is_some() {
            return None;
        }
        let Some(Modal::FormEdit {
            state, cursor_pos, ..
        }) = &self.modal
        else {
            return None;
        };
        let field = state.form.fields.get(state.focused_field)?;
        let areas = self.modal_field_areas.borrow();
        let (_, box_rect) = areas.iter().find(|(idx, _)| *idx == state.focused_field)?;
        form_input_cursor_cell(
            &field.kind,
            state.get(field.id),
            *cursor_pos,
            *box_rect,
            *self.form_textarea_vscroll.borrow(),
            self.form_textarea_follow_cursor.get(),
        )
    }

    pub(super) fn set_cursor_for_active_input(
        &self,
        frame: &mut ratatui::Frame,
    ) -> Option<(u16, u16, char)> {
        let (x, y, ch) = self.active_input_cursor_cell()?;
        let area = frame.area();
        if x < area.x
            || y < area.y
            || x >= area.x.saturating_add(area.width)
            || y >= area.y.saturating_add(area.height)
        {
            return None;
        }
        frame.set_cursor_position((x, y));
        Some((x, y, ch))
    }
}
