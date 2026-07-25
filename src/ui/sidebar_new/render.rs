//! Renderer for the unified sidebar.
//!
//! This is an intentionally small, readable first pass.  It draws the mode
//! header and a row for each visible `SidebarItem`.  It reuses existing palette
//! and status helpers so the output already respects the current theme.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::state::AppState;
use crate::terminal::TerminalRuntimeRegistry;
use crate::ui::shell::{LayoutMode, ShellLayout, SidebarMode};
use crate::ui::sidebar_new::model::{SidebarItem, SidebarItemId};
use crate::ui::status::{state_dot, state_label_color};

/// Render the unified sidebar into the area described by `layout.sidebar.rect`.
///
/// The render is pure: it reads `AppState` and `ShellLayout`, writes to the
/// Ratatui buffer, and mutates nothing else.
pub fn render_sidebar_new(
    app: &AppState,
    _terminal_runtimes: &TerminalRuntimeRegistry,
    frame: &mut Frame,
    layout: &ShellLayout,
    items: &[SidebarItem],
) {
    let area = layout.sidebar.rect;
    if area.width == 0 || area.height == 0 {
        return;
    }

    let p = &app.palette;

    // Separator rail on the right edge of the sidebar.
    let sep_x = area.x + area.width.saturating_sub(1);
    {
        let buf = frame.buffer_mut();
        let sep_style = Style::default().fg(p.surface_dim);
        for y in area.y..area.y + area.height {
            buf[(sep_x, y)].set_symbol("│");
            buf[(sep_x, y)].set_style(sep_style);
        }
    }

    // Mode header.
    let header = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled(
            mode_label(layout.sidebar.mode),
            Style::default()
                .fg(p.text)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(header, top_line_of(area));

    // Rows.
    for row in &layout.sidebar.rows {
        let item = items.iter().find(|item| item.id == row.id);
        let Some(item) = item else {
            continue;
        };
        let selected = is_selected(item, app);
        let active = is_active(item, app);
        let (icon, icon_style) = state_dot(item.state, item.seen, p);
        let label_color = state_label_color(item.state, item.seen, p);

        let bg = if selected {
            p.surface0
        } else if active {
            p.surface_dim
        } else {
            ratatui::style::Color::Reset
        };

        let mut spans = vec![
            Span::raw(" "),
            Span::styled(icon, icon_style),
            Span::raw(" "),
            Span::styled(
                truncate_primary(&item.primary, row.rect.width, 4),
                Style::default()
                    .fg(if selected || active { p.text } else { p.subtext0 })
                    .add_modifier(Modifier::BOLD),
            ),
        ];
        if let Some(secondary) = &item.secondary {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                truncate_primary(secondary, row.rect.width, 4 + display_width(&item.primary) + 1),
                Style::default().fg(label_color).add_modifier(Modifier::DIM),
            ));
        }

        let row_area = row.rect;
        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
            row_area,
        );
    }
}

fn top_line_of(rect: Rect) -> Rect {
    Rect::new(rect.x, rect.y, rect.width, 1.min(rect.height))
}

fn mode_label(mode: SidebarMode) -> &'static str {
    match mode {
        SidebarMode::Workspaces => "Workspaces",
        SidebarMode::Agents => "Agents",
        SidebarMode::Attention => "Attention",
    }
}

fn is_selected(item: &SidebarItem, app: &AppState) -> bool {
    match &item.id {
        SidebarItemId::Workspace { ws_idx } => {
            matches!(app.mode, crate::app::Mode::Navigate) && app.selected == *ws_idx
        }
        _ => false,
    }
}

fn is_active(item: &SidebarItem, app: &AppState) -> bool {
    match &item.id {
        SidebarItemId::Workspace { ws_idx } => app.active == Some(*ws_idx),
        SidebarItemId::Agent { ws_idx, .. } => app.active == Some(*ws_idx),
        _ => false,
    }
}

fn display_width(text: &str) -> usize {
    crate::ui::text::display_width(text)
}

fn truncate_primary(text: &str, max_width: u16, used: usize) -> String {
    let available = max_width.saturating_sub(used as u16) as usize;
    crate::ui::text::truncate_end(text, available)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;
    use crate::terminal::TerminalRuntimeRegistry;
    use crate::ui::shell::{compute_shell_layout, LayoutMode, ScrollState, SidebarMode};
    use crate::ui::sidebar_new::model::SidebarModel;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn renders_header_for_each_mode() {
        let app = AppState::test_new();
        let layout = compute_shell_layout(
            Rect::new(0, 0, 120, 40),
            LayoutMode::Standard,
            false,
            28,
            SidebarMode::Agents,
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
        );
        let registry = TerminalRuntimeRegistry::new();
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| render_sidebar_new(&app, &registry, frame, &layout, &[]))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let header: String = (layout.sidebar.rect.x..layout.sidebar.rect.x + layout.sidebar.rect.width)
            .map(|x| buffer[(x, layout.sidebar.rect.y)].symbol())
            .collect();
        assert!(header.contains("Agents"));
    }
}
