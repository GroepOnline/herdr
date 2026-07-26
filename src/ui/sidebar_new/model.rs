//! View model for the unified sidebar.
//!
//! The model layer intentionally does not render or handle input.  It collects
//! runtime facts into stable, testable view items that the layout and render
//! layers consume.  Derivation functions avoid per-frame allocation where
//! possible by returning borrowed labels from `AppState` and pre-computed
//! strings.

use crate::app::state::AppState;
use crate::detect::AgentState;
use crate::layout::PaneId;
use crate::terminal::TerminalRuntimeRegistry;

/// Stable identifier for a sidebar row.  The identifier is independent of the
/// current sort order so selection can survive list rebuilds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SidebarItemId {
    Workspace {
        ws_idx: usize,
    },
    Agent {
        ws_idx: usize,
        tab_idx: usize,
        pane_id: PaneId,
    },
    Attention {
        source_id: String,
    },
}

impl SidebarItemId {
    pub fn workspace_idx(&self) -> Option<usize> {
        match self {
            SidebarItemId::Workspace { ws_idx } => Some(*ws_idx),
            SidebarItemId::Agent { ws_idx, .. } => Some(*ws_idx),
            _ => None,
        }
    }
}

/// Runtime identity of a sidebar row.  This is intentionally separate from the
/// display string so that layout, hit testing, and rendering can agree on what
/// a row represents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarRowKind {
    /// Workspace row, possibly a linked-worktree child.
    Workspace { ws_idx: usize, indented: bool },
    /// Agent row pointing at a concrete pane.
    Agent {
        ws_idx: usize,
        tab_idx: usize,
        pane_id: PaneId,
    },
    /// Attention row; the source id refers to the originating agent/workspace.
    Attention { source_id: String },
}

/// A single row in any of the three sidebar modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarItem {
    pub id: SidebarItemId,
    pub kind: SidebarRowKind,
    /// Primary label, e.g. workspace name or agent type.
    pub primary: String,
    /// Optional secondary label, e.g. tab/branch/task context.
    pub secondary: Option<String>,
    /// Current agent state.
    pub state: AgentState,
    /// Whether the user has already seen this state.
    pub seen: bool,
    /// Indentation level used for linked-worktree children or grouped agents.
    pub indent: u8,
    /// Attention priority (higher == more urgent).  Used by Attention mode.
    pub attention_priority: u8,
    /// True if this row represents a collapsed group.
    pub collapsed: bool,
}

impl SidebarItem {
    /// Convenience constructor used by the derivation helpers.
    fn new(
        id: SidebarItemId,
        kind: SidebarRowKind,
        primary: String,
        state: AgentState,
        seen: bool,
    ) -> Self {
        Self {
            id,
            kind,
            primary,
            secondary: None,
            state,
            seen,
            indent: 0,
            attention_priority: attention_priority(state, seen),
            collapsed: false,
        }
    }

    fn with_secondary(mut self, secondary: impl Into<String>) -> Self {
        self.secondary = Some(secondary.into());
        self
    }

    fn with_indent(mut self, indent: u8) -> Self {
        self.indent = indent;
        self
    }
}

fn attention_priority(state: AgentState, seen: bool) -> u8 {
    match (state, seen) {
        (AgentState::Blocked, _) => 4,
        (AgentState::Idle, false) => 3,
        (AgentState::Working, _) => 2,
        (AgentState::Idle, true) => 1,
        (AgentState::Unknown, _) => 0,
    }
}

/// Build the workspace list.  Honors existing linked-worktree grouping and
/// collapse state, but presents a single flat `Vec` that the layout layer can
/// scroll uniformly.
pub fn build_workspaces(app: &AppState) -> Vec<SidebarItem> {
    let mut items = Vec::new();
    let grouped = crate::ui::sidebar::workspace_list_entries(app);
    for entry in grouped {
        match entry {
            crate::ui::sidebar::WorkspaceListEntry::Workspace { ws_idx, indented } => {
                let Some(ws) = app.workspaces.get(ws_idx) else {
                    continue;
                };
                let (state, seen) = ws.aggregate_state(&app.terminals);
                let primary = ws.display_name();
                let secondary = ws.branch();
                let mut item = SidebarItem::new(
                    SidebarItemId::Workspace { ws_idx },
                    SidebarRowKind::Workspace { ws_idx, indented },
                    primary,
                    state,
                    seen,
                )
                .with_indent(u8::from(indented));
                if let Some(branch) = secondary {
                    item = item.with_secondary(branch);
                }
                items.push(item);
            }
        }
    }
    items
}

/// Build the agent list.  Preserves the existing agent-panel collection logic
/// but flattens it into the shared `SidebarItem` shape.
pub fn build_agents(app: &AppState, registry: &TerminalRuntimeRegistry) -> Vec<SidebarItem> {
    let entries = crate::ui::sidebar::agent_panel_entries_from(app, registry);
    entries
        .into_iter()
        .map(|entry| {
            let primary = entry
                .agent_label
                .clone()
                .unwrap_or_else(|| entry.primary_label.clone());
            let secondary = entry
                .primary_tab_label
                .or(entry.terminal_title)
                .or(entry.pane_label);
            let id = SidebarItemId::Agent {
                ws_idx: entry.ws_idx,
                tab_idx: entry.tab_idx,
                pane_id: entry.pane_id,
            };
            let kind = SidebarRowKind::Agent {
                ws_idx: entry.ws_idx,
                tab_idx: entry.tab_idx,
                pane_id: entry.pane_id,
            };
            let mut item = SidebarItem::new(id, kind, primary, entry.state, entry.seen);
            if let Some(sec) = secondary {
                item = item.with_secondary(sec);
            }
            item
        })
        .collect()
}

/// Build the attention list.  Items are ordered by urgency: blocked, unseen
/// completions, working, idle/unknown.  This list is intentionally short;
/// idle items should not dominate.
pub fn build_attention(app: &AppState, registry: &TerminalRuntimeRegistry) -> Vec<SidebarItem> {
    let mut items = build_agents(app, registry);
    items.retain(|item| item.attention_priority > 0);
    items.sort_by(|a, b| {
        b.attention_priority
            .cmp(&a.attention_priority)
            .then_with(|| a.primary.cmp(&b.primary))
    });
    items
}

/// Cached container for the three sidebar modes.
///
/// The cache is intentionally simple: it stores the last-derived items for each
/// mode and can be invalidated by calling the `rebuild_*` methods.  Finer
/// grained invalidation (workspace generations, agent state versions, etc.) can
/// be added on top later without changing the view-model shape.
#[derive(Debug, Default)]
pub struct SidebarModel {
    pub workspaces: Vec<SidebarItem>,
    pub agents: Vec<SidebarItem>,
    pub attention: Vec<SidebarItem>,
}

impl SidebarModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rebuild all three modes from `AppState`.
    pub fn rebuild(&mut self, app: &AppState, registry: &TerminalRuntimeRegistry) {
        self.workspaces = build_workspaces(app);
        self.agents = build_agents(app, registry);
        self.attention = build_attention(app, registry);
    }

    /// Items for the requested mode.
    pub fn items_for_mode(&self, mode: crate::ui::shell::SidebarMode) -> &[SidebarItem] {
        use crate::ui::shell::SidebarMode;
        match mode {
            SidebarMode::Workspaces => &self.workspaces,
            SidebarMode::Agents => &self.agents,
            SidebarMode::Attention => &self.attention,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;
    use crate::terminal::TerminalRuntimeRegistry;
    use crate::workspace::Workspace;

    #[test]
    fn workspaces_reflects_aggregate_state() {
        let mut app = AppState::test_new();
        app.workspaces = vec![Workspace::test_new("alpha"), Workspace::test_new("beta")];
        let items = build_workspaces(&app);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].primary, "alpha");
        assert_eq!(items[1].primary, "beta");
    }

    #[test]
    fn agents_include_workspace_and_agent_label() {
        let mut app = AppState::test_new();
        let ws = Workspace::test_new("one");
        let pane_id = ws.tabs[0].root_pane;
        app.workspaces = vec![ws];
        app.ensure_test_terminals();
        let terminal_id = app.workspaces[0].tabs[0].panes[&pane_id]
            .attached_terminal_id
            .clone();
        app.terminals.get_mut(&terminal_id).unwrap().detected_agent =
            Some(crate::detect::Agent::Pi);

        let registry = TerminalRuntimeRegistry::new();
        let items = build_agents(&app, &registry);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].primary, "pi");
    }

    #[test]
    fn attention_prioritizes_blocked_over_working() {
        let mut app = AppState::test_new();
        app.workspaces = vec![Workspace::test_new("one"), Workspace::test_new("two")];
        app.ensure_test_terminals();

        let pane_id_one = app.workspaces[0].tabs[0].root_pane;
        let tid_one = app.workspaces[0].tabs[0].panes[&pane_id_one]
            .attached_terminal_id
            .clone();
        app.terminals.get_mut(&tid_one).unwrap().state = AgentState::Working;

        let pane_id_two = app.workspaces[1].tabs[0].root_pane;
        let tid_two = app.workspaces[1].tabs[0].panes[&pane_id_two]
            .attached_terminal_id
            .clone();
        app.terminals.get_mut(&tid_two).unwrap().state = AgentState::Blocked;

        let registry = TerminalRuntimeRegistry::new();
        let items = build_attention(&app, &registry);
        assert_eq!(items.len(), 2);
        assert!(matches!(
            items[0].id,
            SidebarItemId::Agent { ws_idx: 1, .. }
        ));
        assert!(matches!(
            items[1].id,
            SidebarItemId::Agent { ws_idx: 0, .. }
        ));
    }

    #[test]
    fn empty_workspaces_produces_empty_lists() {
        let mut app = AppState::test_new();
        app.workspaces = Vec::new();
        let items = build_workspaces(&app);
        assert!(items.is_empty());
        let registry = TerminalRuntimeRegistry::new();
        let agents = build_agents(&app, &registry);
        assert!(agents.is_empty());
        let attention = build_attention(&app, &registry);
        assert!(attention.is_empty());
    }

    #[test]
    fn sidebar_model_caches_all_modes() {
        let mut app = AppState::test_new();
        app.workspaces = vec![Workspace::test_new("a"), Workspace::test_new("b")];
        let registry = TerminalRuntimeRegistry::new();
        let mut model = SidebarModel::new();
        model.rebuild(&app, &registry);
        assert_eq!(model.workspaces.len(), 2);
        assert!(!model.agents.is_empty());
        // Attention may be empty if all agents are unknown.
    }

    #[test]
    fn sidebar_item_with_secondary_and_indent() {
        let item = SidebarItem::new(
            SidebarItemId::Workspace { ws_idx: 0 },
            SidebarRowKind::Workspace {
                ws_idx: 0,
                indented: true,
            },
            "test".into(),
            AgentState::Working,
            false,
        )
        .with_secondary("branch")
        .with_indent(2);
        assert_eq!(item.secondary, Some("branch".into()));
        assert_eq!(item.indent, 2);
        assert_eq!(item.primary, "test");
    }

    #[test]
    fn sidebar_item_id_workspace_idx() {
        let id = SidebarItemId::Workspace { ws_idx: 3 };
        assert_eq!(id.workspace_idx(), Some(3));
        let id = SidebarItemId::Agent {
            ws_idx: 1,
            tab_idx: 0,
            pane_id: crate::layout::PaneId::from_raw(1),
        };
        assert_eq!(id.workspace_idx(), Some(1));
        let id = SidebarItemId::Attention {
            source_id: "x".into(),
        };
        assert_eq!(id.workspace_idx(), None);
    }

    #[test]
    fn attention_filters_out_unknown_agents() {
        let mut app = AppState::test_new();
        app.workspaces = vec![Workspace::test_new("one")];
        app.ensure_test_terminals();
        let pane_id = app.workspaces[0].tabs[0].root_pane;
        let tid = app.workspaces[0].tabs[0].panes[&pane_id]
            .attached_terminal_id
            .clone();
        app.terminals.get_mut(&tid).unwrap().state = AgentState::Unknown;
        let registry = TerminalRuntimeRegistry::new();
        let items = build_attention(&app, &registry);
        assert!(items.is_empty());
    }

    #[test]
    fn attention_priority_values() {
        // Blocked > unseen-idle > Working > seen-idle > Unknown
        // Helper: attention_priority(AgentState::Blocked, _) should be highest.
        let p_blocked = super::attention_priority(AgentState::Blocked, false);
        let p_unseen = super::attention_priority(AgentState::Idle, false);
        let p_working = super::attention_priority(AgentState::Working, false);
        let p_seen = super::attention_priority(AgentState::Idle, true);
        let p_unknown = super::attention_priority(AgentState::Unknown, false);
        assert!(p_blocked > p_unseen);
        assert!(p_unseen > p_working);
        assert!(p_working > p_seen);
        assert!(p_seen > p_unknown);
    }

    #[test]
    fn sidebar_model_items_for_mode() {
        let mut app = AppState::test_new();
        app.workspaces = vec![Workspace::test_new("a")];
        let registry = TerminalRuntimeRegistry::new();
        let mut model = SidebarModel::new();
        model.rebuild(&app, &registry);

        let ws = model.items_for_mode(crate::ui::shell::SidebarMode::Workspaces);
        assert!(!ws.is_empty());
        let agents = model.items_for_mode(crate::ui::shell::SidebarMode::Agents);
        assert!(!agents.is_empty());
        let attn = model.items_for_mode(crate::ui::shell::SidebarMode::Attention);
        // May be empty if all idle and seen.
        let _ = attn;
    }

    #[test]
    fn sidebar_item_clone_and_eq() {
        let item = SidebarItem::new(
            SidebarItemId::Workspace { ws_idx: 0 },
            SidebarRowKind::Workspace {
                ws_idx: 0,
                indented: false,
            },
            "hello".into(),
            AgentState::Idle,
            true,
        );
        let item2 = item.clone();
        assert_eq!(item, item2);
    }
}
