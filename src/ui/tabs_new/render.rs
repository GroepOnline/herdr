//! Renderer for the compact tab bar.

use ratatui::{
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

    if layout.scroll_left.width > 0 {
        frame.render_widget(
            Paragraph::new("‹").style(Style::default().fg(palette.overlay0)),
            layout.scroll_left,
        );
    }
    if layout.scroll_right.width > 0 {
        frame.render_widget(
            Paragraph::new("›").style(Style::default().fg(palette.overlay0)),
            layout.scroll_right,
        );
    }
    if layout.new_tab.width > 0 {
        frame.render_widget(
            Paragraph::new(" +").style(Style::default().fg(palette.overlay0)),
            layout.new_tab,
        );
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
) -> (ratatui::style::Color, ratatui::style::Color) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{AppState, Palette};
    use crate::ui::shell::TabBarLayout;
    use crate::ui::tabs_new::model::TabItem;
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};

    fn test_palette() -> Palette {
        Palette::catppuccin()
    }

    #[test]
    fn colors_for_blocked_tab() {
        let tab = TabItem {
            index: 0,
            label: "b".into(),
            state: crate::detect::AgentState::Blocked,
            seen: false,
            active: false,
            working: false,
            blocked: true,
            unseen_done: false,
        };
        let palette = test_palette();
        let (fg, _bg) = colors_for_tab(&tab, &palette);
        assert_eq!(fg, palette.red);
    }

    #[test]
    fn colors_for_working_tab() {
        let tab = TabItem {
            index: 0,
            label: "w".into(),
            state: crate::detect::AgentState::Working,
            seen: false,
            active: false,
            working: true,
            blocked: false,
            unseen_done: false,
        };
        let palette = test_palette();
        let (fg, _bg) = colors_for_tab(&tab, &palette);
        assert_eq!(fg, palette.yellow);
    }

    #[test]
    fn colors_for_unseen_idle_tab() {
        let tab = TabItem {
            index: 0,
            label: "u".into(),
            state: crate::detect::AgentState::Idle,
            seen: false,
            active: false,
            working: false,
            blocked: false,
            unseen_done: true,
        };
        let palette = test_palette();
        let (fg, _bg) = colors_for_tab(&tab, &palette);
        assert_eq!(fg, palette.teal);
    }

    #[test]
    fn colors_for_active_tab_uses_surface_bg() {
        let tab = TabItem {
            index: 0,
            label: "a".into(),
            state: crate::detect::AgentState::Idle,
            seen: true,
            active: true,
            working: false,
            blocked: false,
            unseen_done: false,
        };
        let palette = test_palette();
        let (_fg, bg) = colors_for_tab(&tab, &palette);
        assert_eq!(bg, palette.surface0);
    }

    #[test]
    fn status_icon_for_blocked() {
        let tab = TabItem {
            index: 0,
            label: "b".into(),
            state: crate::detect::AgentState::Blocked,
            seen: false,
            active: false,
            working: false,
            blocked: true,
            unseen_done: false,
        };
        assert_eq!(tab.status_icon(), "◉");
    }

    #[test]
    fn status_icon_for_working() {
        let tab = TabItem {
            index: 0,
            label: "w".into(),
            state: crate::detect::AgentState::Working,
            seen: false,
            active: false,
            working: true,
            blocked: false,
            unseen_done: false,
        };
        assert_eq!(tab.status_icon(), "●");
    }

    #[test]
    fn status_icon_for_unseen_done() {
        let tab = TabItem {
            index: 0,
            label: "d".into(),
            state: crate::detect::AgentState::Idle,
            seen: false,
            active: false,
            working: false,
            blocked: false,
            unseen_done: true,
        };
        assert_eq!(tab.status_icon(), "✓");
    }

    #[test]
    fn status_icon_for_seen_idle() {
        let tab = TabItem {
            index: 0,
            label: "i".into(),
            state: crate::detect::AgentState::Idle,
            seen: true,
            active: false,
            working: false,
            blocked: false,
            unseen_done: false,
        };
        assert_eq!(tab.status_icon(), "");
    }

    #[test]
    fn render_tab_bar_empty_rect_is_noop() {
        let app = AppState::test_new();
        let layout = TabBarLayout {
            rect: Rect::default(),
            tabs: Vec::new(),
            scroll_left: Rect::default(),
            scroll_right: Rect::default(),
            new_tab: Rect::default(),
            overflow: false,
        };
        let palette = test_palette();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let tabs: Vec<TabItem> = Vec::new();
        terminal
            .draw(|frame| render_tab_bar_new(&app, frame, &layout, &tabs, &palette))
            .unwrap();
        // No panic = pass.
    }

    #[test]
    fn render_tab_bar_draws_active_indicator() {
        let app = AppState::test_new();
        let palette = test_palette();
        let layout = TabBarLayout {
            rect: Rect::new(0, 0, 80, 1),
            tabs: vec![crate::ui::shell::TabHitRect {
                rect: Rect::new(0, 0, 10, 1),
                index: 0,
                visible: true,
            }],
            scroll_left: Rect::default(),
            scroll_right: Rect::default(),
            new_tab: Rect::default(),
            overflow: false,
        };
        let tabs = vec![TabItem {
            index: 0,
            label: "test".into(),
            state: crate::detect::AgentState::Idle,
            seen: true,
            active: true,
            working: false,
            blocked: false,
            unseen_done: false,
        }];
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|frame| render_tab_bar_new(&app, frame, &layout, &tabs, &palette))
            .unwrap();
        // The active indicator should have rendered a "─" on the bottom row.
        let buffer = terminal.backend().buffer();
        let symbols: String = (0..10)
            .map(|x| buffer[(x, 0)].symbol().to_string())
            .collect();
        assert!(
            symbols.contains('─'),
            "expected active indicator: {symbols}"
        );
    }
}
