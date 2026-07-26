//! Shell layout model for the Herdr TUI rewrite.
//!
//! The shell owns the high-level geometry of the terminal surface:
//! sidebar, tab bar, terminal area, Fleet Ops line, and overlays.  It is
//! designed to be shared between rendering and hit testing so the two never
//! diverge.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use crate::terminal::TerminalRuntimeRegistry;

/// Width and density breakpoints, expressed in terminal columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Very large terminal: expanded sidebar, full metadata, Fleet Ops line.
    Wide,
    /// Standard desktop layout.
    Standard,
    /// Hide secondary metadata, reduce spacing, shorten labels.
    Medium,
    /// Collapsed rail or overlay navigation, terminal dominates.
    Narrow,
    /// Mobile switcher, minimal chrome.
    Mobile,
    /// Extremely small fallback.
    Tiny,
}

impl LayoutMode {
    pub fn from_area(area: Rect) -> Self {
        match area.width {
            0..=29 => LayoutMode::Tiny,
            30..=49 => LayoutMode::Mobile,
            50..=79 => LayoutMode::Narrow,
            80..=159 => LayoutMode::Standard,
            _ => LayoutMode::Wide,
        }
    }

    /// Whether the sidebar can be shown as a persistent rail.
    pub fn sidebar_visible(self) -> bool {
        matches!(
            self,
            LayoutMode::Wide | LayoutMode::Standard | LayoutMode::Medium
        )
    }

    /// Whether Fleet Ops metadata should be shown inline.
    pub fn fleet_ops_expanded(self) -> bool {
        matches!(self, LayoutMode::Wide | LayoutMode::Standard)
    }
}

/// Geometry for the entire shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellLayout {
    pub area: Rect,
    pub mode: LayoutMode,
    pub sidebar: SidebarLayout,
    pub main: MainLayout,
    pub fleet_ops: FleetOpsLayout,
    pub overlays: OverlayLayout,
}

/// Sidebar geometry and scroll/hit state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarLayout {
    pub rect: Rect,
    pub mode: SidebarMode,
    pub collapsed: bool,
    pub mode_switcher: Rect,
    pub content: Rect,
    pub scroll: ScrollState,
    pub rows: Vec<SidebarRowRect>,
    pub divider: Rect,
}

/// Which content the sidebar is presenting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarMode {
    Workspaces,
    Agents,
    Attention,
}

impl SidebarMode {
    pub fn label(self) -> &'static str {
        match self {
            SidebarMode::Workspaces => "Workspaces",
            SidebarMode::Agents => "Agents",
            SidebarMode::Attention => "Attention",
        }
    }
}

/// Scroll state with clamped offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollState {
    pub offset: usize,
    pub visible: usize,
    pub total: usize,
}

impl ScrollState {
    pub fn max_offset(self) -> usize {
        self.total.saturating_sub(self.visible)
    }

    pub fn clamped(self) -> Self {
        let max = self.max_offset();
        Self {
            offset: self.offset.min(max),
            visible: self.visible,
            total: self.total,
        }
    }
}

/// Per-row geometry for hit testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarRowRect {
    pub rect: Rect,
    pub id: SidebarRowId,
}

/// Identity for a sidebar row. Indices refer to the current mode's item list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarRowId {
    Workspace(usize),
    Agent(usize),
    Attention(usize),
}

/// Main content area layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainLayout {
    pub rect: Rect,
    pub tab_bar: TabBarLayout,
    pub terminal: Rect,
}

/// Tab bar geometry and overflow state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabBarLayout {
    pub rect: Rect,
    pub tabs: Vec<TabHitRect>,
    pub scroll_left: Rect,
    pub scroll_right: Rect,
    pub new_tab: Rect,
    pub overflow: bool,
}

/// Per-tab hit area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabHitRect {
    pub rect: Rect,
    pub index: usize,
    pub visible: bool,
}

/// Fleet Ops geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetOpsLayout {
    pub rect: Rect,
    pub expanded: bool,
    pub summary: Rect,
    pub detail: Option<Rect>,
}

/// Overlay geometry placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayLayout {
    pub area: Rect,
}

impl Default for OverlayLayout {
    fn default() -> Self {
        Self {
            area: Rect::default(),
        }
    }
}

/// Compute the shell layout for a given total area and desired configuration.
///
/// This function is pure: it does not mutate `AppState`.  All scroll clamping
/// happens here using the supplied `ScrollState` values.
pub fn compute_shell_layout(
    area: Rect,
    mode: LayoutMode,
    sidebar_collapsed: bool,
    sidebar_width: u16,
    requested_sidebar_mode: SidebarMode,
    workspace_scroll: ScrollState,
    agent_scroll: ScrollState,
    attention_scroll: ScrollState,
) -> ShellLayout {
    let (sidebar_rect, main_rect) = if mode.sidebar_visible() && !sidebar_collapsed {
        let w = sidebar_width.clamp(22, 40);
        let [s, m] = Layout::horizontal([Constraint::Length(w), Constraint::Min(1)]).areas(area);
        (s, m)
    } else if mode == LayoutMode::Narrow && sidebar_collapsed {
        // In narrow mode a collapsed rail may still be visible.
        let w = 4u16;
        let [s, m] = Layout::horizontal([Constraint::Length(w), Constraint::Min(1)]).areas(area);
        (s, m)
    } else {
        (Rect::default(), area)
    };

    let sidebar = SidebarLayout {
        rect: sidebar_rect,
        mode: requested_sidebar_mode,
        collapsed: sidebar_collapsed,
        mode_switcher: top_line_of(sidebar_rect),
        content: inset_top(sidebar_rect, 1),
        scroll: workspace_scroll.clamped(),
        rows: Vec::new(),
        divider: Rect::default(),
    };

    let fleet_h = if mode.fleet_ops_expanded() {
        1u16
    } else {
        0u16
    };
    let (main_content, fleet_rect) = if fleet_h > 0 && main_rect.height > 2 {
        let [m, f] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(fleet_h)]).areas(main_rect);
        (m, f)
    } else {
        (main_rect, Rect::default())
    };

    let (tab_bar_rect, terminal_rect) = if main_content.height > 1 {
        let [t, tr] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(main_content);
        (t, tr)
    } else {
        (Rect::default(), main_content)
    };

    let main = MainLayout {
        rect: main_content,
        tab_bar: TabBarLayout {
            rect: tab_bar_rect,
            tabs: Vec::new(),
            scroll_left: Rect::default(),
            scroll_right: Rect::default(),
            new_tab: Rect::default(),
            overflow: false,
        },
        terminal: terminal_rect,
    };

    let fleet_ops = FleetOpsLayout {
        rect: fleet_rect,
        expanded: false,
        summary: fleet_rect,
        detail: None,
    };

    ShellLayout {
        area,
        mode,
        sidebar,
        main,
        fleet_ops,
        overlays: OverlayLayout::default(),
    }
}

/// Compute view geometry for the new shell.
///
/// This is the new-shell equivalent of `compute_view_internal` — it computes
/// the new `ShellLayout` and stores it in `app.view`.  Pane resizing is
/// delegated to the existing machinery.
pub fn compute_new_shell_view(
    app: &mut crate::app::AppState,
    _terminal_runtimes: &TerminalRuntimeRegistry,
    area: Rect,
    _resize_panes: bool,
    _cell_size: crate::kitty_graphics::HostCellSize,
) {
    // For Phase 1, use a minimal full-terminal layout.
    // Later phases will wire the full ShellLayout, sidebar, tabs, etc.
    app.view = crate::app::ViewState {
        layout: crate::app::state::ViewLayout::Desktop,
        sidebar_rect: Rect::default(),
        workspace_card_areas: Vec::new(),
        navigator_rows: Vec::new(),
        tab_bar_rect: Rect::default(),
        tab_hit_areas: Vec::new(),
        tab_scroll_left_hit_area: Rect::default(),
        tab_scroll_right_hit_area: Rect::default(),
        new_tab_hit_area: Rect::default(),
        terminal_area: area,
        mobile_header_rect: Rect::default(),
        mobile_menu_hit_area: Rect::default(),
        toast_hit_area: Rect::default(),
        pane_infos: Vec::new(),
        split_borders: Vec::new(),
    };
}

/// Render the new terminal shell UI.
///
/// This is the new-shell equivalent of `render_with_runtime_registry`.
/// Phase 1 renders a minimal placeholder.  Later phases will add the
/// unified sidebar, compact tabs, launcher, settings, etc.
pub fn render_new_shell(
    _app: &crate::app::AppState,
    _terminal_runtimes: &TerminalRuntimeRegistry,
    _frame: &mut ratatui::Frame,
) {
    // Phase 1 placeholder: the terminal pane renders via the existing
    // tab_surface mechanism in `render_with_runtime_registry`.
    // When new_shell is fully implemented, this function will render
    // the complete new UI (sidebar, tabs, terminal, fleet ops, overlays).
}
    Rect::new(rect.x, rect.y, rect.width, 1.min(rect.height))
}

fn inset_top(rect: Rect, n: u16) -> Rect {
    let h = rect.height.saturating_sub(n);
    Rect::new(rect.x, rect.y + n, rect.width, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_mode_selects_correct_breakpoint() {
        assert_eq!(
            LayoutMode::from_area(Rect::new(0, 0, 200, 60)),
            LayoutMode::Wide
        );
        assert_eq!(
            LayoutMode::from_area(Rect::new(0, 0, 100, 40)),
            LayoutMode::Standard
        );
        assert_eq!(
            LayoutMode::from_area(Rect::new(0, 0, 60, 20)),
            LayoutMode::Narrow
        );
        assert_eq!(
            LayoutMode::from_area(Rect::new(0, 0, 40, 20)),
            LayoutMode::Mobile
        );
        assert_eq!(
            LayoutMode::from_area(Rect::new(0, 0, 20, 10)),
            LayoutMode::Tiny
        );
    }

    #[test]
    fn shell_layout_reserves_sidebar_and_terminal_areas() {
        let layout = compute_shell_layout(
            Rect::new(0, 0, 120, 40),
            LayoutMode::Standard,
            false,
            28,
            SidebarMode::Workspaces,
            ScrollState {
                offset: 0,
                visible: 10,
                total: 5,
            },
            ScrollState {
                offset: 0,
                visible: 10,
                total: 5,
            },
            ScrollState {
                offset: 0,
                visible: 10,
                total: 5,
            },
        );
        assert!(layout.sidebar.rect.width > 0);
        assert_eq!(layout.main.terminal.y, 1); // tab bar takes one row
        assert_eq!(layout.main.terminal.height, 38); // rest minus fleet ops
        assert!(layout.fleet_ops.rect.height <= 1);
    }

    #[test]
    fn scroll_state_clamps_offset() {
        let s = ScrollState {
            offset: 100,
            visible: 5,
            total: 10,
        }
        .clamped();
        assert_eq!(s.offset, 5);
    }
}
