//! Geometry for the compact tab bar.

use ratatui::layout::{Constraint, Layout, Rect};

use crate::ui::shell::TabBarLayout;
use crate::ui::tabs_new::model::TabItem;
use crate::ui::text::{display_width, truncate_end};

/// Minimum width reserved for a tab, in cells.
const MIN_TAB_WIDTH: u16 = 4;
/// Width of the close affordance, in cells.
const CLOSE_WIDTH: u16 = 2;

/// Compute tab rectangles and write them into `layout`.
///
/// The layout uses a sliding viewport when tabs do not fit.  It returns the
/// start index of the visible tab range so rendering and hit testing agree.
pub fn layout_tab_bar(layout: &mut TabBarLayout, tabs: &[TabItem]) -> usize {
    let area = layout.rect;
    if area.width == 0 || area.height == 0 || tabs.is_empty() {
        layout.tabs = Vec::new();
        layout.overflow = false;
        return 0;
    }

    let available = area.width.saturating_sub(CLOSE_WIDTH);
    let mut widths = Vec::with_capacity(tabs.len());
    for tab in tabs {
        let label_w = display_width(&tab.label).max(1) as u16;
        widths.push(label_w.min(available));
    }

    let mut visible_start = 0usize;
    let mut visible_end = 0usize;
    let mut used: u16 = 0;
    let mut found_active = false;

    for (idx, w) in widths.iter().enumerate() {
        let needed = w.saturating_add(1); // one-cell separator/rail
        if used.saturating_add(needed) > available {
            layout.overflow = true;
            break;
        }
        visible_end = idx + 1;
        used = used.saturating_add(needed);
        if tabs[idx].active {
            found_active = true;
        }
    }

    // If the active tab is not in the initial visible range, slide the window
    // so the active tab is included.
    if !found_active {
        if let Some(active_idx) = tabs.iter().position(|t| t.active) {
            let mut best_start = 0usize;
            let mut best_used: u16 = 0;
            let mut best_end = 0usize;
            for start in 0..=active_idx.min(tabs.len().saturating_sub(1)) {
                let mut used: u16 = 0;
                let mut end = start;
                for (idx, w) in widths.iter().enumerate().skip(start) {
                    let needed = w.saturating_add(1);
                    if used.saturating_add(needed) > available {
                        break;
                    }
                    used += needed;
                    end = idx + 1;
                    if idx >= active_idx {
                        break;
                    }
                }
                if end > active_idx || (end == active_idx + 1 && used > best_used) {
                    best_start = start;
                    best_used = used;
                    best_end = end;
                    break;
                }
                if end > best_end || (end == best_end && used > best_used) {
                    best_start = start;
                    best_used = used;
                    best_end = end;
                }
            }
            visible_start = best_start;
            visible_end = best_end;
            layout.overflow = visible_end < tabs.len() || visible_start > 0;
        }
    }

    let visible_tabs = &tabs[visible_start..visible_end];
    let mut x = area.x;
    layout.tabs = visible_tabs
        .iter()
        .enumerate()
        .map(|(local_idx, tab)| {
            let global_idx = visible_start + local_idx;
            let w = widths[global_idx].min(available);
            let rect = Rect::new(x, area.y, w, area.height);
            x = x.saturating_add(w.saturating_add(1));
            crate::ui::shell::TabHitRect {
                rect,
                index: global_idx,
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
