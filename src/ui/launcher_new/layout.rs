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
