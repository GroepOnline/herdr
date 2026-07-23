use crate::{
    app::state::{AppState, SettingsSection, THEME_NAMES},
    config::ToastDelivery,
    pane_template::PaneTemplateId,
};

use super::{
    catalog::{
        catalog_entries_available, installed_plugins_sorted, scrollback_presets, SettingsItemId,
    },
    spinner::active_spinner_category,
};

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
    pub id: SettingsItemId,
}

fn matches_filter(filter: &str, label: &str, detail: Option<&str>) -> bool {
    if filter.is_empty() {
        return true;
    }
    let needle = filter.to_ascii_lowercase();
    label.to_ascii_lowercase().contains(&needle)
        || detail.is_some_and(|d| d.to_ascii_lowercase().contains(&needle))
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
                id: SettingsItemId::ThemeAutoSwitch,
            });
            for (idx, name) in THEME_NAMES.iter().enumerate() {
                rows.push(SettingsRow {
                    label: (*name).to_string(),
                    detail: None,
                    kind: SettingsRowKind::Theme,
                    id: SettingsItemId::Theme { index: idx },
                });
            }
            let category = active_spinner_category(app.settings.spinner_category);
            for (idx, style) in category.styles.iter().enumerate() {
                let frames = style.frames();
                let trail = frames
                    .iter()
                    .take(5)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" ");
                rows.push(SettingsRow {
                    label: style.label().to_string(),
                    detail: Some(trail),
                    kind: SettingsRowKind::Spinner,
                    id: SettingsItemId::Spinner { index: idx },
                });
            }
        }
        SettingsSection::Layout => {
            for (label, detail, id) in [
                (
                    "pane borders",
                    "draw borders around split panes",
                    SettingsItemId::PaneBorders,
                ),
                (
                    "pane gaps",
                    "keep split panes visually separated",
                    SettingsItemId::PaneGaps,
                ),
                (
                    "agent labels",
                    "show agent names in pane borders",
                    SettingsItemId::AgentLabels,
                ),
                (
                    "hide tab bar",
                    "hide tab row when only one tab",
                    SettingsItemId::HideTabBar,
                ),
            ] {
                rows.push(SettingsRow {
                    label: label.to_string(),
                    detail: Some(detail.to_string()),
                    kind: SettingsRowKind::Toggle,
                    id,
                });
            }
            rows.push(SettingsRow {
                label: "sidebar collapsed mode".to_string(),
                detail: Some(app.sidebar_collapsed_mode_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::SidebarCollapsedMode,
            });
            rows.push(SettingsRow {
                label: "agent panel sort".to_string(),
                detail: Some(app.agent_panel_sort_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::AgentPanelSort,
            });
            for (idx, id) in PaneTemplateId::ALL.iter().enumerate() {
                let tmpl = id.template();
                rows.push(SettingsRow {
                    label: tmpl.name.to_string(),
                    detail: Some(tmpl.description.to_string()),
                    kind: SettingsRowKind::Template,
                    id: SettingsItemId::PaneTemplate { index: idx },
                });
            }
        }
        SettingsSection::Input => {
            for (label, detail, id) in [
                (
                    "mouse capture",
                    "capture mouse for Herdr UI chrome",
                    SettingsItemId::MouseCapture,
                ),
                (
                    "copy on select",
                    "copy selected terminal text to clipboard",
                    SettingsItemId::CopyOnSelect,
                ),
                (
                    "redraw on focus gained",
                    "refresh panes when Herdr regains focus",
                    SettingsItemId::RedrawOnFocusGained,
                ),
                (
                    "confirm close",
                    "ask before closing tabs and workspaces",
                    SettingsItemId::ConfirmClose,
                ),
                (
                    "prompt new tab name",
                    "ask for a name when creating tabs",
                    SettingsItemId::PromptNewTabName,
                ),
                (
                    "prompt new workspace name",
                    "ask for a name when creating workspaces",
                    SettingsItemId::PromptNewWorkspaceName,
                ),
            ] {
                rows.push(SettingsRow {
                    label: label.to_string(),
                    detail: Some(detail.to_string()),
                    kind: SettingsRowKind::Toggle,
                    id,
                });
            }
            rows.push(SettingsRow {
                label: "host cursor".to_string(),
                detail: Some(app.host_cursor_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::HostCursor,
            });
            let prefix = crate::config::format_key_combo((app.prefix_code, app.prefix_mods));
            rows.push(SettingsRow {
                label: "keybind help".to_string(),
                detail: Some(format!("press {prefix}+? or open prefix help")),
                kind: SettingsRowKind::Note,
                id: SettingsItemId::KeybindHelp,
            });
        }
        SettingsSection::Terminal => {
            rows.push(SettingsRow {
                label: "default shell".to_string(),
                detail: Some(app.default_shell_display()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::DefaultShell,
            });
            rows.push(SettingsRow {
                label: "shell mode".to_string(),
                detail: Some(app.shell_mode_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::ShellMode,
            });
            rows.push(SettingsRow {
                label: "new pane cwd".to_string(),
                detail: Some(app.new_terminal_cwd_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::NewTerminalCwd,
            });
            for (idx, (_bytes, label)) in scrollback_presets().iter().enumerate() {
                rows.push(SettingsRow {
                    label: format!("scrollback {label}"),
                    detail: None,
                    kind: SettingsRowKind::Choice,
                    id: SettingsItemId::ScrollbackPreset { index: idx },
                });
            }
        }
        SettingsSection::Notifications => {
            rows.push(SettingsRow {
                label: "sound alerts".to_string(),
                detail: None,
                kind: SettingsRowKind::Toggle,
                id: SettingsItemId::SoundAlerts,
            });
            for delivery in [
                ToastDelivery::Off,
                ToastDelivery::Herdr,
                ToastDelivery::Terminal,
                ToastDelivery::System,
            ] {
                let label = match delivery {
                    ToastDelivery::Off => "toast off",
                    ToastDelivery::Herdr => "toast inside herdr",
                    ToastDelivery::Terminal => "toast via terminal",
                    ToastDelivery::System => "toast via system",
                };
                rows.push(SettingsRow {
                    label: label.to_string(),
                    detail: None,
                    kind: SettingsRowKind::Choice,
                    id: SettingsItemId::ToastDelivery { delivery },
                });
            }
            rows.push(SettingsRow {
                label: "toast delay".to_string(),
                detail: Some(format!("{}s", app.toast_config.delay_seconds)),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::ToastDelay,
            });
            rows.push(SettingsRow {
                label: "herdr toast position".to_string(),
                detail: Some(app.toast_herdr_position_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::ToastHerdrPosition,
            });
            rows.push(SettingsRow {
                label: "clipboard toast".to_string(),
                detail: Some(app.clipboard_toast_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::ClipboardToast,
            });
        }
        SettingsSection::Agents => {
            rows.push(SettingsRow {
                label: "resume agents on restore".to_string(),
                detail: Some("resume supported agent sessions when restoring".to_string()),
                kind: SettingsRowKind::Toggle,
                id: SettingsItemId::ResumeAgentsOnRestore,
            });
            for (idx, item) in app.integration_recommendations.iter().enumerate() {
                rows.push(SettingsRow {
                    label: item.label.to_string(),
                    detail: Some(item.status_label().to_string()),
                    kind: SettingsRowKind::Integration,
                    id: SettingsItemId::Integration { index: idx },
                });
            }
            if app.integration_recommendations.is_empty() {
                rows.push(SettingsRow {
                    label: "no supported agent CLIs found on PATH".to_string(),
                    detail: None,
                    kind: SettingsRowKind::Note,
                    id: SettingsItemId::IntegrationsEmpty,
                });
            }
        }
        SettingsSection::Plugins => {
            rows.push(SettingsRow {
                label: "your plugins".to_string(),
                detail: Some("toggle to enable or disable".to_string()),
                kind: SettingsRowKind::Note,
                id: SettingsItemId::PluginsInstalledHeader,
            });
            let installed = installed_plugins_sorted(app);
            if installed.is_empty() {
                rows.push(SettingsRow {
                    label: "nothing installed yet".to_string(),
                    detail: Some("pick something below to add".to_string()),
                    kind: SettingsRowKind::Note,
                    id: SettingsItemId::PluginsEmpty,
                });
            } else {
                for (index, plugin) in installed.iter().enumerate() {
                    rows.push(SettingsRow {
                        label: plugin.name.clone(),
                        detail: Some(if plugin.enabled {
                            "on".to_string()
                        } else {
                            "off".to_string()
                        }),
                        kind: SettingsRowKind::Toggle,
                        id: SettingsItemId::InstalledPlugin { index },
                    });
                }
            }
            let catalog = catalog_entries_available(app);
            if !catalog.is_empty() {
                rows.push(SettingsRow {
                    label: "available to install".to_string(),
                    detail: Some("enter installs in one step".to_string()),
                    kind: SettingsRowKind::Note,
                    id: SettingsItemId::PluginsCatalogHeader,
                });
                for (index, entry) in catalog.iter().enumerate() {
                    rows.push(SettingsRow {
                        label: entry.name.to_string(),
                        detail: Some(entry.blurb.to_string()),
                        kind: SettingsRowKind::Integration,
                        id: SettingsItemId::CatalogPlugin { index },
                    });
                }
            }
        }
        SettingsSection::Updates => {
            rows.push(SettingsRow {
                label: "stable channel".to_string(),
                detail: None,
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::UpdateChannelStable,
            });
            rows.push(SettingsRow {
                label: "preview channel".to_string(),
                detail: None,
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::UpdateChannelPreview,
            });
            rows.push(SettingsRow {
                label: "version check".to_string(),
                detail: Some("check for herdr updates in the background".to_string()),
                kind: SettingsRowKind::Toggle,
                id: SettingsItemId::VersionCheck,
            });
            rows.push(SettingsRow {
                label: "manifest check".to_string(),
                detail: Some("check for agent detection manifest updates".to_string()),
                kind: SettingsRowKind::Toggle,
                id: SettingsItemId::ManifestCheck,
            });
        }
        SettingsSection::Advanced => {
            for setting in crate::app::state::ExperimentSetting::ALL {
                rows.push(SettingsRow {
                    label: setting.label().to_string(),
                    detail: None,
                    kind: SettingsRowKind::Toggle,
                    id: SettingsItemId::Experiment(setting),
                });
            }
            rows.push(SettingsRow {
                label: "fleet ops bar".to_string(),
                detail: Some("show fleet operations bar above the terminal".to_string()),
                kind: SettingsRowKind::Toggle,
                id: SettingsItemId::FleetOpsBar,
            });
            for (label, detail, id) in [
                (
                    "manage ssh config",
                    "add keepalive fallbacks for herdr --remote",
                    SettingsItemId::ManageSshConfig,
                ),
                (
                    "clipboard history",
                    "retain recent global clipboard entries",
                    SettingsItemId::ClipboardHistory,
                ),
            ] {
                rows.push(SettingsRow {
                    label: label.to_string(),
                    detail: Some(detail.to_string()),
                    kind: SettingsRowKind::Toggle,
                    id,
                });
            }
            rows.push(SettingsRow {
                label: "worktrees path".to_string(),
                detail: Some(app.worktree_directory.display().to_string()),
                kind: SettingsRowKind::Note,
                id: SettingsItemId::WorktreesPath,
            });
            rows.push(SettingsRow {
                label: "reload config".to_string(),
                detail: Some("prefix reload or herdr server reload-config".to_string()),
                kind: SettingsRowKind::Note,
                id: SettingsItemId::ReloadConfig,
            });
            rows.push(SettingsRow {
                label: "config file".to_string(),
                detail: Some(crate::config::config_path().display().to_string()),
                kind: SettingsRowKind::Note,
                id: SettingsItemId::ConfigFile,
            });
        }
    }

    rows.retain(|row| matches_filter(filter, &row.label, row.detail.as_deref()));
    rows
}

pub(crate) fn row_toggle_checked(app: &AppState, _section: SettingsSection, row: &SettingsRow) -> bool {
    match row.id {
        SettingsItemId::ThemeAutoSwitch => app.settings.config_snapshot.theme_auto_switch,
        SettingsItemId::PaneBorders => app.pane_borders_enabled(),
        SettingsItemId::PaneGaps => app.pane_gaps_enabled(),
        SettingsItemId::AgentLabels => app.agent_border_labels_enabled(),
        SettingsItemId::HideTabBar => app.hide_tab_bar_when_single_tab_enabled(),
        SettingsItemId::MouseCapture => app.mouse_capture,
        SettingsItemId::CopyOnSelect => app.copy_on_select,
        SettingsItemId::RedrawOnFocusGained => app.redraw_on_focus_gained,
        SettingsItemId::ConfirmClose => app.confirm_close,
        SettingsItemId::PromptNewTabName => app.prompt_new_tab_name,
        SettingsItemId::PromptNewWorkspaceName => app.prompt_new_workspace_name,
        SettingsItemId::SoundAlerts => app.sound_enabled(),
        SettingsItemId::ResumeAgentsOnRestore => app.settings.config_snapshot.resume_agents_on_restore,
        SettingsItemId::VersionCheck => app.settings.config_snapshot.version_check,
        SettingsItemId::ManifestCheck => app.settings.config_snapshot.manifest_check,
        SettingsItemId::Experiment(setting) => setting.enabled(app),
        SettingsItemId::ManageSshConfig => app.settings.config_snapshot.manage_ssh_config,
        SettingsItemId::ClipboardHistory => app.settings.config_snapshot.clipboard_history_enabled,
        SettingsItemId::FleetOpsBar => app.fleet_ops_bar_enabled(),
        SettingsItemId::InstalledPlugin { index } => installed_plugins_sorted(app)
            .get(index)
            .is_some_and(|plugin| plugin.enabled),
        _ => false,
    }
}

pub(crate) fn row_choice_selected(app: &AppState, _section: SettingsSection, row: &SettingsRow) -> bool {
    match row.id {
        SettingsItemId::SidebarCollapsedMode
        | SettingsItemId::AgentPanelSort
        | SettingsItemId::HostCursor
        | SettingsItemId::DefaultShell
        | SettingsItemId::ShellMode
        | SettingsItemId::NewTerminalCwd
        | SettingsItemId::ToastDelay
        | SettingsItemId::ToastHerdrPosition
        | SettingsItemId::ClipboardToast => true,
        SettingsItemId::ScrollbackPreset { index } => scrollback_presets()
            .get(index)
            .is_some_and(|(bytes, _)| app.pane_scrollback_limit_bytes == *bytes),
        SettingsItemId::UpdateChannelStable => {
            app.settings.config_snapshot.update_channel
                == crate::config::UpdateChannelConfig::Stable
        }
        SettingsItemId::UpdateChannelPreview => {
            app.settings.config_snapshot.update_channel
                == crate::config::UpdateChannelConfig::Preview
        }
        SettingsItemId::ToastDelivery { delivery } => app.toast_delivery() == delivery,
        _ => false,
    }
}

pub(crate) fn row_theme_current(app: &AppState, row: &SettingsRow) -> bool {
    if let SettingsItemId::Theme { index } = row.id {
        THEME_NAMES
            .get(index)
            .is_some_and(|name| themes_match(name, &app.theme_name))
    } else {
        false
    }
}

pub(crate) fn row_spinner_current(app: &AppState, row: &SettingsRow) -> bool {
    if let SettingsItemId::Spinner { index } = row.id {
        active_spinner_category(app.settings.spinner_category)
            .styles
            .get(index)
            .copied()
            .is_some_and(|style| style == app.spinner_style)
    } else {
        false
    }
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
            format!("on · {:?}", self.toast_config.clipboard.position)
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
