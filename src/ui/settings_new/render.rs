//! Renderer for the rebuilt settings overlay.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::state::{AppState, Palette};
use crate::ui::settings_new::layout::SettingsLayout;
use crate::ui::settings_new::model::SettingsCategory;

/// Render the settings overlay.
pub fn render_settings_new(
    _app: &AppState,
    frame: &mut Frame,
    layout: &SettingsLayout,
    _category: SettingsCategory,
    palette: &Palette,
) {
    if layout.overlay.width == 0 || layout.overlay.height == 0 {
        return;
    }

    frame.render_widget(Clear, layout.overlay);
    frame.render_widget(
        Block::default()
            .title("Settings")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.surface_dim)),
        layout.overlay,
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            "Appearance · Keybinds · Integrations",
            Style::default()
                .fg(palette.subtext0)
                .add_modifier(Modifier::DIM),
        )])),
        layout.footer,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::Palette;
    use crate::ui::settings_new::layout::SettingsLayout;
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};

    fn test_palette() -> Palette {
        Palette::catppuccin()
    }

    fn test_layout() -> SettingsLayout {
        SettingsLayout {
            overlay: Rect::new(10, 3, 80, 30),
            category_rail: Rect::new(10, 3, 16, 30),
            content: Rect::new(26, 3, 64, 29),
            footer: Rect::new(26, 32, 64, 1),
            rows: Vec::new(),
        }
    }

    #[test]
    fn render_noop_when_overlay_zero() {
        let app = AppState::test_new();
        let layout = SettingsLayout {
            overlay: Rect::default(),
            category_rail: Rect::default(),
            content: Rect::default(),
            footer: Rect::default(),
            rows: Vec::new(),
        };
        let palette = test_palette();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|frame| {
                render_settings_new(&app, frame, &layout, SettingsCategory::Appearance, &palette)
            })
            .unwrap();
        // No panic = pass.
    }

    #[test]
    fn render_draws_title_and_border() {
        let app = AppState::test_new();
        let layout = test_layout();
        let palette = test_palette();
        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal
            .draw(|frame| {
                render_settings_new(&app, frame, &layout, SettingsCategory::Appearance, &palette)
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        // Check the title "Settings" is rendered somewhere in the overlay.
        let title_line: String = (0..80)
            .map(|x| buffer[(10 + x, 3)].symbol().to_string())
            .collect();
        assert!(
            title_line.contains("Settings"),
            "expected Settings title: {title_line}"
        );
    }

    #[test]
    fn render_draws_footer_text() {
        let app = AppState::test_new();
        let layout = test_layout();
        let palette = test_palette();
        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal
            .draw(|frame| {
                render_settings_new(&app, frame, &layout, SettingsCategory::Appearance, &palette)
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let footer_line: String = (0..64)
            .map(|x| buffer[(26 + x, 32)].symbol().to_string())
            .collect();
        assert!(
            footer_line.contains("Appearance"),
            "expected footer text: {footer_line}"
        );
    }

    #[test]
    fn render_with_keybinds_category() {
        let app = AppState::test_new();
        let layout = test_layout();
        let palette = test_palette();
        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal
            .draw(|frame| {
                render_settings_new(&app, frame, &layout, SettingsCategory::Keybinds, &palette)
            })
            .unwrap();
        // Should not panic — category is currently unused in rendering but passed through.
    }
}
