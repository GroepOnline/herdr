use crate::{
    api::schema::InstalledPluginInfo,
    app::state::{AgentPanelSort, AppState, ExperimentSetting},
    config::{
        HostCursorModeConfig, NewTerminalCwdConfig, ShellModeConfig, SidebarCollapsedModeConfig,
        SpinnerStyle, ToastClipboardPosition, ToastDelivery, ToastHerdrPosition,
        UpdateChannelConfig,
    },
    pane_template::PaneTemplateId,
};

use super::spinner::active_spinner_category;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsItemId {
    ThemeAutoSwitch,
    Theme { index: usize },
    SpinnerPreview,
    Spinner { index: usize },
    PaneBorders,
    PaneGaps,
    AgentLabels,
    HideTabBar,
    SidebarCollapsedMode,
    AgentPanelSort,
    PaneTemplate { index: usize },
    MouseCapture,
    CopyOnSelect,
    RedrawOnFocusGained,
    ConfirmClose,
    PromptNewTabName,
    PromptNewWorkspaceName,
    HostCursor,
    KeybindHelp,
    DefaultShell,
    ShellMode,
    NewTerminalCwd,
    ScrollbackPreset { index: usize },
    SoundAlerts,
    ToastDelivery { delivery: ToastDelivery },
    ToastDelay,
    ToastHerdrPosition,
    ClipboardToast,
    ResumeAgentsOnRestore,
    Integration { index: usize },
    IntegrationsEmpty,
    PluginsInstalledHeader,
    InstalledPlugin { index: usize },
    PluginsEmpty,
    PluginsCatalogHeader,
    CatalogPlugin { index: usize },
    UpdateChannelStable,
    UpdateChannelPreview,
    VersionCheck,
    ManifestCheck,
    Experiment(ExperimentSetting),
    ManageSshConfig,
    ClipboardHistory,
    FleetOpsBar,
    WorktreesPath,
    ReloadConfig,
    ConfigFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
// The shared `Save` verb is semantic: these actions persist settings.
#[allow(clippy::enum_variant_names)]
pub(crate) enum SettingsAction {
    SaveTheme(String),
    SaveSound(bool),
    SaveToastDelivery(ToastDelivery),
    SaveAgentBorderLabels(bool),
    SavePaneBorders(bool),
    SavePaneGaps(bool),
    SaveHideTabBarWhenSingleTab(bool),
    SavePaneHistory(bool),
    SaveSwitchAsciiInputSourceInPrefix(bool),
    SaveSpinnerStyle(SpinnerStyle),
    ApplyPaneTemplate(PaneTemplateId),
    InstallRecommendedIntegrations,
    SaveMouseCapture(bool),
    SaveCopyOnSelect(bool),
    SaveConfirmClose(bool),
    SavePromptNewTabName(bool),
    SavePromptNewWorkspaceName(bool),
    SaveRedrawOnFocusGained(bool),
    SaveHostCursor(HostCursorModeConfig),
    SaveSidebarCollapsedMode(SidebarCollapsedModeConfig),
    SaveAgentPanelSort(AgentPanelSort),
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
    SaveFleetOpsBar(bool),
    TogglePluginEnabled { plugin_id: String, enabled: bool },
    InstallCatalogPlugin { source: String },
}

const TOAST_DELAY_PRESETS: &[u64] = &[0, 1, 2, 5];

pub(crate) fn scrollback_presets() -> &'static [(usize, &'static str)] {
    &[
        (1_000_000, "1 MB"),
        (5_000_000, "5 MB"),
        (10_000_000, "10 MB"),
        (50_000_000, "50 MB"),
    ]
}

const DEFAULT_SHELL_PRESETS: &[&str] = &["", "bash", "zsh", "fish", "nu"];

#[derive(Debug, Clone, Copy)]
pub(crate) struct PluginCatalogEntry {
    pub name: &'static str,
    pub blurb: &'static str,
    pub source: &'static str,
    pub plugin_id: &'static str,
}

pub(crate) const PLUGIN_CATALOG: &[PluginCatalogEntry] = &[
    PluginCatalogEntry {
        name: "Linear issues",
        blurb: "issue · assignee · cycle on the ops bar",
        source: "OnlineChefGroep/herdr-plugins/chef-linear-context",
        plugin_id: "com.chefgroep.linear-context",
    },
    PluginCatalogEntry {
        name: "GitHub checks",
        blurb: "PR number and CI status on the ops bar",
        source: "OnlineChefGroep/herdr-plugins/chef-github-status",
        plugin_id: "com.chefgroep.github-status",
    },
    PluginCatalogEntry {
        name: "Fleet health",
        blurb: "which hosts are online",
        source: "OnlineChefGroep/herdr-plugins/chef-fleet-health",
        plugin_id: "com.chefgroep.fleet-health",
    },
    PluginCatalogEntry {
        name: "Cloudflare tunnels",
        blurb: "tunnel health at a glance",
        source: "OnlineChefGroep/herdr-plugins/chef-cloudflare-tunnel",
        plugin_id: "com.chefgroep.cloudflare-tunnel",
    },
    PluginCatalogEntry {
        name: "Kater bridge",
        blurb: "Utrecht fleet inventory",
        source: "OnlineChefGroep/herdr-plugins/chef-kater-bridge",
        plugin_id: "com.chefgroep.kater-bridge",
    },
    PluginCatalogEntry {
        name: "Session park",
        blurb: "park and resume long-lived sessions",
        source: "OnlineChefGroep/herdr-plugins/chef-session-park",
        plugin_id: "com.chefgroep.session-park",
    },
];

pub(crate) fn installed_plugins_sorted<'a>(app: &'a AppState) -> Vec<&'a InstalledPluginInfo> {
    let mut plugins = app.installed_plugins.values().collect::<Vec<_>>();
    plugins.sort_by(|left, right| left.plugin_id.cmp(&right.plugin_id));
    plugins
}

pub(crate) fn catalog_entries_available<'a>(app: &'a AppState) -> Vec<&'a PluginCatalogEntry> {
    PLUGIN_CATALOG
        .iter()
        .filter(|entry| !app.installed_plugins.contains_key(entry.plugin_id))
        .collect()
}

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

fn cycle_agent_panel_sort(current: AgentPanelSort) -> AgentPanelSort {
    match current {
        AgentPanelSort::Spaces => AgentPanelSort::Priority,
        AgentPanelSort::Priority => AgentPanelSort::Spaces,
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

fn experiment_toggle_action(
    state: &AppState,
    setting: ExperimentSetting,
) -> Option<SettingsAction> {
    match setting {
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

fn integrations_need_install(state: &AppState) -> bool {
    state
        .integration_recommendations
        .iter()
        .any(crate::integration::IntegrationRecommendation::needs_install)
}

pub(crate) fn activate_item(state: &AppState, id: SettingsItemId) -> Option<SettingsAction> {
    match id {
        SettingsItemId::ThemeAutoSwitch => Some(SettingsAction::SaveThemeAutoSwitch(
            !state.settings.config_snapshot.theme_auto_switch,
        )),
        SettingsItemId::Spinner { index } => {
            active_spinner_category(state.settings.spinner_category)
                .styles
                .get(index)
                .copied()
                .map(SettingsAction::SaveSpinnerStyle)
        }
        SettingsItemId::PaneBorders => Some(SettingsAction::SavePaneBorders(
            !state.pane_borders_enabled(),
        )),
        SettingsItemId::PaneGaps => Some(SettingsAction::SavePaneGaps(!state.pane_gaps_enabled())),
        SettingsItemId::AgentLabels => Some(SettingsAction::SaveAgentBorderLabels(
            !state.agent_border_labels_enabled(),
        )),
        SettingsItemId::HideTabBar => Some(SettingsAction::SaveHideTabBarWhenSingleTab(
            !state.hide_tab_bar_when_single_tab_enabled(),
        )),
        SettingsItemId::SidebarCollapsedMode => Some(SettingsAction::SaveSidebarCollapsedMode(
            cycle_sidebar_collapsed_mode(state.sidebar_collapsed_mode),
        )),
        SettingsItemId::AgentPanelSort => Some(SettingsAction::SaveAgentPanelSort(
            cycle_agent_panel_sort(state.agent_panel_sort),
        )),
        SettingsItemId::PaneTemplate { index } => PaneTemplateId::ALL
            .get(index)
            .copied()
            .map(SettingsAction::ApplyPaneTemplate),
        SettingsItemId::MouseCapture => {
            Some(SettingsAction::SaveMouseCapture(!state.mouse_capture))
        }
        SettingsItemId::CopyOnSelect => {
            Some(SettingsAction::SaveCopyOnSelect(!state.copy_on_select))
        }
        SettingsItemId::RedrawOnFocusGained => Some(SettingsAction::SaveRedrawOnFocusGained(
            !state.redraw_on_focus_gained,
        )),
        SettingsItemId::ConfirmClose => {
            Some(SettingsAction::SaveConfirmClose(!state.confirm_close))
        }
        SettingsItemId::PromptNewTabName => Some(SettingsAction::SavePromptNewTabName(
            !state.prompt_new_tab_name,
        )),
        SettingsItemId::PromptNewWorkspaceName => Some(SettingsAction::SavePromptNewWorkspaceName(
            !state.prompt_new_workspace_name,
        )),
        SettingsItemId::HostCursor => Some(SettingsAction::SaveHostCursor(cycle_host_cursor(
            state.settings.config_snapshot.host_cursor,
        ))),
        SettingsItemId::DefaultShell => Some(SettingsAction::SaveDefaultShell(
            cycle_default_shell(&state.default_shell),
        )),
        SettingsItemId::ShellMode => Some(SettingsAction::SaveShellMode(cycle_shell_mode(
            state.shell_mode,
        ))),
        SettingsItemId::NewTerminalCwd => Some(SettingsAction::SaveNewTerminalCwd(
            cycle_new_terminal_cwd(&state.new_terminal_cwd),
        )),
        SettingsItemId::ScrollbackPreset { index } => scrollback_presets()
            .get(index)
            .map(|(bytes, _)| SettingsAction::SaveScrollbackLimitBytes(*bytes)),
        SettingsItemId::Integration { .. } if integrations_need_install(state) => {
            Some(SettingsAction::InstallRecommendedIntegrations)
        }
        SettingsItemId::SoundAlerts => Some(SettingsAction::SaveSound(!state.sound_enabled())),
        SettingsItemId::ToastDelay => Some(SettingsAction::SaveToastDelaySeconds(
            cycle_toast_delay(state.toast_config.delay_seconds),
        )),
        SettingsItemId::ToastHerdrPosition => Some(SettingsAction::SaveToastHerdrPosition(
            cycle_toast_herdr_position(state.toast_config.herdr.position),
        )),
        SettingsItemId::ClipboardToast => {
            if !state.toast_config.clipboard.enabled {
                return Some(SettingsAction::SaveClipboardToastEnabled(true));
            }
            let next = cycle_clipboard_toast_position(state.toast_config.clipboard.position);
            if next == ToastClipboardPosition::TopLeft
                && state.toast_config.clipboard.position == ToastClipboardPosition::BottomLeft
            {
                return Some(SettingsAction::SaveClipboardToastEnabled(false));
            }
            Some(SettingsAction::SaveClipboardToastPosition(next))
        }
        SettingsItemId::ToastDelivery { delivery } => {
            Some(SettingsAction::SaveToastDelivery(delivery))
        }
        SettingsItemId::ResumeAgentsOnRestore => Some(SettingsAction::SaveResumeAgentsOnRestore(
            !state.settings.config_snapshot.resume_agents_on_restore,
        )),
        SettingsItemId::UpdateChannelStable => Some(SettingsAction::SaveUpdateChannel(
            UpdateChannelConfig::Stable,
        )),
        SettingsItemId::UpdateChannelPreview => Some(SettingsAction::SaveUpdateChannel(
            UpdateChannelConfig::Preview,
        )),
        SettingsItemId::VersionCheck => Some(SettingsAction::SaveVersionCheck(
            !state.settings.config_snapshot.version_check,
        )),
        SettingsItemId::ManifestCheck => Some(SettingsAction::SaveManifestCheck(
            !state.settings.config_snapshot.manifest_check,
        )),
        SettingsItemId::Experiment(setting) => experiment_toggle_action(state, setting),
        SettingsItemId::ManageSshConfig => Some(SettingsAction::SaveManageSshConfig(
            !state.settings.config_snapshot.manage_ssh_config,
        )),
        SettingsItemId::ClipboardHistory => Some(SettingsAction::SaveClipboardHistoryEnabled(
            !state.settings.config_snapshot.clipboard_history_enabled,
        )),
        SettingsItemId::FleetOpsBar => Some(SettingsAction::SaveFleetOpsBar(
            !state.fleet_ops_bar_enabled(),
        )),
        SettingsItemId::InstalledPlugin { index } => installed_plugins_sorted(state)
            .get(index)
            .map(|plugin| SettingsAction::TogglePluginEnabled {
                plugin_id: plugin.plugin_id.clone(),
                enabled: !plugin.enabled,
            }),
        SettingsItemId::CatalogPlugin { index } => {
            catalog_entries_available(state).get(index).map(|entry| {
                SettingsAction::InstallCatalogPlugin {
                    source: entry.source.to_string(),
                }
            })
        }
        SettingsItemId::Theme { .. }
        | SettingsItemId::SpinnerPreview
        | SettingsItemId::KeybindHelp
        | SettingsItemId::Integration { .. }
        | SettingsItemId::IntegrationsEmpty
        | SettingsItemId::PluginsInstalledHeader
        | SettingsItemId::PluginsEmpty
        | SettingsItemId::PluginsCatalogHeader
        | SettingsItemId::WorktreesPath
        | SettingsItemId::ReloadConfig
        | SettingsItemId::ConfigFile => None,
    }
}

pub(crate) fn catalog_plugin_index(id: SettingsItemId) -> Option<usize> {
    if let SettingsItemId::CatalogPlugin { index } = id {
        Some(index)
    } else {
        None
    }
}

pub(crate) fn installed_plugin_index(id: SettingsItemId) -> Option<usize> {
    if let SettingsItemId::InstalledPlugin { index } = id {
        Some(index)
    } else {
        None
    }
}

pub(crate) fn pane_template_index(id: SettingsItemId) -> Option<usize> {
    if let SettingsItemId::PaneTemplate { index } = id {
        Some(index)
    } else {
        None
    }
}

pub(crate) fn integration_index(id: SettingsItemId) -> Option<usize> {
    if let SettingsItemId::Integration { index } = id {
        Some(index)
    } else {
        None
    }
}

pub(crate) fn spinner_index(id: SettingsItemId) -> Option<usize> {
    if let SettingsItemId::Spinner { index } = id {
        Some(index)
    } else {
        None
    }
}

pub(crate) fn theme_index(id: SettingsItemId) -> Option<usize> {
    if let SettingsItemId::Theme { index } = id {
        Some(index)
    } else {
        None
    }
}
