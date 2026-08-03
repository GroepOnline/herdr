use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{state::SettingsSection, AppState};

use super::{
    blocks::{block_height, build_blocks, uses_block_layout, ContentBlock},
    catalog::{integration_index, SettingsItemId},
    layout::{SettingsLayout, SETTINGS_SECTION_DESC_ROWS, SETTINGS_SECTION_GAP_ROWS},
    rows::{
        notifications_toast_delivery_labels, row_choice_selected, row_toggle_checked, section_rows,
        SettingsRow, SettingsRowKind,
    },
    widgets::{
        checklist_group_areas, checklist_item_rects, chip_row_rects, render_checklist_group,
        render_chip_row, render_setting_block_header, ChecklistItem, ChipSpec, CHIP_HORIZONTAL_GAP,
        SETTING_BLOCK_HEADER_ROWS,
    },
};

const BLOCK_GAP_ROWS: u16 = SETTINGS_SECTION_GAP_ROWS;

pub(crate) use super::blocks::uses_block_layout;

#[derive(Debug, Clone)]
struct PlacedBlock {
    y: u16,
    height: u16,
    block: ContentBlock,
}

fn place_blocks(list_y: u16, blocks: &[ContentBlock]) -> Vec<PlacedBlock> {
    let mut y = list_y;
    let mut placed = Vec::with_capacity(blocks.len());
    for (idx, block) in blocks.iter().enumerate() {
        let height = block_height(block);
        placed.push(PlacedBlock {
            y,
            height,
            block: block.clone(),
        });
        y = y.saturating_add(height);
        if idx + 1 < blocks.len() {
            y = y.saturating_add(BLOCK_GAP_ROWS);
        }
    }
    placed
}

fn total_content_height(placed: &[PlacedBlock]) -> u16 {
    if placed.is_empty() {
        return 0;
    }
    let last = placed.last().expect("non-empty placed blocks");
    last.y
        .saturating_add(last.height)
        .saturating_sub(placed[0].y)
}

fn scroll_offset_for_selection(
    placed: &[PlacedBlock],
    list_area: Rect,
    selected_row: usize,
    rows: &[SettingsRow],
) -> u16 {
    let total = total_content_height(placed);
    if total <= list_area.height {
        return 0;
    }

    let selected_y = row_visual_y(placed, selected_row, rows).unwrap_or(0);
    let list_top = list_area.y;
    let rel_selected = selected_y.saturating_sub(list_top);
    let visible = list_area.height;

    if rel_selected >= visible {
        rel_selected.saturating_sub(visible).saturating_add(1)
    } else {
        0
    }
}

fn row_visual_y(placed: &[PlacedBlock], row_index: usize, rows: &[SettingsRow]) -> Option<u16> {
    for block in placed {
        match &block.block {
            ContentBlock::Checklist { row_indices, .. } => {
                for (item_idx, idx) in row_indices.iter().enumerate() {
                    if *idx == row_index {
                        let (_header, items) = checklist_group_areas(
                            Rect::new(0, block.y, 1, block.height),
                            row_indices.len(),
                        );
                        let item_rects = checklist_item_rects(items, row_indices.len());
                        return item_rects.get(item_idx).map(|rect| rect.y);
                    }
                }
            }
            ContentBlock::Chips { row_indices, .. } => {
                if row_indices.len() == 1 {
                    if row_indices[0] == row_index {
                        return Some(block.y);
                    }
                } else if row_indices.contains(&row_index) {
                    return Some(block.y);
                }
            }
            ContentBlock::Plain { row_index: idx } if *idx == row_index => {
                return Some(block.y);
            }
            _ => {}
        }
    }
    let _ = rows;
    None
}

fn chip_selected(
    app: &AppState,
    section: SettingsSection,
    row: &SettingsRow,
    chip_label: &str,
) -> bool {
    match row.id {
        SettingsItemId::HostCursor { .. }
        | SettingsItemId::ShellMode { .. }
        | SettingsItemId::NewTerminalCwd { .. } => {
            row_choice_selected(app, section, row) && row.label == chip_label
        }
        SettingsItemId::ToastDelivery { .. } => notifications_toast_delivery_labels()
            .iter()
            .any(|(label, delivery)| *label == chip_label && app.toast_delivery() == *delivery),
        _ => row_choice_selected(app, section, row),
    }
}

pub(crate) fn content_list_top(layout: &SettingsLayout, _app: &AppState) -> u16 {
    layout.content.y + SETTINGS_SECTION_DESC_ROWS + BLOCK_GAP_ROWS
}

fn list_area(layout: &SettingsLayout, app: &AppState) -> Rect {
    let top = content_list_top(layout, app);
    Rect {
        x: layout.content.x,
        y: top,
        width: layout.content.width,
        height: layout.content.height.saturating_sub(top - layout.content.y),
    }
}

pub(crate) fn content_row_rect(
    app: &AppState,
    layout: &SettingsLayout,
    row_index: usize,
) -> Option<Rect> {
    let section = app.settings.section;
    if !uses_block_layout(section) {
        return None;
    }
    let rows = section_rows(app, section);
    rows.get(row_index)?;
    let blocks = build_blocks(app, section);
    let area = list_area(layout, app);
    let placed = place_blocks(area.y, &blocks);
    let scroll = scroll_offset_for_selection(
        &placed,
        area,
        app.settings.list.selected.min(rows.len().saturating_sub(1)),
        &rows,
    );
    let mut rect = row_visual_y_rect(app, layout, &placed, row_index, &rows)?;
    rect.y = rect.y.saturating_sub(scroll);
    if rect.y < area.y || rect.y >= area.y + area.height {
        return None;
    }
    Some(rect)
}

fn row_visual_y_rect(
    _app: &AppState,
    layout: &SettingsLayout,
    placed: &[PlacedBlock],
    row_index: usize,
    _rows: &[SettingsRow],
) -> Option<Rect> {
    let width = layout.content.width;
    for block in placed {
        match &block.block {
            ContentBlock::Checklist { row_indices, .. } => {
                for (item_idx, idx) in row_indices.iter().enumerate() {
                    if *idx == row_index {
                        let area = Rect::new(layout.content.x, block.y, width, block.height);
                        let (_header, items) = checklist_group_areas(area, row_indices.len());
                        return checklist_item_rects(items, row_indices.len())
                            .get(item_idx)
                            .copied();
                    }
                }
            }
            ContentBlock::Chips {
                row_indices,
                labels,
            } => {
                if row_indices.len() == 1 {
                    if row_indices[0] == row_index {
                        return Some(Rect::new(layout.content.x, block.y, width, 1));
                    }
                } else if let Some(chip_idx) = row_indices.iter().position(|i| *i == row_index) {
                    let label_refs: Vec<&str> = labels.iter().map(String::as_str).collect();
                    let rects = chip_row_rects(
                        Rect::new(layout.content.x, block.y, width, 1),
                        &label_refs,
                        CHIP_HORIZONTAL_GAP,
                    );
                    return rects.get(chip_idx).copied();
                }
            }
            ContentBlock::Plain { row_index: idx } if *idx == row_index => {
                return Some(Rect::new(layout.content.x, block.y, width, 1));
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn content_index_at(
    app: &AppState,
    layout: &SettingsLayout,
    col: u16,
    row: u16,
) -> Option<usize> {
    let section = app.settings.section;
    if !uses_block_layout(section) {
        return None;
    }
    let rows = section_rows(app, section);
    let area = list_area(layout, app);
    if col < area.x || col >= area.x + area.width || row < area.y || row >= area.y + area.height {
        return None;
    }

    for row_index in 0..rows.len() {
        let Some(rect) = content_row_rect(app, layout, row_index) else {
            continue;
        };
        if row == rect.y && col >= rect.x && col < rect.x + rect.width {
            return Some(row_index);
        }
    }
    None
}

pub(crate) fn render_block_section(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    let section = app.settings.section;
    let rows = section_rows(app, section);
    let selected = app.settings.list.selected.min(rows.len().saturating_sub(1));
    let blocks = build_blocks(app, section);
    let area = list_area(layout, app);
    let placed = place_blocks(area.y, &blocks);
    let scroll = scroll_offset_for_selection(&placed, area, selected, &rows);

    for block in &placed {
        let block_y = block.y.saturating_sub(scroll);
        let block_bottom = block_y.saturating_add(block.height);
        if block_bottom <= area.y || block_y >= area.y + area.height {
            continue;
        }

        match &block.block {
            ContentBlock::Header { label } => {
                if block_y >= area.y && block_y < area.y + area.height {
                    render_setting_block_header(
                        frame,
                        Rect::new(area.x, block_y, area.width, SETTING_BLOCK_HEADER_ROWS),
                        label,
                        p,
                    );
                }
            }
            ContentBlock::Checklist { label, row_indices } => {
                let visible_y = block_y.max(area.y);
                let visible_bottom = block_bottom.min(area.y.saturating_add(area.height));
                let clip_height = visible_bottom.saturating_sub(visible_y);
                if clip_height == 0 {
                    continue;
                }
                // Keep full block geometry for item hit math, but never draw above
                // the content viewport (top-clip scrolled blocks).
                let checklist_area = Rect::new(area.x, block_y, area.width, block.height);
                let items: Vec<ChecklistItem<'_>> = row_indices
                    .iter()
                    .filter_map(|idx| rows.get(*idx))
                    .map(|row| ChecklistItem {
                        label: row.label.as_str(),
                        checked: row_toggle_checked(app, section, row),
                    })
                    .collect();
                if block_y >= area.y {
                    render_checklist_group(frame, checklist_area, label, &items, p);
                } else {
                    // Top-clipped: draw only item rows that remain inside bounds.
                    let (_header, items_area) =
                        checklist_group_areas(checklist_area, row_indices.len());
                    for (item, rect) in items
                        .iter()
                        .zip(checklist_item_rects(items_area, row_indices.len()).iter())
                    {
                        if rect.y < area.y || rect.y >= area.y + area.height {
                            continue;
                        }
                        let marker = if item.checked { "[✓]" } else { "[ ]" };
                        frame.render_widget(
                            Paragraph::new(Line::from(vec![
                                Span::styled(
                                    format!(" {marker} "),
                                    Style::default().fg(p.overlay1),
                                ),
                                Span::styled(item.label, Style::default().fg(p.subtext0)),
                            ])),
                            *rect,
                        );
                    }
                }
                for (item_idx, row_idx) in row_indices.iter().enumerate() {
                    if *row_idx != selected {
                        continue;
                    }
                    let Some(row) = rows.get(*row_idx) else {
                        continue;
                    };
                    let (_header, items_area) =
                        checklist_group_areas(checklist_area, row_indices.len());
                    let item_rect = checklist_item_rects(items_area, row_indices.len())
                        .get(item_idx)
                        .copied()
                        .unwrap_or(items_area);
                    if item_rect.y < area.y || item_rect.y >= area.y + area.height {
                        continue;
                    }
                    let marker = if row_toggle_checked(app, section, row) {
                        "[✓]"
                    } else {
                        "[ ]"
                    };
                    let sel_style = Style::default()
                        .bg(p.surface0)
                        .fg(p.text)
                        .add_modifier(Modifier::BOLD);
                    frame.render_widget(
                        Paragraph::new(Line::from(vec![
                            Span::styled(format!(" {marker} "), sel_style),
                            Span::styled(row.label.clone(), sel_style),
                        ])),
                        item_rect,
                    );
                }
            }
            ContentBlock::Chips {
                row_indices,
                labels,
            } => {
                if block_y >= area.y && block_y < area.y + area.height {
                    let chips: Vec<ChipSpec<'_>> = row_indices
                        .iter()
                        .zip(labels.iter())
                        .map(|(row_idx, label)| {
                            let row = rows.get(*row_idx).expect("chip row");
                            ChipSpec {
                                label: label.as_str(),
                                selected: chip_selected(app, section, row, label)
                                    || (*row_idx == selected),
                            }
                        })
                        .collect();
                    render_chip_row(
                        frame,
                        Rect::new(area.x, block_y, area.width, 1),
                        &chips,
                        CHIP_HORIZONTAL_GAP,
                        p,
                    );
                }
            }
            ContentBlock::Plain { row_index } => {
                if block_y >= area.y && block_y < area.y + area.height {
                    let Some(row) = rows.get(*row_index) else {
                        continue;
                    };
                    if section == SettingsSection::Notifications {
                        render_premium_choice_row(
                            app,
                            frame,
                            section,
                            row,
                            *row_index == selected,
                            Rect::new(area.x, block_y, area.width, 1),
                        );
                    } else {
                        render_plain_row(
                            app,
                            frame,
                            section,
                            row,
                            *row_index == selected,
                            Rect::new(area.x, block_y, area.width, 1),
                        );
                    }
                }
            }
        }
    }
}

fn render_premium_choice_row(
    app: &AppState,
    frame: &mut Frame,
    section: SettingsSection,
    row: &SettingsRow,
    is_sel: bool,
    rect: Rect,
) {
    let p = &app.palette;
    let row_style = if is_sel {
        Style::default()
            .bg(p.surface0)
            .fg(p.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.subtext0)
    };
    let marker = if row_choice_selected(app, section, row) {
        "●"
    } else {
        "○"
    };
    let detail = row.detail.as_deref().unwrap_or("tap to cycle");
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!(" {marker} "), row_style),
            Span::styled(row.label.clone(), row_style),
            Span::styled(format!("  ·  {detail}"), Style::default().fg(p.overlay1)),
        ])),
        rect,
    );
}

fn render_plain_row(
    app: &AppState,
    frame: &mut Frame,
    section: SettingsSection,
    row: &SettingsRow,
    is_sel: bool,
    rect: Rect,
) {
    let p = &app.palette;
    let row_style = if is_sel {
        Style::default()
            .bg(p.surface0)
            .fg(p.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.subtext0)
    };

    let marker = match row.kind {
        SettingsRowKind::Toggle => {
            if row_toggle_checked(app, section, row) {
                "[✓]"
            } else {
                "[ ]"
            }
        }
        SettingsRowKind::Choice => {
            if row_choice_selected(app, section, row) {
                "●"
            } else {
                "○"
            }
        }
        SettingsRowKind::Integration => {
            if catalog_plugin_id(row.id).is_some() {
                "+"
            } else {
                integration_marker(app, integration_index(row.id).unwrap_or_default())
            }
        }
        SettingsRowKind::Note => "·",
        _ => " ",
    };

    let mut spans = vec![
        Span::styled(format!(" {marker} "), row_style),
        Span::styled(row.label.clone(), row_style),
    ];
    if let Some(detail) = &row.detail {
        spans.push(Span::styled(
            format!("  — {detail}"),
            Style::default().fg(p.overlay1),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), rect);
}

fn integration_marker(app: &AppState, idx: usize) -> &'static str {
    let Some(item) = app.integration_recommendations.get(idx) else {
        return " ";
    };
    match item.state {
        crate::integration::IntegrationStatusKind::Current => "✓",
        crate::integration::IntegrationStatusKind::Outdated => "↻",
        crate::integration::IntegrationStatusKind::NotInstalled if item.available => "+",
        crate::integration::IntegrationStatusKind::NotInstalled => "–",
    }
}
