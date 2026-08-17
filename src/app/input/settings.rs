use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

use crate::{
    app::{
        state::{AppState, SettingsConfigSnapshot, SettingsFocus, SettingsSection, THEME_NAMES},
        App, Mode,
    },
    ui::settings::{
        catalog::{activate_item, theme_index, SettingsItemId},
        plugin_detail,
        rows::{section_rows, SettingsRowKind},
        SettingsLayout,
    },
};

pub(super) use crate::ui::settings::SettingsAction;

impl App {
    pub(crate) fn handle_settings_key(&mut self, key: KeyEvent) {
        let previous_section = self.state.settings.section;
        if let Some(action) = update_settings_state(&mut self.state, key) {
            self.apply_settings_action(action);
        }
        if previous_section != SettingsSection::Integrations
            && self.state.settings.section == SettingsSection::Integrations
        {
            self.refresh_integration_recommendations();
            self.reload_plugins_for_settings();
        }
    }

    pub(super) fn apply_settings_action(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::SaveTheme(name) => self.save_theme(&name),
            SettingsAction::SaveSidebarWidth(width) => self.save_sidebar_width(width),
            SettingsAction::SaveSidebarCollapsedMode(mode) => {
                self.save_sidebar_collapsed_mode(mode)
            }
            SettingsAction::SaveAgentPanelSort(sort) => self.save_agent_panel_sort(sort),
            SettingsAction::SaveSidebarAgentRowGap(gap) => self.save_sidebar_agent_row_gap(gap),
            SettingsAction::SaveSidebarSpaceRowGap(gap) => self.save_sidebar_space_row_gap(gap),
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
            SettingsAction::SaveStatusIndicators(style) => self.save_status_indicators(style),
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
            SettingsAction::SaveHostCursor(mode) => self.save_host_cursor(mode),
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
            SettingsAction::SaveUpdateChannel(channel) => self.save_update_channel(channel),
            SettingsAction::SaveVersionCheck(enabled) => self.save_version_check(enabled),
            SettingsAction::SaveManifestCheck(enabled) => self.save_manifest_check(enabled),
            SettingsAction::SaveResumeAgentsOnRestore(enabled) => {
                self.save_resume_agents_on_restore(enabled)
            }
            SettingsAction::SaveManageSshConfig(enabled) => self.save_manage_ssh_config(enabled),
            SettingsAction::SaveClipboardHistoryEnabled(enabled) => {
                self.save_clipboard_history_enabled(enabled)
            }
            SettingsAction::SaveAllowNested(enabled) => self.save_allow_nested(enabled),
            SettingsAction::SaveKittyGraphics(enabled) => self.save_kitty_graphics(enabled),
            SettingsAction::SaveRevealHiddenCursorForCjkIme(enabled) => {
                self.save_reveal_hidden_cursor_for_cjk_ime(enabled)
            }
            SettingsAction::SaveThemeAutoSwitch(enabled) => self.save_theme_auto_switch(enabled),
            SettingsAction::SaveFleetOpsBar(enabled) => self.save_fleet_ops_bar(enabled),
            SettingsAction::TogglePluginEnabled { plugin_id, enabled } => {
                if let Err(err) = self.settings_set_plugin_enabled(&plugin_id, enabled) {
                    self.state.plugin_install_messages = vec![err];
                }
            }
            SettingsAction::InstallCatalogPlugin { source } => {
                self.settings_install_catalog_plugin(&source);
            }
            SettingsAction::RefreshInstalledPlugins => {
                self.settings_refresh_installed_plugins();
            }
            SettingsAction::InvokePluginAction {
                plugin_id,
                action_id,
            } => {
                if let Err(err) = self.invoke_plugin_action(&plugin_id, &action_id, "settings") {
                    self.state.plugin_install_messages = vec![err];
                }
            }
        }
        self.state.settings.config_snapshot = SettingsConfigSnapshot::load();
        self.state.theme_runtime.auto_switch =
            self.state.settings.config_snapshot.theme_auto_switch;
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

    let rows = section_rows(state, SettingsSection::Theme);
    let Some(row) = rows.get(state.settings.list.selected) else {
        return;
    };
    if row.kind != SettingsRowKind::Theme {
        return;
    }
    let Some(idx) = theme_index(row.id) else {
        return;
    };
    let Some(name) = THEME_NAMES.get(idx) else {
        return;
    };
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
        SettingsSection::Integrations => Some(SettingsAction::RefreshInstalledPlugins),
        _ => {
            super::modal::leave_modal(state);
            None
        }
    }
}

fn next_section(section: SettingsSection) -> SettingsSection {
    section.next()
}

fn prev_section(section: SettingsSection) -> SettingsSection {
    section.prev()
}

fn first_selectable(state: &AppState, section: SettingsSection) -> usize {
    crate::ui::settings::rows::first_selectable_index(state, section)
}

fn default_selection_for_section(state: &AppState, section: SettingsSection) -> usize {
    let fallback = || first_selectable(state, section);
    match section {
        SettingsSection::Theme => {
            let theme_idx = current_theme_index(&state.theme_name);
            section_rows(state, section)
                .iter()
                .position(|row| theme_index(row.id) == Some(theme_idx))
                .unwrap_or_else(fallback)
        }
        SettingsSection::Sound => section_rows(state, section)
            .iter()
            .position(|row| row.label == "sound alerts")
            .unwrap_or_else(fallback),
        _ => first_selectable(state, section),
    }
}

fn move_selection_next(state: &mut AppState) {
    let count = section_rows(state, state.settings.section).len();
    if count > 0 {
        state.settings.list.selected = (state.settings.list.selected + 1).min(count - 1);
    }
}

fn move_selection_prev(state: &mut AppState) {
    state.settings.list.selected = state.settings.list.selected.saturating_sub(1);
}

fn toggle_collapse_at(state: &mut AppState, row_index: usize) -> bool {
    let rows = section_rows(state, state.settings.section);
    let Some(row) = rows.get(row_index) else {
        return false;
    };
    if row.kind != SettingsRowKind::Header {
        return false;
    }
    if state.settings.collapsed_groups.remove(&row.label) {
        // Group was collapsed; it is now expanded.
    } else {
        state.settings.collapsed_groups.insert(row.label.clone());
    }
    true
}

fn collapse_all_groups(state: &mut AppState) {
    let labels: Vec<String> = section_rows(state, state.settings.section)
        .iter()
        .filter(|row| row.kind == SettingsRowKind::Header)
        .map(|row| row.label.clone())
        .collect();
    state.settings.collapsed_groups.extend(labels);
    state.settings.list.selected = first_selectable(state, state.settings.section);
}

fn expand_all_groups(state: &mut AppState) {
    state.settings.collapsed_groups.clear();
}

fn activate_row(state: &AppState, row_index: usize) -> Option<SettingsAction> {
    let rows = section_rows(state, state.settings.section);
    let row = rows.get(row_index)?;
    activate_item(state, row.id)
}

/// If the row at `row_index` is an installed-plugin row, return that plugin's
/// index in the plugin-id-sorted install list.
fn installed_plugin_index_at(state: &AppState, row_index: usize) -> Option<usize> {
    let rows = section_rows(state, state.settings.section);
    match rows.get(row_index)?.id {
        SettingsItemId::InstalledPlugin { index } => Some(index),
        _ => None,
    }
}

fn open_plugin_detail(state: &mut AppState, index: usize) {
    state.settings.plugin_detail = Some(index);
    state.settings.plugin_detail_cursor = 0;
    state.settings.plugin_detail_scroll = 0;
}

fn close_plugin_detail(state: &mut AppState) {
    state.settings.plugin_detail = None;
    state.settings.plugin_detail_cursor = 0;
    state.settings.plugin_detail_scroll = 0;
}

/// Activate the selected row in the plugin detail view: toggle enable for the
/// first row, otherwise invoke the selected action.
fn activate_plugin_detail_row(state: &AppState) -> Option<SettingsAction> {
    let plugin = plugin_detail::detail_plugin(state)?;
    let cursor = state.settings.plugin_detail_cursor;
    if cursor == 0 {
        return Some(SettingsAction::TogglePluginEnabled {
            plugin_id: plugin.plugin_id.clone(),
            enabled: !plugin.enabled,
        });
    }
    let action = plugin.actions.get(cursor - 1)?;
    Some(SettingsAction::InvokePluginAction {
        plugin_id: plugin.plugin_id.clone(),
        action_id: action.id.clone(),
    })
}

fn handle_plugin_detail_key(state: &mut AppState, key: KeyEvent) -> Option<SettingsAction> {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace | KeyCode::Left | KeyCode::Char('q') => {
            close_plugin_detail(state);
            None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.settings.plugin_detail_cursor =
                state.settings.plugin_detail_cursor.saturating_sub(1);
            sync_plugin_detail_scroll(state);
            None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let count = plugin_detail::selectable_count(state);
            if count > 0 {
                state.settings.plugin_detail_cursor =
                    (state.settings.plugin_detail_cursor + 1).min(count - 1);
            }
            sync_plugin_detail_scroll(state);
            None
        }
        KeyCode::Enter | KeyCode::Char(' ') => activate_plugin_detail_row(state),
        _ => None,
    }
}

fn sync_plugin_detail_scroll(state: &mut AppState) {
    let Some(layout) = state.settings_layout() else {
        return;
    };
    let cursor = state.settings.plugin_detail_cursor;
    let content_height = layout
        .content
        .height
        .saturating_sub(plugin_detail::DETAIL_ACTIONS_OFFSET + 1);
    let visible = content_height.max(1) as usize;
    let scroll = if cursor > 0 {
        let action_idx = cursor - 1;
        if action_idx >= visible {
            (action_idx - visible + 1) as u16
        } else {
            0
        }
    } else {
        0
    };
    state.settings.plugin_detail_scroll = scroll;
}

pub(super) fn update_settings_state(state: &mut AppState, key: KeyEvent) -> Option<SettingsAction> {
    if state.settings.plugin_detail.is_some() {
        return handle_plugin_detail_key(state, key);
    }

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
            && !matches!(
                ch,
                ' ' | '\t' | '\x1b' | '\n' | '\r' | '/' | '[' | ']' | '<' | '>'
            )
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
            move_selection_prev(state);
            if state.settings.section == SettingsSection::Theme {
                preview_selected_theme(state);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            move_selection_next(state);
            if state.settings.section == SettingsSection::Theme {
                preview_selected_theme(state);
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            state.settings.focus = SettingsFocus::Nav;
        }
        KeyCode::Char('[') => {
            collapse_all_groups(state);
        }
        KeyCode::Char(']') => {
            expand_all_groups(state);
        }
        KeyCode::Char('<') => {
            if state.settings.section == SettingsSection::Ui && state.settings.spinner_category > 0
            {
                state.settings.spinner_category -= 1;
            }
        }
        KeyCode::Char('>') => {
            if state.settings.section == SettingsSection::Ui {
                let max = crate::ui::settings::spinner::SPINNER_CATEGORIES
                    .len()
                    .saturating_sub(1);
                if state.settings.spinner_category < max {
                    state.settings.spinner_category += 1;
                }
            }
        }
        KeyCode::Tab => {
            let next = next_section(state.settings.section);
            state.settings.section = next;
            state.settings.list.selected = default_selection_for_section(state, next);
            state.settings.content_scroll = 0;
            close_plugin_detail(state);
        }
        KeyCode::BackTab => {
            let prev = prev_section(state.settings.section);
            state.settings.section = prev;
            state.settings.list.selected = default_selection_for_section(state, prev);
            state.settings.content_scroll = 0;
            close_plugin_detail(state);
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            let idx = state.settings.list.selected;
            if toggle_collapse_at(state, idx) {
                return None;
            }
            if let Some(plugin_idx) = installed_plugin_index_at(state, idx) {
                open_plugin_detail(state, plugin_idx);
                return None;
            }
            return activate_row(state, idx);
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
    open_settings_at(state, SettingsSection::Theme);
}

pub(crate) fn open_settings_at(state: &mut AppState, section: SettingsSection) {
    state.integration_install_messages.clear();
    state.plugin_install_messages.clear();
    state.settings.plugin_install_job = None;
    state.settings.original_palette = Some(state.palette.clone());
    state.settings.original_theme = Some(state.theme_name.clone());
    state.settings.config_snapshot = SettingsConfigSnapshot::load();
    state.settings.section = section;
    state.settings.search.clear();
    state.settings.focus = SettingsFocus::Content;
    state.settings.spinner_category = 0;
    state.settings.content_scroll = 0;
    state.settings.list.selected = default_selection_for_section(state, section);
    state.settings.plugin_detail = None;
    state.settings.plugin_detail_cursor = 0;
    state.settings.plugin_detail_scroll = 0;
    state.mode = Mode::Settings;
    if section == SettingsSection::Integrations {
        let previous_id = crate::ui::settings::rows::selected_settings_row_id(state);
        let _ =
            crate::app::api::plugins::reload_installed_plugins_state(&mut state.installed_plugins);
        crate::ui::settings::rows::clamp_settings_list_selection(state, previous_id);
    }
}

impl AppState {
    fn settings_layout(&self) -> Option<SettingsLayout> {
        SettingsLayout::compute(self.screen_rect(), self)
    }

    pub(super) fn handle_settings_mouse(&mut self, mouse: MouseEvent) -> Option<SettingsAction> {
        let layout = self.settings_layout()?;
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.settings.plugin_detail.is_some() {
                    if let Some(action) = handle_plugin_detail_mouse(self, mouse) {
                        return Some(action);
                    }
                    // Fall through to normal settings handling if click missed detail rows
                }

                if layout.search_index_at(mouse.column, mouse.row) {
                    self.settings.focus = SettingsFocus::Search;
                    return None;
                }

                if let Some(nav_idx) = layout.nav_index_at(mouse.column, mouse.row) {
                    let section = SettingsSection::ALL[nav_idx];
                    self.settings.section = section;
                    self.settings.list.selected = default_selection_for_section(self, section);
                    self.settings.focus = SettingsFocus::Nav;
                    close_plugin_detail(self);
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
                    if toggle_collapse_at(self, idx) {
                        return None;
                    }
                    if let Some(plugin_idx) = installed_plugin_index_at(self, idx) {
                        open_plugin_detail(self, plugin_idx);
                        return None;
                    }
                    if self.settings.section == SettingsSection::Theme {
                        preview_selected_theme(self);
                    }
                    return activate_row(self, idx);
                }

                // The plugin detail view renders a single close button (see
                // render_settings_footer), so it must hit-test the same
                // single-button geometry regardless of the section's normal
                // primary action; otherwise the visible "close" overlaps the
                // two-button layout's "apply" slot and triggers a refresh.
                let show_primary = self.settings.plugin_detail.is_none()
                    && crate::ui::settings_show_primary_action(self);
                let buttons = crate::ui::settings_button_rects(&layout, self, show_primary);
                if let Some(secondary) = buttons.secondary {
                    if super::modal::modal_action_from_buttons(
                        mouse.column,
                        mouse.row,
                        &[(secondary, ())],
                    )
                    .is_some()
                    {
                        return Some(SettingsAction::RefreshInstalledPlugins);
                    }
                }
                let mut modal_buttons = vec![(buttons.close, super::modal::ModalAction::Close)];
                if let Some(apply) = buttons.primary {
                    modal_buttons.insert(0, (apply, super::modal::ModalAction::Apply));
                }
                match super::modal::modal_action_from_buttons(
                    mouse.column,
                    mouse.row,
                    &modal_buttons,
                ) {
                    Some(super::modal::ModalAction::Apply) => apply_settings(self),
                    Some(super::modal::ModalAction::Close) => {
                        // In the plugin detail view the footer "close" goes back
                        // to the plugin list (matching the "esc back" hint and
                        // the keyboard Esc), not out of the whole settings modal.
                        if self.settings.plugin_detail.is_some() {
                            close_plugin_detail(self);
                        } else {
                            cancel_settings(self);
                        }
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

fn handle_plugin_detail_mouse(state: &mut AppState, mouse: MouseEvent) -> Option<SettingsAction> {
    let layout = state.settings_layout()?;
    let idx = plugin_detail::index_at(&layout, state, mouse.column, mouse.row)?;
    state.settings.plugin_detail_cursor = idx;
    state.settings.focus = SettingsFocus::Content;
    sync_plugin_detail_scroll(state);
    activate_plugin_detail_row(state)
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEventKind};

    use super::super::{app_for_mouse_test, mouse, state_with_workspaces};
    use super::*;
    use crate::app::state::ExperimentSetting;
    use crate::ui::settings::catalog::SettingsItemId;

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
        assert_eq!(state.settings.section, SettingsSection::Ui);

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
        open_settings_at(&mut state, SettingsSection::Theme);
        state.settings.focus = SettingsFocus::Nav;

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Ui);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Up, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Theme);
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
    fn settings_sound_toggle_returns_save_action() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Sound);
        let sound_row = section_rows(&state, SettingsSection::Sound)
            .iter()
            .position(|row| row.id == SettingsItemId::SoundAlerts)
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
    fn settings_system_toggles_pane_history() {
        let mut state = state_with_workspaces(&["test"]);
        state.pane_history_persistence = false;
        open_settings_at(&mut state, SettingsSection::System);
        let pane_history_row = section_rows(&state, SettingsSection::System)
            .iter()
            .position(|row| row.id == SettingsItemId::Experiment(ExperimentSetting::PaneHistory))
            .expect("pane history row");
        state.settings.list.selected = pane_history_row;

        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );

        assert_eq!(action, Some(SettingsAction::SavePaneHistory(true)));
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn settings_enter_on_header_toggles_group_collapse() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Theme);

        // Land on the "theme" header (index 0).
        state.settings.list.selected = 0;
        assert_eq!(
            section_rows(&state, SettingsSection::Theme)[0].kind,
            SettingsRowKind::Header
        );

        // Activating a header collapses its group.
        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );
        assert!(action.is_none());
        assert!(state.settings.collapsed_groups.contains("theme"));
        assert!(!section_rows(&state, SettingsSection::Theme)
            .iter()
            .any(|row| row.label == "auto-switch theme with host"));

        // Activating the header again expands the group.
        state.settings.list.selected = 0;
        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );
        assert!(action.is_none());
        assert!(!state.settings.collapsed_groups.contains("theme"));
        assert!(section_rows(&state, SettingsSection::Theme)
            .iter()
            .any(|row| row.label == "auto-switch theme with host"));
    }

    #[test]
    fn settings_navigation_can_land_on_headers() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::System);

        // Default selection is the first non-header row (default shell).
        let rows = section_rows(&state, SettingsSection::System);
        assert_ne!(
            rows[state.settings.list.selected].kind,
            SettingsRowKind::Header
        );

        // Arrow-up lands on the "shell" header.
        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Up, KeyModifiers::empty()),
        );
        let rows = section_rows(&state, SettingsSection::System);
        assert_eq!(
            rows[state.settings.list.selected].kind,
            SettingsRowKind::Header
        );
    }

    #[test]
    fn settings_brackets_collapse_and_expand_all_groups() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::System);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char('['), KeyModifiers::empty()),
        );
        let collapsed = section_rows(&state, SettingsSection::System);
        assert!(collapsed
            .iter()
            .all(|row| row.kind == SettingsRowKind::Header));
        // shell, scrollback, updates, experiments, system, paths & config
        assert_eq!(collapsed.len(), 6);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char(']'), KeyModifiers::empty()),
        );
        assert!(state.settings.collapsed_groups.is_empty());
        assert!(section_rows(&state, SettingsSection::System)
            .iter()
            .any(|row| row.label == "fleet ops bar"));
    }

    #[test]
    fn settings_angle_brackets_cycle_spinner_categories_in_ui() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Ui);
        state.settings.spinner_category = 1;

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char('<'), KeyModifiers::empty()),
        );
        assert_eq!(state.settings.spinner_category, 0);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char('>'), KeyModifiers::empty()),
        );
        assert_eq!(state.settings.spinner_category, 1);
    }

    #[test]
    fn settings_tab_advances_sections() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Theme);
        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        );
        assert_eq!(state.settings.section, SettingsSection::Ui);
    }

    #[test]
    fn ui_rows_expose_sidebar_layout_config_without_token_reimplementation() {
        let state = state_with_workspaces(&["test"]);
        let rows = section_rows(&state, SettingsSection::Ui);

        assert!(rows.iter().any(|row| row.label == "sidebar width"));
        assert!(rows.iter().any(|row| row.label == "collapsed mode"));
        assert!(rows.iter().any(|row| row.label == "agent row gap"));
        assert!(rows.iter().any(|row| row.label == "workspace row gap"));
        assert!(rows.iter().any(|row| row.label == "token layout"));
    }

    #[test]
    fn ui_sidebar_choice_ids_map_to_existing_persistence_actions() {
        let state = state_with_workspaces(&["test"]);
        let rows = section_rows(&state, SettingsSection::Ui);

        for row in rows.iter().filter(|row| {
            matches!(
                row.id,
                SettingsItemId::SidebarWidth
                    | SettingsItemId::SidebarCollapsedMode
                    | SettingsItemId::AgentPanelSort
                    | SettingsItemId::SidebarAgentRowGap
                    | SettingsItemId::SidebarSpaceRowGap
            )
        }) {
            assert!(activate_item(&state, row.id).is_some(), "{}", row.label);
        }
    }

    #[test]
    fn system_choice_ids_map_to_distinct_actions() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::System);
        let rows = section_rows(&state, SettingsSection::System);

        let shell_mode_idx = rows
            .iter()
            .position(|row| row.id == SettingsItemId::ShellMode)
            .expect("shell mode row");
        let cwd_idx = rows
            .iter()
            .position(|row| row.id == SettingsItemId::NewTerminalCwd)
            .expect("cwd row");
        let scrollback_idx = rows
            .iter()
            .position(|row| matches!(row.id, SettingsItemId::ScrollbackPreset { .. }))
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
    fn integrations_enter_toggles_resume_when_no_install_needed() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Integrations);

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
        open_settings_at(&mut app.state, SettingsSection::System);

        let layout = app.state.settings_layout().expect("layout");
        let rows = section_rows(&app.state, SettingsSection::System);
        let pane_history_row = rows
            .iter()
            .position(|row| row.id == SettingsItemId::Experiment(ExperimentSetting::PaneHistory))
            .expect("pane history row");
        // Select the row first so the content viewport scrolls it into view.
        app.state.settings.list.selected = pane_history_row;
        let rect = layout
            .content_row_rect(&app.state, pane_history_row)
            .expect("pane history rect");
        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            rect.x + 2,
            rect.y,
        ));

        assert_eq!(action, Some(SettingsAction::SavePaneHistory(true)));
        assert_eq!(app.state.settings.list.selected, pane_history_row);
    }

    #[test]
    fn integration_update_badge_only_tracks_outdated_recommendations() {
        let mut state = state_with_workspaces(&["test"]);
        state.integration_recommendations = vec![integration_recommendation(
            crate::integration::IntegrationStatusKind::Outdated,
            true,
        )];
        assert!(state.integration_updates_available());
        assert!(state.settings_section_has_badge(SettingsSection::Integrations));
    }

    #[test]
    fn settings_nav_hit_area_matches_layout() {
        let mut state = state_with_workspaces(&["test"]);
        // Settings popup is 96x30; give the synthetic screen enough room.
        state.view.sidebar_rect = ratatui::layout::Rect::new(0, 0, 26, 40);
        state.view.terminal_area = ratatui::layout::Rect::new(26, 0, 80, 40);
        state.integration_recommendations = vec![integration_recommendation(
            crate::integration::IntegrationStatusKind::Outdated,
            true,
        )];
        open_settings(&mut state);

        let layout = state.settings_layout().expect("layout");
        let integrations_idx = SettingsSection::ALL
            .iter()
            .position(|section| *section == SettingsSection::Integrations)
            .expect("integrations section");
        let rect = layout.nav_item_rect(integrations_idx).expect("nav rect");
        assert_eq!(
            layout.nav_index_at(rect.x + 2, rect.y),
            Some(integrations_idx)
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

    #[test]
    fn integrations_apply_action_refreshes_installed_plugins() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Integrations);

        let action = apply_settings(&mut state);

        assert_eq!(action, Some(SettingsAction::RefreshInstalledPlugins));
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn integrations_primary_is_install_when_needed() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Integrations);
        state.integration_recommendations = vec![integration_recommendation(
            crate::integration::IntegrationStatusKind::Outdated,
            true,
        )];

        assert_eq!(
            crate::ui::settings::settings_primary_button_label(&state),
            "install"
        );
        let action = apply_settings(&mut state);
        assert_eq!(action, Some(SettingsAction::InstallRecommendedIntegrations));
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn integrations_can_refresh_plugins_while_install_is_pending() {
        let mut app = app_for_mouse_test();
        app.state.view.sidebar_rect = ratatui::layout::Rect::new(0, 0, 26, 40);
        app.state.view.terminal_area = ratatui::layout::Rect::new(26, 0, 80, 40);
        open_settings_at(&mut app.state, SettingsSection::Integrations);
        app.state.installed_plugins.clear();
        app.state.installed_plugins.insert(
            "com.test.demo".to_string(),
            test_plugin("com.test.demo", vec![]),
        );
        app.state.integration_recommendations = vec![integration_recommendation(
            crate::integration::IntegrationStatusKind::Outdated,
            true,
        )];

        assert_eq!(
            crate::ui::settings::settings_primary_button_label(&app.state),
            "install"
        );
        assert!(crate::ui::settings::settings_show_secondary_action(
            &app.state
        ));

        let layout = app.state.settings_layout().expect("layout");
        let buttons = crate::ui::settings_button_rects(&layout, &app.state, true);
        let refresh = buttons.secondary.expect("refresh footer button");
        let install = buttons.primary.expect("install footer button");
        assert!(
            refresh.x + refresh.width <= install.x || install.x + install.width <= refresh.x,
            "install and refresh footer buttons must not overlap"
        );

        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            refresh.x + refresh.width / 2,
            refresh.y,
        ));
        assert_eq!(action, Some(SettingsAction::RefreshInstalledPlugins));

        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            install.x + install.width / 2,
            install.y,
        ));
        assert_eq!(action, Some(SettingsAction::InstallRecommendedIntegrations));

        let plugin_row = section_rows(&app.state, SettingsSection::Integrations)
            .iter()
            .position(|row| matches!(row.id, SettingsItemId::InstalledPlugin { .. }))
            .expect("installed plugin row");
        app.state.settings.list.selected = plugin_row;
        let action = update_settings_state(
            &mut app.state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );
        assert!(action.is_none());
        assert_eq!(app.state.settings.plugin_detail, Some(0));
    }

    #[test]
    fn integrations_selection_clamps_after_row_list_shrinks() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Integrations);
        state.installed_plugins.clear();
        for i in 0..8 {
            let id = format!("com.test.p{i}");
            state
                .installed_plugins
                .insert(id.clone(), test_plugin(&id, vec![]));
        }
        let rows_before = section_rows(&state, SettingsSection::Integrations);
        let high = rows_before
            .iter()
            .rposition(|row| matches!(row.id, SettingsItemId::InstalledPlugin { .. }))
            .expect("installed plugin row");
        state.settings.list.selected = high;
        let previous_id = crate::ui::settings::rows::selected_settings_row_id(&state);
        assert!(
            matches!(previous_id, Some(SettingsItemId::InstalledPlugin { .. })),
            "high selection should be an installed plugin"
        );

        state.installed_plugins.retain(|id, _| id == "com.test.p0");
        crate::ui::settings::rows::clamp_settings_list_selection(&mut state, previous_id);

        let rows = section_rows(&state, SettingsSection::Integrations);
        assert!(!rows.is_empty());
        assert!(
            state.settings.list.selected < rows.len(),
            "selected {} must be in 0..{}",
            state.settings.list.selected,
            rows.len()
        );
        let _ = activate_row(&state, state.settings.list.selected);
        let _ = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );
    }

    #[test]
    fn integrations_search_matches_catalog_source_and_plugin_id() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Integrations);
        state.settings.search = "chef-linear-context".to_string();

        let rows = section_rows(&state, SettingsSection::Integrations);
        assert!(rows.iter().any(|row| row.label == "Linear issues"));
    }

    fn test_action(id: &str) -> crate::api::schema::PluginManifestAction {
        crate::api::schema::PluginManifestAction {
            id: id.to_string(),
            title: format!("{id} title"),
            description: None,
            contexts: Vec::new(),
            platforms: None,
            command: vec!["echo".to_string(), id.to_string()],
        }
    }

    fn test_plugin(
        plugin_id: &str,
        actions: Vec<crate::api::schema::PluginManifestAction>,
    ) -> crate::api::schema::InstalledPluginInfo {
        crate::api::schema::InstalledPluginInfo {
            plugin_id: plugin_id.to_string(),
            name: format!("{plugin_id} name"),
            version: "0.1.0".to_string(),
            min_herdr_version: String::new(),
            description: Some("a test plugin".to_string()),
            manifest_path: "/tmp/herdr-plugin.toml".to_string(),
            plugin_root: "/tmp".to_string(),
            enabled: true,
            platforms: None,
            build: Vec::new(),
            startup: Vec::new(),
            actions,
            events: Vec::new(),
            panes: Vec::new(),
            link_handlers: Vec::new(),
            source: Default::default(),
            warnings: Vec::new(),
        }
    }

    fn open_plugins_with_demo_plugin(
        actions: Vec<crate::api::schema::PluginManifestAction>,
    ) -> AppState {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Integrations);
        // The section open reloads the registry from disk; replace it with the
        // deterministic test plugin so row indices are stable.
        state.installed_plugins.clear();
        state.installed_plugins.insert(
            "com.test.demo".to_string(),
            test_plugin("com.test.demo", actions),
        );
        state
    }

    #[test]
    fn integrations_enter_on_installed_plugin_opens_detail() {
        let mut state = open_plugins_with_demo_plugin(vec![test_action("run")]);

        let row = section_rows(&state, SettingsSection::Integrations)
            .iter()
            .position(|r| matches!(r.id, SettingsItemId::InstalledPlugin { .. }))
            .expect("installed plugin row");
        state.settings.list.selected = row;

        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );

        assert!(action.is_none());
        assert_eq!(state.settings.plugin_detail, Some(0));
        assert_eq!(state.settings.plugin_detail_cursor, 0);
    }

    #[test]
    fn plugin_detail_enter_toggles_enable_then_invokes_action() {
        let mut state = open_plugins_with_demo_plugin(vec![test_action("run")]);
        state.settings.plugin_detail = Some(0);
        state.settings.plugin_detail_cursor = 0;

        // Enter on the enable toggle row flips the plugin off.
        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );
        assert!(matches!(
            action,
            Some(SettingsAction::TogglePluginEnabled { enabled: false, .. })
        ));

        // Down to the first action, Enter invokes it.
        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
        );
        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );
        assert_eq!(
            action,
            Some(SettingsAction::InvokePluginAction {
                plugin_id: "com.test.demo".to_string(),
                action_id: "run".to_string(),
            })
        );
    }

    #[test]
    fn plugin_detail_esc_closes_back_to_list() {
        let mut state = open_plugins_with_demo_plugin(vec![]);
        state.settings.plugin_detail = Some(0);

        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
        );

        assert!(action.is_none());
        assert_eq!(state.settings.plugin_detail, None);
        assert_eq!(state.settings.plugin_detail_cursor, 0);
    }

    #[test]
    fn plugin_detail_footer_close_click_returns_to_list_without_refresh() {
        let mut app = app_for_mouse_test();
        // Give the synthetic screen room for the 96-wide settings popup.
        app.state.view.sidebar_rect = ratatui::layout::Rect::new(0, 0, 26, 40);
        app.state.view.terminal_area = ratatui::layout::Rect::new(26, 0, 80, 40);
        open_settings_at(&mut app.state, SettingsSection::Integrations);
        app.state.installed_plugins.clear();
        app.state.installed_plugins.insert(
            "com.test.demo".to_string(),
            test_plugin("com.test.demo", vec![test_action("run")]),
        );
        app.state.settings.plugin_detail = Some(0);

        // The detail view renders a single close button; click its center.
        let layout = app.state.settings_layout().expect("layout");
        let close_rect = crate::ui::settings_button_rects(&layout, &app.state, false).close;
        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            close_rect.x + close_rect.width / 2,
            close_rect.y,
        ));

        // Close goes back to the list, not out of the modal, and never refreshes.
        assert!(action.is_none());
        assert_eq!(app.state.settings.plugin_detail, None);
        assert_eq!(app.state.mode, Mode::Settings);
    }
}
