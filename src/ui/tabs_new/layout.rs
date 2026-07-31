//! Geometry for the compact tab bar.

use ratatui::layout::Rect;

use crate::ui::shell::TabBarLayout;
use crate::ui::tabs_new::model::TabItem;
use crate::ui::text::{display_width, truncate_end};

/// Minimum width reserved for a tab, in cells.
const MIN_TAB_WIDTH: u16 = 4;
const TAB_GAP: u16 = 1;
const NEW_TAB_WIDTH: u16 = 3;
const SCROLL_WIDTH: u16 = 2;

/// Compute tab rectangles and write them into `layout`.
pub fn layout_tab_bar(layout: &mut TabBarLayout, tabs: &[TabItem]) -> usize {
    let area = layout.rect;
    layout.tabs.clear();
    layout.scroll_left = Rect::default();
    layout.scroll_right = Rect::default();
    layout.new_tab = Rect::default();
    layout.overflow = false;

    if area.width == 0 || area.height == 0 {
        return 0;
    }

    let new_tab_width = area.width.min(NEW_TAB_WIDTH);
    layout.new_tab = Rect::new(
        area.x + area.width.saturating_sub(new_tab_width),
        area.y,
        new_tab_width,
        area.height,
    );
    if tabs.is_empty() {
        return 0;
    }

    let base_available = layout.new_tab.x.saturating_sub(area.x);
    if base_available == 0 {
        layout.overflow = true;
        return 0;
    }

    let widths: Vec<u16> = tabs
        .iter()
        .map(|tab| {
            let label_width = display_width(&tab.label).max(1) as u16;
            label_width.max(MIN_TAB_WIDTH).min(base_available)
        })
        .collect();
    let total_width = widths.iter().fold(0u16, |total, width| {
        total.saturating_add(width.saturating_add(TAB_GAP))
    });
    let needs_overflow = total_width > base_available;

    let (tab_start_x, available) =
        if needs_overflow && base_available > SCROLL_WIDTH.saturating_mul(2) {
            layout.scroll_left = Rect::new(area.x, area.y, SCROLL_WIDTH, area.height);
            layout.scroll_right = Rect::new(
                layout.new_tab.x.saturating_sub(SCROLL_WIDTH),
                area.y,
                SCROLL_WIDTH,
                area.height,
            );
            let start = area.x.saturating_add(SCROLL_WIDTH);
            (start, layout.scroll_right.x.saturating_sub(start))
        } else {
            (area.x, base_available)
        };

    let fit_end = |start: usize| -> usize {
        let mut used = 0u16;
        let mut end = start;
        for (index, width) in widths.iter().enumerate().skip(start) {
            let needed = width.saturating_add(TAB_GAP);
            if used.saturating_add(needed) > available {
                break;
            }
            used = used.saturating_add(needed);
            end = index + 1;
        }
        end
    };

    let active_index = tabs.iter().position(|tab| tab.active).unwrap_or(0);
    let mut visible_start = 0usize;
    let mut visible_end = fit_end(visible_start);
    if active_index >= visible_end {
        visible_start = active_index;
        let mut used = widths[active_index].saturating_add(TAB_GAP);
        while visible_start > 0 {
            let previous = widths[visible_start - 1].saturating_add(TAB_GAP);
            if used.saturating_add(previous) > available {
                break;
            }
            visible_start -= 1;
            used = used.saturating_add(previous);
        }
        visible_end = fit_end(visible_start);
    }

    layout.overflow = visible_start > 0 || visible_end < tabs.len();
    let mut x = tab_start_x;
    layout.tabs = (visible_start..visible_end)
        .map(|index| {
            let width = widths[index].min(available);
            let rect = Rect::new(x, area.y, width, area.height);
            x = x.saturating_add(width.saturating_add(TAB_GAP));
            crate::ui::shell::TabHitRect {
                rect,
                index,
                visible: true,
            }
        })
        .collect();

    visible_start
}

/// Return the visible tab index at the given x coordinate, if any.
pub fn tab_at(layout: &TabBarLayout, x: u16, _y: u16) -> Option<usize> {
    layout
        .tabs
        .iter()
        .find(|hit| x >= hit.rect.x && x < hit.rect.x + hit.rect.width)
        .map(|hit| hit.index)
}

/// Truncate a tab label to fit inside the tab width.
pub fn fit_tab_label(label: &str, width: u16) -> String {
    let max = width.saturating_sub(2) as usize;
    truncate_end(label, max.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::shell::TabBarLayout;
    use crate::ui::tabs_new::model::TabItem;
    use crate::ui::text::display_width;
    use ratatui::layout::Rect;

    fn test_tabs(count: usize, active_idx: usize) -> Vec<TabItem> {
        (0..count)
            .map(|i| TabItem {
                index: i,
                label: format!("tab{}", i),
                state: crate::detect::AgentState::Idle,
                seen: true,
                active: i == active_idx,
                working: false,
                blocked: false,
                unseen_done: false,
            })
            .collect()
    }

    fn empty_tab_bar() -> TabBarLayout {
        TabBarLayout {
            rect: Rect::default(),
            tabs: Vec::new(),
            scroll_left: Rect::default(),
            scroll_right: Rect::default(),
            new_tab: Rect::default(),
            overflow: false,
        }
    }

    #[test]
    fn empty_area_or_tabs_produces_no_layout() {
        let mut layout = empty_tab_bar();
        let start = layout_tab_bar(&mut layout, &[]);
        assert_eq!(start, 0);
        assert!(layout.tabs.is_empty());
        assert!(!layout.overflow);

        let mut layout = TabBarLayout {
            rect: Rect::new(0, 0, 0, 1),
            ..empty_tab_bar()
        };
        let tabs = test_tabs(3, 0);
        let start = layout_tab_bar(&mut layout, &tabs);
        assert_eq!(start, 0);
        assert!(layout.tabs.is_empty());
    }

    #[test]
    fn fits_all_tabs_when_space_available() {
        let mut layout = TabBarLayout {
            rect: Rect::new(0, 0, 80, 1),
            ..empty_tab_bar()
        };
        let tabs = test_tabs(3, 0);
        let start = layout_tab_bar(&mut layout, &tabs);
        assert_eq!(start, 0);
        assert_eq!(layout.tabs.len(), 3);
        assert!(!layout.overflow);
    }

    #[test]
    fn marks_overflow_when_tabs_exceed_width() {
        let mut layout = TabBarLayout {
            rect: Rect::new(0, 0, 20, 1),
            ..empty_tab_bar()
        };
        let tabs = test_tabs(10, 0);
        layout_tab_bar(&mut layout, &tabs);
        assert!(layout.overflow);
        assert!(layout.tabs.len() < 10);
        assert!(layout.scroll_left.width > 0);
        assert!(layout.scroll_right.width > 0);
        assert!(layout.new_tab.width > 0);
        assert!(layout.scroll_left.x < layout.scroll_right.x);
    }

    #[test]
    fn slides_to_include_active_tab() {
        let mut layout = TabBarLayout {
            rect: Rect::new(0, 0, 30, 1),
            ..empty_tab_bar()
        };
        // Active tab is at the end — should slide viewport to include it.
        let tabs = test_tabs(8, 7);
        layout_tab_bar(&mut layout, &tabs);
        assert!(layout.overflow);
        assert!(layout.tabs.iter().any(|t| t.index == 7));
    }

    #[test]
    fn tab_at_returns_correct_index() {
        let mut layout = TabBarLayout {
            rect: Rect::new(0, 0, 80, 1),
            ..empty_tab_bar()
        };
        let tabs = test_tabs(3, 0);
        layout_tab_bar(&mut layout, &tabs);

        // First tab starts at x=0 with width 4 ("tab0") = 4
        let first = tab_at(&layout, 0, 0);
        assert_eq!(first, Some(0));

        // Click outside tabs returns None.
        assert_eq!(tab_at(&layout, 79, 0), None);
    }

    #[test]
    fn fit_tab_label_truncates_long_labels() {
        let label = "very long tab label that exceeds space";
        let fitted = fit_tab_label(label, 10);
        assert!(display_width(&fitted) <= 8); // width - 2 = 8
        let prefix = fitted.trim_end_matches('…');
        assert!(label.starts_with(prefix));
    }

    #[test]
    fn fit_tab_label_handles_zero_width() {
        let fitted = fit_tab_label("test", 0);
        assert_eq!(display_width(&fitted), 1); // max(1, 0) = 1
        assert_eq!(fitted, "…");
    }

    #[test]
    fn layout_preserves_active_tab_in_small_viewport() {
        let mut layout = TabBarLayout {
            rect: Rect::new(0, 0, 25, 1),
            ..empty_tab_bar()
        };
        // Active tab is #5 out of 8; viewport should slide to include it.
        let tabs = test_tabs(8, 5);
        layout_tab_bar(&mut layout, &tabs);
        let visible_indices: Vec<usize> = layout.tabs.iter().map(|t| t.index).collect();
        assert!(visible_indices.contains(&5));
    }

    #[test]
    fn single_tab_fills_available_width() {
        let mut layout = TabBarLayout {
            rect: Rect::new(0, 0, 40, 1),
            ..empty_tab_bar()
        };
        let tabs = test_tabs(1, 0);
        layout_tab_bar(&mut layout, &tabs);
        assert_eq!(layout.tabs.len(), 1);
        assert!(!layout.overflow);
    }
}
