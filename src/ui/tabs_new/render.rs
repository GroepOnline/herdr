//! Renderer for the compact tab bar.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::state::AppState;
use crate::ui::shell::TabBarLayout;
use crate::ui::tabs_new::layout::fit_tab_label;
use crate::ui::tabs_new::model::TabItem;

/// Render the compact tab bar.
///
/// The active tab is shown with an underline/rail on the bottom row and bold
/// text.  Working/blocked/unseen-done states get a small glyph marker.
pub fn render_tab_bar_new(
    _app: &AppState,
    frame: &mut Frame,
    layout: &TabBarLayout,
    tabs: &[TabItem],
    palette: &crate::app::state::Palette,
) {
    if layout.rect.width == 0 || layout.rect.height == 0 {
        return;
    }

    for hit in &layout.tabs {
        let Some(tab) = tabs.get(hit.index) else {
            continue;
        };
        let rect = hit.rect;
        if rect.width == 0 {
            continue;
        }

        let (fg, bg) = colors_for_tab(tab, palette);
        let icon = tab.clone().status_icon();
        let label = fit_tab_label(&tab.label, rect.width);

        let mut spans = vec![Span::raw(" ")];
        if !icon.is_empty() {
            spans.push(Span::styled(icon, Style::default().fg(fg)));
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(
            label,
            Style::default().fg(fg).bg(bg).add_modifier(if tab.active {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
        ));

        let style = Style::default().bg(bg);
        frame.render_widget(Paragraph::new(Line::from(spans)).style(style), rect);

        // Active indicator: bottom one-cell rail.
        if tab.active && rect.height > 0 {
            let indicator_y = rect.y + rect.height.saturating_sub(1);
            let buf = frame.buffer_mut();
            for x in rect.x..rect.x + rect.width {
                let cell = &mut buf[(x, indicator_y)];
                cell.set_symbol("─");
                cell.set_style(Style::default().fg(palette.accent));
            }
        }
    }
}

fn colors_for_tab(
    tab: &TabItem,
    palette: &crate::app::state::Palette,
) -> (crate::app::state::Color, crate::app::state::Color) {
    use ratatui::style::Color;
    let fg = match tab.state {
        crate::detect::AgentState::Blocked => palette.red,
        crate::detect::AgentState::Working => palette.yellow,
        crate::detect::AgentState::Idle if !tab.seen => palette.teal,
        crate::detect::AgentState::Idle => palette.green,
        crate::detect::AgentState::Unknown => palette.overlay0,
    };
    let bg = if tab.active {
        palette.surface0
    } else {
        Color::Reset
    };
    (fg, bg)
}
