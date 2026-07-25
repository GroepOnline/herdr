//! View model for the new compact tab bar.

use crate::app::state::AppState;
use crate::detect::AgentState;
use crate::workspace::aggregate::Tab;

/// Display-ready tab row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem {
    pub index: usize,
    pub label: String,
    pub state: AgentState,
    pub seen: bool,
    pub active: bool,
    pub working: bool,
    pub blocked: bool,
    pub unseen_done: bool,
}

impl TabItem {
    pub fn status_icon(self) -> &'static str {
        if self.blocked {
            "◉"
        } else if self.working {
            "●"
        } else if self.unseen_done {
            "✓"
        } else {
            ""
        }
    }
}

/// Build tab view models from the active workspace.
pub fn build_tabs(app: &AppState) -> Vec<TabItem> {
    let Some(ws_idx) = app.active else {
        return Vec::new();
    };
    let Some(ws) = app.workspaces.get(ws_idx) else {
        return Vec::new();
    };

    let active_tab = ws.active_tab;
    ws.tabs
        .iter()
        .enumerate()
        .map(|(idx, tab)| tab_item_from_runtime(idx, tab, active_tab == Some(idx), &app))
        .collect()
}

fn tab_item_from_runtime(idx: usize, tab: &Tab, active: bool, app: &AppState) -> TabItem {
    let (state, seen) = tab.aggregate_state(&app.terminals);
    let working = state == AgentState::Working;
    let blocked = state == AgentState::Blocked;
    let unseen_done = state == AgentState::Idle && !seen;
    TabItem {
        index: idx,
        label: tab.label.clone(),
        state,
        seen,
        active,
        working,
        blocked,
        unseen_done,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;

    #[test]
    fn build_tabs_reflects_active_tab() {
        let app = AppState::test_new();
        let tabs = build_tabs(&app);
        assert!(!tabs.is_empty());
        assert!(tabs.iter().any(|t| t.active));
    }
}
