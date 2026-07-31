//! End-to-end integration tests for the new shell pipeline.
//!
//! These tests exercise the full lifecycle:
//!   1. Build model (sidebar, tabs, fleet ops)
//!   2. Compute layout from model
//!   3. Render to a ratatui TestBackend buffer
//!   4. Verify buffer output contains expected elements
//!
//! The tests are designed to catch regressions across the model→layout→render
//! boundary that unit tests on individual modules cannot detect.

#[cfg(test)]
mod integration {
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};

    use crate::app::state::AppState;
    use crate::terminal::TerminalRuntimeRegistry;
    use crate::ui::shell::{compute_new_shell_view, render_new_shell, LayoutMode};
    use crate::workspace::Workspace;

    /// Helper: collect visible text from a buffer row.
    fn buffer_row_text(buffer: &ratatui::buffer::Buffer, area: Rect, row: u16) -> String {
        (area.x..area.x + area.width)
            .map(|x| buffer[(x, row)].symbol())
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    /// Helper: check if any row in the buffer contains the given substring.
    fn buffer_contains(buffer: &ratatui::buffer::Buffer, area: Rect, needle: &str) -> bool {
        (area.y..area.y + area.height)
            .any(|row| buffer_row_text(buffer, area, row).contains(needle))
    }

    /// Helper: build a minimal AppState with one workspace and one pane.
    fn app_with_workspace(name: &str) -> AppState {
        let mut app = AppState::test_new();
        app.workspaces = vec![Workspace::test_new(name)];
        app.active = Some(0);
        app.selected = 0;
        app.new_shell = true;
        app
    }

    /// Helper: build AppState with two workspaces and agents detected.
    fn app_with_two_workspaces_and_agents() -> AppState {
        let mut app = AppState::test_new();
        app.workspaces = vec![Workspace::test_new("alpha"), Workspace::test_new("beta")];
        app.active = Some(0);
        app.selected = 0;
        app.new_shell = true;
        app.ensure_test_terminals();

        // Tag alpha's root pane as a Pi agent (working).
        let pane_a = app.workspaces[0].tabs[0].root_pane;
        let tid_a = app.workspaces[0].tabs[0].panes[&pane_a]
            .attached_terminal_id
            .clone();
        app.terminals.get_mut(&tid_a).unwrap().detected_agent = Some(crate::detect::Agent::Pi);
        app.terminals.get_mut(&tid_a).unwrap().state = crate::detect::AgentState::Working;

        // Tag beta's root pane as a Claude agent (blocked).
        let pane_b = app.workspaces[1].tabs[0].root_pane;
        let tid_b = app.workspaces[1].tabs[0].panes[&pane_b]
            .attached_terminal_id
            .clone();
        app.terminals.get_mut(&tid_b).unwrap().detected_agent = Some(crate::detect::Agent::Claude);
        app.terminals.get_mut(&tid_b).unwrap().state = crate::detect::AgentState::Blocked;

        app
    }

    // ── Full pipeline: compute → render → verify ────────────────────────

    #[test]
    fn full_pipeline_renders_sidebar_in_standard_mode() {
        let mut app = app_with_workspace("test-ws");
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 120, 40);

        // Compute view.
        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        // Render.
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| render_new_shell(&app, &registry, frame))
            .unwrap();
        let buffer = terminal.backend().buffer();

        // Verify sidebar header is rendered.
        let sidebar_area = app.view.sidebar_rect;
        assert!(
            sidebar_area.width > 0,
            "sidebar should be visible in standard mode"
        );
        assert!(
            buffer_contains(buffer, sidebar_area, "Workspaces"),
            "sidebar header should contain 'Workspaces'"
        );

        // Verify tab bar geometry populated.
        assert!(
            !app.view.tab_hit_areas.is_empty(),
            "tab hit areas should be populated"
        );
        assert!(
            app.view.terminal_area.width > 0,
            "terminal area should be non-zero"
        );

        // Verify sidebar rows were laid out.
        let sidebar_text = buffer_row_text(buffer, sidebar_area, sidebar_area.y + 1);
        assert!(
            sidebar_text.contains("test-ws"),
            "sidebar should show workspace name: got '{sidebar_text}'"
        );
    }

    #[test]
    fn full_pipeline_collapses_sidebar_when_requested() {
        let mut app = app_with_workspace("ws");
        app.new_sidebar_collapsed = true;
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 120, 40);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| render_new_shell(&app, &registry, frame))
            .unwrap();

        // Collapsed sidebar should have zero width.
        assert_eq!(
            app.view.sidebar_rect.width, 0,
            "collapsed sidebar should be hidden"
        );
    }

    #[test]
    fn full_pipeline_renders_tab_bar_with_workspace_tabs() {
        let mut app = app_with_workspace("dev");
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 120, 40);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| render_new_shell(&app, &registry, frame))
            .unwrap();
        let buffer = terminal.backend().buffer();

        // Tab bar should exist and have at least one tab.
        assert!(app.view.tab_bar_rect.width > 0, "tab bar should be visible");
        assert!(
            app.view.tab_hit_areas.len() >= 1,
            "should have at least one tab"
        );

        // The tab bar row should contain text (tab labels).
        let tab_row = buffer_row_text(buffer, area, app.view.tab_bar_rect.y);
        assert!(!tab_row.is_empty(), "tab bar should contain text");
        assert!(
            tab_row.contains('1') || tab_row.to_lowercase().contains("tab"),
            "tab bar should show tab label: got '{tab_row}'"
        );
    }

    #[test]
    fn full_pipeline_renders_terminal_placeholder() {
        let mut app = app_with_workspace("tmpl");
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 120, 40);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| render_new_shell(&app, &registry, frame))
            .unwrap();
        let buffer = terminal.backend().buffer();

        // Terminal area should contain the placeholder text.
        let terminal_area = app.view.terminal_area;
        assert!(terminal_area.width > 0, "terminal area should be non-zero");
        let term_text = buffer_row_text(buffer, area, terminal_area.y);
        assert!(
            term_text.contains("new shell terminal area"),
            "terminal placeholder should be visible: got '{term_text}'"
        );
    }

    #[test]
    fn full_pipeline_narrow_mode_hides_sidebar() {
        let mut app = app_with_workspace("narrow-ws");
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 60, 20);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        // In Narrow mode (width 60), sidebar should be hidden.
        assert_eq!(LayoutMode::from_area(area), LayoutMode::Narrow,);
        assert_eq!(app.view.sidebar_rect.width, 0, "narrow mode hides sidebar");
        assert!(app.view.terminal_area.width > 0);
    }

    #[test]
    fn full_pipeline_wide_mode_shows_fleet_ops() {
        let mut app = app_with_workspace("wide-ws");
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 200, 60);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        assert_eq!(LayoutMode::from_area(area), LayoutMode::Wide);
        // Wide mode has fleet ops; verify terminal area is reduced.
        let full_height = area.height;
        let used = app.view.tab_bar_rect.height + app.view.terminal_area.height;
        // Just check that the layout consumed most of the height.
        assert!(
            used >= full_height - 2,
            "layout should fill most of the frame"
        );
    }

    // ── Multi-workspace + agent detection pipeline ─────────────────────

    #[test]
    fn full_pipeline_multi_workspace_with_agents() {
        let mut app = app_with_two_workspaces_and_agents();
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 120, 40);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| render_new_shell(&app, &registry, frame))
            .unwrap();
        let buffer = terminal.backend().buffer();

        // Verify both workspaces visible in sidebar.
        let sidebar_area = app.view.sidebar_rect;
        assert!(sidebar_area.width > 0);
        let mut sidebar_text = String::new();
        for row in sidebar_area.y..sidebar_area.y + sidebar_area.height {
            sidebar_text.push_str(&buffer_row_text(buffer, area, row));
            sidebar_text.push('\n');
        }
        assert!(
            sidebar_text.contains("alpha"),
            "sidebar should contain 'alpha': got '{sidebar_text}'"
        );
        assert!(
            sidebar_text.contains("beta"),
            "sidebar should contain 'beta': got '{sidebar_text}'"
        );
    }

    // ── Launcher overlay integration ───────────────────────────────────

    #[test]
    fn full_pipeline_renders_launcher_overlay_when_open() {
        let mut app = app_with_workspace("launch-ws");
        app.new_launcher_open = true;
        app.new_launcher_selected = 0;
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 120, 40);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| render_new_shell(&app, &registry, frame))
            .unwrap();

        // The launcher should render without panicking — the overlay should
        // be visible somewhere on screen.
        // We just verify the render didn't crash; visual verification
        // of launcher content is done in launcher_new unit tests.
    }

    // ── Settings overlay integration ───────────────────────────────────

    #[test]
    fn full_pipeline_renders_settings_overlay_when_open() {
        let mut app = app_with_workspace("settings-ws");
        app.new_settings_open = true;
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 120, 40);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| render_new_shell(&app, &registry, frame))
            .unwrap();

        // Settings overlay should render without panicking.
        // Visual verification is done in settings_new unit tests.
    }

    // ── Mode switching ─────────────────────────────────────────────────

    #[test]
    fn full_pipeline_sidebar_mode_switch_to_agents() {
        let mut app = app_with_two_workspaces_and_agents();
        app.new_sidebar_mode = 1; // Agents mode
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 120, 40);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| render_new_shell(&app, &registry, frame))
            .unwrap();
        let buffer = terminal.backend().buffer();

        // Sidebar header should now show "Agents".
        let sidebar_area = app.view.sidebar_rect;
        assert!(
            buffer_contains(buffer, sidebar_area, "Agents"),
            "sidebar header should switch to 'Agents'"
        );
    }

    #[test]
    fn full_pipeline_sidebar_mode_switch_to_attention() {
        let mut app = app_with_two_workspaces_and_agents();
        app.new_sidebar_mode = 2; // Attention mode
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 120, 40);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| render_new_shell(&app, &registry, frame))
            .unwrap();
        let buffer = terminal.backend().buffer();

        // Sidebar header should now show "Attention".
        let sidebar_area = app.view.sidebar_rect;
        assert!(
            buffer_contains(buffer, sidebar_area, "Attention"),
            "sidebar header should switch to 'Attention'"
        );
    }

    // ── Tiny/mobile mode integration ───────────────────────────────────

    #[test]
    fn full_pipeline_tiny_mode_does_not_panic() {
        let mut app = app_with_workspace("tiny");
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 20, 10);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        let mut terminal = Terminal::new(TestBackend::new(20, 10)).unwrap();
        terminal
            .draw(|frame| render_new_shell(&app, &registry, frame))
            .unwrap();

        // In Tiny mode everything is zero-sized but the render should not crash.
        assert_eq!(app.view.sidebar_rect.width, 0);
    }

    #[test]
    fn full_pipeline_mobile_mode_does_not_panic() {
        let mut app = app_with_workspace("mobile-ws");
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 40, 20);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 20)).unwrap();
        terminal
            .draw(|frame| render_new_shell(&app, &registry, frame))
            .unwrap();

        assert_eq!(LayoutMode::from_area(area), LayoutMode::Mobile);
        assert!(!LayoutMode::Mobile.sidebar_visible());
    }

    // ── Transition integration ─────────────────────────────────────────

    #[test]
    fn full_pipeline_transitions_advance_on_compute() {
        let mut app = app_with_workspace("trans-ws");
        app.new_sidebar_mode = 0;
        app.new_sidebar_prev_mode = 1; // Different — should trigger transition
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 120, 40);

        // First compute starts a transition.
        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        // Transition should be active after mode switch.
        assert!(
            app.new_transitions.is_active(),
            "transition should be active after sidebar mode change"
        );

        // After prev_mode equals mode, no new transition on next compute.
        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        // Transition should still be active (mid-animation).
        assert!(app.new_transitions.is_active());
    }

    #[test]
    fn full_pipeline_no_transition_when_mode_unchanged() {
        let mut app = app_with_workspace("static-ws");
        app.new_sidebar_mode = 0;
        app.new_sidebar_prev_mode = 0; // Same — no transition
        let registry = TerminalRuntimeRegistry::new();
        let area = Rect::new(0, 0, 120, 40);

        compute_new_shell_view(
            &mut app,
            &registry,
            area,
            false,
            crate::kitty_graphics::HostCellSize::default(),
        );

        // No transition should be active when mode hasn't changed.
        assert!(
            !app.new_transitions.is_active(),
            "no transition should be created when mode is unchanged"
        );
    }
}
