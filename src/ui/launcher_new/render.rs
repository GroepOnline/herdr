//! Renderer for the global launcher overlay.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::state::AppState;
use crate::ui::launcher_new::layout::LauncherLayout;
use crate::ui::launcher_new::model::LauncherItem;

/// Render the launcher overlay.
pub fn render_launcher_new(
    _app: &AppState,
    frame: &mut Frame,
    layout: &LauncherLayout,
    items: &[LauncherItem],
    selected: usize,
    palette: &crate::app::state::Palette,
) {
    if layout.overlay.width == 0 || layout.overlay.height == 0 {
        return;
    }

    frame.render_widget(Clear, layout.overlay);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.surface_dim)),
        layout.overlay,
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            "› ",
            Style::default().fg(palette.accent),
        )])),
        layout.input,
    );

    for (idx, (row, item)) in layout.rows.iter().zip(items.iter()).enumerate() {
        let is_selected = idx == selected;
        let fg = if is_selected {
            palette.text
        } else {
            palette.subtext0
        };
        let bg = if is_selected {
            palette.surface0
        } else {
            continue;
        };
        let mut spans = vec![
            Span::styled(item.icon, Style::default().fg(palette.accent)),
            Span::raw(" "),
            Span::styled(
                crate::ui::text::truncate_end(&item.primary, row.width.saturating_sub(4) as usize),
                Style::default().fg(fg),
            ),
        ];
        if let Some(secondary) = &item.secondary {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                crate::ui::text::truncate_end(secondary, row.width.saturating_sub(6) as usize),
                Style::default()
                    .fg(palette.overlay0)
                    .add_modifier(Modifier::DIM),
            ));
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
            *row,
        );
    }
}
