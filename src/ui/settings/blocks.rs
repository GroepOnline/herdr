use crate::app::{state::SettingsSection, AppState};

use super::{
    catalog::{catalog_plugin_id, SettingsItemId},
    rows::{section_rows, SettingsRow, SettingsRowKind},
    widgets::{checklist_group_height, SETTING_BLOCK_HEADER_ROWS},
};

// Section specification for the premium block layout (headers/checklists/chips).

pub(crate) fn uses_block_layout(section: SettingsSection) -> bool {
    matches!(
        section,
        SettingsSection::Layout
            | SettingsSection::Notifications
            | SettingsSection::Input
            | SettingsSection::Terminal
            | SettingsSection::Agents
            | SettingsSection::Plugins
            | SettingsSection::Updates
            | SettingsSection::Advanced
    )
}

#[derive(Debug, Clone)]
pub(super) enum ContentBlock {
    Header {
        label: &'static str,
    },
    Checklist {
        label: &'static str,
        row_indices: Vec<usize>,
    },
    Chips {
        row_indices: Vec<usize>,
        labels: Vec<String>,
    },
    Plain {
        row_index: usize,
    },
}

#[derive(Debug, Clone)]
struct PlacedBlock {
    y: u16,
    height: u16,
    block: ContentBlock,
}

pub(super) fn row_index_for_id(rows: &[SettingsRow], id: SettingsItemId) -> Option<usize> {
    rows.iter().position(|row| row.id == id)
}

fn row_indices_for_ids(rows: &[SettingsRow], ids: &[SettingsItemId]) -> Vec<usize> {
    ids.iter()
        .filter_map(|id| row_index_for_id(rows, *id))
        .collect()
}

fn checklist_height(item_count: usize) -> u16 {
    checklist_group_height(item_count)
}

pub(super) fn block_height(block: &ContentBlock) -> u16 {
    match block {
        ContentBlock::Header { .. } => SETTING_BLOCK_HEADER_ROWS,
        ContentBlock::Checklist { row_indices, .. } => checklist_height(row_indices.len()),
        ContentBlock::Chips { .. } => 1,
        ContentBlock::Plain { .. } => 1,
    }
}

pub(super) fn build_blocks(app: &AppState, section: SettingsSection) -> Vec<ContentBlock> {
    let rows = section_rows(app, section);
    let mut blocks = Vec::new();

    match section {
        SettingsSection::Layout => {
            let chrome = row_indices_for_ids(
                &rows,
                &[
                    SettingsItemId::PaneBorders,
                    SettingsItemId::PaneGaps,
                    SettingsItemId::AgentLabels,
                    SettingsItemId::HideTabBar,
                ],
            );
            if !chrome.is_empty() {
                blocks.push(ContentBlock::Header { label: "chrome" });
                blocks.push(ContentBlock::Checklist {
                    label: "components",
                    row_indices: chrome,
                });
            }

            for id in [
                SettingsItemId::SidebarCollapsedMode,
                SettingsItemId::AgentPanelSort,
            ] {
                if let Some(idx) = row_index_for_id(&rows, id) {
                    blocks.push(ContentBlock::Plain { row_index: idx });
                }
            }

            let templates: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter_map(|(idx, row)| {
                    if matches!(row.kind, SettingsRowKind::Template) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
            if !templates.is_empty() {
                blocks.push(ContentBlock::Header { label: "templates" });
                for idx in templates {
                    blocks.push(ContentBlock::Plain { row_index: idx });
                }
            }
        }
        SettingsSection::Notifications => {
            if let Some(idx) = row_index_for_id(&rows, SettingsItemId::SoundAlerts) {
                blocks.push(ContentBlock::Header { label: "sound" });
                blocks.push(ContentBlock::Checklist {
                    label: "alerts",
                    row_indices: vec![idx],
                });
            }

            let deliveries: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter_map(|(idx, row)| {
                    if matches!(row.id, SettingsItemId::ToastDelivery { .. }) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
            if !deliveries.is_empty() {
                blocks.push(ContentBlock::Header { label: "toasts" });
                let labels = notifications_toast_delivery_labels()
                    .iter()
                    .map(|(label, _)| (*label).to_string())
                    .collect();
                blocks.push(ContentBlock::Chips {
                    row_indices: deliveries,
                    labels,
                });
            }

            for id in [
                SettingsItemId::ToastDelay,
                SettingsItemId::ToastHerdrPosition,
                SettingsItemId::ClipboardToast,
            ] {
                if let Some(idx) = row_index_for_id(&rows, id) {
                    blocks.push(ContentBlock::Plain { row_index: idx });
                }
            }
        }
        SettingsSection::Input => {
            let pointer = row_indices_for_ids(
                &rows,
                &[SettingsItemId::MouseCapture, SettingsItemId::CopyOnSelect],
            );
            if !pointer.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "pointer & clipboard",
                });
                blocks.push(ContentBlock::Checklist {
                    label: "toggles",
                    row_indices: pointer,
                });
            }

            let prompts = row_indices_for_ids(
                &rows,
                &[
                    SettingsItemId::RedrawOnFocusGained,
                    SettingsItemId::ConfirmClose,
                    SettingsItemId::PromptNewTabName,
                    SettingsItemId::PromptNewWorkspaceName,
                ],
            );
            if !prompts.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "focus & prompts",
                });
                blocks.push(ContentBlock::Checklist {
                    label: "toggles",
                    row_indices: prompts,
                });
            }

            let host_cursor: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter_map(|(idx, row)| {
                    matches!(row.id, SettingsItemId::HostCursor { .. }).then_some(idx)
                })
                .collect();
            if !host_cursor.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "host cursor",
                });
                let labels = host_cursor
                    .iter()
                    .filter_map(|idx| rows.get(*idx).map(|row| row.label.clone()))
                    .collect();
                blocks.push(ContentBlock::Chips {
                    row_indices: host_cursor,
                    labels,
                });
            }

            if let Some(idx) = row_index_for_id(&rows, SettingsItemId::KeybindHelp) {
                blocks.push(ContentBlock::Plain { row_index: idx });
            }
        }
        SettingsSection::Terminal => {
            if let Some(idx) = row_index_for_id(&rows, SettingsItemId::DefaultShell) {
                blocks.push(ContentBlock::Header { label: "shell" });
                blocks.push(ContentBlock::Plain { row_index: idx });
            }

            let shell_mode: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter_map(|(idx, row)| {
                    matches!(row.id, SettingsItemId::ShellMode { .. }).then_some(idx)
                })
                .collect();
            if !shell_mode.is_empty() {
                let labels = shell_mode
                    .iter()
                    .filter_map(|idx| rows.get(*idx).map(|row| row.label.clone()))
                    .collect();
                blocks.push(ContentBlock::Chips {
                    row_indices: shell_mode,
                    labels,
                });
            }

            let new_cwd: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter_map(|(idx, row)| {
                    matches!(row.id, SettingsItemId::NewTerminalCwd { .. }).then_some(idx)
                })
                .collect();
            if !new_cwd.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "new pane cwd",
                });
                let labels = new_cwd
                    .iter()
                    .filter_map(|idx| rows.get(*idx).map(|row| row.label.clone()))
                    .collect();
                blocks.push(ContentBlock::Chips {
                    row_indices: new_cwd,
                    labels,
                });
            }

            let scrollback: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter_map(|(idx, row)| {
                    if matches!(row.id, SettingsItemId::ScrollbackPreset { .. }) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
            if !scrollback.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "scrollback limit",
                });
                let labels = scrollback
                    .iter()
                    .filter_map(|idx| rows.get(*idx))
                    .filter_map(|row| {
                        if let SettingsItemId::ScrollbackPreset { index } = row.id {
                            super::catalog::scrollback_presets()
                                .get(index)
                                .map(|(_, label)| (*label).to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                blocks.push(ContentBlock::Chips {
                    row_indices: scrollback,
                    labels,
                });
            }
        }
        SettingsSection::Updates => {
            let channels = row_indices_for_ids(
                &rows,
                &[
                    SettingsItemId::UpdateChannelStable,
                    SettingsItemId::UpdateChannelPreview,
                ],
            );
            if !channels.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "release channel",
                });
                let labels = channels
                    .iter()
                    .filter_map(|idx| rows.get(*idx))
                    .map(|row| row.label.clone())
                    .collect();
                blocks.push(ContentBlock::Chips {
                    row_indices: channels,
                    labels,
                });
            }

            let checks = row_indices_for_ids(
                &rows,
                &[SettingsItemId::VersionCheck, SettingsItemId::ManifestCheck],
            );
            if !checks.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "background checks",
                });
                blocks.push(ContentBlock::Checklist {
                    label: "toggles",
                    row_indices: checks,
                });
            }
        }
        SettingsSection::Advanced => {
            let experiments: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter_map(|(idx, row)| {
                    if matches!(row.id, SettingsItemId::Experiment(_)) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
            if !experiments.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "experiments",
                });
                blocks.push(ContentBlock::Checklist {
                    label: "flags",
                    row_indices: experiments,
                });
            }

            let ops = row_indices_for_ids(
                &rows,
                &[
                    SettingsItemId::FleetOpsBar,
                    SettingsItemId::ManageSshConfig,
                    SettingsItemId::ClipboardHistory,
                ],
            );
            if !ops.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "operations",
                });
                blocks.push(ContentBlock::Checklist {
                    label: "toggles",
                    row_indices: ops,
                });
            }

            let notes = row_indices_for_ids(
                &rows,
                &[
                    SettingsItemId::WorktreesPath,
                    SettingsItemId::ReloadConfig,
                    SettingsItemId::ConfigFile,
                ],
            );
            if !notes.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "paths & config",
                });
                for idx in notes {
                    blocks.push(ContentBlock::Plain { row_index: idx });
                }
            }
        }
        SettingsSection::Agents => {
            if let Some(idx) = row_index_for_id(&rows, SettingsItemId::ResumeAgentsOnRestore) {
                blocks.push(ContentBlock::Header {
                    label: "session restore",
                });
                blocks.push(ContentBlock::Checklist {
                    label: "toggles",
                    row_indices: vec![idx],
                });
            }

            let integrations: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter_map(|(idx, row)| {
                    if matches!(row.id, SettingsItemId::Integration { .. }) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
            if !integrations.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "agent CLIs",
                });
                for idx in integrations {
                    blocks.push(ContentBlock::Plain { row_index: idx });
                }
            } else if row_index_for_id(&rows, SettingsItemId::IntegrationsEmpty).is_some() {
                blocks.push(ContentBlock::Header {
                    label: "agent CLIs",
                });
                blocks.push(ContentBlock::Plain {
                    row_index: row_index_for_id(&rows, SettingsItemId::IntegrationsEmpty)
                        .expect("empty integrations row"),
                });
            }
        }
        SettingsSection::Plugins => {
            let installed: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter_map(|(idx, row)| {
                    if matches!(row.id, SettingsItemId::InstalledPlugin { .. }) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
            blocks.push(ContentBlock::Header {
                label: "your plugins",
            });
            if installed.is_empty() {
                if let Some(idx) = row_index_for_id(&rows, SettingsItemId::PluginsEmpty) {
                    blocks.push(ContentBlock::Plain { row_index: idx });
                }
            } else {
                blocks.push(ContentBlock::Checklist {
                    label: "installed",
                    row_indices: installed,
                });
            }

            let catalog: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter_map(|(idx, row)| {
                    if catalog_plugin_id(row.id).is_some() {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
            if !catalog.is_empty() {
                blocks.push(ContentBlock::Header {
                    label: "available to install",
                });
                for idx in catalog {
                    blocks.push(ContentBlock::Plain { row_index: idx });
                }
            }
        }
        _ => {}
    }

    blocks
}
