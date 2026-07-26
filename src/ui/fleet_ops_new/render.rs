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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::Palette;
    use crate::ui::shell::FleetOpsLayout;
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};

    fn test_palette() -> Palette {
        Palette::default()
    }

    #[test]
    fn render_noop_when_rect_zero() {
        let app = crate::app::state::AppState::test_new();
        let layout = FleetOpsLayout {
            rect: Rect::default(),
            expanded: false,
            summary: Rect::default(),
            detail: None,
        };
        let ctx = FleetOpsContext {
            summary: "data".into(),
            expanded: "data".into(),
            has_data: true,
        };
        let palette = test_palette();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|frame| render_fleet_ops_new(&app, frame, &layout, &ctx, &palette))
            .unwrap();
        // No panic = pass.
    }

    #[test]
    fn render_noop_when_no_data() {
        let app = crate::app::state::AppState::test_new();
        let layout = FleetOpsLayout {
            rect: Rect::new(0, 39, 80, 1),
            expanded: false,
            summary: Rect::new(0, 39, 80, 1),
            detail: None,
        };
        let ctx = FleetOpsContext {
            summary: String::new(),
            expanded: String::new(),
            has_data: false,
        };
        let palette = test_palette();
        let mut terminal = Terminal::new(TestBackend::new(80, 40)).unwrap();
        terminal
            .draw(|frame| render_fleet_ops_new(&app, frame, &layout, &ctx, &palette))
            .unwrap();
        // No panic, no output = pass.
    }

    #[test]
    fn render_draws_summary_text() {
        let app = crate::app::state::AppState::test_new();
        let layout = FleetOpsLayout {
            rect: Rect::new(0, 39, 80, 1),
            expanded: false,
            summary: Rect::new(0, 39, 80, 1),
            detail: None,
        };
        let ctx = FleetOpsContext {
            summary: "test-summary · more".into(),
            expanded: "full".into(),
            has_data: true,
        };
        let palette = test_palette();
        let mut terminal = Terminal::new(TestBackend::new(80, 40)).unwrap();
        terminal
            .draw(|frame| render_fleet_ops_new(&app, frame, &layout, &ctx, &palette))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let line: String = (0..80)
            .map(|x| buffer[(x, 39)].symbol().to_string())
            .collect();
        assert!(line.contains("test-summary"), "expected fleet ops text: {line}");
    }
}
