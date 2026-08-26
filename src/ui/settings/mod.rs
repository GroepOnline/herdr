pub(crate) mod catalog;
mod layout;
pub(crate) mod plugin_detail;
pub(crate) mod rows;
mod sections;
pub(crate) mod spinner;

pub(crate) use catalog::SettingsAction;

pub(crate) use layout::{settings_button_rects, settings_show_primary_action, SettingsLayout};

#[cfg(test)]
pub(crate) use layout::{settings_primary_button_label, settings_show_secondary_action};

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
    fn settings_overlay_renders_left_nav_tabs() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::System;

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

        assert!(rendered.contains("theme"));
        assert!(rendered.contains("system"));
        assert!(rendered.contains("customize herdr"));
    }

    #[test]
    fn system_section_renders_experiment_rows() {
        let mut app = AppState::test_new();
        app.pane_history_persistence = true;
        app.settings.section = SettingsSection::System;
        let pane_history_row = super::rows::section_rows(&app, SettingsSection::System)
            .iter()
            .position(|row| {
                matches!(
                    row.id,
                    crate::ui::settings::catalog::SettingsItemId::Experiment(
                        crate::app::state::ExperimentSetting::PaneHistory
                    )
                )
            })
            .expect("pane history row");
        app.settings.list.selected = pane_history_row;
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

        assert!(rendered.contains("[✓] pane screen history"));
    }

    #[test]
    fn remote_and_graphics_owns_remote_graphics_and_nesting_rows() {
        let app = AppState::test_new();
        let labels = super::rows::section_rows(&app, SettingsSection::RemoteGraphics)
            .iter()
            .map(|row| row.label.clone())
            .collect::<Vec<_>>();

        for expected in [
            "kitty graphics protocol",
            "allow nested herdr sessions",
            "manage ssh config",
            "clipboard history",
        ] {
            assert!(labels.iter().any(|label| label == expected), "{expected}");
        }
    }

    #[test]
    fn keys_and_pointer_owns_input_experiments() {
        let app = AppState::test_new();
        let labels = super::rows::section_rows(&app, SettingsSection::Keys)
            .iter()
            .map(|row| row.label.clone())
            .collect::<Vec<_>>();

        assert!(labels
            .iter()
            .any(|label| label == "switch to ascii input source in prefix (macOS/Windows)"));
        assert!(labels
            .iter()
            .any(|label| label == "reveal hidden cursor for cjk ime"));
    }

    #[test]
    fn system_keeps_only_system_experiment_rows() {
        let app = AppState::test_new();
        let labels = super::rows::section_rows(&app, SettingsSection::System)
            .iter()
            .map(|row| row.label.clone())
            .collect::<Vec<_>>();

        assert!(labels.iter().any(|label| label == "pane screen history"));
        for moved in [
            "kitty graphics protocol",
            "allow nested herdr sessions",
            "switch to ascii input source in prefix (macOS/Windows)",
            "reveal hidden cursor for cjk ime",
            "manage ssh config",
            "clipboard history",
        ] {
            assert!(!labels.iter().any(|label| label == moved), "{moved}");
        }
    }
}
