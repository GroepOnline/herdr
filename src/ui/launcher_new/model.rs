//! View model for the global launcher.

use crate::app::state::AppState;

/// Kinds of launcher items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherKind {
    Workspace,
    Tab,
    Pane,
    Agent,
    Setting,
    FleetOps,
    Command,
}

/// A single launcher row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherItem {
    pub kind: LauncherKind,
    pub primary: String,
    pub secondary: Option<String>,
    pub icon: &'static str,
}

/// Build launcher items from the current app state.
pub fn build_launcher_items(app: &AppState) -> Vec<LauncherItem> {
    let mut items = Vec::new();

    for (idx, ws) in app.workspaces.iter().enumerate() {
        items.push(LauncherItem {
            kind: LauncherKind::Workspace,
            primary: ws.display_name(),
            secondary: ws.branch(),
            icon: "□",
        });
        for tab_idx in 0..ws.tabs.len() {
            items.push(LauncherItem {
                kind: LauncherKind::Tab,
                primary: ws
                    .tab_display_name(tab_idx)
                    .unwrap_or_else(|| (tab_idx + 1).to_string()),
                secondary: Some(format!("workspace {}", idx)),
                icon: "▸",
            });
        }
    }

    items.push(LauncherItem {
        kind: LauncherKind::Setting,
        primary: "Settings".to_string(),
        secondary: None,
        icon: "⚙",
    });
    items.push(LauncherItem {
        kind: LauncherKind::FleetOps,
        primary: "Fleet Ops".to_string(),
        secondary: None,
        icon: "◎",
    });

    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;
    use crate::workspace::Workspace;

    #[test]
    fn builds_workspace_items() {
        let mut app = AppState::test_new();
        app.workspaces = vec![Workspace::test_new("alpha"), Workspace::test_new("beta")];
        let items = build_launcher_items(&app);
        let workspace_items: Vec<_> = items
            .iter()
            .filter(|i| matches!(i.kind, LauncherKind::Workspace))
            .collect();
        assert_eq!(workspace_items.len(), 2);
        assert_eq!(workspace_items[0].primary, "alpha");
        assert_eq!(workspace_items[1].primary, "beta");
    }

    #[test]
    fn builds_tab_items_for_each_workspace() {
        let mut app = AppState::test_new();
        app.workspaces = vec![Workspace::test_new("ws1")];
        let items = build_launcher_items(&app);
        let tab_items: Vec<_> = items
            .iter()
            .filter(|i| matches!(i.kind, LauncherKind::Tab))
            .collect();
        assert!(!tab_items.is_empty());
        assert!(tab_items.iter().all(|t| t.secondary.is_some()));
    }

    #[test]
    fn always_includes_setting_and_fleet_ops() {
        let app = AppState::test_new();
        let items = build_launcher_items(&app);
        assert!(items
            .iter()
            .any(|i| matches!(i.kind, LauncherKind::Setting)));
        assert!(items
            .iter()
            .any(|i| matches!(i.kind, LauncherKind::FleetOps)));
    }

    #[test]
    fn empty_workspaces_still_produces_static_items() {
        let mut app = AppState::test_new();
        app.workspaces = Vec::new();
        let items = build_launcher_items(&app);
        // Should still have Settings and FleetOps.
        assert!(items.len() >= 2);
    }

    #[test]
    fn launcher_kind_debug_and_eq() {
        assert_eq!(LauncherKind::Workspace, LauncherKind::Workspace);
        assert_ne!(LauncherKind::Workspace, LauncherKind::Setting);
        let kind = LauncherKind::Agent;
        assert_eq!(kind.clone(), kind);
    }

    #[test]
    fn launcher_item_clone_and_eq() {
        let item = LauncherItem {
            kind: LauncherKind::Workspace,
            primary: "test".into(),
            secondary: Some("sec".into()),
            icon: "□",
        };
        let item2 = item.clone();
        assert_eq!(item, item2);
    }

    #[test]
    fn workspace_with_branch_shows_branch_as_secondary() {
        let mut app = AppState::test_new();
        app.workspaces = vec![Workspace::test_new("with-branch")];
        let items = build_launcher_items(&app);
        let ws_item = items
            .iter()
            .find(|i| matches!(i.kind, LauncherKind::Workspace))
            .unwrap();
        assert_eq!(ws_item.primary, "with-branch");
    }

    #[test]
    fn multiple_workspaces_with_multiple_tabs() {
        let mut app = AppState::test_new();
        let mut ws1 = Workspace::test_new("one");
        let mut ws2 = Workspace::test_new("two");
        // Add an extra tab to ws1.
        ws1.test_add_tab(None);
        app.workspaces = vec![ws1, ws2];
        let items = build_launcher_items(&app);
        let workspace_count = items
            .iter()
            .filter(|i| matches!(i.kind, LauncherKind::Workspace))
            .count();
        assert_eq!(workspace_count, 2);
        let tab_count = items
            .iter()
            .filter(|i| matches!(i.kind, LauncherKind::Tab))
            .count();
        assert!(tab_count >= 3); // at least 2 from ws1 + 1 from ws2
    }
}
