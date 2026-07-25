//! View model for the compact Fleet Ops line.

use crate::app::state::AppState;
use crate::fleet::ops::{FleetOpsMetadata, FleetOpsBarKind};

/// Display-ready Fleet Ops context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetOpsContext {
    pub summary: String,
    pub expanded: String,
    pub has_data: bool,
}

/// Build a compact one-line summary from the active pane's Fleet Ops metadata.
pub fn build_fleet_ops_context(app: &AppState) -> FleetOpsContext {
    let Some(meta) = active_pane_metadata(app) else {
        return FleetOpsContext {
            summary: String::new(),
            expanded: String::new(),
            has_data: false,
        };
    };

    let parts = meta.bar_parts("agent", crate::detect::AgentState::Working, None);
    let summary_parts: Vec<String> = parts
        .iter()
        .filter(|p| !p.text.is_empty() && p.kind != FleetOpsBarKind::Name)
        .map(|p| p.text.clone())
        .collect();

    let expanded = parts
        .iter()
        .map(|p| p.text.clone())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");

    FleetOpsContext {
        summary: summary_parts.join(" · "),
        expanded,
        has_data: true,
    }
}

fn active_pane_metadata(app: &AppState) -> Option<FleetOpsMetadata> {
    let ws_idx = app.active?;
    let ws = app.workspaces.get(ws_idx)?;
    let active_pane = ws.focused_pane().or_else(|| {
        ws.tabs
            .iter()
            .flat_map(|tab| tab.panes.values())
            .next()
    })?;
    let terminal = app.terminals.get(&active_pane.attached_terminal_id)?;
    let host = crate::platform::hostname().unwrap_or_else(|| "local".to_string());
    Some(FleetOpsMetadata::from_terminal(terminal, &host, &app.fleet_ops_cache))
}
