//! View model for the global launcher.

use crate::app::state::AppState;
use crate::ui::shell::SidebarMode;

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
        for (tab_idx, tab) in ws.tabs.iter().enumerate() {
            items.push(LauncherItem {
                kind: LauncherKind::Tab,
                primary: tab.label.clone(),
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
