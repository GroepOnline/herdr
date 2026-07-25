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
            Style::default().fg(palette.subtext0).add_modifier(Modifier::DIM),
        )])),
        layout.footer,
    );
}
