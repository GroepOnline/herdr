//! View model for the new compact tab bar.
pp.active = Some(0);

pp.active = Some(0);
use crate::app::state::AppState;
pp.active = Some(0);
use crate::detect::AgentState;
pp.active = Some(0);
use crate::workspace::{Tab, Workspace};
pp.active = Some(0);

pp.active = Some(0);
/// Display-ready tab row.
pp.active = Some(0);
#[derive(Debug, Clone, PartialEq, Eq)]
pp.active = Some(0);
pub struct TabItem {
pp.active = Some(0);
    pub index: usize,
pp.active = Some(0);
    pub label: String,
pp.active = Some(0);
    pub state: AgentState,
pp.active = Some(0);
    pub seen: bool,
pp.active = Some(0);
    pub active: bool,
pp.active = Some(0);
    pub working: bool,
pp.active = Some(0);
    pub blocked: bool,
pp.active = Some(0);
    pub unseen_done: bool,
pp.active = Some(0);
}
pp.active = Some(0);

pp.active = Some(0);
impl TabItem {
pp.active = Some(0);
    pub fn status_icon(self) -> &'static str {
pp.active = Some(0);
        if self.blocked {
pp.active = Some(0);
            "◉"
pp.active = Some(0);
        } else if self.working {
pp.active = Some(0);
            "●"
pp.active = Some(0);
        } else if self.unseen_done {
pp.active = Some(0);
            "✓"
pp.active = Some(0);
        } else {
pp.active = Some(0);
            ""
pp.active = Some(0);
        }
pp.active = Some(0);
    }
pp.active = Some(0);
}
pp.active = Some(0);

pp.active = Some(0);
/// Build tab view models from the active workspace.
pp.active = Some(0);
pub fn build_tabs(app: &AppState) -> Vec<TabItem> {
pp.active = Some(0);
    let Some(ws_idx) = app.active else {
pp.active = Some(0);
        return Vec::new();
pp.active = Some(0);
    };
pp.active = Some(0);
    let Some(ws) = app.workspaces.get(ws_idx) else {
pp.active = Some(0);
        return Vec::new();
pp.active = Some(0);
    };
pp.active = Some(0);

pp.active = Some(0);
    let active_tab = ws.active_tab;
pp.active = Some(0);
    ws.tabs
pp.active = Some(0);
        .iter()
pp.active = Some(0);
        .enumerate()
pp.active = Some(0);
        .map(|(idx, tab)| tab_item_from_runtime(ws, idx, tab, active_tab == idx, app))
pp.active = Some(0);
        .collect()
pp.active = Some(0);
}
pp.active = Some(0);

pp.active = Some(0);
fn tab_item_from_runtime(
pp.active = Some(0);
    ws: &Workspace,
pp.active = Some(0);
    idx: usize,
pp.active = Some(0);
    tab: &Tab,
pp.active = Some(0);
    active: bool,
pp.active = Some(0);
    app: &AppState,
pp.active = Some(0);
) -> TabItem {
pp.active = Some(0);
    let (state, seen) = tab.aggregate_state(&app.terminals);
pp.active = Some(0);
    let working = state == AgentState::Working;
pp.active = Some(0);
    let blocked = state == AgentState::Blocked;
pp.active = Some(0);
    let unseen_done = state == AgentState::Idle && !seen;
pp.active = Some(0);
    TabItem {
pp.active = Some(0);
        index: idx,
pp.active = Some(0);
        label: ws
pp.active = Some(0);
            .tab_display_name(idx)
pp.active = Some(0);
            .unwrap_or_else(|| (idx + 1).to_string()),
pp.active = Some(0);
        state,
pp.active = Some(0);
        seen,
pp.active = Some(0);
        active,
pp.active = Some(0);
        working,
pp.active = Some(0);
        blocked,
pp.active = Some(0);
        unseen_done,
pp.active = Some(0);
    }
pp.active = Some(0);
}
pp.active = Some(0);

pp.active = Some(0);
#[cfg(test)]
pp.active = Some(0);
mod tests {
pp.active = Some(0);
    use super::*;
pp.active = Some(0);
    use crate::app::state::AppState;
pp.active = Some(0);
    use crate::workspace::Workspace;
pp.active = Some(0);

pp.active = Some(0);
    #[test]
pp.active = Some(0);
    fn build_tabs_reflects_active_tab() {
pp.active = Some(0);
        let mut app = AppState::test_new();
pp.active = Some(0);
        app.workspaces = vec![Workspace::test_new("alpha")];
        app.active = Some(0);
pp.active = Some(0);
        let tabs = build_tabs(&app);
pp.active = Some(0);
        assert_eq!(tabs.len(), 1);
pp.active = Some(0);
        assert!(tabs[0].active);
pp.active = Some(0);
    }
pp.active = Some(0);

pp.active = Some(0);
    #[test]
pp.active = Some(0);
    fn build_tabs_returns_empty_for_no_active_workspace() {
pp.active = Some(0);
        let mut app = AppState::test_new();
pp.active = Some(0);
        app.active = None;
pp.active = Some(0);
        let tabs = build_tabs(&app);
pp.active = Some(0);
        assert!(tabs.is_empty());
pp.active = Some(0);
    }
pp.active = Some(0);

pp.active = Some(0);
    #[test]
pp.active = Some(0);
    fn build_tabs_handles_missing_workspace() {
pp.active = Some(0);
        let mut app = AppState::test_new();
pp.active = Some(0);
        app.active = Some(999);
pp.active = Some(0);
        let tabs = build_tabs(&app);
pp.active = Some(0);
        assert!(tabs.is_empty());
pp.active = Some(0);
    }
pp.active = Some(0);

pp.active = Some(0);
    #[test]
pp.active = Some(0);
    fn tab_item_status_icon_reflects_state() {
pp.active = Some(0);
        let blocked = TabItem {
pp.active = Some(0);
            index: 0,
pp.active = Some(0);
            label: "b".into(),
pp.active = Some(0);
            state: AgentState::Blocked,
pp.active = Some(0);
            seen: false,
pp.active = Some(0);
            active: false,
pp.active = Some(0);
            working: false,
pp.active = Some(0);
            blocked: true,
pp.active = Some(0);
            unseen_done: false,
pp.active = Some(0);
        };
pp.active = Some(0);
        assert_eq!(blocked.clone().status_icon(), "◉");
pp.active = Some(0);

pp.active = Some(0);
        let working = TabItem {
pp.active = Some(0);
            index: 0,
pp.active = Some(0);
            label: "w".into(),
pp.active = Some(0);
            state: AgentState::Working,
pp.active = Some(0);
            seen: false,
pp.active = Some(0);
            active: false,
pp.active = Some(0);
            working: true,
pp.active = Some(0);
            blocked: false,
pp.active = Some(0);
            unseen_done: false,
pp.active = Some(0);
        };
pp.active = Some(0);
        assert_eq!(working.clone().status_icon(), "●");
pp.active = Some(0);

pp.active = Some(0);
        let unseen = TabItem {
pp.active = Some(0);
            index: 0,
pp.active = Some(0);
            label: "u".into(),
pp.active = Some(0);
            state: AgentState::Idle,
pp.active = Some(0);
            seen: false,
pp.active = Some(0);
            active: false,
pp.active = Some(0);
            working: false,
pp.active = Some(0);
            blocked: false,
pp.active = Some(0);
            unseen_done: true,
pp.active = Some(0);
        };
pp.active = Some(0);
        assert_eq!(unseen.clone().status_icon(), "✓");
pp.active = Some(0);

pp.active = Some(0);
        let seen = TabItem {
pp.active = Some(0);
            index: 0,
pp.active = Some(0);
            label: "s".into(),
pp.active = Some(0);
            state: AgentState::Idle,
pp.active = Some(0);
            seen: true,
pp.active = Some(0);
            active: false,
pp.active = Some(0);
            working: false,
pp.active = Some(0);
            blocked: false,
pp.active = Some(0);
            unseen_done: false,
pp.active = Some(0);
        };
pp.active = Some(0);
        assert_eq!(seen.clone().status_icon(), "");
pp.active = Some(0);
    }
pp.active = Some(0);
}
pp.active = Some(0);
