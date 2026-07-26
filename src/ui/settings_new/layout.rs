//! Layout for the settings overlay.

use ratatui::layout::{Constraint, Layout, Rect};

/// Geometry for the settings overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsLayout {
    pub overlay: Rect,
    pub category_rail: Rect,
    pub content: Rect,
    pub footer: Rect,
    pub rows: Vec<Rect>,
}

/// Compute settings overlay geometry.
pub fn layout_settings(area: Rect) -> SettingsLayout {
    let width = area.width.saturating_sub(4).min(100).max(area.width / 2);
    let height = area.height.saturating_sub(4);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + 2;
    let overlay = Rect::new(x, y, width, height);

    let [category_rail, content] =
        Layout::horizontal([Constraint::Length(16), Constraint::Min(1)]).areas(overlay);
    let [content, footer] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(content);

    SettingsLayout {
        overlay,
        category_rail,
        content,
        footer,
        rows: Vec::new(),
    }
}

/// Compute row rectangles for settings items.
pub fn settings_row_rect(_layout: &SettingsLayout, _index: usize) -> Option<Rect> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_centers_overlay() {
        let area = Rect::new(0, 0, 120, 40);
        let layout = layout_settings(area);
        assert!(layout.overlay.x > 0);
        assert!(layout.overlay.y > 0);
        assert!(layout.overlay.width > 40);
        assert!(layout.overlay.height > 0);
    }

    #[test]
    fn layout_clamps_width_to_100() {
        let area = Rect::new(0, 0, 200, 60);
        let layout = layout_settings(area);
        assert!(layout.overlay.width <= 100);
    }

    #[test]
    fn layout_minimum_width_is_half_area() {
        let area = Rect::new(0, 0, 20, 10);
        let layout = layout_settings(area);
        assert!(layout.overlay.width >= 10);
    }

    #[test]
    fn category_rail_is_16_columns() {
        let area = Rect::new(0, 0, 100, 40);
        let layout = layout_settings(area);
        assert_eq!(layout.category_rail.width, 16);
    }

    #[test]
    fn footer_is_one_line() {
        let area = Rect::new(0, 0, 100, 40);
        let layout = layout_settings(area);
        assert_eq!(layout.footer.height, 1);
    }

    #[test]
    fn content_is_remaining_space() {
        let area = Rect::new(0, 0, 100, 40);
        let layout = layout_settings(area);
        assert_eq!(layout.content.width, layout.overlay.width - 16);
        assert_eq!(
            layout.content.height + 1,
            layout.overlay.height
        );
    }

    #[test]
    fn settings_row_rect_returns_none() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = layout_settings(area);
        assert!(settings_row_rect(&layout, 0).is_none());
    }

    #[test]
    fn rows_start_empty() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = layout_settings(area);
        assert!(layout.rows.is_empty());
    }

    #[test]
    fn layout_handles_tiny_terminal() {
        let area = Rect::new(0, 0, 10, 10);
        let layout = layout_settings(area);
        assert!(layout.overlay.width >= 5);
        assert!(layout.overlay.height >= 6);
    }

    #[test]
    fn layout_eq_and_clone() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = layout_settings(area);
        let layout2 = layout.clone();
        assert_eq!(layout, layout2);
    }
}
