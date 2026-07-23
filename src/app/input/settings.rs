use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::{
    app::{
        state::{AppState, ExperimentSetting, SettingsSection, THEME_NAMES, UI_SPINNER_OFFSET},
        App, Mode,
    },
    config::ToastDelivery,
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
    SaveFleetOpsBar(bool),
    SaveSpinnerStyle(crate::config::SpinnerStyle),
    ApplyPaneTemplate(crate::pane_template::PaneTemplateId),
    InstallRecommendedIntegrations,
}

/// Map an Experiments row index to the toggle action that flips it.
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
                SettingsAction::SaveFleetOpsBar(enabled) => self.save_fleet_ops_bar(enabled),
                SettingsAction::SaveSpinnerStyle(style) => self.save_spinner_style(style),
                SettingsAction::ApplyPaneTemplate(template) => self.apply_pane_template(template),
                SettingsAction::InstallRecommendedIntegrations => {
                    self.install_recommended_integrations()
                }
            }
        }
        if previous_section != SettingsSection::Integrations
            && self.state.settings.section == SettingsSection::Integrations
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

fn toast_delivery_for_index(idx: usize) -> ToastDelivery {
    match idx {
        0 => ToastDelivery::Off,
        1 => ToastDelivery::Herdr,
        2 => ToastDelivery::Terminal,
        _ => ToastDelivery::System,
    }
}

fn preview_selected_theme(state: &mut AppState) {
    use crate::app::state::Palette;

    let name = THEME_NAMES[state.settings.list.selected];
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
        SettingsSection::Theme => {
            let theme_name = state.theme_name.clone();
            state.settings.original_palette = None;
            state.settings.original_theme = None;
            super::modal::leave_modal(state);
            Some(SettingsAction::SaveTheme(theme_name))
        }
        SettingsSection::Integrations if integrations_need_install(state) => {
            Some(SettingsAction::InstallRecommendedIntegrations)
        }
        SettingsSection::Integrations => None,
        SettingsSection::Fleet => Some(SettingsAction::SaveFleetOpsBar(
            !state.fleet_ops_bar_enabled(),
        )),
        SettingsSection::Plugins => None,
        SettingsSection::Ui
        | SettingsSection::Sound
        | SettingsSection::System
        | SettingsSection::Templates => {
            super::modal::leave_modal(state);
            None
        }
    }
}

/// Total items in the Sound tab: 1 sound toggle + 4 toast delivery options.
const SOUND_TOTAL: usize = 5;

fn ui_total_items() -> usize {
    crate::app::state::UI_SPINNER_OFFSET + crate::config::SpinnerStyle::ALL.len()
}

fn plugins_total_items(state: &AppState) -> usize {
    state.installed_plugins.len() + crate::fleet::FLEET_OPS_PLUGIN_IDS.len()
}

fn default_settings_index(state: &AppState, section: SettingsSection) -> usize {
    match section {
        SettingsSection::Theme => current_theme_index(&state.theme_name),
        SettingsSection::Ui => 0,
        SettingsSection::Sound => usize::from(!state.sound_enabled()),
        SettingsSection::System => 0,
        SettingsSection::Fleet => 0,
        SettingsSection::Plugins => 0,
        SettingsSection::Templates => 0,
        SettingsSection::Integrations => 0,
    }
}

fn select_settings_section(state: &mut AppState, section: SettingsSection) {
    state.settings.section = section;
    state.settings.list.selected = default_settings_index(state, section);
}

fn adjacent_settings_section(current: SettingsSection, direction: isize) -> SettingsSection {
    let len = SettingsSection::ALL.len() as isize;
    let idx = SettingsSection::ALL
        .iter()
        .position(|section| *section == current)
        .unwrap_or(0) as isize;
    let next = (idx + direction).rem_euclid(len) as usize;
    SettingsSection::ALL[next]
}

fn select_next_settings_section(state: &mut AppState) {
    select_settings_section(state, adjacent_settings_section(state.settings.section, 1));
}

fn select_previous_settings_section(state: &mut AppState) {
    select_settings_section(state, adjacent_settings_section(state.settings.section, -1));
}

pub(super) fn update_settings_state(state: &mut AppState, key: KeyEvent) -> Option<SettingsAction> {
    match state.settings.section {
        // ── Theme tab ──────────────────────────────────────────────
        SettingsSection::Theme => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let prev = state.settings.list.selected;
                state.settings.list.move_prev();
                if state.settings.list.selected != prev {
                    preview_selected_theme(state);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let prev = state.settings.list.selected;
                state.settings.list.move_next(THEME_NAMES.len());
                if state.settings.list.selected != prev {
                    preview_selected_theme(state);
                }
            }
            KeyCode::Tab => {
                select_next_settings_section(state);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                select_next_settings_section(state);
            }
            KeyCode::BackTab => {
                select_previous_settings_section(state);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                select_previous_settings_section(state);
            }
            _ => match super::modal::modal_action_from_key(&key, super::modal::SETTINGS_ACTIONS) {
                Some(super::modal::ModalAction::Apply) => return apply_settings(state),
                Some(super::modal::ModalAction::Close) => cancel_settings(state),
                _ => {}
            },
        },

        // ── Ui tab: toggles + spinner grid ─────────────────────────
        SettingsSection::Ui => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let s = state.settings.list.selected;
                if s >= UI_SPINNER_OFFSET + 2 {
                    state.settings.list.selected = s - 2;
                } else if s >= UI_SPINNER_OFFSET {
                    // First spinner row: jump to last toggle.
                    state.settings.list.selected = UI_SPINNER_OFFSET - 1;
                } else {
                    state.settings.list.selected = s.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = ui_total_items() - 1;
                let s = state.settings.list.selected;
                if s < UI_SPINNER_OFFSET {
                    state.settings.list.selected = (s + 1).min(max);
                } else {
                    state.settings.list.selected = (s + 2).min(max);
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                state.settings.list.selected = state.settings.list.selected.saturating_sub(1);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let max = ui_total_items() - 1;
                state.settings.list.selected = (state.settings.list.selected + 1).min(max);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let s = state.settings.list.selected;
                if s < UI_SPINNER_OFFSET {
                    return match s {
                        0 => Some(SettingsAction::SavePaneBorders(
                            !state.pane_borders_enabled(),
                        )),
                        1 => Some(SettingsAction::SavePaneGaps(!state.pane_gaps_enabled())),
                        2 => Some(SettingsAction::SaveAgentBorderLabels(
                            !state.agent_border_labels_enabled(),
                        )),
                        3 => Some(SettingsAction::SaveHideTabBarWhenSingleTab(
                            !state.hide_tab_bar_when_single_tab_enabled(),
                        )),
                        _ => None,
                    };
                }
                let spinner_idx = s - UI_SPINNER_OFFSET;
                return crate::config::SpinnerStyle::ALL
                    .get(spinner_idx)
                    .copied()
                    .map(SettingsAction::SaveSpinnerStyle);
            }
            KeyCode::Tab => {
                select_next_settings_section(state);
            }
            KeyCode::BackTab => {
                select_previous_settings_section(state);
            }
            _ => {
                if let Some(super::modal::ModalAction::Close) =
                    super::modal::modal_action_from_key(&key, super::modal::SETTINGS_ACTIONS)
                {
                    cancel_settings(state);
                }
            }
        },

        // ── Sound tab: sound toggle + toast delivery ───────────────
        SettingsSection::Sound => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.settings.list.move_prev();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.settings.list.move_next(SOUND_TOTAL);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let s = state.settings.list.selected;
                if s == 0 {
                    return Some(SettingsAction::SaveSound(!state.sound_enabled()));
                }
                let delivery = toast_delivery_for_index(s - 1);
                return Some(SettingsAction::SaveToastDelivery(delivery));
            }
            KeyCode::Tab => {
                select_next_settings_section(state);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                select_next_settings_section(state);
            }
            KeyCode::BackTab => {
                select_previous_settings_section(state);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                select_previous_settings_section(state);
            }
            _ => {
                if let Some(super::modal::ModalAction::Close) =
                    super::modal::modal_action_from_key(&key, super::modal::SETTINGS_ACTIONS)
                {
                    cancel_settings(state);
                }
            }
        },

        // ── System tab: experiments ────────────────────────────────
        SettingsSection::System => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.settings.list.move_prev();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.settings.list.move_next(ExperimentSetting::ALL.len());
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                return experiment_toggle_action(state, state.settings.list.selected);
            }
            KeyCode::Tab => {
                select_next_settings_section(state);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                select_next_settings_section(state);
            }
            KeyCode::BackTab => {
                select_previous_settings_section(state);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                select_previous_settings_section(state);
            }
            _ => {
                if let Some(super::modal::ModalAction::Close) =
                    super::modal::modal_action_from_key(&key, super::modal::SETTINGS_ACTIONS)
                {
                    cancel_settings(state);
                }
            }
        },

        // ── Fleet tab: CHEF personal context ───────────────────────
        SettingsSection::Fleet => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.settings.list.move_prev();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.settings.list.move_next(1);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                return Some(SettingsAction::SaveFleetOpsBar(
                    !state.fleet_ops_bar_enabled(),
                ));
            }
            KeyCode::Tab => {
                select_next_settings_section(state);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                select_next_settings_section(state);
            }
            KeyCode::BackTab => {
                select_previous_settings_section(state);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                select_previous_settings_section(state);
            }
            _ => match super::modal::modal_action_from_key(&key, super::modal::SETTINGS_ACTIONS) {
                Some(super::modal::ModalAction::Apply) => {
                    return Some(SettingsAction::SaveFleetOpsBar(
                        !state.fleet_ops_bar_enabled(),
                    ));
                }
                Some(super::modal::ModalAction::Close) => cancel_settings(state),
                _ => {}
            },
        },

        // ── Plugins tab: installed + browse catalog ────────────────
        SettingsSection::Plugins => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.settings.list.move_prev();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.settings.list.move_next(plugins_total_items(state));
            }
            KeyCode::Tab => {
                select_next_settings_section(state);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                select_next_settings_section(state);
            }
            KeyCode::BackTab => {
                select_previous_settings_section(state);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                select_previous_settings_section(state);
            }
            _ => {
                if let Some(super::modal::ModalAction::Close) =
                    super::modal::modal_action_from_key(&key, super::modal::SETTINGS_ACTIONS)
                {
                    cancel_settings(state);
                }
            }
        },

        // ── Templates tab: pane layout templates ───────────────────
        SettingsSection::Templates => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.settings.list.move_prev();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state
                    .settings
                    .list
                    .move_next(crate::pane_template::PaneTemplateId::ALL.len());
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                return crate::pane_template::PaneTemplateId::ALL
                    .get(state.settings.list.selected)
                    .copied()
                    .map(SettingsAction::ApplyPaneTemplate);
            }
            KeyCode::Tab => {
                select_next_settings_section(state);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                select_next_settings_section(state);
            }
            KeyCode::BackTab => {
                select_previous_settings_section(state);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                select_previous_settings_section(state);
            }
            _ => {
                if let Some(super::modal::ModalAction::Close) =
                    super::modal::modal_action_from_key(&key, super::modal::SETTINGS_ACTIONS)
                {
                    cancel_settings(state);
                }
            }
        },

        // ── Integrations tab ──────────────────────────────────────
        SettingsSection::Integrations => match key.code {
            KeyCode::Enter | KeyCode::Char(' ') if integrations_need_install(state) => {
                return Some(SettingsAction::InstallRecommendedIntegrations);
            }
            KeyCode::Tab => {
                select_next_settings_section(state);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                select_next_settings_section(state);
            }
            KeyCode::BackTab => {
                select_previous_settings_section(state);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                select_previous_settings_section(state);
            }
            _ => match super::modal::modal_action_from_key(&key, super::modal::SETTINGS_ACTIONS) {
                Some(super::modal::ModalAction::Apply) => return apply_settings(state),
                Some(super::modal::ModalAction::Close) => cancel_settings(state),
                _ => {}
            },
        },
    }

    None
}

pub(crate) fn open_settings(state: &mut AppState) {
    open_settings_at(state, SettingsSection::Theme);
}

pub(crate) fn open_settings_at(state: &mut AppState, section: SettingsSection) {
    state.integration_install_messages.clear();
    state.settings.original_palette = Some(state.palette.clone());
    state.settings.original_theme = Some(state.theme_name.clone());
    state.settings.section = section;
    state.settings.list.selected = default_settings_index(state, section);
    state.mode = Mode::Settings;
}

impl AppState {
    fn settings_popup_rect(&self) -> Rect {
        crate::ui::centered_popup_rect(
            self.screen_rect(),
            crate::ui::SETTINGS_POPUP_WIDTH,
            crate::ui::settings_popup_height(self),
        )
        .unwrap_or_default()
    }

    fn settings_inner_rect(&self) -> Rect {
        let popup = self.settings_popup_rect();
        Rect::new(
            popup.x + 1,
            popup.y + 1,
            popup.width.saturating_sub(2),
            popup.height.saturating_sub(2),
        )
    }

    fn settings_tab_at(&self, col: u16, row: u16) -> Option<SettingsSection> {
        let inner = self.settings_inner_rect();
        let tab_y = inner.y + 1;
        if row != tab_y {
            return None;
        }
        let mut x = inner.x;
        for section in SettingsSection::ALL {
            let badge_width = if self.settings_section_has_badge(*section) {
                2
            } else {
                0
            };
            let width = section.label().len() as u16 + 2 + badge_width;
            if col >= x && col < x + width {
                return Some(*section);
            }
            x += width + 1;
        }
        None
    }

    pub(crate) fn settings_content_rect(&self) -> Rect {
        let inner = self.settings_inner_rect();
        crate::ui::modal_stack_areas(inner, 3, 2, 0, 1).content
    }

    fn settings_list_index_at(&self, col: u16, row: u16) -> Option<usize> {
        let area = self.settings_content_rect();
        if row < area.y || row >= area.y + area.height || col < area.x || col >= area.x + area.width
        {
            return None;
        }

        let list_area = crate::ui::settings_list_area(area);

        match self.settings.section {
            SettingsSection::Ui => {
                crate::ui::settings_ui_index_at(list_area, col, row, self.settings.list.selected)
            }
            SettingsSection::Theme => {
                let max_visible = area.height as usize;
                let scroll = if self.settings.list.selected >= max_visible {
                    self.settings.list.selected - max_visible + 1
                } else {
                    0
                };
                let idx = scroll + (row - area.y) as usize;
                (idx < THEME_NAMES.len()).then_some(idx)
            }
            SettingsSection::Sound => crate::ui::settings_sound_index_at(list_area, col, row),
            SettingsSection::System => {
                if row >= list_area.y && row < list_area.y + ExperimentSetting::ALL.len() as u16 {
                    Some((row - list_area.y) as usize)
                } else {
                    None
                }
            }
            SettingsSection::Fleet => {
                if row == list_area.y {
                    Some(0)
                } else {
                    None
                }
            }
            SettingsSection::Plugins => {
                let installed_count = self.installed_plugins.len();
                if row >= list_area.y && row < list_area.y + installed_count as u16 {
                    return Some((row - list_area.y) as usize);
                }
                let browse_y = list_area.y + installed_count.max(1) as u16 + 1;
                if row >= browse_y
                    && row < browse_y + crate::fleet::FLEET_OPS_PLUGIN_IDS.len() as u16
                {
                    Some(installed_count + (row - browse_y) as usize)
                } else {
                    None
                }
            }
            SettingsSection::Templates => {
                crate::ui::settings_template_index_at(list_area, col, row)
            }
            SettingsSection::Integrations => {
                if row >= list_area.y
                    && !self.integration_recommendations.is_empty()
                    && row < list_area.y + self.integration_recommendations.len() as u16
                {
                    Some((row - list_area.y) as usize)
                } else {
                    None
                }
            }
        }
    }

    pub(super) fn handle_settings_mouse(&mut self, mouse: MouseEvent) -> Option<SettingsAction> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(section) = self.settings_tab_at(mouse.column, mouse.row) {
                    select_settings_section(self, section);
                    return None;
                }
                if let Some(idx) = self.settings_list_index_at(mouse.column, mouse.row) {
                    self.settings.list.select(idx);
                    return match self.settings.section {
                        SettingsSection::Theme => {
                            preview_selected_theme(self);
                            None
                        }
                        SettingsSection::Ui => {
                            if idx < UI_SPINNER_OFFSET {
                                match idx {
                                    0 => Some(SettingsAction::SavePaneBorders(
                                        !self.pane_borders_enabled(),
                                    )),
                                    1 => Some(SettingsAction::SavePaneGaps(
                                        !self.pane_gaps_enabled(),
                                    )),
                                    2 => Some(SettingsAction::SaveAgentBorderLabels(
                                        !self.agent_border_labels_enabled(),
                                    )),
                                    3 => Some(SettingsAction::SaveHideTabBarWhenSingleTab(
                                        !self.hide_tab_bar_when_single_tab_enabled(),
                                    )),
                                    _ => None,
                                }
                            } else {
                                crate::config::SpinnerStyle::ALL
                                    .get(idx - UI_SPINNER_OFFSET)
                                    .copied()
                                    .map(SettingsAction::SaveSpinnerStyle)
                            }
                        }
                        SettingsSection::Sound => {
                            if idx == 0 {
                                Some(SettingsAction::SaveSound(!self.sound_enabled()))
                            } else {
                                let delivery = toast_delivery_for_index(idx - 1);
                                Some(SettingsAction::SaveToastDelivery(delivery))
                            }
                        }
                        SettingsSection::System => experiment_toggle_action(self, idx),
                        SettingsSection::Fleet => {
                            if idx == 0 {
                                Some(SettingsAction::SaveFleetOpsBar(
                                    !self.fleet_ops_bar_enabled(),
                                ))
                            } else {
                                None
                            }
                        }
                        SettingsSection::Plugins => None,
                        SettingsSection::Templates => crate::pane_template::PaneTemplateId::ALL
                            .get(idx)
                            .copied()
                            .map(SettingsAction::ApplyPaneTemplate),
                        SettingsSection::Integrations => {
                            // Selecting a row highlights it; Install remains the primary action.
                            None
                        }
                    };
                }

                let inner = self.settings_inner_rect();
                let show_primary = crate::ui::settings_show_primary_action(self);
                let (apply, close) =
                    crate::ui::settings_button_rects(inner, self.settings.section, show_primary);
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
                    _ => {
                        // Inside chrome: no-op. Outside popup: dismiss (navigator pattern).
                        let popup = self.settings_popup_rect();
                        let inside = mouse.column >= popup.x
                            && mouse.column < popup.x.saturating_add(popup.width)
                            && mouse.row >= popup.y
                            && mouse.row < popup.y.saturating_add(popup.height);
                        if !inside {
                            cancel_settings(self);
                        }
                        None
                    }
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
        assert_eq!(
            state.settings.section,
            crate::app::state::SettingsSection::Ui
        );

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
    fn settings_sound_toggle_returns_save_action() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings(&mut state);
        state.settings.section = crate::app::state::SettingsSection::Sound;
        state.settings.list.selected = 0;

        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );

        assert_eq!(action, Some(SettingsAction::SaveSound(true)));
        assert!(!state.sound.enabled);
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn settings_system_toggles_pane_history() {
        let mut state = state_with_workspaces(&["test"]);
        state.pane_history_persistence = false;
        open_settings_at(&mut state, SettingsSection::System);

        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );

        assert_eq!(action, Some(SettingsAction::SavePaneHistory(true)));
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn settings_system_down_then_toggle_switches_ascii_input_source() {
        let mut state = state_with_workspaces(&["test"]);
        state.switch_ascii_input_source_in_prefix = false;
        open_settings_at(&mut state, SettingsSection::System);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.list.selected, 1);

        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );

        assert_eq!(
            action,
            Some(SettingsAction::SaveSwitchAsciiInputSourceInPrefix(true))
        );
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn settings_fleet_enter_toggles_fleet_ops_bar() {
        let mut state = state_with_workspaces(&["test"]);
        state.fleet_ops_bar = false;
        open_settings_at(&mut state, SettingsSection::Fleet);

        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );

        assert_eq!(action, Some(SettingsAction::SaveFleetOpsBar(true)));
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn settings_tab_ui_to_sound() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Ui);
        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Sound);
    }

    #[test]
    fn settings_tab_sound_to_system() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Sound);
        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::System);
    }

    #[test]
    fn settings_tab_system_to_fleet() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::System);
        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Fleet);
    }

    #[test]
    fn settings_tab_fleet_to_plugins() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Fleet);
        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Plugins);
    }

    #[test]
    fn settings_tab_plugins_to_templates() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Plugins);
        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Templates);
    }

    #[test]
    fn settings_backtab_templates_to_plugins() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Templates);
        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::BackTab, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Plugins);
    }

    #[test]
    fn integrations_enter_does_nothing_when_nothing_needs_install() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Integrations);

        let enter_action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );
        assert_eq!(enter_action, None);

        let space_action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()),
        );
        assert_eq!(space_action, None);
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
    fn settings_click_inside_chrome_does_not_cancel() {
        let mut app = app_for_mouse_test();
        let original_theme = app.state.theme_name.clone();
        open_settings(&mut app.state);
        crate::ui::compute_view(&mut app.state, ratatui::layout::Rect::new(0, 0, 106, 28));

        let popup = app.state.settings_popup_rect();
        // Title row inside the border — not a tab, list row, or button.
        let chrome_col = popup.x + 2;
        let chrome_row = popup.y + 1;
        assert!(app.state.settings_tab_at(chrome_col, chrome_row).is_none());
        assert!(app
            .state
            .settings_list_index_at(chrome_col, chrome_row)
            .is_none());

        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            chrome_col,
            chrome_row,
        ));

        assert_eq!(action, None);
        assert_eq!(app.state.mode, Mode::Settings);
        assert_eq!(app.state.theme_name, original_theme);
    }

    #[test]
    fn settings_click_outside_popup_cancels() {
        let mut app = app_for_mouse_test();
        open_settings(&mut app.state);
        crate::ui::compute_view(&mut app.state, ratatui::layout::Rect::new(0, 0, 106, 28));
        let popup = app.state.settings_popup_rect();

        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            0,
            0,
        ));

        assert_eq!(action, None);
        assert_ne!(app.state.mode, Mode::Settings);
        assert!(popup.width > 0);
    }

    #[test]
    fn settings_mouse_click_toggles_pane_history() {
        let mut app = app_for_mouse_test();
        app.state.pane_history_persistence = false;
        open_settings_at(&mut app.state, SettingsSection::System);

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
    fn settings_mouse_click_toggles_switch_ascii_input_source_row() {
        let mut app = app_for_mouse_test();
        app.state.switch_ascii_input_source_in_prefix = false;
        open_settings_at(&mut app.state, SettingsSection::System);

        // Navigate down to the second experiment.
        update_settings_state(
            &mut app.state,
            KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
        );
        let action = update_settings_state(
            &mut app.state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );

        assert_eq!(
            action,
            Some(SettingsAction::SaveSwitchAsciiInputSourceInPrefix(true))
        );
    }

    #[test]
    fn settings_plugins_list_index_at_installed_and_browse_rows() {
        let mut app = app_for_mouse_test();
        app.state.installed_plugins.insert(
            "com.chefgroep.linear-context".into(),
            installed_plugin("com.chefgroep.linear-context", true),
        );
        open_settings_at(&mut app.state, SettingsSection::Plugins);
        crate::ui::compute_view(&mut app.state, ratatui::layout::Rect::new(0, 0, 106, 30));

        let list_area = crate::ui::settings_list_area(app.state.settings_content_rect());
        assert_eq!(
            app.state
                .settings_list_index_at(list_area.x + 1, list_area.y),
            Some(0)
        );

        let browse_y = list_area.y + app.state.installed_plugins.len() as u16 + 1;
        assert_eq!(
            app.state.settings_list_index_at(list_area.x + 1, browse_y),
            Some(app.state.installed_plugins.len())
        );
    }

    #[test]
    fn settings_integrations_click_selects_row() {
        let mut app = app_for_mouse_test();
        app.state.integration_recommendations = vec![
            integration_recommendation(crate::integration::IntegrationStatusKind::Current, true),
            integration_recommendation(crate::integration::IntegrationStatusKind::Outdated, true),
        ];
        open_settings_at(&mut app.state, SettingsSection::Integrations);
        crate::ui::compute_view(&mut app.state, ratatui::layout::Rect::new(0, 0, 106, 30));

        let list_area = crate::ui::settings_list_area(app.state.settings_content_rect());
        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            list_area.x + 1,
            list_area.y + 1,
        ));

        assert_eq!(action, None);
        assert_eq!(app.state.settings.list.selected, 1);
    }

    #[test]
    fn integration_update_badge_only_tracks_outdated_recommendations() {
        let mut state = state_with_workspaces(&["test"]);
        state.integration_recommendations = vec![integration_recommendation(
            crate::integration::IntegrationStatusKind::NotInstalled,
            true,
        )];
        assert!(!state.integration_updates_available());

        state.integration_recommendations = vec![integration_recommendation(
            crate::integration::IntegrationStatusKind::NotInstalled,
            false,
        )];
        assert!(!state.integration_updates_available());

        state.integration_recommendations = vec![integration_recommendation(
            crate::integration::IntegrationStatusKind::Current,
            true,
        )];
        assert!(!state.integration_updates_available());

        state.integration_recommendations = vec![integration_recommendation(
            crate::integration::IntegrationStatusKind::Outdated,
            true,
        )];
        assert!(state.integration_updates_available());
    }

    #[test]
    fn settings_tab_hit_area_includes_integration_update_badge() {
        let mut state = state_with_workspaces(&["test"]);
        state.integration_recommendations = vec![integration_recommendation(
            crate::integration::IntegrationStatusKind::Outdated,
            true,
        )];
        open_settings(&mut state);

        let inner = state.settings_inner_rect();
        let tab_y = inner.y + 1;
        let integrations_idx = SettingsSection::ALL
            .iter()
            .position(|section| *section == SettingsSection::Integrations)
            .expect("integrations section should be present");
        let integrations_x = inner.x
            + SettingsSection::ALL[..integrations_idx]
                .iter()
                .map(|section| {
                    let badge_width = if state.settings_section_has_badge(*section) {
                        2
                    } else {
                        0
                    };
                    section.label().len() as u16 + 3 + badge_width
                })
                .sum::<u16>();
        let dotted_width = SettingsSection::Integrations.label().len() as u16 + 4;

        assert_eq!(
            state.settings_tab_at(integrations_x + dotted_width - 1, tab_y),
            Some(SettingsSection::Integrations)
        );
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

    fn installed_plugin(id: &str, enabled: bool) -> crate::api::schema::InstalledPluginInfo {
        crate::api::schema::InstalledPluginInfo {
            plugin_id: id.into(),
            name: "Test Plugin".into(),
            version: "0.1.0".into(),
            min_herdr_version: crate::build_info::BASE_VERSION.into(),
            description: None,
            manifest_path: format!("/tmp/{id}/herdr-plugin.toml"),
            plugin_root: format!("/tmp/{id}"),
            enabled,
            platforms: None,
            build: vec![],
            startup: vec![],
            actions: vec![],
            events: vec![],
            panes: vec![],
            link_handlers: vec![],
            source: Default::default(),
            warnings: vec![],
        }
    }
}
