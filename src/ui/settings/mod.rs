pub(crate) mod catalog;
mod blocks;
mod content;
mod layout;
pub(crate) mod rows;
mod sections;
pub(crate) mod spinner;
#[allow(dead_code)]
pub(crate) mod widgets;

pub(crate) use catalog::SettingsAction;

pub(crate) use layout::{settings_button_rects, settings_show_tertiary_action, SettingsLayout};

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
    use crate::ui::settings::catalog::SettingsItemId;
    use crate::ui::settings::rows::{
        scroll_list_row_indices, section_rows, SettingsRowKind,
    };
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn settings_overlay_renders_left_nav_sections() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Advanced;

        let mut terminal =
            Terminal::new(TestBackend::new(160, 50)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 160, 50)))
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
        assert!(rendered.contains("CONFIG MENU"));
        assert!(rendered.contains("advanced settings"));
        assert!(rendered.contains("save"));
        assert!(rendered.contains("cancel"));
    }

    #[test]
    fn advanced_section_renders_experiment_checklist() {
        let mut app = AppState::test_new();
        app.pane_history_persistence = true;
        app.settings.section = SettingsSection::Advanced;
        app.settings.list.selected = 0;
        app.mode = Mode::Settings;

        let mut terminal =
            Terminal::new(TestBackend::new(160, 50)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 160, 50)))
            .expect("settings overlay should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("experiments"));
        assert!(rendered.contains("[✓] pane screen history"));
    }

    #[test]
    fn input_section_renders_block_headers() {
        let mut app = AppState::test_new();
        app.settings.section = SettingsSection::Input;
        app.mode = Mode::Settings;

        let mut terminal =
            Terminal::new(TestBackend::new(160, 50)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 160, 50)))
            .expect("settings overlay should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("pointer & clipboard"));
        assert!(rendered.contains("focus & prompts"));
        assert!(rendered.contains("host cursor"));
        assert!(rendered.contains("mouse capture"));
        assert!(rendered.contains("auto"));
        assert!(rendered.contains("native"));
        assert!(rendered.contains("drawn"));
    }

    #[test]
    fn terminal_section_renders_all_shell_and_cwd_chips() {
        let mut app = AppState::test_new();
        app.settings.section = SettingsSection::Terminal;
        app.mode = Mode::Settings;

        let mut terminal =
            Terminal::new(TestBackend::new(160, 50)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 160, 50)))
            .expect("settings overlay should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("shell"));
        assert!(rendered.contains("new pane cwd"));
        assert!(rendered.contains("login"));
        assert!(rendered.contains("non_login"));
        assert!(rendered.contains("follow"));
        assert!(rendered.contains("home"));
        assert!(rendered.contains("current"));
    }

    #[test]
    fn appearance_section_renders_theme_chip_labels() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Appearance;

        let mut terminal =
            Terminal::new(TestBackend::new(160, 50)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 160, 50)))
            .expect("settings overlay should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("appearance"));
        assert!(rendered.contains("catppuccin"));
        assert!(rendered.contains("dracula"));
        assert!(rendered.contains("spinner"));
        assert!(rendered.contains("auto-switch theme with host"));
        assert_eq!(
            rendered.matches("auto-switch theme with host").count(),
            1,
            "auto-switch should render once in the appearance showcase"
        );
    }

    #[test]
    fn appearance_scroll_list_excludes_theme_showcase_rows() {
        let app = AppState::test_new();
        let indices = scroll_list_row_indices(&app, SettingsSection::Appearance);
        let rows = section_rows(&app, SettingsSection::Appearance);
        for idx in indices {
            assert_ne!(rows[idx].id, SettingsItemId::ThemeAutoSwitch);
            assert_ne!(rows[idx].kind, SettingsRowKind::Theme);
        }
    }

    #[test]
    fn layout_section_renders_chrome_checklist_labels() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Layout;

        let mut terminal =
            Terminal::new(TestBackend::new(160, 50)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 160, 50)))
            .expect("settings overlay should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("chrome"));
        assert!(rendered.contains("pane borders"));
        assert!(rendered.contains("templates"));
    }

    #[test]
    fn notifications_section_renders_sound_and_toast_blocks() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Notifications;

        let mut terminal =
            Terminal::new(TestBackend::new(160, 50)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 160, 50)))
            .expect("settings overlay should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("sound"));
        assert!(rendered.contains("toasts"));
        assert!(rendered.contains("herdr"));
    }
}
