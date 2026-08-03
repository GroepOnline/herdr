use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::state::Palette;

use super::super::widgets::{action_button_width, panel_contrast_fg};

pub(crate) const CHIP_HORIZONTAL_GAP: u16 = 1;
pub(crate) const CHECKLIST_HEADER_ROWS: u16 = 1;
pub(crate) const CHECKLIST_TOP_GAP: u16 = 1;
pub(crate) const SETTING_BLOCK_HEADER_ROWS: u16 = 1;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ChipSpec<'a> {
    pub label: &'a str,
    pub selected: bool,
}

pub(crate) fn chip_width(label: &str) -> u16 {
    (label.len().saturating_add(2)) as u16
}

pub(crate) fn chip_row_rects(area: Rect, labels: &[&str], gap: u16) -> Vec<Rect> {
    let mut rects = Vec::with_capacity(labels.len());
    let mut x = area.x;
    for label in labels {
        let width = chip_width(label).min(area.width.saturating_sub(x.saturating_sub(area.x)));
        if width == 0 {
            break;
        }
        rects.push(Rect::new(x, area.y, width, 1));
        x = x.saturating_add(width).saturating_add(gap);
        if x >= area.x + area.width {
            break;
        }
    }
    rects
}

pub(crate) fn chip_row_rects_from_specs(area: Rect, chips: &[ChipSpec<'_>], gap: u16) -> Vec<Rect> {
    let labels: Vec<&str> = chips.iter().map(|chip| chip.label).collect();
    chip_row_rects(area, &labels, gap)
}

pub(crate) fn chip_row_index_at(
    area: Rect,
    labels: &[&str],
    gap: u16,
    col: u16,
    row: u16,
) -> Option<usize> {
    if row != area.y {
        return None;
    }
    for (idx, rect) in chip_row_rects(area, labels, gap).into_iter().enumerate() {
        if col >= rect.x && col < rect.x + rect.width {
            return Some(idx);
        }
    }
    None
}

pub(crate) fn chip_wrap_rects(area: Rect, labels: &[&str], gap: u16) -> Vec<Rect> {
    let mut rects = Vec::with_capacity(labels.len());
    if area.width == 0 || area.height == 0 {
        return rects;
    }

    let mut x = area.x;
    let mut y = area.y;
    for label in labels {
        let width = chip_width(label).min(area.width);
        if width == 0 {
            break;
        }
        if x > area.x && x.saturating_add(width) > area.x + area.width {
            x = area.x;
            y = y.saturating_add(1);
            if y >= area.y + area.height {
                break;
            }
        }
        rects.push(Rect::new(x, y, width, 1));
        x = x.saturating_add(width).saturating_add(gap);
        if x >= area.x + area.width {
            x = area.x;
            y = y.saturating_add(1);
            if y >= area.y + area.height {
                break;
            }
        }
    }
    rects
}

pub(crate) fn chip_wrap_row_count(area_width: u16, labels: &[&str], gap: u16) -> u16 {
    if area_width == 0 || labels.is_empty() {
        return 0;
    }
    let area = Rect::new(0, 0, area_width, u16::MAX);
    let rects = chip_wrap_rects(area, labels, gap);
    rects
        .last()
        .map(|rect| rect.y.saturating_add(1))
        .unwrap_or(0)
}

pub(crate) fn chip_wrap_index_at(
    area: Rect,
    labels: &[&str],
    gap: u16,
    col: u16,
    row: u16,
) -> Option<usize> {
    for (idx, rect) in chip_wrap_rects(area, labels, gap).into_iter().enumerate() {
        if row == rect.y && col >= rect.x && col < rect.x + rect.width {
            return Some(idx);
        }
    }
    None
}

pub(crate) fn render_chip_row(
    frame: &mut Frame,
    area: Rect,
    chips: &[ChipSpec<'_>],
    gap: u16,
    p: &Palette,
) {
    let rects = if area.height <= 1 {
        chip_row_rects_from_specs(area, chips, gap)
    } else {
        let labels: Vec<&str> = chips.iter().map(|chip| chip.label).collect();
        chip_wrap_rects(area, &labels, gap)
    };
    for (chip, rect) in chips.iter().zip(rects.iter()) {
        let style = if chip.selected {
            Style::default()
                .fg(panel_contrast_fg(p))
                .bg(p.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.overlay1).bg(p.surface0)
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(format!(" {} ", chip.label), style))),
            *rect,
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ChecklistItem<'a> {
    pub label: &'a str,
    pub checked: bool,
}

pub(crate) fn checklist_group_height(item_count: usize) -> u16 {
    CHECKLIST_HEADER_ROWS
        .saturating_add(CHECKLIST_TOP_GAP)
        .saturating_add(item_count as u16)
        .saturating_add(2)
}

pub(crate) fn checklist_group_areas(area: Rect, _item_count: usize) -> (Rect, Rect) {
    let header = Rect::new(area.x, area.y, area.width, CHECKLIST_HEADER_ROWS);
    let items_y = header
        .y
        .saturating_add(CHECKLIST_HEADER_ROWS)
        .saturating_add(CHECKLIST_TOP_GAP);
    let items = Rect::new(
        area.x,
        items_y,
        area.width,
        area.height.saturating_sub(items_y - area.y),
    );
    (header, items)
}

pub(crate) fn checklist_item_rects(items_area: Rect, item_count: usize) -> Vec<Rect> {
    (0..item_count)
        .map(|idx| Rect::new(items_area.x, items_area.y + idx as u16, items_area.width, 1))
        .take(items_area.height as usize)
        .collect()
}

pub(crate) fn checklist_item_index_at(
    items_area: Rect,
    item_count: usize,
    col: u16,
    row: u16,
) -> Option<usize> {
    if col < items_area.x || col >= items_area.x + items_area.width {
        return None;
    }
    for (idx, rect) in checklist_item_rects(items_area, item_count)
        .into_iter()
        .enumerate()
    {
        if row == rect.y {
            return Some(idx);
        }
    }
    None
}

pub(crate) fn render_checklist_group(
    frame: &mut Frame,
    area: Rect,
    group_label: &str,
    items: &[ChecklistItem<'_>],
    p: &Palette,
) {
    let (header, items_area) = checklist_group_areas(area, items.len());
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {group_label}"),
            Style::default().fg(p.text).add_modifier(Modifier::BOLD),
        ))),
        header,
    );

    let box_top = items_area.y.saturating_sub(1);
    let box_bottom = items_area.y.saturating_add(items.len() as u16);
    if items_area.width > 0 {
        for y in box_top..=box_bottom.min(items_area.y + items_area.height) {
            frame.render_widget(
                Paragraph::new(Span::styled(
                    if y == box_top || y == box_bottom {
                        "─".repeat(items_area.width as usize)
                    } else {
                        format!(
                            "{} {}",
                            "│",
                            " ".repeat(items_area.width.saturating_sub(2) as usize)
                        )
                    },
                    Style::default().fg(p.surface0),
                )),
                Rect::new(items_area.x, y, items_area.width, 1),
            );
        }
    }

    for (item, rect) in items
        .iter()
        .zip(checklist_item_rects(items_area, items.len()).iter())
    {
        let marker = if item.checked { "[✓]" } else { "[ ]" };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!(" {marker} "), Style::default().fg(p.overlay1)),
                Span::styled(item.label, Style::default().fg(p.subtext0)),
            ])),
            *rect,
        );
    }
}

pub(crate) fn setting_block_header_rect(area: Rect) -> Rect {
    Rect::new(area.x, area.y, area.width, SETTING_BLOCK_HEADER_ROWS)
}

pub(crate) fn render_setting_block_header(frame: &mut Frame, area: Rect, label: &str, p: &Palette) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" {label}"),
                Style::default().fg(p.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " ─".to_string()
                    + &"─".repeat(area.width.saturating_sub(label.len() as u16 + 3) as usize),
                Style::default().fg(p.surface0),
            ),
        ])),
        setting_block_header_rect(area),
    );
}

pub(crate) fn right_aligned_button_row(
    area: Rect,
    hints_and_labels: &[(&str, &str)],
    gap: u16,
) -> Vec<Rect> {
    let widths: Vec<u16> = hints_and_labels
        .iter()
        .map(|(hint, label)| {
            let hint = if hint.is_empty() { None } else { Some(*hint) };
            action_button_width(hint, label)
        })
        .collect();
    let total_w = widths
        .iter()
        .copied()
        .sum::<u16>()
        .saturating_add(gap.saturating_mul(widths.len().saturating_sub(1) as u16));
    let mut x = area.x.saturating_add(area.width.saturating_sub(total_w));
    let y = area.y;
    widths
        .iter()
        .map(|w| {
            let max_w = area.x.saturating_add(area.width).saturating_sub(x);
            let rect = Rect::new(x, y, (*w).min(max_w), 1);
            x = x.saturating_add(*w).saturating_add(gap);
            rect
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_row_rects_align_with_index_at() {
        let area = Rect::new(4, 2, 40, 1);
        let labels = ["auto", "dark", "light"];
        let rects = chip_row_rects(area, &labels, CHIP_HORIZONTAL_GAP);
        assert_eq!(rects.len(), 3);
        for (idx, rect) in rects.iter().enumerate() {
            assert_eq!(
                chip_row_index_at(area, &labels, CHIP_HORIZONTAL_GAP, rect.x + 1, rect.y),
                Some(idx)
            );
        }
    }

    #[test]
    fn chip_wrap_rects_wrap_to_multiple_rows() {
        let labels = ["catppuccin", "dracula", "nord", "gruvbox", "one-dark"];
        let row_count = chip_wrap_row_count(24, &labels, CHIP_HORIZONTAL_GAP);
        assert!(row_count > 1);
        let area = Rect::new(0, 0, 24, row_count);
        let rects = chip_wrap_rects(area, &labels, CHIP_HORIZONTAL_GAP);
        assert_eq!(rects.len(), labels.len());
        for (idx, rect) in rects.iter().enumerate() {
            assert_eq!(
                chip_wrap_index_at(area, &labels, CHIP_HORIZONTAL_GAP, rect.x + 1, rect.y),
                Some(idx)
            );
        }
    }

    #[test]
    fn checklist_item_rects_match_index_at() {
        let area = Rect::new(2, 5, 30, 7);
        let (_header, items) = checklist_group_areas(area, 3);
        let rects = checklist_item_rects(items, 3);
        assert_eq!(rects.len(), 3);
        for (idx, rect) in rects.iter().enumerate() {
            assert_eq!(
                checklist_item_index_at(items, 3, rect.x + 2, rect.y),
                Some(idx)
            );
        }
    }

    #[test]
    fn right_aligned_buttons_fit_inside_area() {
        let area = Rect::new(10, 20, 50, 1);
        let rects = right_aligned_button_row(area, &[("↵", "save"), ("esc", "cancel")], 2);
        assert_eq!(rects.len(), 2);
        let last = rects.last().expect("cancel rect");
        assert!(last.x + last.width <= area.x + area.width);
    }

    #[test]
    fn right_aligned_buttons_saturate_on_tiny_terminal_footer() {
        let area = Rect::new(0, 10, 8, 1);
        let rects = right_aligned_button_row(area, &[("↵", "save"), ("esc", "cancel")], 2);
        assert_eq!(rects.len(), 2);
        for rect in rects {
            assert!(rect.x + rect.width <= area.x + area.width);
            assert!(rect.width <= area.width);
        }
    }
}
