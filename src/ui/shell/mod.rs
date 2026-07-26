//! Shell layout model for the Herdr TUI rewrite.
//!
//! The shell owns the high-level geometry of the terminal surface:
//! sidebar, tab bar, terminal area, Fleet Ops line, and overlays.  It is
//! designed to be shared between rendering and hit testing so the two never
//! diverge.

use crate::terminal::TerminalRuntimeRegistry;
use crate::ui::sidebar_new::model::SidebarItemId;
use ratatui::layout::{Constraint, Layout, Rect};

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
    pub id: SidebarItemId,
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OverlayLayout {
    pub area: Rect,
}

/// Compute the shell layout for a given total area and desired configuration.
///
/// This function is pure: it does not mutate `AppState`.  All scroll clamping
/// happens here using the supplied `ScrollState` values.
///
/// `sidebar_anim_progress` is an optional 0.0–1.0 value from the motion
/// system that interpolates the sidebar width during expand/collapse
/// transitions.  When `None` (or 1.0), the full target width is used.
pub fn compute_shell_layout(
    area: Rect,
    mode: LayoutMode,
    sidebar_collapsed: bool,
    sidebar_width: u16,
    requested_sidebar_mode: SidebarMode,
    workspace_scroll: ScrollState,
    _agent_scroll: ScrollState,
    _attention_scroll: ScrollState,
) -> ShellLayout {
    compute_shell_layout_with_anim(
        area,
        mode,
        sidebar_collapsed,
        sidebar_width,
        requested_sidebar_mode,
        workspace_scroll,
        _agent_scroll,
        _attention_scroll,
        1.0,
    )
}

/// Like `compute_shell_layout` but accepts a `sidebar_anim_progress` parameter
/// (0.0 = fully collapsed / zero width, 1.0 = fully expanded).
pub fn compute_shell_layout_with_anim(
    area: Rect,
    mode: LayoutMode,
    sidebar_collapsed: bool,
    sidebar_width: u16,
    requested_sidebar_mode: SidebarMode,
    workspace_scroll: ScrollState,
    _agent_scroll: ScrollState,
    _attention_scroll: ScrollState,
    sidebar_anim_progress: f32,
) -> ShellLayout {
    let progress = sidebar_anim_progress.clamp(0.0, 1.0);
    let (sidebar_rect, main_rect) = if mode.sidebar_visible() && !sidebar_collapsed {
        let w = sidebar_width.clamp(22, 40);
        let animated_w = ((w as f32) * progress) as u16;
        let w_final = if animated_w < 2 && progress < 0.5 {
            0
        } else {
            animated_w.max(2)
        };
        let [s, m] =
            Layout::horizontal([Constraint::Length(w_final), Constraint::Min(1)]).areas(area);
        (s, m)
    } else if mode == LayoutMode::Narrow && sidebar_collapsed {
        // In narrow mode a collapsed rail may still be visible.
        let w = 4u16;
        let animated_w = ((w as f32) * progress) as u16;
        let [s, m] =
            Layout::horizontal([Constraint::Length(animated_w), Constraint::Min(1)]).areas(area);
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
/// the full `ShellLayout`, builds sidebar model + tab items, and stores
/// geometry in `app.view`.
pub fn compute_new_shell_view(
    app: &mut crate::app::AppState,
    terminal_runtimes: &TerminalRuntimeRegistry,
    area: Rect,
    _resize_panes: bool,
    _cell_size: crate::kitty_graphics::HostCellSize,
) {
    let now = std::time::Instant::now();

    // ── Phase 7: Sidebar mode-switch transition ────────────────────
    // When the user switches sidebar modes (Workspaces→Agents→Attention),
    // start a short ease-out transition that render_new_shell samples
    // to cross-fade the header label.
    if app.new_sidebar_mode != app.new_sidebar_prev_mode {
        let from = app.new_sidebar_prev_mode as f32;
        let to = app.new_sidebar_mode as f32;
        app.new_transitions.set(crate::ui::motion::Transition::new(
            crate::ui::motion::UiRegion::SidebarMode,
            now,
            app.new_motion_policy
                .resolve_duration(std::time::Duration::from_millis(140)),
            from,
            to,
            crate::ui::motion::Easing::ease_out,
            crate::ui::motion::InterruptionPolicy::Retarget,
        ));
        app.new_sidebar_prev_mode = app.new_sidebar_mode;
    }

    // ── Phase 7: Sidebar expand/collapse transition ─────────────────
    // When the sidebar is toggled, animate the width from 0→1 or 1→0.
    // We use a stored prev-collapsed field to detect edges.  (For now
    // we infer the transition purely from the collapsed flag changing
    // between frames; a dedicated field will be added in Phase 9.)
    {
        let target_progress: f32 = if app.new_sidebar_collapsed { 0.0 } else { 1.0 };
        let current = app
            .new_transitions
            .sample_scalar(crate::ui::motion::UiRegion::Sidebar, now)
            .unwrap_or(target_progress);
        // If the target differs from current, start (or retarget) a transition.
        if (target_progress - current).abs() > 0.01 {
            app.new_transitions.set(crate::ui::motion::Transition::new(
                crate::ui::motion::UiRegion::Sidebar,
                now,
                app.new_motion_policy
                    .resolve_duration(std::time::Duration::from_millis(180)),
                current,
                target_progress,
                crate::ui::motion::Easing::smooth,
                crate::ui::motion::InterruptionPolicy::Retarget,
            ));
        }
    }

    // ── Phase 7: Attention pulse for blocked agents ─────────────────
    // When any agent is in Blocked state, maintain a pulsing transition
    // on UiRegion::Toast that the render path can sample for visual emphasis.
    {
        let has_blocked = app
            .workspaces
            .iter()
            .any(|ws| ws.aggregate_state(&app.terminals).0 == crate::detect::AgentState::Blocked);
        if has_blocked {
            let pulse = app
                .new_transitions
                .sample_scalar(crate::ui::motion::UiRegion::Toast, now)
                .unwrap_or(0.0);
            if pulse < 0.01 {
                // Start a repeating pulse: 0→1 over 600ms, ease-in-out.
                app.new_transitions.set(crate::ui::motion::Transition::new(
                    crate::ui::motion::UiRegion::Toast,
                    now,
                    app.new_motion_policy
                        .resolve_duration(std::time::Duration::from_millis(600)),
                    0.0,
                    1.0,
                    crate::ui::motion::Easing::ease_in_out,
                    crate::ui::motion::InterruptionPolicy::Retarget,
                ));
            } else if pulse > 0.99 {
                // Reverse: 1→0 over 600ms.
                app.new_transitions.set(crate::ui::motion::Transition::new(
                    crate::ui::motion::UiRegion::Toast,
                    now,
                    app.new_motion_policy
                        .resolve_duration(std::time::Duration::from_millis(600)),
                    1.0,
                    0.0,
                    crate::ui::motion::Easing::ease_in_out,
                    crate::ui::motion::InterruptionPolicy::Retarget,
                ));
            }
        }
    }

    // Advance all transitions (removes completed ones, returns next wake time).
    app.new_transitions.advance(now);

    // Sample the sidebar animation progress for layout.
    let sidebar_anim = app
        .new_transitions
        .sample_scalar(crate::ui::motion::UiRegion::Sidebar, now)
        .unwrap_or(if app.new_sidebar_collapsed { 0.0 } else { 1.0 });

    let mode = LayoutMode::from_area(area);
    let sidebar_mode = match app.new_sidebar_mode {
        0 => SidebarMode::Workspaces,
        1 => SidebarMode::Agents,
        _ => SidebarMode::Attention,
    };

    // ── Phase 8: Skip rebuild when nothing changed ─────────────────
    // When the shell is not dirty and the area hasn't changed, reuse the
    // cached layout.  State changes (agent state, workspace changes, resize)
    // set new_shell_dirty = true so this gate is bypassed.
    let hash = crate::ui::shell::layout_hash(area, app.new_sidebar_collapsed, app.new_sidebar_mode);
    if !app.new_shell_dirty && app.new_shell_layout_hash == hash && app.new_shell_layout_hash != 0 {
        // Cache hit — skip rebuild, just advance transitions for motion.
        app.new_transitions.advance(std::time::Instant::now());
        return;
    }
    app.new_shell_layout_hash = hash;
    app.new_shell_dirty = false;

    // Build sidebar model and compute shell layout.
    let mut sidebar_model = crate::ui::sidebar_new::model::SidebarModel::new();
    sidebar_model.rebuild(app, terminal_runtimes);

    let workspace_scroll = ScrollState {
        offset: app.workspace_scroll,
        visible: 20,
        total: sidebar_model.items_for_mode(sidebar_mode).len(),
    };
    let agent_scroll = ScrollState {
        offset: app.agent_panel_scroll,
        visible: 20,
        total: sidebar_model.items_for_mode(SidebarMode::Agents).len(),
    };

    let mut shell_layout = compute_shell_layout_with_anim(
        area,
        mode,
        app.new_sidebar_collapsed,
        app.sidebar_width,
        sidebar_mode,
        workspace_scroll,
        agent_scroll,
        ScrollState {
            offset: 0,
            visible: 20,
            total: 0,
        },
        sidebar_anim,
    );

    // Layout sidebar rows.
    crate::ui::sidebar_new::layout::layout_sidebar(&mut shell_layout, &sidebar_model);

    // Build and layout tab bar.
    let tab_items = crate::ui::tabs_new::model::build_tabs(app);
    crate::ui::tabs_new::layout::layout_tab_bar(&mut shell_layout.main.tab_bar, &tab_items);

    // Populate app.view from the shell layout.
    let tab_hit_areas: Vec<Rect> = shell_layout
        .main
        .tab_bar
        .tabs
        .iter()
        .map(|h| h.rect)
        .collect();

    app.view = crate::app::ViewState {
        layout: crate::app::state::ViewLayout::Desktop,
        sidebar_rect: shell_layout.sidebar.rect,
        workspace_card_areas: Vec::new(),
        navigator_rows: Vec::new(),
        tab_bar_rect: shell_layout.main.tab_bar.rect,
        tab_hit_areas,
        tab_scroll_left_hit_area: shell_layout.main.tab_bar.scroll_left,
        tab_scroll_right_hit_area: shell_layout.main.tab_bar.scroll_right,
        new_tab_hit_area: shell_layout.main.tab_bar.new_tab,
        terminal_area: shell_layout.main.terminal,
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
/// Renders the unified sidebar, compact tabs, and terminal area.
pub fn render_new_shell(
    app: &crate::app::AppState,
    terminal_runtimes: &TerminalRuntimeRegistry,
    frame: &mut ratatui::Frame,
) {
    let area = frame.area();
    let mode = LayoutMode::from_area(area);
    let sidebar_mode = match app.new_sidebar_mode {
        0 => SidebarMode::Workspaces,
        1 => SidebarMode::Agents,
        _ => SidebarMode::Attention,
    };

    // Build sidebar model.
    let mut sidebar_model = crate::ui::sidebar_new::model::SidebarModel::new();
    sidebar_model.rebuild(app, terminal_runtimes);

    let workspace_scroll = ScrollState {
        offset: app.workspace_scroll,
        visible: 20,
        total: sidebar_model.items_for_mode(sidebar_mode).len(),
    };
    let agent_scroll = ScrollState {
        offset: app.agent_panel_scroll,
        visible: 20,
        total: sidebar_model.items_for_mode(SidebarMode::Agents).len(),
    };

    // Compute shell layout with animation progress sampled from transitions.
    let now = std::time::Instant::now();
    let sidebar_anim = app
        .new_transitions
        .sample_scalar(crate::ui::motion::UiRegion::Sidebar, now)
        .unwrap_or(if app.new_sidebar_collapsed { 0.0 } else { 1.0 });
    let mut shell_layout = compute_shell_layout_with_anim(
        area,
        mode,
        app.new_sidebar_collapsed,
        app.sidebar_width,
        sidebar_mode,
        workspace_scroll,
        agent_scroll,
        ScrollState {
            offset: 0,
            visible: 20,
            total: 0,
        },
        sidebar_anim,
    );

    // Layout sidebar rows from the model.
    crate::ui::sidebar_new::layout::layout_sidebar(&mut shell_layout, &sidebar_model);

    // Build and layout tab bar.
    let tab_items = crate::ui::tabs_new::model::build_tabs(app);
    crate::ui::tabs_new::layout::layout_tab_bar(&mut shell_layout.main.tab_bar, &tab_items);

    // Render sidebar.
    let items = sidebar_model.items_for_mode(sidebar_mode).to_vec();
    crate::ui::sidebar_new::render::render_sidebar_new(
        app,
        terminal_runtimes,
        frame,
        &shell_layout,
        &items,
    );

    // Render compact tab bar.
    crate::ui::tabs_new::render::render_tab_bar_new(
        app,
        frame,
        &shell_layout.main.tab_bar,
        &tab_items,
        &app.palette,
    );

    // Render Fleet Ops compact status line.
    let fleet_ctx = crate::ui::fleet_ops_new::model::build_fleet_ops_context(app);
    if shell_layout.fleet_ops.rect.width > 0 && shell_layout.fleet_ops.rect.height > 0 {
        crate::ui::fleet_ops_new::render::render_fleet_ops_new(
            app,
            frame,
            &shell_layout.fleet_ops,
            &fleet_ctx,
            &app.palette,
        );
    }

    // Render terminal area — placeholder for now; full terminal rendering
    // integration comes in later phases.
    let terminal_area = shell_layout.main.terminal;
    if terminal_area.width > 0 && terminal_area.height > 0 {
        use ratatui::widgets::Paragraph;
        frame.render_widget(
            Paragraph::new("[new shell terminal area — Phase 3]"),
            terminal_area,
        );
    }

    // Render launcher overlay if open.
    if app.new_launcher_open {
        let mut launcher_layout = crate::ui::launcher_new::layout::layout_launcher(area);
        let launcher_items = crate::ui::launcher_new::model::build_launcher_items(app);
        crate::ui::launcher_new::layout::layout_launcher_rows(
            &mut launcher_layout,
            launcher_items.len(),
        );
        crate::ui::launcher_new::render::render_launcher_new(
            app,
            frame,
            &launcher_layout,
            &launcher_items,
            app.new_launcher_selected,
            &app.palette,
        );
    }

    // Render settings overlay if open.
    if app.new_settings_open {
        let settings_layout = crate::ui::settings_new::layout::layout_settings(area);
        crate::ui::settings_new::render::render_settings_new(
            app,
            frame,
            &settings_layout,
            crate::ui::settings_new::model::SettingsCategory::Appearance,
            &app.palette,
        );
    }
}

fn top_line_of(rect: Rect) -> Rect {
    Rect::new(rect.x, rect.y, rect.width, 1.min(rect.height))
}

fn inset_top(rect: Rect, n: u16) -> Rect {
    let h = rect.height.saturating_sub(n);
    Rect::new(rect.x, rect.y + n, rect.width, h)
}

/// Phase 8: Compute a cheap hash of layout inputs so we can skip recomputation
/// when nothing has changed.  Uses the area dimensions, collapsed flag, and
/// sidebar mode — all layout-determining inputs that don't require rebuilding
/// the model.
pub fn layout_hash(area: Rect, collapsed: bool, sidebar_mode: u8) -> u64 {
    let mut h: u64 = 0;
    h = h.wrapping_mul(31).wrapping_add(area.x as u64);
    h = h.wrapping_mul(31).wrapping_add(area.y as u64);
    h = h.wrapping_mul(31).wrapping_add(area.width as u64);
    h = h.wrapping_mul(31).wrapping_add(area.height as u64);
    h = h.wrapping_mul(31).wrapping_add(u64::from(collapsed));
    h = h.wrapping_mul(31).wrapping_add(sidebar_mode as u64);
    h
}

// ── Phase 9: Hit-test routing for the new shell ────────────────────

/// Resolve a mouse click at `(col, row)` in the new shell to the appropriate
/// target.  Returns `None` when the click hits dead space or the terminal area
/// (which is handled by the existing pane input path).
pub fn hit_test_new_shell(
    app: &crate::app::AppState,
    shell_layout: &ShellLayout,
    col: u16,
    row: u16,
) -> Option<HitTarget> {
    use crate::ui::sidebar_new::layout::row_at;
    use crate::ui::sidebar_new::model::SidebarItemId;

    let sidebar = &shell_layout.sidebar;
    if col < sidebar.rect.x + sidebar.rect.width && sidebar.rect.width > 0 {
        // Click is in the sidebar area.
        // Check mode-switcher row (top row).
        if row == sidebar.mode_switcher.y {
            return Some(HitTarget::SidebarModeToggle);
        }
        // Check a sidebar row.
        if let Some(hit) = row_at(shell_layout, col, row) {
            return match &hit.id {
                SidebarItemId::Workspace { ws_idx } => {
                    Some(HitTarget::SidebarWorkspace { ws_idx: *ws_idx })
                }
                SidebarItemId::Agent { ws_idx, tab_idx, pane_id } => {
                    Some(HitTarget::SidebarAgent {
                        ws_idx: *ws_idx,
                        tab_idx: *tab_idx,
                        pane_id: *pane_id,
                    })
                }
                SidebarItemId::Attention { source_id } => {
                    Some(HitTarget::SidebarAttention {
                        source_id: source_id.clone(),
                    })
                }
            };
        }
        return None;
    }

    // Tab bar hit testing.
    let tab_bar = &shell_layout.main.tab_bar;
    if row == tab_bar.rect.y && tab_bar.rect.width > 0 {
        for tab in &tab_bar.tabs {
            if tab.visible && col >= tab.rect.x && col < tab.rect.x + tab.rect.width {
                return Some(HitTarget::Tab { index: tab.index });
            }
        }
        if col >= tab_bar.new_tab.x && col < tab_bar.new_tab.x + tab_bar.new_tab.width {
            return Some(HitTarget::NewTab);
        }
        if col >= tab_bar.scroll_left.x && col < tab_bar.scroll_left.x + tab_bar.scroll_left.width {
            return Some(HitTarget::TabScrollLeft);
        }
        if col >= tab_bar.scroll_right.x && col < tab_bar.scroll_right.x + tab_bar.scroll_right.width {
            return Some(HitTarget::TabScrollRight);
        }
    }

    None
}

/// Hit-test target for the new shell UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitTarget {
    /// Clicked the sidebar mode switcher header.
    SidebarModeToggle,
    /// Clicked a workspace row.
    SidebarWorkspace { ws_idx: usize },
    /// Clicked an agent row.
    SidebarAgent {
        ws_idx: usize,
        tab_idx: usize,
        pane_id: crate::layout::PaneId,
    },
    /// Clicked an attention row.
    SidebarAttention { source_id: String },
    /// Clicked a tab.
    Tab { index: usize },
    /// Clicked the new-tab button.
    NewTab,
    /// Clicked the tab scroll-left arrow.
    TabScrollLeft,
    /// Clicked the tab scroll-right arrow.
    TabScrollRight,
}

// ── Integration tests (end-to-end model→layout→render pipeline) ────
#[cfg(test)]
mod integration_tests;

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

    #[test]
    fn scroll_state_zero_offset_when_total_lt_visible() {
        let s = ScrollState {
            offset: 3,
            visible: 10,
            total: 5,
        }
        .clamped();
        assert_eq!(s.offset, 0);
    }

    #[test]
    fn mobile_mode_hides_sidebar() {
        assert!(!LayoutMode::Mobile.sidebar_visible());
        assert!(!LayoutMode::Narrow.sidebar_visible());
        assert!(!LayoutMode::Tiny.sidebar_visible());
    }

    #[test]
    fn wide_mode_shows_fleet_ops() {
        assert!(LayoutMode::Wide.fleet_ops_expanded());
        assert!(LayoutMode::Standard.fleet_ops_expanded());
        assert!(!LayoutMode::Narrow.fleet_ops_expanded());
        assert!(!LayoutMode::Mobile.fleet_ops_expanded());
    }

    #[test]
    fn collapsed_sidebar_in_standard_mode_zeroes_sidebar_rect() {
        let layout = compute_shell_layout(
            Rect::new(0, 0, 120, 40),
            LayoutMode::Standard,
            true, // collapsed
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
        assert_eq!(layout.sidebar.rect.width, 0);
    }

    #[test]
    fn narrow_collapsed_shows_rail() {
        let layout = compute_shell_layout(
            Rect::new(0, 0, 60, 20),
            LayoutMode::Narrow,
            true, // collapsed
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
        assert_eq!(layout.sidebar.rect.width, 4);
    }

    #[test]
    fn tiny_mode_produces_no_sidebar_or_fleet_ops() {
        let layout = compute_shell_layout(
            Rect::new(0, 0, 20, 10),
            LayoutMode::Tiny,
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
        assert_eq!(layout.sidebar.rect.width, 0);
        assert_eq!(layout.fleet_ops.rect.width, 0);
    }

    #[test]
    fn all_layout_modes_have_labels() {
        let modes = [
            LayoutMode::Wide,
            LayoutMode::Standard,
            LayoutMode::Medium,
            LayoutMode::Narrow,
            LayoutMode::Mobile,
            LayoutMode::Tiny,
        ];
        for mode in &modes {
            let layout = compute_shell_layout(
                Rect::new(0, 0, 200, 60),
                *mode,
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
            // Every mode should produce a ShellLayout without panic.
            assert!(layout.area.width > 0);
        }
    }

    #[test]
    fn sidebar_mode_labels() {
        assert_eq!(SidebarMode::Workspaces.label(), "Workspaces");
        assert_eq!(SidebarMode::Agents.label(), "Agents");
        assert_eq!(SidebarMode::Attention.label(), "Attention");
    }

    #[test]
    fn scroll_state_max_offset() {
        let s = ScrollState {
            offset: 0,
            visible: 5,
            total: 10,
        };
        assert_eq!(s.max_offset(), 5);
        let s = ScrollState {
            offset: 0,
            visible: 10,
            total: 5,
        };
        assert_eq!(s.max_offset(), 0);
        let s = ScrollState {
            offset: 0,
            visible: 0,
            total: 0,
        };
        assert_eq!(s.max_offset(), 0);
    }

    #[test]
    fn shell_layout_fleet_ops_hidden_when_too_short() {
        let layout = compute_shell_layout(
            Rect::new(0, 0, 120, 2),
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
        assert_eq!(layout.fleet_ops.rect.width, 0);
    }

    #[test]
    fn animation_progress_interpolates_sidebar_width() {
        // Full progress (1.0) → full sidebar width.
        let layout = compute_shell_layout_with_anim(
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
            1.0,
        );
        assert!(layout.sidebar.rect.width > 0);

        // Zero progress → zero sidebar width.
        let layout = compute_shell_layout_with_anim(
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
            0.0,
        );
        assert_eq!(layout.sidebar.rect.width, 0);

        // Half progress → sidebar width around half.
        let layout = compute_shell_layout_with_anim(
            Rect::new(0, 0, 120, 40),
            LayoutMode::Standard,
            false,
            30,
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
            0.5,
        );
        assert_eq!(layout.sidebar.rect.width, 15); // 30 * 0.5 = 15
    }

    #[test]
    fn narrow_collapsed_animation_progress_scales_rail() {
        let layout = compute_shell_layout_with_anim(
            Rect::new(0, 0, 60, 20),
            LayoutMode::Narrow,
            true,
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
            0.5,
        );
        assert_eq!(layout.sidebar.rect.width, 2); // 4 * 0.5 = 2
    }
}
