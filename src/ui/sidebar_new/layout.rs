//! Geometry layout for the unified sidebar.
//!
//! This module computes `SidebarRowRect`s from a `SidebarModel` and the current
//! `ShellLayout`.  Rendering and hit testing use the same rectangles so they
//! cannot drift apart.

use ratatui::layout::Rect;

use crate::ui::shell::{compute_shell_layout, ShellLayout, SidebarRowRect, SidebarMode};
use crate::ui::sidebar_new::model::SidebarModel;

/// Compute per-row rectangles for the current sidebar mode and write them into
/// `layout.sidebar.rows`.  The function is pure: it mutates only the derived
/// geometry field, never `AppState`.
pub fn layout_sidebar(layout: &mut ShellLayout, model: &SidebarModel) {
    let mode = layout.sidebar.mode;
    let items = model.items_for_mode(mode);
    let content = layout.sidebar.content;
    if content.width == 0 || content.height == 0 {
        layout.sidebar.rows = Vec::new();
        return;
    }

    let total = items.len();
    let viewport_height = content.height as usize;
    let max_scroll = total.saturating_sub(viewport_height);
    let scroll = layout.sidebar.scroll.offset.min(max_scroll);

    let mut rows = Vec::with_capacity(viewport_height.min(total));
    let mut y = content.y;
    for item in items.iter().skip(scroll) {
        if rows.len() >= viewport_height || y >= content.y + content.height {
            break;
        }
        rows.push(SidebarRowRect {
            rect: Rect::new(content.x, y, content.width, 1),
            id: item.id.clone(),
        });
        y = y.saturating_add(1);
    }

    layout.sidebar.rows = rows;
}

/// Find the row rect (if any) that contains the given point.  This is the
/// inverse of the layout computation and is used for mouse hit testing.
pub fn row_at(layout: &ShellLayout, x: u16, y: u16) -> Option<&SidebarRowRect> {
    layout.sidebar.rows.iter().find(|row| {
        let r = row.rect;
        x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;
    use crate::terminal::TerminalRuntimeRegistry;
    use crate::ui::shell::{LayoutMode, ScrollState, ShellLayout, SidebarMode};
    use crate::ui::sidebar_new::model::SidebarModel;

    fn test_layout() -> ShellLayout {
        compute_shell_layout(
            Rect::new(0, 0, 120, 40),
            LayoutMode::Standard,
            false,
            28,
            SidebarMode::Workspaces,
            ScrollState {
                offset: 0,
                visible: 10,
                total: 5,
            },
            ScrollState {
                offset: 0,
                visible: 10,
                total: 5,
            },
            ScrollState {
                offset: 0,
                visible: 10,
                total: 5,
            },
        )
    }

    #[test]
    fn rows_fill_content_area_one_per_row() {
        let mut layout = test_layout();
        let mut app = AppState::test_new();
        app.workspaces = vec![
            crate::workspace::Workspace::test_new("a"),
            crate::workspace::Workspace::test_new("b"),
            crate::workspace::Workspace::test_new("c"),
        ];
        let registry = TerminalRuntimeRegistry::new();
        let mut model = SidebarModel::new();
        model.rebuild(&app, &registry);

        layout_sidebar(&mut layout, &model);

        assert!(!layout.sidebar.rows.is_empty());
        assert_eq!(layout.sidebar.rows[0].rect.y, layout.sidebar.content.y);
        assert_eq!(layout.sidebar.rows[0].rect.height, 1);
        assert_eq!(layout.sidebar.rows[0].rect.width, layout.sidebar.content.width);
    }

    #[test]
    fn row_at_finds_hit() {
        let mut layout = test_layout();
        let mut app = AppState::test_new();
        app.workspaces = vec![
            crate::workspace::Workspace::test_new("a"),
            crate::workspace::Workspace::test_new("b"),
        ];
        let registry = TerminalRuntimeRegistry::new();
        let mut model = SidebarModel::new();
        model.rebuild(&app, &registry);
        layout_sidebar(&mut layout, &model);

        let row = row_at(&layout, layout.sidebar.content.x, layout.sidebar.content.y).unwrap();
        assert!(matches!(row.id, crate::ui::sidebar_new::model::SidebarItemId::Workspace { ws_idx: 0, .. }));
    }
}
