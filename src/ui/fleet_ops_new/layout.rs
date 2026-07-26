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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::shell::FleetOpsLayout;
    use ratatui::layout::Rect;

    #[test]
    fn layout_collapsed_only_has_summary() {
        let area = Rect::new(0, 39, 120, 1);
        let mut layout = FleetOpsLayout {
            rect: Rect::default(),
            expanded: false,
            summary: Rect::default(),
            detail: None,
        };
        layout_fleet_ops(&mut layout, area, false);
        assert_eq!(layout.rect, area);
        assert!(!layout.expanded);
        assert_eq!(layout.summary, area);
        assert!(layout.detail.is_none());
    }

    #[test]
    fn layout_expanded_has_detail_when_tall_enough() {
        let area = Rect::new(0, 36, 120, 4);
        let mut layout = FleetOpsLayout {
            rect: Rect::default(),
            expanded: false,
            summary: Rect::default(),
            detail: None,
        };
        layout_fleet_ops(&mut layout, area, true);
        assert!(layout.expanded);
        assert!(layout.detail.is_some());
        if let Some(detail) = layout.detail {
            assert_eq!(detail.y, area.y + area.height - (area.height - 1));
            assert_eq!(detail.width, area.width);
        }
    }

    #[test]
    fn layout_expanded_no_detail_when_too_short() {
        let area = Rect::new(0, 39, 120, 1);
        let mut layout = FleetOpsLayout {
            rect: Rect::default(),
            expanded: false,
            summary: Rect::default(),
            detail: None,
        };
        layout_fleet_ops(&mut layout, area, true);
        assert!(layout.expanded);
        assert!(layout.detail.is_none());
    }

    #[test]
    fn toggle_rect_returns_area() {
        let area = Rect::new(0, 0, 120, 1);
        assert_eq!(toggle_rect(area), area);
    }

    #[test]
    fn bottom_n_clamps_to_height() {
        let area = Rect::new(0, 0, 80, 3);
        let result = rect_map::bottom_n(area, 10);
        assert_eq!(result.height, 3);
    }

    #[test]
    fn bottom_n_for_one_row() {
        let area = Rect::new(0, 0, 80, 1);
        let result = rect_map::bottom_n(area, 1);
        assert_eq!(result.y, 0);
        assert_eq!(result.height, 1);
    }
}
