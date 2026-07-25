//! Layout for the compact Fleet Ops status line.

use ratatui::layout::Rect;

use crate::ui::shell::FleetOpsLayout;

/// Compute the Fleet Ops layout. The summary occupies the full one-line rect;
/// an optional detail rect is provided when expanded.
pub fn layout_fleet_ops(layout: &mut FleetOpsLayout, area: Rect, expanded: bool) {
    layout.rect = area;
    layout.expanded = expanded;
    layout.summary = area;
    layout.detail = if expanded && area.height > 2 {
        Some(rect_map::bottom_n(area, area.height.saturating_sub(1)))
    } else {
        None
    };
}

/// Toggle hit area for expanding/collapsing the Fleet Ops detail.
pub fn toggle_rect(area: Rect) -> Rect {
    area
}

mod rect_map {
    use ratatui::layout::Rect;

    pub fn bottom_n(area: Rect, n: u16) -> Rect {
        let h = n.min(area.height);
        Rect::new(area.x, area.y + area.height - h, area.width, h)
    }
}
