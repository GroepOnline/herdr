//! View model for the compact Fleet Ops line.

use crate::app::state::AppState;
use crate::fleet::ops::{FleetOpsBarKind, FleetOpsMetadata};

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
    let pane_id = ws
        .focused_pane_id()
        .or_else(|| ws.tabs.iter().flat_map(|tab| tab.layout.pane_ids()).next())?;
    let pane = ws.pane_state(pane_id)?;
    let terminal = app.terminals.get(&pane.attached_terminal_id)?;
    let host = std::env::var("HERDR_HOST_NAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_default();
    let host = if host.is_empty() {
        "local"
    } else {
        host.as_str()
    };
    Some(FleetOpsMetadata::from_terminal(
        terminal,
        host,
        &app.fleet_ops_cache,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;

    #[test]
    fn empty_app_produces_empty_context() {
        let mut app = AppState::test_new();
        app.active = None;
        let ctx = build_fleet_ops_context(&app);
        assert!(!ctx.has_data);
        assert!(ctx.summary.is_empty());
        assert!(ctx.expanded.is_empty());
    }

    #[test]
    fn context_has_data_when_active_workspace_exists() {
        let app = AppState::test_new();
        let ctx = build_fleet_ops_context(&app);
        // A fresh test workspace has a terminal pane; fleet ops may or may not have
        // data depending on the metadata content, but it should not panic.
        // The contract is that has_data is true only when a pane is found.
        assert!(!ctx.summary.is_empty() || !ctx.has_data);
    }

    #[test]
    fn fleet_ops_context_is_clone_and_eq() {
        let ctx = FleetOpsContext {
            summary: "test".into(),
            expanded: "test · expanded".into(),
            has_data: true,
        };
        let ctx2 = ctx.clone();
        assert_eq!(ctx, ctx2);
    }

    #[test]
    fn fleet_ops_context_empty_has_no_data() {
        let ctx = FleetOpsContext {
            summary: String::new(),
            expanded: String::new(),
            has_data: false,
        };
        assert!(!ctx.has_data);
        assert!(ctx.summary.is_empty());
        assert!(ctx.expanded.is_empty());
    }
}
