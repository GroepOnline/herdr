//! Renderer for the compact Fleet Ops status line.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::state::AppState;
use crate::ui::fleet_ops_new::model::FleetOpsContext;
use crate::ui::shell::FleetOpsLayout;
use crate::ui::text::truncate_end;

/// Render the compact one-line Fleet Ops summary.
pub fn render_fleet_ops_new(
    _app: &AppState,
    frame: &mut Frame,
    layout: &FleetOpsLayout,
    context: &FleetOpsContext,
    palette: &crate::app::state::Palette,
) {
    if layout.rect.width == 0 || layout.rect.height == 0 {
        return;
    }
    if !context.has_data {
        return;
    }

    let summary = truncate_end(&context.summary, layout.rect.width as usize);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ", Style::default().fg(palette.accent)),
            Span::styled(
                summary,
                Style::default()
                    .fg(palette.subtext0)
                    .add_modifier(Modifier::DIM),
            ),
        ])),
        layout.rect,
    );
}
