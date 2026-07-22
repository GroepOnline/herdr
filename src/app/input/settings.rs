use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::{
    app::{
        state::{AppState, ExperimentSetting, SettingsSection, THEME_NAMES},
        App, Mode,
    },
    config::ToastDelivery,
    ui::settings::{
        rows::{section_rows, spinner_style_for_row, SettingsRowKind},
        SettingsLayout, SETTINGS_POPUP_WIDTH,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
// The shared `Save` verb is semantic: these actions persist settings.
#[allow(clippy::enum_variant_names)]
pub(super) enum SettingsAction {
    SaveTheme(String),
    SaveSound(bool),
    SaveToastDelivery(ToastDelivery),
    SaveAgentBorderLabels(bool),
    SavePaneBorders(bool),
    SavePaneGaps(bool),
    SaveHideTabBarWhenSingleTab(bool),
    SavePaneHistory(bool),
    SaveSwitchAsciiInputSourceInPrefix(bool),
    SaveSpinnerStyle(crate::config::SpinnerStyle),
    ApplyPaneTemplate(crate::pane_template::PaneTemplateId),
    InstallRecommendedIntegrations,
}

fn experiment_toggle_action(state: &AppState, idx: usize) -> Option<SettingsAction> {
    match ExperimentSetting::ALL.get(idx).copied()? {
        ExperimentSetting::PaneHistory => Some(SettingsAction::SavePaneHistory(
            !ExperimentSetting::PaneHistory.enabled(state),
        )),
        ExperimentSetting::SwitchAsciiInputSourceInPrefix => {
            Some(SettingsAction::SaveSwitchAsciiInputSourceInPrefix(
                !ExperimentSetting::SwitchAsciiInputSourceInPrefix.enabled(state),
            ))
        }
    }
}

impl App {
    pub(crate) fn handle_settings_key(&mut self, key: KeyEvent) {
        let previous_section = self.state.settings.section;
        if let Some(action) = update_settings_state(&mut self.state, key) {
            match action {
                SettingsAction::SaveTheme(name) => self.save_theme(&name),
                SettingsAction::SaveSound(enabled) => self.save_sound(enabled),
                SettingsAction::SaveToastDelivery(delivery) => self.save_toast_delivery(delivery),
                SettingsAction::SaveAgentBorderLabels(enabled) => {
                    self.save_agent_border_labels(enabled)
                }
                SettingsAction::SavePaneBorders(enabled) => self.save_pane_borders(enabled),
                SettingsAction::SavePaneGaps(enabled) => self.save_pane_gaps(enabled),
                SettingsAction::SaveHideTabBarWhenSingleTab(enabled) => {
                    self.save_hide_tab_bar_when_single_tab(enabled)
                }
                SettingsAction::SavePaneHistory(enabled) => {
                    self.save_pane_history_persistence(enabled)
                }
                SettingsAction::SaveSwitchAsciiInputSourceInPrefix(enabled) => {
                    self.save_switch_ascii_input_source_in_prefix(enabled)
                }
                SettingsAction::SaveSpinnerStyle(style) => self.save_spinner_style(style),
                SettingsAction::ApplyPaneTemplate(template) => self.apply_pane_template(template),
                SettingsAction::InstallRecommendedIntegrations => {
                    self.install_recommended_integrations()
                }
            }
        }
        if previous_section != SettingsSection::Agents
            && self.state.settings.section == SettingsSection::Agents
        {
            self.refresh_integration_recommendations();
        }
    }
}

fn normalize_theme_name(name: &str) -> String {
    name.to_lowercase().replace([' ', '_'], "-")
}

fn current_theme_index(theme_name: &str) -> usize {
    let normalized = normalize_theme_name(theme_name);
    THEME_NAMES
        .iter()
        .position(|name| normalize_theme_name(name) == normalized)
        .unwrap_or(0)
}

fn preview_selected_theme(state: &mut AppState) {
    use crate::app::state::Palette;

    let rows = section_rows(state, SettingsSection::Appearance);
    let Some(row) = rows.get(state.settings.list.selected) else {
        return;
    };
    if row.kind != SettingsRowKind::Theme {
        return;
    }
    let name = THEME_NAMES[row.payload];
    if let Some(mut palette) = Palette::from_name(name) {
        if let Some(custom) = &state.theme_runtime.custom {
            palette = palette.with_overrides(custom);
        }
        if let Some(accent) = &state.theme_runtime.legacy_accent {
            palette.accent = crate::config::parse_color(accent);
        }
        state.palette = palette;
        state.theme_name = name.to_string();
    }
}

fn cancel_settings(state: &mut AppState) {
    if let Some(palette) = state.settings.original_palette.take() {
        state.palette = palette;
    }
    if let Some(theme_name) = state.settings.original_theme.take() {
        state.theme_name = theme_name;
    }
    super::modal::leave_modal(state);
}

fn integrations_need_install(state: &AppState) -> bool {
    state
        .integration_recommendations
        .iter()
        .any(crate::integration::IntegrationRecommendation::needs_install)
}

fn apply_settings(state: &mut AppState) -> Option<SettingsAction> {
    match state.settings.section {
        SettingsSection::Appearance => {
            let theme_name = state.theme_name.clone();
            state.settings.original_palette = None;
            state.settings.original_theme = None;
            super::modal::leave_modal(state);
            Some(SettingsAction::SaveTheme(theme_name))
        }
        SettingsSection::Agents if integrations_need_install(state) => {
            Some(SettingsAction::InstallRecommendedIntegrations)
        }
        SettingsSection::Agents => None,
        _ => {
            super::modal::leave_modal(state);
            None
        }
    }
}

fn section_item_count(state: &AppState) -> usize {
    section_rows(state, state.settings.section).len()
}

fn next_section(section: SettingsSection) -> SettingsSection {
    let idx = SettingsSection::ALL
        .iter()
        .position(|item| *item == section)
        .unwrap_or(0);
    SettingsSection::ALL[(idx + 1) % SettingsSection::ALL.len()]
}

fn prev_section(section: SettingsSection) -> SettingsSection {
    let idx = SettingsSection::ALL
        .iter()
        .position(|item| *item == section)
        .unwrap_or(0);
    let len = SettingsSection::ALL.len();
    SettingsSection::ALL[(idx + len - 1) % len]
}

fn default_selection_for_section(state: &AppState, section: SettingsSection) -> usize {
    match section {
        SettingsSection::Appearance => {
            let theme_idx = current_theme_index(&state.theme_name);
            section_rows(state, section)
                .iter()
                .position(|row| row.kind == SettingsRowKind::Theme && row.payload == theme_idx)
                .unwrap_or(0)
        }
        SettingsSection::Notifications => section_rows(state, section)
            .iter()
            .position(|row| row.label == "sound alerts")
            .unwrap_or(0),
        _ => 0,
    }
}

fn activate_row(state: &AppState, row_index: usize) -> Option<SettingsAction> {
    let section = state.settings.section;
    let row = section_rows(state, section).get(row_index)?;

    match (section, row.kind) {
        (SettingsSection::Appearance, SettingsRowKind::Spinner) => {
            spinner_style_for_row(state, row).map(SettingsAction::SaveSpinnerStyle)
        }
        (SettingsSection::Layout, SettingsRowKind::Toggle) => match row.payload {
            0 => Some(SettingsAction::SavePaneBorders(!state.pane_borders_enabled())),
            1 => Some(SettingsAction::SavePaneGaps(!state.pane_gaps_enabled())),
            2 => Some(SettingsAction::SaveAgentBorderLabels(
                !state.agent_border_labels_enabled(),
            )),
            3 => Some(SettingsAction::SaveHideTabBarWhenSingleTab(
                !state.hide_tab_bar_when_single_tab_enabled(),
            )),
            _ => None,
        },
        (SettingsSection::Layout, SettingsRowKind::Template) => {
            crate::pane_template::PaneTemplateId::ALL
                .get(row.payload)
                .copied()
                .map(SettingsAction::ApplyPaneTemplate)
        }
        (SettingsSection::Notifications, SettingsRowKind::Toggle) => {
            Some(SettingsAction::SaveSound(!state.sound_enabled()))
        }
        (SettingsSection::Notifications, SettingsRowKind::Choice) => {
            let delivery = match row.payload {
                1 => ToastDelivery::Off,
                2 => ToastDelivery::Herdr,
                3 => ToastDelivery::Terminal,
                4 => ToastDelivery::System,
                _ => return None,
            };
            Some(SettingsAction::SaveToastDelivery(delivery))
        }
        (SettingsSection::Advanced, SettingsRowKind::Toggle) => {
            if row.payload == 100 {
                return None;
            }
            experiment_toggle_action(state, row.payload)
        }
        _ => None,
    }
}

pub(super) fn update_settings_state(state: &mut AppState, key: KeyEvent) -> Option<SettingsAction> {
    if matches!(key.code, KeyCode::Char('/'))
        && key.modifiers.is_empty()
    {
        state.settings.search.clear();
        return None;
    }

    if let KeyCode::Char(ch) = key.code {
        if !key.modifiers.is_empty() || ch == ' ' || ch == '\n' || ch == '\r' {
            // fall through
        } else if ch.is_ascii() && !matches!(ch, '\t' | '\x1b') {
            state.settings.search.push(ch);
            state.settings.list.selected = 0;
            return None;
        }
    }

    if matches!(key.code, KeyCode::Backspace) && key.modifiers.is_empty() {
        state.settings.search.pop();
        state.settings.list.selected = 0;
        return None;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.settings.list.move_prev();
            if state.settings.section == SettingsSection::Appearance {
                preview_selected_theme(state);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.settings.list.move_next(section_item_count(state));
            if state.settings.section == SettingsSection::Appearance {
                preview_selected_theme(state);
            }
        }
        KeyCode::Tab => {
            let next = next_section(state.settings.section);
            state.settings.section = next;
            state.settings.list.selected = default_selection_for_section(state, next);
            state.settings.content_scroll = 0;
        }
        KeyCode::BackTab => {
            let prev = prev_section(state.settings.section);
            state.settings.section = prev;
            state.settings.list.selected = default_selection_for_section(state, prev);
            state.settings.content_scroll = 0;
        }
        KeyCode::Left | KeyCode::Char('h')
            if state.settings.section == SettingsSection::Appearance =>
        {
            if state.settings.spinner_category > 0 {
                state.settings.spinner_category -= 1;
            }
        }
        KeyCode::Right | KeyCode::Char('l')
            if state.settings.section == SettingsSection::Appearance =>
        {
            let max = crate::ui::settings::spinner::SPINNER_CATEGORIES.len().saturating_sub(1);
            if state.settings.spinner_category < max {
                state.settings.spinner_category += 1;
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            return activate_row(state, state.settings.list.selected);
        }
        _ => match super::modal::modal_action_from_key(&key, super::modal::SETTINGS_ACTIONS) {
            Some(super::modal::ModalAction::Apply) => return apply_settings(state),
            Some(super::modal::ModalAction::Close) => cancel_settings(state),
            _ => {}
        },
    }

    None
}

pub(crate) fn open_settings(state: &mut AppState) {
    open_settings_at(state, SettingsSection::Appearance);
}

pub(crate) fn open_settings_at(state: &mut AppState, section: SettingsSection) {
    state.integration_install_messages.clear();
    state.settings.original_palette = Some(state.palette.clone());
    state.settings.original_theme = Some(state.theme_name.clone());
    state.settings.section = section;
    state.settings.search.clear();
    state.settings.spinner_category = 0;
    state.settings.content_scroll = 0;
    state.settings.list.selected = default_selection_for_section(state, section);
    state.mode = Mode::Settings;
}

impl AppState {
    fn settings_popup_rect(&self) -> Rect {
        crate::ui::centered_popup_rect(
            self.screen_rect(),
            SETTINGS_POPUP_WIDTH,
            crate::ui::settings_popup_height(self),
        )
        .unwrap_or_default()
    }

    fn settings_layout(&self) -> Option<SettingsLayout> {
        SettingsLayout::compute(self.screen_rect(), self)
    }

    pub(crate) fn settings_content_rect(&self) -> Rect {
        self.settings_layout()
            .map(|layout| layout.content)
            .unwrap_or_default()
    }

    pub(crate) fn settings_nav_index_at(&self, col: u16, row: u16) -> Option<usize> {
        self.settings_layout()?.nav_index_at(col, row)
    }

    pub(crate) fn settings_content_index_at(&self, col: u16, row: u16) -> Option<usize> {
        self.settings_layout()?.content_index_at(self, col, row)
    }

    pub(super) fn handle_settings_mouse(&mut self, mouse: MouseEvent) -> Option<SettingsAction> {
        let layout = self.settings_layout()?;
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(nav_idx) = layout.nav_index_at(mouse.column, mouse.row) {
                    let section = SettingsSection::ALL[nav_idx];
                    self.settings.section = section;
                    self.settings.list.selected = default_selection_for_section(self, section);
                    return None;
                }

                if let Some(category) =
                    layout.spinner_category_index_at(self, mouse.column, mouse.row)
                {
                    self.settings.spinner_category = category;
                    return None;
                }

                if let Some(idx) = layout.content_index_at(self, mouse.column, mouse.row) {
                    self.settings.list.select(idx);
                    if self.settings.section == SettingsSection::Appearance {
                        preview_selected_theme(self);
                    }
                    return activate_row(self, idx);
                }

                let show_primary = crate::ui::settings_show_primary_action(self);
                let (apply, close) =
                    crate::ui::settings_button_rects(&layout, self.settings.section, show_primary);
                let mut buttons = vec![(close, super::modal::ModalAction::Close)];
                if let Some(apply) = apply {
                    buttons.insert(0, (apply, super::modal::ModalAction::Apply));
                }
                match super::modal::modal_action_from_buttons(mouse.column, mouse.row, &buttons) {
                    Some(super::modal::ModalAction::Apply) => apply_settings(self),
                    Some(super::modal::ModalAction::Close) => {
                        cancel_settings(self);
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEventKind};

    use super::super::{app_for_mouse_test, mouse, state_with_workspaces};
    use super::*;

    #[test]
    fn settings_cancel_restores_previewed_theme_from_other_sections() {
        let mut state = state_with_workspaces(&["test"]);
        let original_palette = state.palette.clone();
        let original_theme = state.theme_name.clone();

        open_settings(&mut state);
        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
        );
        assert_ne!(state.theme_name, original_theme);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Layout);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
        );

        assert_eq!(state.mode, Mode::Terminal);
        assert_eq!(state.theme_name, original_theme);
        assert_eq!(state.palette.accent, original_palette.accent);
        assert_eq!(state.palette.panel_bg, original_palette.panel_bg);
    }

    #[test]
    fn settings_notifications_toggle_returns_save_action() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Notifications);
        let sound_row = section_rows(&state, SettingsSection::Notifications)
            .iter()
            .position(|row| row.label == "sound alerts")
            .expect("sound row");
        state.settings.list.selected = sound_row;

        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );

        assert_eq!(action, Some(SettingsAction::SaveSound(true)));
        assert!(!state.sound.enabled);
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn settings_advanced_toggles_pane_history() {
        let mut state = state_with_workspaces(&["test"]);
        state.pane_history_persistence = false;
        open_settings_at(&mut state, SettingsSection::Advanced);

        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );

        assert_eq!(action, Some(SettingsAction::SavePaneHistory(true)));
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn settings_tab_advances_sections() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Appearance);
        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Layout);
    }

    #[test]
    fn agents_enter_does_nothing_when_nothing_needs_install() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Agents);

        let enter_action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );
        assert_eq!(enter_action, None);
    }

    #[test]
    fn settings_hover_does_not_change_selection() {
        let mut app = app_for_mouse_test();
        open_settings(&mut app.state);
        app.state.settings.list.select(0);

        let area = app.state.settings_content_rect();
        app.handle_mouse(mouse(MouseEventKind::Moved, area.x + 2, area.y + 2));

        assert_eq!(app.state.settings.list.selected, 0);
    }

    #[test]
    fn settings_mouse_click_toggles_pane_history() {
        let mut app = app_for_mouse_test();
        app.state.pane_history_persistence = false;
        open_settings_at(&mut app.state, SettingsSection::Advanced);

        let area = app.state.settings_content_rect();
        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            area.x + 2,
            area.y + 3,
        ));

        assert_eq!(action, Some(SettingsAction::SavePaneHistory(true)));
        assert_eq!(app.state.settings.list.selected, 0);
    }

    #[test]
    fn integration_update_badge_only_tracks_outdated_recommendations() {
        let mut state = state_with_workspaces(&["test"]);
        state.integration_recommendations = vec![integration_recommendation(
            crate::integration::IntegrationStatusKind::Outdated,
            true,
        )];
        assert!(state.integration_updates_available());
        assert!(state.settings_section_has_badge(SettingsSection::Agents));
    }

    #[test]
    fn settings_nav_hit_area_matches_layout() {
        let mut state = state_with_workspaces(&["test"]);
        state.integration_recommendations = vec![integration_recommendation(
            crate::integration::IntegrationStatusKind::Outdated,
            true,
        )];
        open_settings(&mut state);

        let layout = state.settings_layout().expect("layout");
        let agents_idx = SettingsSection::ALL
            .iter()
            .position(|section| *section == SettingsSection::Agents)
            .expect("agents section");
        let rect = layout.nav_item_rect(agents_idx).expect("nav rect");
        assert_eq!(state.settings_nav_index_at(rect.x + 2, rect.y), Some(agents_idx));
    }

    fn integration_recommendation(
        state: crate::integration::IntegrationStatusKind,
        available: bool,
    ) -> crate::integration::IntegrationRecommendation {
        crate::integration::IntegrationRecommendation {
            target: crate::api::schema::IntegrationTarget::Claude,
            label: "claude",
            command: "claude",
            available,
            path: std::path::PathBuf::from("/tmp/herdr-test-integration"),
            state,
        }
    }
}
