use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

use crate::{
    app::{
        state::{AppState, SettingsConfigSnapshot, SettingsFocus, SettingsSection, THEME_NAMES},
        App, Mode,
    },
    ui::settings::{
        catalog::{
            activate_item, coalesce_pending_action, settings_action_is_immediate, theme_index,
        },
        rows::{
            appearance_theme_index_at_label_index, navigable_row_indices, section_rows,
            SettingsRowKind,
        },
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
        if previous_section != SettingsSection::Agents
            && self.state.settings.section == SettingsSection::Agents
        {
            self.refresh_integration_recommendations();
        }
        if previous_section != SettingsSection::Plugins
            && self.state.settings.section == SettingsSection::Plugins
        {
            self.reload_plugins_for_settings();
        }
    }

    pub(super) fn apply_settings_action(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::CommitPending => {
                self.commit_settings_pending();
                return;
            }
            SettingsAction::DiscardPending => {
                self.discard_settings_pending();
                return;
            }
            _ => {}
        }

        if self.state.mode == Mode::Settings && !settings_action_is_immediate(&action) {
            self.apply_settings_draft(&action);
            coalesce_pending_action(&mut self.settings_draft_pending, action);
            return;
        }

        self.persist_settings_action(action);
        self.state.settings.config_snapshot = SettingsConfigSnapshot::load();
        self.state.theme_runtime.auto_switch =
            self.state.settings.config_snapshot.theme_auto_switch;
    }

    fn persist_settings_action(&mut self, action: SettingsAction) {
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
            SettingsAction::SaveHostCursor(mode) => self.save_host_cursor(mode),
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
            SettingsAction::CommitPending | SettingsAction::DiscardPending => {}
        }
    }

    fn apply_settings_draft(&mut self, action: &SettingsAction) {
        match action {
            SettingsAction::SaveTheme(name) => {
                if let Some(mut palette) = crate::app::state::Palette::from_name(name) {
                    if let Some(custom) = &self.state.theme_runtime.custom {
                        palette = palette.with_overrides(custom);
                    }
                    if let Some(accent) = &self.state.theme_runtime.legacy_accent {
                        palette.accent = crate::config::parse_color(accent);
                    }
                    self.state.palette = palette;
                    self.state.theme_name = name.clone();
                    self.state.theme_runtime.manual_name = name.clone();
                    self.state.theme_runtime.auto_switch = false;
                    self.state.settings.config_snapshot.theme_auto_switch = false;
                }
            }
            SettingsAction::SaveSound(enabled) => self.state.sound.enabled = *enabled,
            SettingsAction::SaveToastDelivery(delivery) => {
                self.state.toast_config.delivery = *delivery;
            }
            SettingsAction::SaveAgentBorderLabels(enabled) => {
                self.state.show_agent_labels_on_pane_borders = *enabled;
            }
            SettingsAction::SavePaneBorders(enabled) => self.state.pane_borders = *enabled,
            SettingsAction::SavePaneGaps(enabled) => self.state.pane_gaps = *enabled,
            SettingsAction::SaveHideTabBarWhenSingleTab(enabled) => {
                self.state.hide_tab_bar_when_single_tab = *enabled;
            }
            SettingsAction::SavePaneHistory(enabled) => {
                self.state.pane_history_persistence = *enabled;
                self.persist_pane_history = *enabled;
            }
            SettingsAction::SaveSwitchAsciiInputSourceInPrefix(enabled) => {
                self.state.switch_ascii_input_source_in_prefix = *enabled;
            }
            SettingsAction::SaveSpinnerStyle(style) => self.state.spinner_style = *style,
            SettingsAction::SaveMouseCapture(enabled) => self.state.mouse_capture = *enabled,
            SettingsAction::SaveCopyOnSelect(enabled) => self.state.copy_on_select = *enabled,
            SettingsAction::SaveConfirmClose(enabled) => self.state.confirm_close = *enabled,
            SettingsAction::SavePromptNewTabName(enabled) => {
                self.state.prompt_new_tab_name = *enabled;
            }
            SettingsAction::SavePromptNewWorkspaceName(enabled) => {
                self.state.prompt_new_workspace_name = *enabled;
            }
            SettingsAction::SaveRedrawOnFocusGained(enabled) => {
                self.state.redraw_on_focus_gained = *enabled;
            }
            SettingsAction::SaveHostCursor(mode) => {
                self.state.settings.config_snapshot.host_cursor = *mode;
                self.loaded_host_cursor = *mode;
            }
            SettingsAction::SaveSidebarCollapsedMode(mode) => {
                self.state.sidebar_collapsed_mode = *mode;
            }
            SettingsAction::SaveAgentPanelSort(sort) => self.state.agent_panel_sort = *sort,
            SettingsAction::SaveShellMode(mode) => self.state.shell_mode = *mode,
            SettingsAction::SaveDefaultShell(shell) => self.state.default_shell = shell.clone(),
            SettingsAction::SaveNewTerminalCwd(cwd) => self.state.new_terminal_cwd = cwd.clone(),
            SettingsAction::SaveScrollbackLimitBytes(bytes) => {
                self.state.pane_scrollback_limit_bytes = *bytes;
            }
            SettingsAction::SaveToastDelaySeconds(seconds) => {
                self.state.toast_config.delay_seconds = *seconds;
            }
            SettingsAction::SaveToastHerdrPosition(position) => {
                self.state.toast_config.herdr.position = *position;
            }
            SettingsAction::SaveClipboardToastEnabled(enabled) => {
                self.state.toast_config.clipboard.enabled = *enabled;
            }
            SettingsAction::SaveClipboardToastPosition(position) => {
                self.state.toast_config.clipboard.position = *position;
            }
            SettingsAction::SaveUpdateChannel(channel) => {
                self.state.settings.config_snapshot.update_channel = *channel;
            }
            SettingsAction::SaveVersionCheck(enabled) => {
                self.state.settings.config_snapshot.version_check = *enabled;
                self.update_version_check_enabled = *enabled;
            }
            SettingsAction::SaveManifestCheck(enabled) => {
                self.state.settings.config_snapshot.manifest_check = *enabled;
                self.update_manifest_check_enabled = *enabled;
            }
            SettingsAction::SaveResumeAgentsOnRestore(enabled) => {
                self.state.settings.config_snapshot.resume_agents_on_restore = *enabled;
            }
            SettingsAction::SaveManageSshConfig(enabled) => {
                self.state.settings.config_snapshot.manage_ssh_config = *enabled;
            }
            SettingsAction::SaveClipboardHistoryEnabled(enabled) => {
                self.state.settings.config_snapshot.clipboard_history_enabled = *enabled;
            }
            SettingsAction::SaveAllowNested(enabled) => {
                self.state.settings.config_snapshot.allow_nested = *enabled;
            }
            SettingsAction::SaveKittyGraphics(enabled) => {
                self.state.kitty_graphics_enabled = *enabled;
            }
            SettingsAction::SaveRevealHiddenCursorForCjkIme(enabled) => {
                self.state.reveal_hidden_cursor_for_cjk_ime = *enabled;
            }
            SettingsAction::SaveThemeAutoSwitch(enabled) => {
                self.state.settings.config_snapshot.theme_auto_switch = *enabled;
                self.state.theme_runtime.auto_switch = *enabled;
            }
            SettingsAction::SaveFleetOpsBar(enabled) => self.state.fleet_ops_bar = *enabled,
            _ => {}
        }
    }

    fn commit_settings_pending(&mut self) {
        let pending = std::mem::take(&mut self.settings_draft_pending);
        let theme_dirty = self
            .state
            .settings
            .original_theme
            .as_ref()
            .is_some_and(|original| original != &self.state.theme_name);
        let had_theme_save = pending
            .iter()
            .any(|action| matches!(action, SettingsAction::SaveTheme(_)));

        for action in pending {
            self.persist_settings_action(action);
        }
        if theme_dirty && !had_theme_save {
            // Theme may have been previewed without a queued SaveTheme action.
            let theme_name = self.state.theme_name.clone();
            self.save_theme(&theme_name);
        }

        self.state.settings.original_palette = None;
        self.state.settings.original_theme = None;
        self.state.settings.runtime_baseline = None;
        self.state.settings.config_snapshot = SettingsConfigSnapshot::load();
        self.state.theme_runtime.auto_switch =
            self.state.settings.config_snapshot.theme_auto_switch;
        super::modal::leave_modal(&mut self.state);
    }

    fn discard_settings_pending(&mut self) {
        self.settings_draft_pending.clear();
        if let Some(baseline) = self.state.settings.runtime_baseline.take() {
            let host_cursor = baseline.config_snapshot.host_cursor;
            let version_check = baseline.config_snapshot.version_check;
            let manifest_check = baseline.config_snapshot.manifest_check;
            let pane_history = baseline.pane_history_persistence;
            baseline.restore_into(&mut self.state);
            self.loaded_host_cursor = host_cursor;
            self.update_version_check_enabled = version_check;
            self.update_manifest_check_enabled = manifest_check;
            self.persist_pane_history = pane_history;
        } else if let Some(palette) = self.state.settings.original_palette.take() {
            self.state.palette = palette;
            if let Some(theme_name) = self.state.settings.original_theme.take() {
                self.state.theme_name = theme_name;
            }
        }
        self.state.settings.original_palette = None;
        self.state.settings.original_theme = None;
        super::modal::leave_modal(&mut self.state);
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

fn integrations_need_install(state: &AppState) -> bool {
    state
        .integration_recommendations
        .iter()
        .any(crate::integration::IntegrationRecommendation::needs_install)
}

fn tertiary_settings_action(state: &AppState) -> Option<SettingsAction> {
    match state.settings.section {
        SettingsSection::Agents if integrations_need_install(state) => {
            Some(SettingsAction::InstallRecommendedIntegrations)
        }
        SettingsSection::Plugins => Some(SettingsAction::RefreshInstalledPlugins),
        _ => None,
    }
}

fn save_settings(_state: &mut AppState) -> Option<SettingsAction> {
    Some(SettingsAction::CommitPending)
}

fn discard_settings(_state: &mut AppState) -> Option<SettingsAction> {
    Some(SettingsAction::DiscardPending)
}

fn footer_hit_contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row == rect.y
}

fn popup_hit_contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

fn move_settings_selection_prev(state: &mut AppState) {
    let indices = navigable_row_indices(state, state.settings.section);
    let Some(&first) = indices.first() else {
        return;
    };
    let current = state.settings.list.selected;
    if let Some(pos) = indices.iter().position(|&idx| idx == current) {
        if pos > 0 {
            state.settings.list.selected = indices[pos - 1];
        }
    } else {
        state.settings.list.selected = indices
            .iter()
            .copied()
            .rev()
            .find(|idx| *idx < current)
            .unwrap_or(first);
    }
}

fn move_settings_selection_next(state: &mut AppState) {
    let indices = navigable_row_indices(state, state.settings.section);
    let Some(&last) = indices.last() else {
        return;
    };
    let current = state.settings.list.selected;
    if let Some(pos) = indices.iter().position(|&idx| idx == current) {
        if pos + 1 < indices.len() {
            state.settings.list.selected = indices[pos + 1];
        }
    } else {
        state.settings.list.selected = indices
            .iter()
            .copied()
            .find(|idx| *idx > current)
            .unwrap_or(last);
    }
}

fn default_navigable_selection(state: &AppState, section: SettingsSection) -> usize {
    match section {
        SettingsSection::Appearance => {
            let theme_idx = current_theme_index(&state.theme_name);
            section_rows(state, section)
                .iter()
                .position(|row| theme_index(row.id) == Some(theme_idx))
                .unwrap_or(0)
        }
        SettingsSection::Notifications => section_rows(state, section)
            .iter()
            .position(|row| row.label == "sound alerts")
            .unwrap_or(0),
        _ => navigable_row_indices(state, section)
            .first()
            .copied()
            .unwrap_or(0),
    }
}

fn default_selection_for_section(state: &AppState, section: SettingsSection) -> usize {
    let preferred = default_navigable_selection(state, section);
    navigable_row_indices(state, section)
        .into_iter()
        .find(|idx| *idx == preferred)
        .or_else(|| navigable_row_indices(state, section).first().copied())
        .unwrap_or(0)
}

fn next_section(section: SettingsSection) -> SettingsSection {
    section.next()
}

fn prev_section(section: SettingsSection) -> SettingsSection {
    section.prev()
}

fn activate_row(state: &AppState, row_index: usize) -> Option<SettingsAction> {
    let rows = section_rows(state, state.settings.section);
    let row = rows.get(row_index)?;
    activate_item(state, row.id)
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
        let is_content_command = matches!(ch, 'h' | 'j' | 'k')
            || (ch == 'i'
                && state.settings.section == SettingsSection::Agents
                && integrations_need_install(state))
            || (matches!(ch, '[' | ']')
                && state.settings.section == SettingsSection::Appearance);
        if key.modifiers.is_empty()
            && ch.is_ascii()
            && !matches!(ch, ' ' | '\t' | '\x1b' | '\n' | '\r' | '/')
            && !is_content_command
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
            move_settings_selection_prev(state);
            if state.settings.section == SettingsSection::Appearance {
                preview_selected_theme(state);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            move_settings_selection_next(state);
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
        KeyCode::Char(' ') => {
            return activate_row(state, state.settings.list.selected);
        }
        KeyCode::Char('i')
            if state.settings.section == SettingsSection::Agents
                && integrations_need_install(state) =>
        {
            return tertiary_settings_action(state);
        }
        _ => match super::modal::modal_action_from_key(&key, super::modal::SETTINGS_ACTIONS) {
            Some(super::modal::ModalAction::Apply) => return save_settings(state),
            Some(super::modal::ModalAction::Close) => return discard_settings(state),
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
                return discard_settings(state);
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
    state.plugin_install_messages.clear();
    state.settings.plugin_install_job = None;
    state.settings.config_snapshot = SettingsConfigSnapshot::load();
    state.settings.original_palette = Some(state.palette.clone());
    state.settings.original_theme = Some(state.theme_name.clone());
    state.settings.runtime_baseline =
        Some(crate::app::state::SettingsRuntimeBaseline::capture(state));
    state.settings.section = section;
    state.settings.search.clear();
    state.settings.focus = SettingsFocus::Content;
    state.settings.spinner_category = 0;
    state.settings.content_scroll = 0;
    state.settings.list.selected = default_selection_for_section(state, section);
    state.mode = Mode::Settings;
    if section == SettingsSection::Plugins {
        let _ =
            crate::app::api::plugins::reload_installed_plugins_state(&mut state.installed_plugins);
    }
}

impl App {
    pub(crate) fn open_settings_menu(&mut self) {
        self.settings_draft_pending.clear();
        open_settings(&mut self.state);
    }

    pub(crate) fn open_settings_menu_at(&mut self, section: SettingsSection) {
        self.settings_draft_pending.clear();
        open_settings_at(&mut self.state, section);
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
                if !popup_hit_contains(layout.popup, mouse.column, mouse.row) {
                    return discard_settings(self);
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
                    return None;
                }

                if self.settings.section == SettingsSection::Appearance {
                    if let Some(category) =
                        layout.spinner_category_index_at(self, mouse.column, mouse.row)
                    {
                        self.settings.spinner_category = category;
                        return None;
                    }

                    if let Some(chip_idx) =
                        layout.appearance_theme_chip_index_at(self, mouse.column, mouse.row)
                    {
                        if let Some(theme_idx) =
                            appearance_theme_index_at_label_index(self, chip_idx)
                        {
                            let rows = section_rows(self, SettingsSection::Appearance);
                            if let Some(row_idx) = rows
                                .iter()
                                .position(|row| theme_index(row.id) == Some(theme_idx))
                            {
                                self.settings.list.select(row_idx);
                                self.settings.focus = SettingsFocus::Content;
                                preview_selected_theme(self);
                                if let Some(row) = rows.get(row_idx) {
                                    return activate_item(self, row.id);
                                }
                            }
                        }
                        return None;
                    }
                } else if let Some(category) =
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

                let show_tertiary = crate::ui::settings_show_tertiary_action(self);
                let buttons =
                    crate::ui::settings_button_rects(&layout, self.settings.section, show_tertiary);
                if footer_hit_contains(buttons.cancel, mouse.column, mouse.row) {
                    return discard_settings(self);
                }
                if footer_hit_contains(buttons.save, mouse.column, mouse.row) {
                    return save_settings(self);
                }
                if let Some(tertiary) = buttons.tertiary {
                    if footer_hit_contains(tertiary, mouse.column, mouse.row) {
                        return tertiary_settings_action(self);
                    }
                }
                None
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
    use crate::ui::settings::catalog::SettingsItemId;

    #[test]
    fn settings_cancel_restores_previewed_theme_from_other_sections() {
        let mut app = app_for_mouse_test();
        let original_palette = app.state.palette.clone();
        let original_theme = app.state.theme_name.clone();

        open_settings(&mut app.state);
        update_settings_state(
            &mut app.state,
            KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
        );
        assert_ne!(app.state.theme_name, original_theme);

        update_settings_state(
            &mut app.state,
            KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        );
        assert_eq!(app.state.settings.section, SettingsSection::Layout);

        let action = update_settings_state(
            &mut app.state,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
        );
        assert_eq!(action, Some(SettingsAction::DiscardPending));
        app.apply_settings_action(action.expect("discard"));

        assert_eq!(app.state.mode, Mode::Terminal);
        assert_eq!(app.state.theme_name, original_theme);
        assert_eq!(app.state.palette.accent, original_palette.accent);
        assert_eq!(app.state.palette.panel_bg, original_palette.panel_bg);
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
            .position(|row| row.id == SettingsItemId::SoundAlerts)
            .expect("sound row");
        state.settings.list.selected = sound_row;

        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()),
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
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()),
        );

        assert_eq!(action, Some(SettingsAction::SavePaneHistory(true)));
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn settings_enter_commits_pending_via_keyboard() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Notifications);
        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );
        assert_eq!(action, Some(SettingsAction::CommitPending));
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn settings_content_hotkeys_do_not_start_search() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Notifications);
        let initial = state.settings.list.selected;

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()),
        );
        assert!(state.settings.list.selected > initial);
        assert!(state.settings.search.is_empty());
        assert_eq!(state.settings.focus, SettingsFocus::Content);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty()),
        );
        assert!(state.settings.search.is_empty());
        assert_eq!(state.settings.focus, SettingsFocus::Nav);
    }

    #[test]
    fn settings_appearance_category_hotkey_does_not_start_search() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Appearance);

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char(']'), KeyModifiers::empty()),
        );

        assert_eq!(state.settings.spinner_category, 1);
        assert!(state.settings.search.is_empty());
        assert_eq!(state.settings.focus, SettingsFocus::Content);
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
    fn terminal_choice_ids_map_to_distinct_actions() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Terminal);
        let rows = section_rows(&state, SettingsSection::Terminal);

        let shell_mode_idx = rows
            .iter()
            .position(|row| {
                matches!(
                    row.id,
                    SettingsItemId::ShellMode {
                        mode: crate::config::ShellModeConfig::Login
                    }
                )
            })
            .expect("shell mode login row");
        let cwd_idx = rows
            .iter()
            .position(|row| {
                matches!(
                    row.id,
                    SettingsItemId::NewTerminalCwd {
                        choice: crate::ui::settings::catalog::NewTerminalCwdChoice::Home
                    }
                )
            })
            .expect("cwd home row");
        let scrollback_idx = rows
            .iter()
            .position(|row| matches!(row.id, SettingsItemId::ScrollbackPreset { .. }))
            .expect("scrollback row");

        state.settings.list.selected = shell_mode_idx;
        assert_eq!(
            activate_row(&state, shell_mode_idx),
            Some(SettingsAction::SaveShellMode(
                crate::config::ShellModeConfig::Login
            ))
        );
        state.settings.list.selected = cwd_idx;
        assert_eq!(
            activate_row(&state, cwd_idx),
            Some(SettingsAction::SaveNewTerminalCwd(
                crate::config::NewTerminalCwdConfig::Home
            ))
        );
        state.settings.list.selected = scrollback_idx;
        assert!(matches!(
            activate_row(&state, scrollback_idx),
            Some(SettingsAction::SaveScrollbackLimitBytes(_))
        ));
    }

    #[test]
    fn agents_space_toggles_resume_when_no_install_needed() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Agents);

        let action = update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()),
        );
        assert_eq!(
            action,
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
        app.state.view.sidebar_rect = ratatui::layout::Rect::new(0, 0, 28, 50);
        app.state.view.terminal_area = ratatui::layout::Rect::new(28, 0, 132, 50);
        app.state.pane_history_persistence = false;
        open_settings_at(&mut app.state, SettingsSection::Advanced);

        let layout = app.state.settings_layout().expect("layout");
        let row_idx = section_rows(&app.state, SettingsSection::Advanced)
            .iter()
            .position(|row| {
                row.id
                    == SettingsItemId::Experiment(crate::app::state::ExperimentSetting::PaneHistory)
            })
            .expect("pane history row");
        let rect = layout
            .content_row_rect(&app.state, row_idx)
            .expect("pane history row should have geometry");
        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            rect.x + 2,
            rect.y,
        ));

        assert_eq!(action, Some(SettingsAction::SavePaneHistory(true)));
        assert_eq!(app.state.settings.list.selected, row_idx);
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
        // Settings popup is 100x32; give the synthetic screen enough room.
        state.view.sidebar_rect = ratatui::layout::Rect::new(0, 0, 28, 50);
        state.view.terminal_area = ratatui::layout::Rect::new(28, 0, 132, 50);
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

    #[test]
    fn plugins_tertiary_action_refreshes_installed_plugins() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Plugins);

        let action = tertiary_settings_action(&state);

        assert_eq!(action, Some(SettingsAction::RefreshInstalledPlugins));
        assert_eq!(state.mode, Mode::Settings);
    }

    #[test]
    fn plugins_search_matches_catalog_source_and_plugin_id() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Plugins);
        state.settings.search = "chef-linear-context".to_string();

        let rows = section_rows(&state, SettingsSection::Plugins);
        assert!(rows.iter().any(|row| row.label == "Linear issues"));
    }

    #[test]
    fn settings_mouse_theme_chip_selects_theme_row() {
        use crate::ui::settings::rows::appearance_theme_labels;
        use crate::ui::settings::widgets::{chip_wrap_rects, CHIP_HORIZONTAL_GAP};

        let mut app = app_for_mouse_test();
        open_settings_at(&mut app.state, SettingsSection::Appearance);
        let layout = app.state.settings_layout().expect("layout");
        let labels = appearance_theme_labels(&app.state);
        let dracula_idx = labels
            .iter()
            .position(|label| *label == "dracula")
            .expect("dracula label");
        let chip_rect = chip_wrap_rects(
            layout.appearance_theme_chips_rect(&app.state),
            &labels,
            CHIP_HORIZONTAL_GAP,
        )
        .into_iter()
        .nth(dracula_idx)
        .expect("dracula chip rect");

        app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            chip_rect.x + 1,
            chip_rect.y,
        ));

        let rows = section_rows(&app.state, SettingsSection::Appearance);
        let dracula_row = rows
            .iter()
            .position(|row| row.label == "dracula")
            .expect("dracula row");
        assert_eq!(app.state.settings.list.selected, dracula_row);
        assert_eq!(app.state.theme_name, "dracula");
    }

    #[test]
    fn settings_mouse_layout_checklist_toggles_pane_borders() {
        let mut app = app_for_mouse_test();
        app.state.view.sidebar_rect = ratatui::layout::Rect::new(0, 0, 28, 50);
        app.state.view.terminal_area = ratatui::layout::Rect::new(28, 0, 132, 50);
        app.state.pane_borders = false;
        open_settings_at(&mut app.state, SettingsSection::Layout);
        let layout = app.state.settings_layout().expect("layout");
        let row_idx = section_rows(&app.state, SettingsSection::Layout)
            .iter()
            .position(|row| row.id == SettingsItemId::PaneBorders)
            .expect("pane borders row");
        let rect = layout
            .content_row_rect(&app.state, row_idx)
            .expect("pane borders checklist row");
        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            rect.x + 2,
            rect.y,
        ));

        assert_eq!(action, Some(SettingsAction::SavePaneBorders(true)));
        assert_eq!(app.state.settings.list.selected, row_idx);
    }

    #[test]
    fn settings_outside_click_cancels() {
        let mut app = app_for_mouse_test();
        app.state.view.sidebar_rect = ratatui::layout::Rect::new(0, 0, 28, 50);
        app.state.view.terminal_area = ratatui::layout::Rect::new(28, 0, 132, 50);
        open_settings(&mut app.state);
        assert_eq!(app.state.mode, Mode::Settings);

        let layout = app.state.settings_layout().expect("layout");
        let outside_row = layout.popup.y.saturating_sub(1);
        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            layout.popup.x,
            outside_row,
        ));
        assert_eq!(action, Some(SettingsAction::DiscardPending));
        app.apply_settings_action(action.expect("discard"));

        assert_ne!(app.state.mode, Mode::Settings);
    }

    #[test]
    fn plugins_keyboard_skips_section_header_rows() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Plugins);
        let rows = section_rows(&state, SettingsSection::Plugins);
        let header_idx = rows
            .iter()
            .position(|row| row.id == SettingsItemId::PluginsInstalledHeader)
            .expect("plugins installed header row");
        state.settings.list.selected = header_idx;

        update_settings_state(
            &mut state,
            KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
        );

        let selected = &rows[state.settings.list.selected];
        assert!(!matches!(
            selected.id,
            SettingsItemId::PluginsInstalledHeader | SettingsItemId::PluginsCatalogHeader
        ));
    }

    #[test]
    fn input_keyboard_skips_non_actionable_keybind_help() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Input);
        let indices = navigable_row_indices(&state, SettingsSection::Input);
        let rows = section_rows(&state, SettingsSection::Input);
        assert!(indices.iter().all(|idx| rows[*idx].id != SettingsItemId::KeybindHelp));
        assert!(indices.iter().any(|idx| {
            matches!(rows[*idx].id, SettingsItemId::HostCursor { .. })
        }));
    }

    #[test]
    fn draft_toggle_is_memory_only_until_commit() {
        let mut app = app_for_mouse_test();
        app.state.sound.enabled = false;
        open_settings_at(&mut app.state, SettingsSection::Notifications);
        let sound_row = section_rows(&app.state, SettingsSection::Notifications)
            .iter()
            .position(|row| row.id == SettingsItemId::SoundAlerts)
            .expect("sound row");
        app.state.settings.list.selected = sound_row;

        let action = update_settings_state(
            &mut app.state,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()),
        );
        assert_eq!(action, Some(SettingsAction::SaveSound(true)));
        app.apply_settings_action(action.expect("draft toggle"));
        assert!(app.state.sound.enabled);
        assert_eq!(app.settings_draft_pending.len(), 1);

        let discard = update_settings_state(
            &mut app.state,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
        );
        assert_eq!(discard, Some(SettingsAction::DiscardPending));
        app.apply_settings_action(discard.expect("discard"));
        assert!(!app.state.sound.enabled);
        assert!(app.settings_draft_pending.is_empty());
        assert_ne!(app.state.mode, Mode::Settings);
    }

    #[test]
    fn draft_commit_persists_once_via_pending_queue() {
        let mut app = app_for_mouse_test();
        app.state.pane_borders = false;
        open_settings_at(&mut app.state, SettingsSection::Layout);
        let row_idx = section_rows(&app.state, SettingsSection::Layout)
            .iter()
            .position(|row| row.id == SettingsItemId::PaneBorders)
            .expect("pane borders");
        app.state.settings.list.selected = row_idx;

        let toggle = update_settings_state(
            &mut app.state,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()),
        );
        app.apply_settings_action(toggle.expect("toggle"));
        assert!(app.state.pane_borders);
        assert_eq!(app.settings_draft_pending.len(), 1);

        // Coalesce repeated toggles into one pending slot.
        let toggle_again = update_settings_state(
            &mut app.state,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()),
        );
        app.apply_settings_action(toggle_again.expect("toggle again"));
        assert!(!app.state.pane_borders);
        assert_eq!(app.settings_draft_pending.len(), 1);

        let commit = update_settings_state(
            &mut app.state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        );
        assert_eq!(commit, Some(SettingsAction::CommitPending));
        app.apply_settings_action(commit.expect("commit"));
        assert!(app.settings_draft_pending.is_empty());
        assert_ne!(app.state.mode, Mode::Settings);
    }

    #[test]
    fn host_cursor_shell_mode_and_cwd_expose_all_chip_labels() {
        let mut state = state_with_workspaces(&["test"]);
        open_settings_at(&mut state, SettingsSection::Input);
        let input_rows = section_rows(&state, SettingsSection::Input);
        let host_labels: Vec<&str> = input_rows
            .iter()
            .filter_map(|row| match row.id {
                SettingsItemId::HostCursor { .. } => Some(row.label.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(host_labels, ["auto", "native", "drawn"]);

        open_settings_at(&mut state, SettingsSection::Terminal);
        let terminal_rows = section_rows(&state, SettingsSection::Terminal);
        let shell_labels: Vec<&str> = terminal_rows
            .iter()
            .filter_map(|row| match row.id {
                SettingsItemId::ShellMode { .. } => Some(row.label.as_str()),
                _ => None,
            })
            .collect();
        let cwd_labels: Vec<&str> = terminal_rows
            .iter()
            .filter_map(|row| match row.id {
                SettingsItemId::NewTerminalCwd { .. } => Some(row.label.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(shell_labels, ["auto", "login", "non_login"]);
        assert_eq!(cwd_labels, ["follow", "home", "current"]);
    }

    #[test]
    fn host_cursor_chip_mouse_selects_concrete_mode() {
        let mut app = app_for_mouse_test();
        app.state.view.sidebar_rect = ratatui::layout::Rect::new(0, 0, 28, 50);
        app.state.view.terminal_area = ratatui::layout::Rect::new(28, 0, 132, 50);
        app.state.settings.config_snapshot.host_cursor = crate::config::HostCursorModeConfig::Auto;
        open_settings_at(&mut app.state, SettingsSection::Input);
        let layout = app.state.settings_layout().expect("layout");
        let rows = section_rows(&app.state, SettingsSection::Input);
        let drawn_idx = rows
            .iter()
            .position(|row| {
                matches!(
                    row.id,
                    SettingsItemId::HostCursor {
                        mode: crate::config::HostCursorModeConfig::Drawn
                    }
                )
            })
            .expect("drawn chip row");
        let rect = layout
            .content_row_rect(&app.state, drawn_idx)
            .expect("drawn chip geometry");
        let action = app.state.handle_settings_mouse(mouse(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            rect.x + 1,
            rect.y,
        ));
        assert_eq!(
            action,
            Some(SettingsAction::SaveHostCursor(
                crate::config::HostCursorModeConfig::Drawn
            ))
        );
    }
}
