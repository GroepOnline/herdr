use crate::{
    app::state::{AppState, SettingsSection, THEME_NAMES},
    config::{SpinnerStyle, ToastDelivery},
    pane_template::PaneTemplateId,
};

use super::spinner::active_spinner_category;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsRowKind {
    Toggle,
    Choice,
    Theme,
    Spinner,
    Template,
    Integration,
    Note,
}

#[derive(Debug, Clone)]
pub(crate) struct SettingsRow {
    pub label: String,
    pub detail: Option<String>,
    pub kind: SettingsRowKind,
    /// Section-local payload index (theme idx, spinner style, template idx, etc.).
    pub payload: usize,
}

fn matches_filter(filter: &str, label: &str, detail: Option<&str>) -> bool {
    if filter.is_empty() {
        return true;
    }
    let needle = filter.to_ascii_lowercase();
    label.to_ascii_lowercase().contains(&needle)
        || detail
            .is_some_and(|d| d.to_ascii_lowercase().contains(&needle))
}

pub(crate) fn section_rows(app: &AppState, section: SettingsSection) -> Vec<SettingsRow> {
    let filter = app.settings.search.as_str();
    let mut rows = Vec::new();

    match section {
        SettingsSection::Appearance => {
            rows.push(SettingsRow {
                label: "auto-switch theme with host".to_string(),
                detail: Some("follow terminal light/dark appearance".to_string()),
                kind: SettingsRowKind::Toggle,
                payload: 0,
            });
            for (idx, name) in THEME_NAMES.iter().enumerate() {
                rows.push(SettingsRow {
                    label: (*name).to_string(),
                    detail: None,
                    kind: SettingsRowKind::Theme,
                    payload: idx,
                });
            }
            rows.push(SettingsRow {
                label: "spinner preview".to_string(),
                detail: Some("animated preview of the selected style".to_string()),
                kind: SettingsRowKind::Note,
                payload: 0,
            });
            let category = active_spinner_category(app.settings.spinner_category);
            for (idx, style) in category.styles.iter().enumerate() {
                rows.push(SettingsRow {
                    label: style.label().to_string(),
                    detail: None,
                    kind: SettingsRowKind::Spinner,
                    payload: idx,
                });
            }
        }
        SettingsSection::Layout => {
            for (idx, (label, detail)) in [
                ("pane borders", "draw borders around split panes"),
                ("pane gaps", "keep split panes visually separated"),
                ("agent labels", "show agent names in pane borders"),
                ("hide tab bar", "hide tab row when only one tab"),
            ]
            .iter()
            .enumerate()
            {
                rows.push(SettingsRow {
                    label: (*label).to_string(),
                    detail: Some((*detail).to_string()),
                    kind: SettingsRowKind::Toggle,
                    payload: idx,
                });
            }
            rows.push(SettingsRow {
                label: "sidebar collapsed mode".to_string(),
                detail: Some(app.sidebar_collapsed_mode_label()),
                kind: SettingsRowKind::Choice,
                payload: 0,
            });
            rows.push(SettingsRow {
                label: "agent panel sort".to_string(),
                detail: Some(app.agent_panel_sort_label()),
                kind: SettingsRowKind::Choice,
                payload: 1,
            });
            for (idx, id) in PaneTemplateId::ALL.iter().enumerate() {
                let tmpl = id.template();
                rows.push(SettingsRow {
                    label: tmpl.name.to_string(),
                    detail: Some(tmpl.description.to_string()),
                    kind: SettingsRowKind::Template,
                    payload: idx,
                });
            }
        }
        SettingsSection::Input => {
            for (idx, (label, detail)) in [
                ("mouse capture", "capture mouse for Herdr UI chrome"),
                ("copy on select", "copy selected terminal text to clipboard"),
                ("redraw on focus gained", "refresh panes when Herdr regains focus"),
                ("confirm close", "ask before closing tabs and workspaces"),
                ("prompt new tab name", "ask for a name when creating tabs"),
                ("prompt new workspace name", "ask for a name when creating workspaces"),
            ]
            .iter()
            .enumerate()
            {
                rows.push(SettingsRow {
                    label: (*label).to_string(),
                    detail: Some((*detail).to_string()),
                    kind: SettingsRowKind::Toggle,
                    payload: idx,
                });
            }
            rows.push(SettingsRow {
                label: "host cursor".to_string(),
                detail: Some(app.host_cursor_label()),
                kind: SettingsRowKind::Choice,
                payload: 0,
            });
            let prefix = crate::config::format_key_combo((app.prefix_code, app.prefix_mods));
            rows.push(SettingsRow {
                label: "keybind help".to_string(),
                detail: Some(format!("press {prefix}+? or open prefix help")),
                kind: SettingsRowKind::Note,
                payload: 1,
            });
        }
        SettingsSection::Terminal => {
            rows.push(SettingsRow {
                label: "default shell".to_string(),
                detail: Some(app.default_shell_display()),
                kind: SettingsRowKind::Choice,
                payload: 0,
            });
            rows.push(SettingsRow {
                label: "shell mode".to_string(),
                detail: Some(app.shell_mode_label()),
                kind: SettingsRowKind::Choice,
                payload: 0,
            });
            rows.push(SettingsRow {
                label: "new pane cwd".to_string(),
                detail: Some(app.new_terminal_cwd_label()),
                kind: SettingsRowKind::Choice,
                payload: 1,
            });
            for (idx, (bytes, label)) in scrollback_presets().iter().enumerate() {
                rows.push(SettingsRow {
                    label: format!("scrollback {label}"),
                    detail: None,
                    kind: SettingsRowKind::Choice,
                    payload: idx + 2,
                });
            }
        }
        SettingsSection::Notifications => {
            rows.push(SettingsRow {
                label: "sound alerts".to_string(),
                detail: None,
                kind: SettingsRowKind::Toggle,
                payload: 0,
            });
            for (idx, label) in [
                "toast off",
                "toast inside herdr",
                "toast via terminal",
                "toast via system",
            ]
            .iter()
            .enumerate()
            {
                rows.push(SettingsRow {
                    label: (*label).to_string(),
                    detail: None,
                    kind: SettingsRowKind::Choice,
                    payload: idx + 1,
                });
            }
            rows.push(SettingsRow {
                label: "toast delay".to_string(),
                detail: Some(format!("{}s", app.toast_config.delay_seconds)),
                kind: SettingsRowKind::Choice,
                payload: 0,
            });
            rows.push(SettingsRow {
                label: "herdr toast position".to_string(),
                detail: Some(app.toast_herdr_position_label()),
                kind: SettingsRowKind::Choice,
                payload: 1,
            });
            rows.push(SettingsRow {
                label: "clipboard toast".to_string(),
                detail: Some(app.clipboard_toast_label()),
                kind: SettingsRowKind::Choice,
                payload: 2,
            });
        }
        SettingsSection::Agents => {
            rows.push(SettingsRow {
                label: "resume agents on restore".to_string(),
                detail: Some("resume supported agent sessions when restoring".to_string()),
                kind: SettingsRowKind::Toggle,
                payload: 0,
            });
            for (idx, item) in app.integration_recommendations.iter().enumerate() {
                rows.push(SettingsRow {
                    label: item.label.to_string(),
                    detail: Some(item.status_label().to_string()),
                    kind: SettingsRowKind::Integration,
                    payload: idx,
                });
            }
            if app.integration_recommendations.is_empty() {
                rows.push(SettingsRow {
                    label: "no supported agent CLIs found on PATH".to_string(),
                    detail: None,
                    kind: SettingsRowKind::Note,
                    payload: 0,
                });
            }
        }
        SettingsSection::Updates => {
            rows.push(SettingsRow {
                label: "stable channel".to_string(),
                detail: None,
                kind: SettingsRowKind::Choice,
                payload: 0,
            });
            rows.push(SettingsRow {
                label: "preview channel".to_string(),
                detail: None,
                kind: SettingsRowKind::Choice,
                payload: 1,
            });
            rows.push(SettingsRow {
                label: "version check".to_string(),
                detail: Some("check for herdr updates in the background".to_string()),
                kind: SettingsRowKind::Toggle,
                payload: 0,
            });
            rows.push(SettingsRow {
                label: "manifest check".to_string(),
                detail: Some("check for agent detection manifest updates".to_string()),
                kind: SettingsRowKind::Toggle,
                payload: 1,
            });
        }
        SettingsSection::Advanced => {
            for (idx, setting) in crate::app::state::ExperimentSetting::ALL.iter().enumerate() {
                rows.push(SettingsRow {
                    label: setting.label().to_string(),
                    detail: None,
                    kind: SettingsRowKind::Toggle,
                    payload: idx,
                });
            }
            for (idx, (label, detail)) in [
                ("manage ssh config", "add keepalive fallbacks for herdr --remote"),
                ("clipboard history", "retain recent global clipboard entries"),
            ]
            .iter()
            .enumerate()
            {
                rows.push(SettingsRow {
                    label: (*label).to_string(),
                    detail: Some((*detail).to_string()),
                    kind: SettingsRowKind::Toggle,
                    payload: 100 + idx,
                });
            }
            for (label, detail) in [
                ("worktrees path", "git worktrees under repo/.git/worktrees"),
                ("reload config", "prefix reload or herdr server reload-config"),
            ] {
                rows.push(SettingsRow {
                    label: label.to_string(),
                    detail: Some(detail.to_string()),
                    kind: SettingsRowKind::Note,
                    payload: 0,
                });
            }
            rows.push(SettingsRow {
                label: "config file".to_string(),
                detail: Some(crate::config::config_path().display().to_string()),
                kind: SettingsRowKind::Note,
                payload: 0,
            });
        }
    }

    rows.retain(|row| matches_filter(filter, &row.label, row.detail.as_deref()));
    rows
}

pub(crate) fn scrollback_presets() -> &'static [(usize, &'static str)] {
    &[
        (1_000_000, "1 MB"),
        (5_000_000, "5 MB"),
        (10_000_000, "10 MB"),
        (50_000_000, "50 MB"),
    ]
}

pub(crate) fn row_toggle_checked(app: &AppState, section: SettingsSection, row: &SettingsRow) -> bool {
    match section {
        SettingsSection::Appearance if row.kind == SettingsRowKind::Toggle => {
            app.settings.config_snapshot.theme_auto_switch
        }
        SettingsSection::Layout if row.kind == SettingsRowKind::Toggle => match row.payload {
            0 => app.pane_borders_enabled(),
            1 => app.pane_gaps_enabled(),
            2 => app.agent_border_labels_enabled(),
            3 => app.hide_tab_bar_when_single_tab_enabled(),
            _ => false,
        },
        SettingsSection::Input if row.kind == SettingsRowKind::Toggle => match row.payload {
            0 => app.mouse_capture,
            1 => app.copy_on_select,
            2 => app.redraw_on_focus_gained,
            3 => app.confirm_close,
            4 => app.prompt_new_tab_name,
            5 => app.prompt_new_workspace_name,
            _ => false,
        },
        SettingsSection::Notifications if row.kind == SettingsRowKind::Toggle => app.sound_enabled(),
        SettingsSection::Agents if row.kind == SettingsRowKind::Toggle => {
            app.settings.config_snapshot.resume_agents_on_restore
        }
        SettingsSection::Updates if row.kind == SettingsRowKind::Toggle => match row.payload {
            0 => app.settings.config_snapshot.version_check,
            1 => app.settings.config_snapshot.manifest_check,
            _ => false,
        },
        SettingsSection::Advanced if row.kind == SettingsRowKind::Toggle => {
            if row.payload >= 100 {
                match row.payload - 100 {
                    0 => app.settings.config_snapshot.manage_ssh_config,
                    1 => app.settings.config_snapshot.clipboard_history_enabled,
                    _ => false,
                }
            } else {
                crate::app::state::ExperimentSetting::ALL
                    .get(row.payload)
                    .copied()
                    .is_some_and(|setting| setting.enabled(app))
            }
        }
        _ => false,
    }
}

pub(crate) fn row_choice_selected(app: &AppState, section: SettingsSection, row: &SettingsRow) -> bool {
    match section {
        SettingsSection::Updates if row.kind == SettingsRowKind::Choice => match row.payload {
            0 => {
                app.settings.config_snapshot.update_channel
                    == crate::config::UpdateChannelConfig::Stable
            }
            1 => {
                app.settings.config_snapshot.update_channel
                    == crate::config::UpdateChannelConfig::Preview
            }
            _ => false,
        },
        SettingsSection::Notifications if row.kind == SettingsRowKind::Choice => {
            let delivery = match row.payload {
                1 => ToastDelivery::Off,
                2 => ToastDelivery::Herdr,
                3 => ToastDelivery::Terminal,
                4 => ToastDelivery::System,
                _ => return false,
            };
            app.toast_delivery() == delivery
        }
        SettingsSection::Terminal if row.kind == SettingsRowKind::Choice && row.payload >= 2 => {
            let preset_idx = row.payload - 2;
            scrollback_presets()
                .get(preset_idx)
                .is_some_and(|(bytes, _)| app.pane_scrollback_limit_bytes == *bytes)
        }
        _ => false,
    }
}

pub(crate) fn row_theme_current(app: &AppState, row: &SettingsRow) -> bool {
    THEME_NAMES
        .get(row.payload)
        .is_some_and(|name| themes_match(name, &app.theme_name))
}

pub(crate) fn row_spinner_current(app: &AppState, row: &SettingsRow) -> bool {
    active_spinner_category(app.settings.spinner_category)
        .styles
        .get(row.payload)
        .copied()
        .is_some_and(|style| style == app.spinner_style)
}

pub(crate) fn spinner_style_for_row(app: &AppState, row: &SettingsRow) -> Option<SpinnerStyle> {
    active_spinner_category(app.settings.spinner_category)
        .styles
        .get(row.payload)
        .copied()
}

fn themes_match(a: &str, b: &str) -> bool {
    a.to_lowercase().replace([' ', '_'], "-") == b.to_lowercase().replace([' ', '_'], "-")
}

trait SettingsDisplayLabels {
    fn sidebar_collapsed_mode_label(&self) -> String;
    fn agent_panel_sort_label(&self) -> String;
    fn default_shell_display(&self) -> String;
    fn shell_mode_label(&self) -> String;
    fn new_terminal_cwd_label(&self) -> String;
    fn toast_herdr_position_label(&self) -> String;
    fn clipboard_toast_label(&self) -> String;
    fn host_cursor_label(&self) -> String;
}

impl SettingsDisplayLabels for AppState {
    fn sidebar_collapsed_mode_label(&self) -> String {
        match self.sidebar_collapsed_mode {
            crate::config::SidebarCollapsedModeConfig::Compact => "compact".to_string(),
            crate::config::SidebarCollapsedModeConfig::Hidden => "hidden".to_string(),
        }
    }

    fn agent_panel_sort_label(&self) -> String {
        match self.agent_panel_sort {
            crate::app::state::AgentPanelSort::Spaces => "spaces".to_string(),
            crate::app::state::AgentPanelSort::Priority => "priority".to_string(),
        }
    }

    fn default_shell_display(&self) -> String {
        if self.default_shell.is_empty() {
            "SHELL or /bin/sh".to_string()
        } else {
            self.default_shell.clone()
        }
    }

    fn shell_mode_label(&self) -> String {
        match self.shell_mode {
            crate::config::ShellModeConfig::Auto => "auto".to_string(),
            crate::config::ShellModeConfig::Login => "login".to_string(),
            crate::config::ShellModeConfig::NonLogin => "non_login".to_string(),
        }
    }

    fn new_terminal_cwd_label(&self) -> String {
        match &self.new_terminal_cwd {
            crate::config::NewTerminalCwdConfig::Follow => "follow".to_string(),
            crate::config::NewTerminalCwdConfig::Home => "home".to_string(),
            crate::config::NewTerminalCwdConfig::Current => "current".to_string(),
            crate::config::NewTerminalCwdConfig::Path(path) => path.clone(),
        }
    }

    fn toast_herdr_position_label(&self) -> String {
        format!("{:?}", self.toast_config.herdr.position)
            .to_ascii_lowercase()
            .replace('_', " ")
    }

    fn clipboard_toast_label(&self) -> String {
        if self.toast_config.clipboard.enabled {
            format!(
                "on · {:?}",
                self.toast_config.clipboard.position
            )
            .to_ascii_lowercase()
            .replace('_', " ")
        } else {
            "off".to_string()
        }
    }

    fn host_cursor_label(&self) -> String {
        match self.settings.config_snapshot.host_cursor {
            crate::config::HostCursorModeConfig::Auto => "auto".to_string(),
            crate::config::HostCursorModeConfig::Native => "native".to_string(),
            crate::config::HostCursorModeConfig::Drawn => "drawn".to_string(),
        }
    }
}
