//! Layout for the global launcher overlay.

use ratatui::layout::{Constraint, Layout, Rect};

/// Geometry for the launcher overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherLayout {
    pub overlay: Rect,
    pub input: Rect,
    pub list: Rect,
    pub rows: Vec<Rect>,
}

/// Compute launcher geometry centered over the given area.
pub fn layout_launcher(area: Rect) -> LauncherLayout {
    let width = (area.width as f32 * 0.7).min(area.width as f32) as u16;
    let height = (area.height as f32 * 0.7).min(area.height as f32) as u16;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let overlay = Rect::new(x, y, width, height);

    let [input, list] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(overlay);

    LauncherLayout {
        overlay,
        input,
        list,
        rows: Vec::new(),
    }
}

/// Compute per-row rectangles for the launcher list.
pub fn layout_launcher_rows(layout: &mut LauncherLayout, count: usize) {
    layout.rows.clear();
    let mut y = layout.list.y;
    for _ in 0..count.min(layout.list.height as usize) {
        if y >= layout.list.y + layout.list.height {
            break;
        }
        layout
            .rows
            .push(Rect::new(layout.list.x, y, layout.list.width, 1));
        y += 1;
    }
}

/// Hit test for a launcher row.
pub fn launcher_row_rect(layout: &LauncherLayout, index: usize) -> Option<Rect> {
    layout.rows.get(index).copied()
}

/// Toggle button rect (for future use).
pub fn toggle_button_rect(_layout: &LauncherLayout) -> Rect {
    Rect::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_centers_overlay() {
        let area = Rect::new(0, 0, 100, 40);
        let layout = layout_launcher(area);
        assert!(layout.overlay.x > 0);
        assert!(layout.overlay.y > 0);
        assert!(layout.overlay.width <= 70);
        assert!(layout.overlay.height <= 28);
    }

    #[test]
    fn layout_has_input_and_list_areas() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = layout_launcher(area);
        assert_eq!(layout.input.height, 1);
        assert_eq!(layout.list.y, layout.input.y + 1);
    }

    #[test]
    fn layout_clamps_to_area() {
        let area = Rect::new(0, 0, 10, 5);
        let layout = layout_launcher(area);
        assert!(layout.overlay.width <= 10);
        assert!(layout.overlay.height <= 5);
    }

    #[test]
    fn rows_fill_list_area() {
        let area = Rect::new(0, 0, 80, 24);
        let mut layout = layout_launcher(area);
        let count = 5;
        layout_launcher_rows(&mut layout, count);
        assert_eq!(layout.rows.len(), count);
        assert_eq!(layout.rows[0].y, layout.list.y);
        assert_eq!(layout.rows[4].y, layout.list.y + 4);
    }

    #[test]
    fn rows_truncated_when_count_exceeds_height() {
        let area = Rect::new(0, 0, 80, 10);
        let mut layout = layout_launcher(area);
        layout_launcher_rows(&mut layout, 100);
        assert!(layout.rows.len() <= layout.list.height as usize);
    }

    #[test]
    fn launcher_row_rect_out_of_bounds() {
        let area = Rect::new(0, 0, 80, 24);
        let mut layout = layout_launcher(area);
        layout_launcher_rows(&mut layout, 3);
        assert!(launcher_row_rect(&layout, 0).is_some());
        assert!(launcher_row_rect(&layout, 2).is_some());
        assert!(launcher_row_rect(&layout, 5).is_none());
    }

    #[test]
    fn toggle_button_rect_is_default() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = layout_launcher(area);
        assert_eq!(toggle_button_rect(&layout), Rect::default());
    }

    #[test]
    fn zero_count_clears_rows() {
        let area = Rect::new(0, 0, 80, 24);
        let mut layout = layout_launcher(area);
        layout_launcher_rows(&mut layout, 5);
        assert_eq!(layout.rows.len(), 5);
        layout_launcher_rows(&mut layout, 0);
        assert!(layout.rows.is_empty());
    }

    #[test]
    fn tiny_area_still_produces_layout() {
        let area = Rect::new(0, 0, 4, 4);
        let layout = layout_launcher(area);
        assert!(layout.overlay.width >= 2);
        assert!(layout.overlay.height >= 2);
    }
}
