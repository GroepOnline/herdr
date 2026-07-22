use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

use crate::{
    app::{
        state::{
            AppState, ExperimentSetting, SettingsConfigSnapshot, SettingsFocus, SettingsSection,
            THEME_NAMES,
        },
        App, Mode,
    },
    config::{
        HostCursorModeConfig, NewTerminalCwdConfig, ShellModeConfig, SidebarCollapsedModeConfig,
        ToastClipboardPosition, ToastDelivery, ToastHerdrPosition, UpdateChannelConfig,
    },
    ui::settings::{
        rows::{scrollback_presets, section_rows, spinner_style_for_row, SettingsRowKind},
        SettingsLayout,
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
    SaveMouseCapture(bool),
    SaveCopyOnSelect(bool),
    SaveConfirmClose(bool),
    SavePromptNewTabName(bool),
    SavePromptNewWorkspaceName(bool),
    SaveRedrawOnFocusGained(bool),
    SaveHostCursor(HostCursorModeConfig),
    SaveSidebarCollapsedMode(SidebarCollapsedModeConfig),
    SaveAgentPanelSort(crate::app::state::AgentPanelSort),
    SaveShellMode(ShellModeConfig),
    SaveDefaultShell(String),
    SaveNewTerminalCwd(NewTerminalCwdConfig),
    SaveScrollbackLimitBytes(usize),
    SaveToastDelaySeconds(u64),
    SaveToastHerdrPosition(ToastHerdrPosition),
    SaveClipboardToastEnabled(bool),
    SaveClipboardToastPosition(ToastClipboardPosition),
    SaveUpdateChannel(UpdateChannelConfig),
    SaveVersionCheck(bool),
    SaveManifestCheck(bool),
    SaveResumeAgentsOnRestore(bool),
    SaveManageSshConfig(bool),
    SaveClipboardHistoryEnabled(bool),
    SaveAllowNested(bool),
    SaveKittyGraphics(bool),
    SaveRevealHiddenCursorForCjkIme(bool),
    SaveThemeAutoSwitch(bool),
}

const DEFAULT_SHELL_PRESETS: &[&str] = &["", "bash", "zsh", "fish", "nu"];
const TOAST_DELAY_PRESETS: &[u64] = &[0, 1, 2, 5];

fn cycle_host_cursor(current: HostCursorModeConfig) -> HostCursorModeConfig {
    match current {
        HostCursorModeConfig::Auto => HostCursorModeConfig::Native,
        HostCursorModeConfig::Native => HostCursorModeConfig::Drawn,
        HostCursorModeConfig::Drawn => HostCursorModeConfig::Auto,
    }
}

fn cycle_sidebar_collapsed_mode(current: SidebarCollapsedModeConfig) -> SidebarCollapsedModeConfig {
    match current {
        SidebarCollapsedModeConfig::Compact => SidebarCollapsedModeConfig::Hidden,
        SidebarCollapsedModeConfig::Hidden => SidebarCollapsedModeConfig::Compact,
    }
}

fn cycle_agent_panel_sort(
    current: crate::app::state::AgentPanelSort,
) -> crate::app::state::AgentPanelSort {
    match current {
        crate::app::state::AgentPanelSort::Spaces => crate::app::state::AgentPanelSort::Priority,
        crate::app::state::AgentPanelSort::Priority => crate::app::state::AgentPanelSort::Spaces,
    }
}

fn cycle_shell_mode(current: ShellModeConfig) -> ShellModeConfig {
    match current {
        ShellModeConfig::Auto => ShellModeConfig::Login,
        ShellModeConfig::Login => ShellModeConfig::NonLogin,
        ShellModeConfig::NonLogin => ShellModeConfig::Auto,
    }
}

fn cycle_new_terminal_cwd(current: &NewTerminalCwdConfig) -> NewTerminalCwdConfig {
    match current {
        NewTerminalCwdConfig::Follow => NewTerminalCwdConfig::Home,
        NewTerminalCwdConfig::Home => NewTerminalCwdConfig::Current,
        NewTerminalCwdConfig::Current | NewTerminalCwdConfig::Path(_) => {
            NewTerminalCwdConfig::Follow
        }
    }
}

fn cycle_default_shell(current: &str) -> String {
    let idx = DEFAULT_SHELL_PRESETS
        .iter()
        .position(|shell| *shell == current)
        .unwrap_or(0);
    DEFAULT_SHELL_PRESETS[(idx + 1) % DEFAULT_SHELL_PRESETS.len()].to_string()
}

fn cycle_toast_delay(current: u64) -> u64 {
    let idx = TOAST_DELAY_PRESETS
        .iter()
        .position(|delay| *delay == current)
        .unwrap_or(1);
    TOAST_DELAY_PRESETS[(idx + 1) % TOAST_DELAY_PRESETS.len()]
}

fn cycle_toast_herdr_position(current: ToastHerdrPosition) -> ToastHerdrPosition {
    match current {
        ToastHerdrPosition::TopLeft => ToastHerdrPosition::TopRight,
        ToastHerdrPosition::TopRight => ToastHerdrPosition::BottomRight,
        ToastHerdrPosition::BottomRight => ToastHerdrPosition::BottomLeft,
        ToastHerdrPosition::BottomLeft => ToastHerdrPosition::TopLeft,
    }
}

fn cycle_clipboard_toast_position(current: ToastClipboardPosition) -> ToastClipboardPosition {
    match current {
        ToastClipboardPosition::TopLeft => ToastClipboardPosition::TopCenter,
        ToastClipboardPosition::TopCenter => ToastClipboardPosition::TopRight,
        ToastClipboardPosition::TopRight => ToastClipboardPosition::BottomRight,
        ToastClipboardPosition::BottomRight => ToastClipboardPosition::BottomCenter,
        ToastClipboardPosition::BottomCenter => ToastClipboardPosition::BottomLeft,
        ToastClipboardPosition::BottomLeft => ToastClipboardPosition::TopLeft,
    }
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
        ExperimentSetting::KittyGraphics => Some(SettingsAction::SaveKittyGraphics(
            !ExperimentSetting::KittyGraphics.enabled(state),
        )),
        ExperimentSetting::AllowNested => Some(SettingsAction::SaveAllowNested(
            !ExperimentSetting::AllowNested.enabled(state),
        )),
        ExperimentSetting::RevealHiddenCursorForCjkIme => {
            Some(SettingsAction::SaveRevealHiddenCursorForCjkIme(
                !ExperimentSetting::RevealHiddenCursorForCjkIme.enabled(state),
            ))
        }
    }
}

impl App {
    pub(crate) fn handle_settings_key(&mut self, key: KeyEvent) {
        let previous_section = self.state.settings.section;
        if let Some(action) = update_settings_state(&mut self.state, key) {
            self.apply_settings_action(action);
        }
        if previous_section != SettingsSection::Agents
            && self.state.settings.section == SettingsSection::Agents
        {
            self.refresh_integration_recommendations();
        }
    }

    pub(super) fn apply_settings_action(&mut self, action: SettingsAction) {
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
            SettingsAction::SavePaneHistory(enabled) => self.save_pane_history_persistence(enabled),
            SettingsAction::SaveSwitchAsciiInputSourceInPrefix(enabled) => {
                self.save_switch_ascii_input_source_in_prefix(enabled)
            }
            SettingsAction::SaveSpinnerStyle(style) => self.save_spinner_style(style),
            SettingsAction::ApplyPaneTemplate(template) => self.apply_pane_template(template),
            SettingsAction::InstallRecommendedIntegrations => {
                self.install_recommended_integrations()
            }
            SettingsAction::SaveMouseCapture(enabled) => self.save_mouse_capture(enabled),
            SettingsAction::SaveCopyOnSelect(enabled) => self.save_copy_on_select(enabled),
            SettingsAction::SaveConfirmClose(enabled) => self.save_confirm_close(enabled),
            SettingsAction::SavePromptNewTabName(enabled) => self.save_prompt_new_tab_name(enabled),
            SettingsAction::SavePromptNewWorkspaceName(enabled) => {
                self.save_prompt_new_workspace_name(enabled)
            }
            SettingsAction::SaveRedrawOnFocusGained(enabled) => {
                self.save_redraw_on_focus_gained(enabled)
            }
            SettingsAction::SaveHostCursor(mode) => {
                self.state.settings.config_snapshot.host_cursor = mode;
                self.save_host_cursor(mode);
            }
            SettingsAction::SaveSidebarCollapsedMode(mode) => {
                self.save_sidebar_collapsed_mode(mode)
            }
            SettingsAction::SaveAgentPanelSort(sort) => self.save_agent_panel_sort(sort),
            SettingsAction::SaveShellMode(mode) => self.save_shell_mode(mode),
            SettingsAction::SaveDefaultShell(shell) => self.save_default_shell(&shell),
            SettingsAction::SaveNewTerminalCwd(cwd) => self.save_new_terminal_cwd(cwd),
            SettingsAction::SaveScrollbackLimitBytes(bytes) => {
                self.save_scrollback_limit_bytes(bytes)
            }
            SettingsAction::SaveToastDelaySeconds(seconds) => {
                self.save_toast_delay_seconds(seconds)
            }
            SettingsAction::SaveToastHerdrPosition(position) => {
                self.save_toast_herdr_position(position)
            }
            SettingsAction::SaveClipboardToastEnabled(enabled) => {
                self.save_clipboard_toast_enabled(enabled)
            }
            SettingsAction::SaveClipboardToastPosition(position) => {
                self.save_clipboard_toast_position(position)
            }
            SettingsAction::SaveUpdateChannel(channel) => {
                self.state.settings.config_snapshot.update_channel = channel;
                self.save_update_channel(channel);
            }
            SettingsAction::SaveVersionCheck(enabled) => {
                self.state.settings.config_snapshot.version_check = enabled;
                self.save_version_check(enabled);
            }
            SettingsAction::SaveManifestCheck(enabled) => {
                self.state.settings.config_snapshot.manifest_check = enabled;
                self.save_manifest_check(enabled);
            }
            SettingsAction::SaveResumeAgentsOnRestore(enabled) => {
                self.state.settings.config_snapshot.resume_agents_on_restore = enabled;
                self.save_resume_agents_on_restore(enabled);
            }
            SettingsAction::SaveManageSshConfig(enabled) => {
                self.state.settings.config_snapshot.manage_ssh_config = enabled;
                self.save_manage_ssh_config(enabled);
            }
            SettingsAction::SaveClipboardHistoryEnabled(enabled) => {
                self.state
                    .settings
                    .config_snapshot
                    .clipboard_history_enabled = enabled;
                self.save_clipboard_history_enabled(enabled);
            }
            SettingsAction::SaveAllowNested(enabled) => {
                self.state.settings.config_snapshot.allow_nested = enabled;
                self.save_allow_nested(enabled);
            }
            SettingsAction::SaveKittyGraphics(enabled) => self.save_kitty_graphics(enabled),
            SettingsAction::SaveRevealHiddenCursorForCjkIme(enabled) => {
                self.save_reveal_hidden_cursor_for_cjk_ime(enabled)
            }
            SettingsAction::SaveThemeAutoSwitch(enabled) => {
                self.state.settings.config_snapshot.theme_auto_switch = enabled;
                self.state.theme_runtime.auto_switch = enabled;
                self.save_theme_auto_switch(enabled);
            }
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
    section.next()
}

fn prev_section(section: SettingsSection) -> SettingsSection {
    section.prev()
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
    let rows = section_rows(state, section);
    let row = rows.get(row_index)?;

    match (section, row.kind) {
        (SettingsSection::Appearance, SettingsRowKind::Toggle) => Some(
            SettingsAction::SaveThemeAutoSwitch(!state.settings.config_snapshot.theme_auto_switch),
        ),
        (SettingsSection::Appearance, SettingsRowKind::Spinner) => {
            spinner_style_for_row(state, row).map(SettingsAction::SaveSpinnerStyle)
        }
        (SettingsSection::Layout, SettingsRowKind::Toggle) => match row.payload {
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
        },
        (SettingsSection::Layout, SettingsRowKind::Choice) => match row.payload {
            0 => Some(SettingsAction::SaveSidebarCollapsedMode(
                cycle_sidebar_collapsed_mode(state.sidebar_collapsed_mode),
            )),
            1 => Some(SettingsAction::SaveAgentPanelSort(cycle_agent_panel_sort(
                state.agent_panel_sort,
            ))),
            _ => None,
        },
        (SettingsSection::Layout, SettingsRowKind::Template) => {
            crate::pane_template::PaneTemplateId::ALL
                .get(row.payload)
                .copied()
                .map(SettingsAction::ApplyPaneTemplate)
        }
        (SettingsSection::Input, SettingsRowKind::Toggle) => match row.payload {
            0 => Some(SettingsAction::SaveMouseCapture(!state.mouse_capture)),
            1 => Some(SettingsAction::SaveCopyOnSelect(!state.copy_on_select)),
            2 => Some(SettingsAction::SaveRedrawOnFocusGained(
                !state.redraw_on_focus_gained,
            )),
            3 => Some(SettingsAction::SaveConfirmClose(!state.confirm_close)),
            4 => Some(SettingsAction::SavePromptNewTabName(
                !state.prompt_new_tab_name,
            )),
            5 => Some(SettingsAction::SavePromptNewWorkspaceName(
                !state.prompt_new_workspace_name,
            )),
            _ => None,
        },
        (SettingsSection::Input, SettingsRowKind::Choice) => Some(SettingsAction::SaveHostCursor(
            cycle_host_cursor(state.settings.config_snapshot.host_cursor),
        )),
        (SettingsSection::Terminal, SettingsRowKind::Choice) => match row.payload {
            0 => Some(SettingsAction::SaveDefaultShell(cycle_default_shell(
                &state.default_shell,
            ))),
            1 => Some(SettingsAction::SaveShellMode(cycle_shell_mode(
                state.shell_mode,
            ))),
            2 => Some(SettingsAction::SaveNewTerminalCwd(cycle_new_terminal_cwd(
                &state.new_terminal_cwd,
            ))),
            preset_idx if preset_idx >= 3 => scrollback_presets()
                .get(preset_idx - 3)
                .map(|(bytes, _)| SettingsAction::SaveScrollbackLimitBytes(*bytes)),
            _ => None,
        },
        (SettingsSection::Agents, SettingsRowKind::Integration) => integrations_need_install(state)
            .then_some(SettingsAction::InstallRecommendedIntegrations),
        (SettingsSection::Notifications, SettingsRowKind::Toggle) => {
            Some(SettingsAction::SaveSound(!state.sound_enabled()))
        }
        (SettingsSection::Notifications, SettingsRowKind::Choice) => {
            if row.label == "toast delay" {
                return Some(SettingsAction::SaveToastDelaySeconds(cycle_toast_delay(
                    state.toast_config.delay_seconds,
                )));
            }
            if row.label == "herdr toast position" {
                return Some(SettingsAction::SaveToastHerdrPosition(
                    cycle_toast_herdr_position(state.toast_config.herdr.position),
                ));
            }
            if row.label == "clipboard toast" {
                if !state.toast_config.clipboard.enabled {
                    return Some(SettingsAction::SaveClipboardToastEnabled(true));
                }
                let next = cycle_clipboard_toast_position(state.toast_config.clipboard.position);
                // After a full position cycle back to TopLeft, turn clipboard toasts off.
                if next == ToastClipboardPosition::TopLeft
                    && state.toast_config.clipboard.position == ToastClipboardPosition::BottomLeft
                {
                    return Some(SettingsAction::SaveClipboardToastEnabled(false));
                }
                return Some(SettingsAction::SaveClipboardToastPosition(next));
            }
            let delivery = match row.payload {
                1 => ToastDelivery::Off,
                2 => ToastDelivery::Herdr,
                3 => ToastDelivery::Terminal,
                4 => ToastDelivery::System,
                _ => return None,
            };
            Some(SettingsAction::SaveToastDelivery(delivery))
        }
        (SettingsSection::Agents, SettingsRowKind::Toggle) => {
            Some(SettingsAction::SaveResumeAgentsOnRestore(
                !state.settings.config_snapshot.resume_agents_on_restore,
            ))
        }
        (SettingsSection::Updates, SettingsRowKind::Choice) => match row.payload {
            0 => Some(SettingsAction::SaveUpdateChannel(
                UpdateChannelConfig::Stable,
            )),
            1 => Some(SettingsAction::SaveUpdateChannel(
                UpdateChannelConfig::Preview,
            )),
            _ => None,
        },
        (SettingsSection::Updates, SettingsRowKind::Toggle) => match row.payload {
            0 => Some(SettingsAction::SaveVersionCheck(
                !state.settings.config_snapshot.version_check,
            )),
            1 => Some(SettingsAction::SaveManifestCheck(
                !state.settings.config_snapshot.manifest_check,
            )),
            _ => None,
        },
        (SettingsSection::Advanced, SettingsRowKind::Toggle) => {
            if row.payload >= 100 {
                return match row.payload - 100 {
                    0 => Some(SettingsAction::SaveManageSshConfig(
                        !state.settings.config_snapshot.manage_ssh_config,
                    )),
                    1 => Some(SettingsAction::SaveClipboardHistoryEnabled(
                        !state.settings.config_snapshot.clipboard_history_enabled,
                    )),
                    _ => None,
                };
            }
            experiment_toggle_action(state, row.payload)
        }
        _ => None,
    }
}

pub(super) fn update_settings_state(state: &mut AppState, key: KeyEvent) -> Option<SettingsAction> {
    if matches!(key.code, KeyCode::Char('/')) && key.modifiers.is_empty() {
        state.settings.focus = SettingsFocus::Search;
        state.settings.search.clear();
        return None;
    }

    if state.settings.focus == SettingsFocus::Search {
        return handle_settings_search_key(state, key);
    }

    if state.settings.focus == SettingsFocus::Nav {
        return handle_settings_nav_key(state, key);
    }

    if let KeyCode::Char(ch) = key.code {
        if key.modifiers.is_empty()
            && ch.is_ascii()
            && !matches!(ch, ' ' | '\t' | '\x1b' | '\n' | '\r' | '/')
        {
            state.settings.focus = SettingsFocus::Search;
            state.settings.search.push(ch);
            state.settings.list.selected = 0;
            return None;
        }
    }

    if matches!(key.code, KeyCode::Backspace) && key.modifiers.is_empty() {
        if !state.settings.search.is_empty() {
            state.settings.search.pop();
            state.settings.list.selected = 0;
        }
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
        KeyCode::Left | KeyCode::Char('h') => {
            state.settings.focus = SettingsFocus::Nav;
        }
        KeyCode::Char('[') if state.settings.section == SettingsSection::Appearance => {
            if state.settings.spinner_category > 0 {
                state.settings.spinner_category -= 1;
            }
        }
        KeyCode::Char(']') if state.settings.section == SettingsSection::Appearance => {
            let max = crate::ui::settings::spinner::SPINNER_CATEGORIES
                .len()
                .saturating_sub(1);
            if state.settings.spinner_category < max {
                state.settings.spinner_category += 1;
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

fn handle_settings_search_key(state: &mut AppState, key: KeyEvent) -> Option<SettingsAction> {
    match key.code {
        KeyCode::Esc => {
            state.settings.focus = SettingsFocus::Content;
            state.settings.search.clear();
        }
        KeyCode::Backspace => {
            state.settings.search.pop();
            state.settings.list.selected = 0;
        }
        KeyCode::Enter => state.settings.focus = SettingsFocus::Content,
        KeyCode::Char(ch) if key.modifiers.is_empty() => {
            state.settings.search.push(ch);
            state.settings.list.selected = 0;
        }
        _ => {}
    }
    None
}

fn handle_settings_nav_key(state: &mut AppState, key: KeyEvent) -> Option<SettingsAction> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.settings.section = prev_section(state.settings.section);
            state.settings.list.selected =
                default_selection_for_section(state, state.settings.section);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.settings.section = next_section(state.settings.section);
            state.settings.list.selected =
                default_selection_for_section(state, state.settings.section);
        }
        KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') | KeyCode::Tab => {
            state.settings.focus = SettingsFocus::Content;
        }
        KeyCode::BackTab => {
            state.settings.section = prev_section(state.settings.section);
            state.settings.list.selected =
                default_selection_for_section(state, state.settings.section);
        }
        _ => {
            if let Some(super::modal::ModalAction::Close) =
                super::modal::modal_action_from_key(&key, super::modal::SETTINGS_ACTIONS)
            {
                cancel_settings(state);
            }
        }
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
    state.settings.config_snapshot = SettingsConfigSnapshot::load();
    state.settings.section = section;
    state.settings.search.clear();
    state.settings.focus = SettingsFocus::Content;
    state.settings.spinner_category = 0;
    state.settings.content_scroll = 0;
    state.settings.list.selected = default_selection_for_section(state, section);
    state.mode = Mode::Settings;
}

impl AppState {
    fn settings_layout(&self) -> Option<SettingsLayout> {
        SettingsLayout::compute(self.screen_rect(), self)
    }

    pub(super) fn handle_settings_mouse(&mut self, mouse: MouseEvent) -> Option<SettingsAction> {
        let layout = self.settings_layout()?;
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if layout.search_index_at(mouse.column, mouse.row) {
                    self.settings.focus = SettingsFocus::Search;
                    return None;
                }

                if let Some(nav_idx) = layout.nav_index_at(mouse.column, mouse.row) {
                    let section = SettingsSection::ALL[nav_idx];
                    self.settings.section = section;
                    self.settings.list.selected = default_selection_for_section(self, section);
                    self.settings.focus = SettingsFocus::Nav;
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
                    self.settings.focus = SettingsFocus::Content;
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
    fn settings_nav_cycle_forward_and_back() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Appearance);
        state.settings.focus = SettingsFocus::Nav;

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Layout);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Up, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Appearance);
    }

    #[test]
    fn settings_search_focus_and_clear() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings(&mut state);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()),
        );
        assert_eq!(state.settings.focus, SettingsFocus::Search);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::empty()),
        );
        assert_eq!(state.settings.search, "m");

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.focus, SettingsFocus::Content);
        assert!(state.settings.search.is_empty());
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
    fn terminal_choice_payloads_map_to_distinct_actions() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Terminal);
        let rows = section_rows(&state, SettingsSection::Terminal);

        let shell_mode_idx = rows
            .iter()
            .position(|row| row.label == "shell mode")
            .expect("shell mode row");
        let cwd_idx = rows
            .iter()
            .position(|row| row.label == "new pane cwd")
            .expect("cwd row");
        let scrollback_idx = rows
            .iter()
            .position(|row| row.label.starts_with("scrollback"))
            .expect("scrollback row");

        state.settings.list.selected = shell_mode_idx;
        assert!(matches!(
            activate_row(&state, shell_mode_idx),
            Some(SettingsAction::SaveShellMode(_))
        ));
        state.settings.list.selected = cwd_idx;
        assert!(matches!(
            activate_row(&state, cwd_idx),
            Some(SettingsAction::SaveNewTerminalCwd(_))
        ));
        state.settings.list.selected = scrollback_idx;
        assert!(matches!(
            activate_row(&state, scrollback_idx),
            Some(SettingsAction::SaveScrollbackLimitBytes(_))
        ));
    }

    #[test]
    fn agents_enter_toggles_resume_when_no_install_needed() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Agents);

        let enter_action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );
        assert_eq!(
            enter_action,
            Some(SettingsAction::SaveResumeAgentsOnRestore(
                !state.settings.config_snapshot.resume_agents_on_restore
            ))
        );
    }

    #[test]
    fn settings_hover_does_not_change_selection() {
        let mut app = app_for_mouse_test();
        open_settings(&mut app.state);
        app.state.settings.list.select(0);

        let area = app.state.settings_layout().expect("layout").content;
        app.handle_mouse(mouse(MouseEventKind::Moved, area.x + 2, area.y + 2));

        assert_eq!(app.state.settings.list.selected, 0);
    }

    #[test]
    fn settings_mouse_click_toggles_pane_history() {
        let mut app = app_for_mouse_test();
        app.state.pane_history_persistence = false;
        open_settings_at(&mut app.state, SettingsSection::Advanced);

        let area = app.state.settings_layout().expect("layout").content;
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
        assert_eq!(layout.nav_index_at(rect.x + 2, rect.y), Some(agents_idx));
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
