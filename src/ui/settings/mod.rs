mod layout;
pub(crate) mod rows;
mod sections;
pub(crate) mod spinner;

pub(crate) use layout::{
    settings_button_rects, settings_show_primary_action, SettingsLayout,
};

use ratatui::{layout::Rect, Frame};

use crate::app::AppState;

use self::sections::{
    render_settings_content, render_settings_footer, render_settings_header, render_settings_nav,
};

pub(super) fn render_settings_overlay(app: &AppState, frame: &mut Frame, area: Rect) {
    let p = &app.palette;
    let Some(layout) = SettingsLayout::compute(area, app) else {
        return;
    };

    super::dim_background(frame, area);

    let Some(_inner) =
        super::widgets::render_panel_shell(frame, layout.popup, p.accent, p.panel_bg)
    else {
        return;
    };

    render_settings_header(app, frame, &layout);
    render_settings_nav(app, frame, &layout);
    render_settings_content(app, frame, &layout);
    render_settings_footer(app, frame, &layout);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{state::SettingsSection, Mode};
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn settings_overlay_renders_left_nav_sections() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Advanced;

        let mut terminal =
            Terminal::new(TestBackend::new(120, 40)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 120, 40)))
            .expect("settings overlay should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("appearance"));
        assert!(rendered.contains("advanced"));
        assert!(rendered.contains("customize herdr"));
    }

    #[test]
    fn advanced_section_renders_experiment_rows() {
        let mut app = AppState::test_new();
        app.pane_history_persistence = true;
        app.settings.section = SettingsSection::Advanced;
        app.settings.list.selected = 0;
        app.mode = Mode::Settings;

        let mut terminal =
            Terminal::new(TestBackend::new(120, 40)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 120, 40)))
            .expect("settings overlay should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("pane screen history [✓]"));
    }
}
