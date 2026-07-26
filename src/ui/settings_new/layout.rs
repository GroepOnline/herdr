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
