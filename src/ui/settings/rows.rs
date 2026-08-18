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
    Header,
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
    /// Extra haystack for settings search (e.g. plugin id / source).
    pub search_extra: Option<String>,
}

fn header_row(label: &str) -> SettingsRow {
    SettingsRow {
        label: label.to_string(),
        detail: None,
        kind: SettingsRowKind::Header,
        id: SettingsItemId::Header,
        search_extra: None,
    }
}

fn matches_filter(filter: &str, label: &str, detail: Option<&str>, extra: Option<&str>) -> bool {
    if filter.is_empty() {
        return true;
    }
    let needle = filter.to_ascii_lowercase();
    label.to_ascii_lowercase().contains(&needle)
        || detail.is_some_and(|d| d.to_ascii_lowercase().contains(&needle))
        || extra.is_some_and(|e| e.to_ascii_lowercase().contains(&needle))
}

pub(crate) fn section_rows(app: &AppState, section: SettingsSection) -> Vec<SettingsRow> {
    let filter = app.settings.search.as_str();
    let show_headers = filter.is_empty();
    let mut rows = Vec::new();

    match section {
        SettingsSection::Theme => {
            if show_headers {
                rows.push(header_row("theme"));
            }
            rows.push(SettingsRow {
                label: "auto-switch theme with host".to_string(),
                detail: Some("follow terminal light/dark appearance".to_string()),
                kind: SettingsRowKind::Toggle,
                id: SettingsItemId::ThemeAutoSwitch,
                search_extra: None,
            });
            for (idx, name) in THEME_NAMES.iter().enumerate() {
                rows.push(SettingsRow {
                    label: (*name).to_string(),
                    detail: None,
                    kind: SettingsRowKind::Theme,
                    id: SettingsItemId::Theme { index: idx },
                    search_extra: None,
                });
            }
        }
        SettingsSection::Ui => {
            if show_headers {
                rows.push(header_row("spinner"));
            }
            let category = active_spinner_category(app.settings.spinner_category);
            for (idx, style) in category.styles.iter().enumerate() {
                let frames = style.frames();
                let trail = frames.iter().take(5).copied().collect::<Vec<_>>().join(" ");
                rows.push(SettingsRow {
                    label: style.label().to_string(),
                    detail: Some(trail),
                    kind: SettingsRowKind::Spinner,
                    id: SettingsItemId::Spinner { index: idx },
                    search_extra: None,
                });
            }
            if show_headers {
                rows.push(header_row("status indicators"));
            }
            for (idx, (label, detail)) in [
                ("dots", "animated spinner dots"),
                ("symbols", "distinct static shapes per state"),
            ]
            .iter()
            .enumerate()
            {
                rows.push(SettingsRow {
                    label: (*label).to_string(),
                    detail: Some((*detail).to_string()),
                    kind: SettingsRowKind::Choice,
                    id: SettingsItemId::StatusIndicators { index: idx },
                    search_extra: None,
                });
            }
            if show_headers {
                rows.push(header_row("pane chrome"));
            }
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
                    search_extra: None,
                });
            }
            if show_headers {
                rows.push(header_row("sidebar"));
            }
            for (label, detail, id) in [
                (
                    "sidebar width",
                    format!(
                        "{} columns · {}–{}",
                        app.sidebar_width, app.sidebar_min_width, app.sidebar_max_width
                    ),
                    SettingsItemId::SidebarWidth,
                ),
                (
                    "collapsed mode",
                    app.sidebar_collapsed_mode_label(),
                    SettingsItemId::SidebarCollapsedMode,
                ),
                (
                    "agent ordering",
                    app.agent_panel_sort_label(),
                    SettingsItemId::AgentPanelSort,
                ),
                (
                    "agent row gap",
                    format!("{} rows", app.sidebar_agents.row_gap),
                    SettingsItemId::SidebarAgentRowGap,
                ),
                (
                    "workspace row gap",
                    format!("{} rows", app.sidebar_spaces.row_gap),
                    SettingsItemId::SidebarSpaceRowGap,
                ),
            ] {
                rows.push(SettingsRow {
                    label: label.to_string(),
                    detail: Some(detail.to_string()),
                    kind: SettingsRowKind::Choice,
                    id,
                    search_extra: None,
                });
            }
            rows.push(SettingsRow {
                label: "token layout".to_string(),
                detail: Some(
                    "edit [ui.sidebar] rows in config.toml · typed tokens + per-agent overrides"
                        .to_string(),
                ),
                kind: SettingsRowKind::Note,
                id: SettingsItemId::ConfigFile,
                search_extra: Some("agents spaces rows rows_by_agent token custom".to_string()),
            });
            if show_headers {
                rows.push(header_row("mouse & clipboard"));
            }
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
                    search_extra: None,
                });
            }
            if show_headers {
                rows.push(header_row("cursor & keys"));
            }
            rows.push(SettingsRow {
                label: "host cursor".to_string(),
                detail: Some(app.host_cursor_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::HostCursor,
                search_extra: None,
            });
            let prefix = crate::config::format_key_combo((app.prefix_code, app.prefix_mods));
            rows.push(SettingsRow {
                label: "keybind help".to_string(),
                detail: Some(format!("press {prefix}+? or open prefix help")),
                kind: SettingsRowKind::Note,
                id: SettingsItemId::KeybindHelp,
                search_extra: None,
            });
        }
        SettingsSection::Sound => {
            if show_headers {
                rows.push(header_row("sound"));
            }
            rows.push(SettingsRow {
                label: "sound alerts".to_string(),
                detail: None,
                kind: SettingsRowKind::Toggle,
                id: SettingsItemId::SoundAlerts,
                search_extra: None,
            });
            if show_headers {
                rows.push(header_row("toasts"));
            }
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
                    search_extra: None,
                });
            }
            rows.push(SettingsRow {
                label: "toast delay".to_string(),
                detail: Some(format!("{}s", app.toast_config.delay_seconds)),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::ToastDelay,
                search_extra: None,
            });
            rows.push(SettingsRow {
                label: "herdr toast position".to_string(),
                detail: Some(app.toast_herdr_position_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::ToastHerdrPosition,
                search_extra: None,
            });
            rows.push(SettingsRow {
                label: "clipboard toast".to_string(),
                detail: Some(app.clipboard_toast_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::ClipboardToast,
                search_extra: None,
            });
        }
        SettingsSection::System => {
            if show_headers {
                rows.push(header_row("shell"));
            }
            rows.push(SettingsRow {
                label: "default shell".to_string(),
                detail: Some(app.default_shell_display()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::DefaultShell,
                search_extra: None,
            });
            rows.push(SettingsRow {
                label: "shell mode".to_string(),
                detail: Some(app.shell_mode_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::ShellMode,
                search_extra: None,
            });
            rows.push(SettingsRow {
                label: "new pane cwd".to_string(),
                detail: Some(app.new_terminal_cwd_label()),
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::NewTerminalCwd,
                search_extra: None,
            });
            if show_headers {
                rows.push(header_row("scrollback"));
            }
            for (idx, (_bytes, label)) in scrollback_presets().iter().enumerate() {
                rows.push(SettingsRow {
                    label: format!("scrollback {label}"),
                    detail: None,
                    kind: SettingsRowKind::Choice,
                    id: SettingsItemId::ScrollbackPreset { index: idx },
                    search_extra: None,
                });
            }
            if show_headers {
                rows.push(header_row("updates"));
            }
            rows.push(SettingsRow {
                label: "stable channel".to_string(),
                detail: None,
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::UpdateChannelStable,
                search_extra: None,
            });
            rows.push(SettingsRow {
                label: "preview channel".to_string(),
                detail: None,
                kind: SettingsRowKind::Choice,
                id: SettingsItemId::UpdateChannelPreview,
                search_extra: None,
            });
            rows.push(SettingsRow {
                label: "version check".to_string(),
                detail: Some("check for herdr updates in the background".to_string()),
                kind: SettingsRowKind::Toggle,
                id: SettingsItemId::VersionCheck,
                search_extra: None,
            });
            rows.push(SettingsRow {
                label: "manifest check".to_string(),
                detail: Some("check for agent detection manifest updates".to_string()),
                kind: SettingsRowKind::Toggle,
                id: SettingsItemId::ManifestCheck,
                search_extra: None,
            });
            if show_headers {
                rows.push(header_row("experiments"));
            }
            for setting in crate::app::state::ExperimentSetting::ALL {
                rows.push(SettingsRow {
                    label: setting.label().to_string(),
                    detail: None,
                    kind: SettingsRowKind::Toggle,
                    id: SettingsItemId::Experiment(setting),
                    search_extra: None,
                });
            }
            if show_headers {
                rows.push(header_row("system"));
            }
            rows.push(SettingsRow {
                label: "fleet ops bar".to_string(),
                detail: Some("show fleet operations bar above the terminal".to_string()),
                kind: SettingsRowKind::Toggle,
                id: SettingsItemId::FleetOpsBar,
                search_extra: None,
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
                    search_extra: None,
                });
            }
            if show_headers {
                rows.push(header_row("paths & config"));
            }
            rows.push(SettingsRow {
                label: "worktrees path".to_string(),
                detail: Some(app.worktree_directory.display().to_string()),
                kind: SettingsRowKind::Note,
                id: SettingsItemId::WorktreesPath,
                search_extra: None,
            });
            rows.push(SettingsRow {
                label: "reload config".to_string(),
                detail: Some("prefix reload or herdr server reload-config".to_string()),
                kind: SettingsRowKind::Note,
                id: SettingsItemId::ReloadConfig,
                search_extra: None,
            });
            rows.push(SettingsRow {
                label: "config file".to_string(),
                detail: Some(crate::config::config_path().display().to_string()),
                kind: SettingsRowKind::Note,
                id: SettingsItemId::ConfigFile,
                search_extra: None,
            });
        }
        SettingsSection::Templates => {
            if show_headers {
                rows.push(header_row("templates"));
            }
            for (idx, id) in PaneTemplateId::ALL.iter().enumerate() {
                let tmpl = id.template();
                rows.push(SettingsRow {
                    label: tmpl.name.to_string(),
                    detail: Some(tmpl.description.to_string()),
                    kind: SettingsRowKind::Template,
                    id: SettingsItemId::PaneTemplate { index: idx },
                    search_extra: None,
                });
            }
        }
        SettingsSection::Integrations => {
            if show_headers {
                rows.push(header_row("sessions"));
            }
            rows.push(SettingsRow {
                label: "resume agents on restore".to_string(),
                detail: Some("resume supported agent sessions when restoring".to_string()),
                kind: SettingsRowKind::Toggle,
                id: SettingsItemId::ResumeAgentsOnRestore,
                search_extra: None,
            });
            if show_headers {
                rows.push(header_row("integrations"));
            }
            for (idx, item) in app.integration_recommendations.iter().enumerate() {
                rows.push(SettingsRow {
                    label: item.label.to_string(),
                    detail: Some(item.status_label().to_string()),
                    kind: SettingsRowKind::Integration,
                    id: SettingsItemId::Integration { index: idx },
                    search_extra: None,
                });
            }
            if app.integration_recommendations.is_empty() {
                rows.push(SettingsRow {
                    label: "no supported agent CLIs found on PATH".to_string(),
                    detail: None,
                    kind: SettingsRowKind::Note,
                    id: SettingsItemId::IntegrationsEmpty,
                    search_extra: None,
                });
            }
            if show_headers {
                rows.push(header_row("your plugins"));
            }
            let installed = installed_plugins_sorted(app);
            if installed.is_empty() {
                rows.push(SettingsRow {
                    label: "nothing installed yet".to_string(),
                    detail: Some("pick something below to add".to_string()),
                    kind: SettingsRowKind::Note,
                    id: SettingsItemId::PluginsEmpty,
                    search_extra: None,
                });
            } else {
                for (index, plugin) in installed.iter().enumerate() {
                    let source_label = plugin_source_search_label(plugin);
                    rows.push(SettingsRow {
                        label: plugin.name.clone(),
                        detail: Some(if plugin.enabled {
                            "on".to_string()
                        } else {
                            "off".to_string()
                        }),
                        kind: SettingsRowKind::Toggle,
                        id: SettingsItemId::InstalledPlugin { index },
                        search_extra: Some(format!("{} {source_label}", plugin.plugin_id)),
                    });
                }
            }
            let catalog = catalog_entries_available(app);
            if !catalog.is_empty() {
                if show_headers {
                    rows.push(header_row("available to install"));
                }
                for entry in catalog {
                    rows.push(SettingsRow {
                        label: entry.name.to_string(),
                        detail: Some(entry.blurb.to_string()),
                        kind: SettingsRowKind::Integration,
                        id: SettingsItemId::CatalogPlugin {
                            plugin_id: entry.plugin_id,
                        },
                        search_extra: Some(format!(
                            "{} {} {}",
                            entry.blurb, entry.source, entry.plugin_id
                        )),
                    });
                }
            }
        }
    }

    // Collapse: hide children of collapsed groups. Headers only exist when the
    // search filter is empty, so collapsing is a no-op while searching.
    if filter.is_empty() {
        let mut collapse_next = false;
        rows.retain(|row| {
            if row.kind == SettingsRowKind::Header {
                collapse_next = app.settings.collapsed_groups.contains(&row.label);
                true
            } else {
                !collapse_next
            }
        });
    }

    rows.retain(|row| {
        matches_filter(
            filter,
            &row.label,
            row.detail.as_deref(),
            row.search_extra.as_deref(),
        )
    });
    rows
}

fn plugin_source_search_label(plugin: &crate::api::schema::InstalledPluginInfo) -> String {
    let source = &plugin.source;
    match (&source.owner, &source.repo) {
        (Some(owner), Some(repo)) => {
            let mut label = format!("{owner}/{repo}");
            if let Some(subdir) = &source.subdir {
                label.push('/');
                label.push_str(subdir);
            }
            label
        }
        _ => plugin.plugin_id.clone(),
    }
}

pub(crate) fn row_toggle_checked(
    app: &AppState,
    _section: SettingsSection,
    row: &SettingsRow,
) -> bool {
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
        SettingsItemId::ResumeAgentsOnRestore => {
            app.settings.config_snapshot.resume_agents_on_restore
        }
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

pub(crate) fn row_choice_selected(
    app: &AppState,
    _section: SettingsSection,
    row: &SettingsRow,
) -> bool {
    match row.id {
        SettingsItemId::SidebarWidth
        | SettingsItemId::SidebarCollapsedMode
        | SettingsItemId::AgentPanelSort
        | SettingsItemId::SidebarAgentRowGap
        | SettingsItemId::SidebarSpaceRowGap
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
        SettingsItemId::StatusIndicators { index } => {
            (index == 0) == (app.status_indicators == crate::config::StatusIndicatorStyle::Dots)
        }
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

pub(crate) fn first_selectable_index(state: &AppState, section: SettingsSection) -> usize {
    section_rows(state, section)
        .iter()
        .position(|row| row.kind != SettingsRowKind::Header)
        .unwrap_or(0)
}

pub(crate) fn selected_settings_row_id(state: &AppState) -> Option<SettingsItemId> {
    section_rows(state, state.settings.section)
        .get(state.settings.list.selected)
        .map(|row| row.id)
}

pub(crate) fn clamp_settings_list_selection(
    state: &mut AppState,
    previous_id: Option<SettingsItemId>,
) {
    if state.settings.section != SettingsSection::Integrations {
        return;
    }
    let rows = section_rows(state, state.settings.section);
    if let Some(id) = previous_id {
        if let Some(idx) = rows.iter().position(|row| row.id == id) {
            state.settings.list.selected = idx;
            return;
        }
    }
    if rows.is_empty() {
        state.settings.list.selected = 0;
        return;
    }
    state.settings.list.selected = state.settings.list.selected.min(rows.len() - 1);
    if rows[state.settings.list.selected].kind == SettingsRowKind::Header {
        state.settings.list.selected = first_selectable_index(state, state.settings.section);
    }
}
