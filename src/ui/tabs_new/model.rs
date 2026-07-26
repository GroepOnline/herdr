//! View model for the new compact tab bar.

use crate::app::state::AppState;
use crate::detect::AgentState;
use crate::workspace::{Tab, Workspace};

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
        .map(|(idx, tab)| tab_item_from_runtime(ws, idx, tab, active_tab == idx, app))
        .collect()
}

fn tab_item_from_runtime(
    ws: &Workspace,
    idx: usize,
    tab: &Tab,
    active: bool,
    app: &AppState,
) -> TabItem {
    let (state, seen) = tab.aggregate_state(&app.terminals);
    let working = state == AgentState::Working;
    let blocked = state == AgentState::Blocked;
    let unseen_done = state == AgentState::Idle && !seen;
    TabItem {
        index: idx,
        label: ws
            .tab_display_name(idx)
            .unwrap_or_else(|| (idx + 1).to_string()),
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

    #[test]
    fn build_tabs_returns_empty_for_no_active_workspace() {
        let mut app = AppState::test_new();
        app.active = None;
        let tabs = build_tabs(&app);
        assert!(tabs.is_empty());
    }

    #[test]
    fn build_tabs_handles_missing_workspace() {
        let mut app = AppState::test_new();
        app.active = Some(999);
        let tabs = build_tabs(&app);
        assert!(tabs.is_empty());
    }

    #[test]
    fn tab_item_status_icon_reflects_state() {
        let blocked = TabItem {
            index: 0,
            label: "b".into(),
            state: AgentState::Blocked,
            seen: false,
            active: false,
            working: false,
            blocked: true,
            unseen_done: false,
        };
        assert_eq!(blocked.clone().status_icon(), "◉");

        let working = TabItem {
            index: 0,
            label: "w".into(),
            state: AgentState::Working,
            seen: false,
            active: false,
            working: true,
            blocked: false,
            unseen_done: false,
        };
        assert_eq!(working.clone().status_icon(), "●");

        let unseen = TabItem {
            index: 0,
            label: "u".into(),
            state: AgentState::Idle,
            seen: false,
            active: false,
            working: false,
            blocked: false,
            unseen_done: true,
        };
        assert_eq!(unseen.clone().status_icon(), "✓");

        let seen = TabItem {
            index: 0,
            label: "s".into(),
            state: AgentState::Idle,
            seen: true,
            active: false,
            working: false,
            blocked: false,
            unseen_done: false,
        };
        assert_eq!(seen.clone().status_icon(), "");
    }
}
