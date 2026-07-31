from pathlib import Path


def replace(path: str, old: str, new: str, expected: int = 1) -> None:
    file = Path(path)
    text = file.read_text()
    actual = text.count(old)
    if actual != expected:
        raise SystemExit(
            f"{path}: expected {expected} occurrences, found {actual}: {old!r}"
        )
    file.write_text(text.replace(old, new))


def replace_between(path: str, start: str, end: str, replacement: str) -> None:
    file = Path(path)
    text = file.read_text()
    start_index = text.find(start)
    if start_index < 0:
        raise SystemExit(f"{path}: start marker not found: {start!r}")
    end_index = text.find(end, start_index)
    if end_index < 0:
        raise SystemExit(f"{path}: end marker not found: {end!r}")
    file.write_text(text[:start_index] + replacement + text[end_index:])


# Mouse routing must distinguish dead space from a handled click with no action.
replace(
    "src/app/input/mouse.rs",
    "impl AppState {",
    "enum NewShellClickOutcome {\n"
    "    Unhandled,\n"
    "    Handled,\n"
    "    Action(MouseAction),\n"
    "}\n\n"
    "impl AppState {",
)
replace_between(
    "src/app/input/mouse.rs",
    "    /// Map a new-shell hit-test result to a legacy `MouseAction`.\n",
    "    pub(super) fn handle_mouse(\n",
    "    /// Route a new-shell click without allowing handled shell controls to\n"
    "    /// fall through to the legacy mouse handler.\n"
    "    fn route_new_shell_click(&mut self, col: u16, row: u16) -> NewShellClickOutcome {\n"
    "        use crate::ui::shell::{hit_test_new_shell, HitTarget};\n\n"
    "        let Some(layout) = self.new_shell_layout.as_ref() else {\n"
    "            return NewShellClickOutcome::Unhandled;\n"
    "        };\n"
    "        let Some(target) = hit_test_new_shell(self, layout, col, row) else {\n"
    "            return NewShellClickOutcome::Unhandled;\n"
    "        };\n\n"
    "        match target {\n"
    "            HitTarget::SidebarModeToggle => {\n"
    "                self.new_sidebar_mode = (self.new_sidebar_mode + 1) % 3;\n"
    "                self.new_shell_dirty = true;\n"
    "                NewShellClickOutcome::Handled\n"
    "            }\n"
    "            HitTarget::SidebarWorkspace { ws_idx } => {\n"
    "                self.mode = Mode::Terminal;\n"
    "                NewShellClickOutcome::Action(MouseAction::FocusWorkspace { ws_idx })\n"
    "            }\n"
    "            HitTarget::SidebarAgent {\n"
    "                ws_idx, pane_id, ..\n"
    "            } => {\n"
    "                self.mode = Mode::Terminal;\n"
    "                NewShellClickOutcome::Action(MouseAction::FocusPane { ws_idx, pane_id })\n"
    "            }\n"
    "            HitTarget::SidebarAttention { .. } => {\n"
    "                self.mode = Mode::Terminal;\n"
    "                NewShellClickOutcome::Handled\n"
    "            }\n"
    "            HitTarget::Tab { index } => {\n"
    "                self.mode = Mode::Terminal;\n"
    "                NewShellClickOutcome::Action(MouseAction::FocusTab { tab_idx: index })\n"
    "            }\n"
    "            HitTarget::NewTab => {\n"
    "                if self.prompt_new_tab_name {\n"
    "                    super::modal::open_new_tab_dialog(self);\n"
    "                } else {\n"
    "                    self.request_new_tab = true;\n"
    "                    self.mode = Mode::Terminal;\n"
    "                }\n"
    "                NewShellClickOutcome::Handled\n"
    "            }\n"
    "            HitTarget::TabScrollLeft => {\n"
    "                self.scroll_tabs_left();\n"
    "                NewShellClickOutcome::Handled\n"
    "            }\n"
    "            HitTarget::TabScrollRight => {\n"
    "                self.scroll_tabs_right();\n"
    "                NewShellClickOutcome::Handled\n"
    "            }\n"
    "        }\n"
    "    }\n\n"
    "    #[cfg(test)]\n"
    "    fn handle_new_shell_click(&mut self, col: u16, row: u16) -> Option<MouseAction> {\n"
    "        match self.route_new_shell_click(col, row) {\n"
    "            NewShellClickOutcome::Action(action) => Some(action),\n"
    "            NewShellClickOutcome::Handled | NewShellClickOutcome::Unhandled => None,\n"
    "        }\n"
    "    }\n\n",
)
replace(
    "src/app/input/mouse.rs",
    "            if let Some(action) = self.handle_new_shell_click(mouse.column, mouse.row) {\n"
    "                return Some(action);\n"
    "            }",
    "            match self.route_new_shell_click(mouse.column, mouse.row) {\n"
    "                NewShellClickOutcome::Action(action) => return Some(action),\n"
    "                NewShellClickOutcome::Handled => return None,\n"
    "                NewShellClickOutcome::Unhandled => {}\n"
    "            }",
)
replace(
    "src/app/input/mouse.rs",
    "    use crate::ui::shell::{self, HitTarget, LayoutMode, ScrollState, ShellLayout, SidebarMode};\n"
    "    use crate::ui::sidebar_new::layout::row_at;\n",
    "    use crate::ui::shell::{self, LayoutMode, ScrollState, ShellLayout, SidebarMode};\n",
)
replace_between(
    "src/app/input/mouse.rs",
    "    /// Build a realistic `ShellLayout` in Standard mode with sidebar visible.\n",
    "    fn app_for_new_shell_test() -> crate::app::AppState {\n",
    "    /// Build a realistic `ShellLayout` in Standard mode with sidebar and\n"
    "    overflow controls populated.\n"
    "    fn new_shell_test_layout() -> ShellLayout {\n"
    "        let mut layout = shell::compute_shell_layout(\n"
    "            Rect::new(0, 0, 120, 40),\n"
    "            LayoutMode::Standard,\n"
    "            false,\n"
    "            28,\n"
    "            SidebarMode::Workspaces,\n"
    "            ScrollState { offset: 0, visible: 20, total: 5 },\n"
    "            ScrollState { offset: 0, visible: 20, total: 5 },\n"
    "            ScrollState { offset: 0, visible: 20, total: 5 },\n"
    "        );\n"
    "        let tabs: Vec<crate::ui::tabs_new::model::TabItem> = (0..24)\n"
    "            .map(|index| crate::ui::tabs_new::model::TabItem {\n"
    "                index,\n"
    "                label: format!(\"tab-{index}\"),\n"
    "                state: crate::detect::AgentState::Idle,\n"
    "                seen: true,\n"
    "                active: index == 0,\n"
    "                working: false,\n"
    "                blocked: false,\n"
    "                unseen_done: false,\n"
    "            })\n"
    "            .collect();\n"
    "        crate::ui::tabs_new::layout::layout_tab_bar(&mut layout.main.tab_bar, &tabs);\n"
    "        layout\n"
    "    }\n\n",
)

# Expose the current target so transitions are only retargeted when intent changes.
replace(
    "src/ui/motion.rs",
    "    /// Sample a scalar transition at the given time.\n"
    "    pub fn sample_scalar(&self, region: UiRegion, now: Instant) -> Option<f32> {\n"
    "        self.scalar.get(&region).map(|t| t.sample(now).0)\n"
    "    }",
    "    /// Sample a scalar transition at the given time.\n"
    "    pub fn sample_scalar(&self, region: UiRegion, now: Instant) -> Option<f32> {\n"
    "        self.scalar.get(&region).map(|t| t.sample(now).0)\n"
    "    }\n\n"
    "    /// Return the currently registered target for a scalar region.\n"
    "    pub fn scalar_target(&self, region: UiRegion) -> Option<f32> {\n"
    "        self.scalar.get(&region).map(Transition::target)\n"
    "    }",
)

# Use configured sidebar bounds, stop restarting transitions, and always rebuild
# the authoritative layout until explicit model invalidation exists.
replace(
    "src/ui/shell/mod.rs",
    "        let w = sidebar_width.clamp(22, 40);",
    "        let w = sidebar_width.min(area.width.saturating_sub(1));",
)
replace_between(
    "src/ui/shell/mod.rs",
    "    // ── Phase 7: Sidebar expand/collapse transition ─────────────────\n",
    "    // ── Phase 7: Attention pulse for blocked agents ─────────────────\n",
    "    // ── Phase 7: Sidebar expand/collapse transition ─────────────────\n"
    "    {\n"
    "        let region = crate::ui::motion::UiRegion::Sidebar;\n"
    "        let target_progress = if app.new_sidebar_collapsed { 0.0 } else { 1.0 };\n"
    "        let current = app\n"
    "            .new_transitions\n"
    "            .sample_scalar(region, now)\n"
    "            .unwrap_or(target_progress);\n"
    "        let target_changed = app\n"
    "            .new_transitions\n"
    "            .scalar_target(region)\n"
    "            .is_some_and(|target| (target - target_progress).abs() > 0.01);\n"
    "        if target_changed {\n"
    "            app.new_transitions.set(crate::ui::motion::Transition::new(\n"
    "                region,\n"
    "                now,\n"
    "                app.new_motion_policy\n"
    "                    .resolve_duration(std::time::Duration::from_millis(180)),\n"
    "                current,\n"
    "                target_progress,\n"
    "                crate::ui::motion::Easing::smooth,\n"
    "                crate::ui::motion::InterruptionPolicy::Retarget,\n"
    "            ));\n"
    "        }\n"
    "    }\n\n",
)
replace_between(
    "src/ui/shell/mod.rs",
    "    // ── Phase 8: Skip rebuild when nothing changed ─────────────────\n",
    "    // Build sidebar model and compute shell layout.\n",
    "    // Record the latest layout inputs. The model is rebuilt every pass so\n"
    "    // state and animation changes cannot leave cached geometry stale.\n"
    "    app.new_shell_layout_hash =\n"
    "        crate::ui::shell::layout_hash(area, app.new_sidebar_collapsed, app.new_sidebar_mode);\n"
    "    app.new_shell_dirty = false;\n\n",
)
replace(
    "src/ui/shell/mod.rs",
    "        app.sidebar_width,",
    "        app.sidebar_width\n"
    "            .clamp(app.sidebar_min_width, app.sidebar_max_width),",
    1,
)

# Render exactly the layout computed for hit testing instead of recomputing it.
replace_between(
    "src/ui/shell/mod.rs",
    "pub fn render_new_shell(\n",
    "fn top_line_of(rect: Rect) -> Rect {\n",
    "pub fn render_new_shell(\n"
    "    app: &crate::app::AppState,\n"
    "    terminal_runtimes: &TerminalRuntimeRegistry,\n"
    "    frame: &mut ratatui::Frame,\n"
    ") {\n"
    "    let area = frame.area();\n"
    "    let Some(shell_layout) = app.new_shell_layout.as_ref() else {\n"
    "        return;\n"
    "    };\n"
    "    let sidebar_mode = shell_layout.sidebar.mode;\n\n"
    "    let mut sidebar_model = crate::ui::sidebar_new::model::SidebarModel::new();\n"
    "    sidebar_model.rebuild(app, terminal_runtimes);\n"
    "    let tab_items = crate::ui::tabs_new::model::build_tabs(app);\n\n"
    "    let items = sidebar_model.items_for_mode(sidebar_mode).to_vec();\n"
    "    crate::ui::sidebar_new::render::render_sidebar_new(\n"
    "        app, terminal_runtimes, frame, shell_layout, &items,\n"
    "    );\n"
    "    crate::ui::tabs_new::render::render_tab_bar_new(\n"
    "        app, frame, &shell_layout.main.tab_bar, &tab_items, &app.palette,\n"
    "    );\n\n"
    "    let fleet_ctx = crate::ui::fleet_ops_new::model::build_fleet_ops_context(app);\n"
    "    if shell_layout.fleet_ops.rect.width > 0 && shell_layout.fleet_ops.rect.height > 0 {\n"
    "        crate::ui::fleet_ops_new::render::render_fleet_ops_new(\n"
    "            app, frame, &shell_layout.fleet_ops, &fleet_ctx, &app.palette,\n"
    "        );\n"
    "    }\n\n"
    "    let terminal_area = shell_layout.main.terminal;\n"
    "    if terminal_area.width > 0 && terminal_area.height > 0 {\n"
    "        use ratatui::widgets::Paragraph;\n"
    "        frame.render_widget(\n"
    "            Paragraph::new(\"[new shell terminal area — integration pending]\"),\n"
    "            terminal_area,\n"
    "        );\n"
    "    }\n\n"
    "    if app.new_launcher_open {\n"
    "        let mut launcher_layout = crate::ui::launcher_new::layout::layout_launcher(area);\n"
    "        let launcher_items = crate::ui::launcher_new::model::build_launcher_items(app);\n"
    "        crate::ui::launcher_new::layout::layout_launcher_rows(\n"
    "            &mut launcher_layout, launcher_items.len(),\n"
    "        );\n"
    "        crate::ui::launcher_new::render::render_launcher_new(\n"
    "            app, frame, &launcher_layout, &launcher_items,\n"
    "            app.new_launcher_selected, &app.palette,\n"
    "        );\n"
    "    }\n\n"
    "    if app.new_settings_open {\n"
    "        let settings_layout = crate::ui::settings_new::layout::layout_settings(area);\n"
    "        crate::ui::settings_new::render::render_settings_new(\n"
    "            app, frame, &settings_layout,\n"
    "            crate::ui::settings_new::model::SettingsCategory::Appearance,\n"
    "            &app.palette,\n"
    "        );\n"
    "    }\n"
    "}\n\n",
)

# Allocate concrete tab controls and keep the active tab in the visible window.
replace_between(
    "src/ui/tabs_new/layout.rs",
    "/// Minimum width reserved for a tab, in cells.\n",
    "/// Return the visible tab index at the given x coordinate, if any.\n",
    "/// Minimum width reserved for a tab, in cells.\n"
    "const MIN_TAB_WIDTH: u16 = 4;\n"
    "const TAB_GAP: u16 = 1;\n"
    "const NEW_TAB_WIDTH: u16 = 3;\n"
    "const SCROLL_WIDTH: u16 = 2;\n\n"
    "/// Compute tab rectangles and write them into `layout`.\n"
    "pub fn layout_tab_bar(layout: &mut TabBarLayout, tabs: &[TabItem]) -> usize {\n"
    "    let area = layout.rect;\n"
    "    layout.tabs.clear();\n"
    "    layout.scroll_left = Rect::default();\n"
    "    layout.scroll_right = Rect::default();\n"
    "    layout.new_tab = Rect::default();\n"
    "    layout.overflow = false;\n\n"
    "    if area.width == 0 || area.height == 0 {\n"
    "        return 0;\n"
    "    }\n\n"
    "    let new_tab_width = area.width.min(NEW_TAB_WIDTH);\n"
    "    layout.new_tab = Rect::new(\n"
    "        area.x + area.width.saturating_sub(new_tab_width),\n"
    "        area.y, new_tab_width, area.height,\n"
    "    );\n"
    "    if tabs.is_empty() {\n"
    "        return 0;\n"
    "    }\n\n"
    "    let base_available = layout.new_tab.x.saturating_sub(area.x);\n"
    "    if base_available == 0 {\n"
    "        layout.overflow = true;\n"
    "        return 0;\n"
    "    }\n\n"
    "    let widths: Vec<u16> = tabs\n"
    "        .iter()\n"
    "        .map(|tab| {\n"
    "            let label_width = display_width(&tab.label).max(1) as u16;\n"
    "            label_width.max(MIN_TAB_WIDTH).min(base_available)\n"
    "        })\n"
    "        .collect();\n"
    "    let total_width = widths.iter().fold(0u16, |total, width| {\n"
    "        total.saturating_add(width.saturating_add(TAB_GAP))\n"
    "    });\n"
    "    let needs_overflow = total_width > base_available;\n\n"
    "    let (tab_start_x, available) = if needs_overflow\n"
    "        && base_available > SCROLL_WIDTH.saturating_mul(2)\n"
    "    {\n"
    "        layout.scroll_left = Rect::new(area.x, area.y, SCROLL_WIDTH, area.height);\n"
    "        layout.scroll_right = Rect::new(\n"
    "            layout.new_tab.x.saturating_sub(SCROLL_WIDTH),\n"
    "            area.y, SCROLL_WIDTH, area.height,\n"
    "        );\n"
    "        let start = area.x.saturating_add(SCROLL_WIDTH);\n"
    "        (start, layout.scroll_right.x.saturating_sub(start))\n"
    "    } else {\n"
    "        (area.x, base_available)\n"
    "    };\n\n"
    "    let fit_end = |start: usize| -> usize {\n"
    "        let mut used = 0u16;\n"
    "        let mut end = start;\n"
    "        for (index, width) in widths.iter().enumerate().skip(start) {\n"
    "            let needed = width.saturating_add(TAB_GAP);\n"
    "            if used.saturating_add(needed) > available {\n"
    "                break;\n"
    "            }\n"
    "            used = used.saturating_add(needed);\n"
    "            end = index + 1;\n"
    "        }\n"
    "        end\n"
    "    };\n\n"
    "    let active_index = tabs.iter().position(|tab| tab.active).unwrap_or(0);\n"
    "    let mut visible_start = 0usize;\n"
    "    let mut visible_end = fit_end(visible_start);\n"
    "    if active_index >= visible_end {\n"
    "        visible_start = active_index;\n"
    "        let mut used = widths[active_index].saturating_add(TAB_GAP);\n"
    "        while visible_start > 0 {\n"
    "            let previous = widths[visible_start - 1].saturating_add(TAB_GAP);\n"
    "            if used.saturating_add(previous) > available {\n"
    "                break;\n"
    "            }\n"
    "            visible_start -= 1;\n"
    "            used = used.saturating_add(previous);\n"
    "        }\n"
    "        visible_end = fit_end(visible_start);\n"
    "    }\n\n"
    "    layout.overflow = visible_start > 0 || visible_end < tabs.len();\n"
    "    let mut x = tab_start_x;\n"
    "    layout.tabs = (visible_start..visible_end)\n"
    "        .map(|index| {\n"
    "            let width = widths[index].min(available);\n"
    "            let rect = Rect::new(x, area.y, width, area.height);\n"
    "            x = x.saturating_add(width.saturating_add(TAB_GAP));\n"
    "            crate::ui::shell::TabHitRect { rect, index, visible: true }\n"
    "        })\n"
    "        .collect();\n\n"
    "    visible_start\n"
    "}\n\n",
)
replace(
    "src/ui/tabs_new/layout.rs",
    "        assert!(layout.overflow);\n        assert!(layout.tabs.len() < 10);",
    "        assert!(layout.overflow);\n"
    "        assert!(layout.tabs.len() < 10);\n"
    "        assert!(layout.scroll_left.width > 0);\n"
    "        assert!(layout.scroll_right.width > 0);\n"
    "        assert!(layout.new_tab.width > 0);\n"
    "        assert!(layout.scroll_left.x < layout.scroll_right.x);",
)

# Draw the controls whose rectangles are used by hit testing.
replace(
    "src/ui/tabs_new/render.rs",
    "    widgets::Paragraph,",
    "    widgets::Paragraph,",
)
replace(
    "src/ui/tabs_new/render.rs",
    "    for hit in &layout.tabs {",
    "    if layout.scroll_left.width > 0 {\n"
    "        frame.render_widget(\n"
    "            Paragraph::new(\"‹\").style(Style::default().fg(palette.overlay0)),\n"
    "            layout.scroll_left,\n"
    "        );\n"
    "    }\n"
    "    if layout.scroll_right.width > 0 {\n"
    "        frame.render_widget(\n"
    "            Paragraph::new(\"›\").style(Style::default().fg(palette.overlay0)),\n"
    "            layout.scroll_right,\n"
    "        );\n"
    "    }\n"
    "    if layout.new_tab.width > 0 {\n"
    "        frame.render_widget(\n"
    "            Paragraph::new(\" +\").style(Style::default().fg(palette.overlay0)),\n"
    "            layout.new_tab,\n"
    "        );\n"
    "    }\n\n"
    "    for hit in &layout.tabs {",
)
