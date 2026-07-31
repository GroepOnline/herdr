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


replace(
    "src/ui/shell/mod.rs",
    "    _resize_panes: bool,\n    _cell_size: crate::kitty_graphics::HostCellSize,",
    "    resize_panes: bool,\n    cell_size: crate::kitty_graphics::HostCellSize,",
)

replace(
    "src/ui/shell/mod.rs",
    "    crate::ui::tabs_new::layout::layout_tab_bar(&mut shell_layout.main.tab_bar, &tab_items);\n\n    // Cache the shell layout for hit-test routing.",
    "    crate::ui::tabs_new::layout::layout_tab_bar(&mut shell_layout.main.tab_bar, &tab_items);\n\n"
    "    // Reuse the established pane geometry/runtime path inside the new shell.\n"
    "    let tab_surface = super::compute_tab_surface(\n"
    "        app,\n"
    "        terminal_runtimes,\n"
    "        shell_layout.main.terminal,\n"
    "        resize_panes,\n"
    "        cell_size,\n"
    "    );\n"
    "    if resize_panes {\n"
    "        super::resize_background_tab_panes_to_area(\n"
    "            app,\n"
    "            terminal_runtimes,\n"
    "            shell_layout.main.terminal,\n"
    "            cell_size,\n"
    "        );\n"
    "        super::resize_popup_pane(\n"
    "            app,\n"
    "            terminal_runtimes,\n"
    "            shell_layout.main.terminal,\n"
    "            cell_size,\n"
    "        );\n"
    "    }\n\n"
    "    // Cache the shell layout for hit-test routing.",
)

replace(
    "src/ui/shell/mod.rs",
    "        pane_infos: Vec::new(),\n        split_borders: Vec::new(),\n    };\n}",
    "        pane_infos: tab_surface.pane_infos,\n"
    "        split_borders: tab_surface.split_borders,\n"
    "    };\n"
    "    app.sync_copy_mode_search_geometry();\n"
    "}",
)

replace(
    "src/ui/shell/mod.rs",
    "    let terminal_area = shell_layout.main.terminal;\n"
    "    if terminal_area.width > 0 && terminal_area.height > 0 {\n"
    "        use ratatui::widgets::Paragraph;\n"
    "        frame.render_widget(\n"
    "            Paragraph::new(\"[new shell terminal area — integration pending]\"),\n"
    "            terminal_area,\n"
    "        );\n"
    "    }",
    "    let terminal_area = shell_layout.main.terminal;\n"
    "    if app\n"
    "        .active\n"
    "        .and_then(|ws_idx| app.workspaces.get(ws_idx))\n"
    "        .is_some()\n"
    "    {\n"
    "        super::render_tab_surface(app, terminal_runtimes, app.view.tab_surface(), frame);\n"
    "    } else {\n"
    "        super::render_empty(app, frame, terminal_area);\n"
    "    }\n\n"
    "    super::render_notifications(app, frame, terminal_area);\n"
    "    super::render_popup_pane(app, terminal_runtimes, frame, terminal_area);",
)

replace(
    "src/ui/shell/integration_tests.rs",
    "    fn full_pipeline_renders_terminal_placeholder() {\n"
    "        let mut app = app_with_workspace(\"tmpl\");\n"
    "        let registry = TerminalRuntimeRegistry::new();",
    "    fn full_pipeline_renders_real_terminal_surface() {\n"
    "        let mut app = app_with_workspace(\"tmpl\");\n"
    "        let pane_id = app.workspaces[0].tabs[0].root_pane;\n"
    "        app.workspaces[0].insert_test_runtime(\n"
    "            pane_id,\n"
    "            crate::terminal::TerminalRuntime::test_with_screen_bytes(\n"
    "                40,\n"
    "                10,\n"
    "                b\"NEW-SHELL-TERMINAL\",\n"
    "            ),\n"
    "        );\n"
    "        app.mode = crate::app::Mode::Terminal;\n"
    "        let registry = TerminalRuntimeRegistry::new();",
)
replace(
    "src/ui/shell/integration_tests.rs",
    "        // Terminal area should contain the placeholder text.\n"
    "        let terminal_area = app.view.terminal_area;\n"
    "        assert!(terminal_area.width > 0, \"terminal area should be non-zero\");\n"
    "        let term_text = buffer_row_text(buffer, area, terminal_area.y);\n"
    "        assert!(\n"
    "            term_text.contains(\"new shell terminal area\"),\n"
    "            \"terminal placeholder should be visible: got '{term_text}'\"\n"
    "        );",
    "        let terminal_area = app.view.terminal_area;\n"
    "        assert!(terminal_area.width > 0, \"terminal area should be non-zero\");\n"
    "        assert!(\n"
    "            buffer_contains(buffer, terminal_area, \"NEW-SHELL-TERMINAL\"),\n"
    "            \"terminal runtime content should be visible\"\n"
    "        );\n"
    "        assert!(!buffer_contains(\n"
    "            buffer,\n"
    "            terminal_area,\n"
    "            \"new shell terminal area\"\n"
    "        ));",
)

replace(
    "src/app/runtime.rs",
    "        if self.agent_panel_has_animation() || self.state.mode == crate::app::Mode::Settings {",
    "        if self.agent_panel_has_animation()\n"
    "            || self.state.mode == crate::app::Mode::Settings\n"
    "            || (self.state.new_shell && self.state.new_transitions.is_active())\n"
    "        {",
)

marker = "    #[test]\n    fn headless_deadline_can_suppress_git_refresh_timer() {"
test = """    #[test]
    fn new_shell_transition_schedules_deterministic_animation_deadline() {
        let mut app = super::super::App::new(
            &crate::config::Config::default(),
            true,
            None,
            tokio::sync::mpsc::unbounded_channel().1,
            crate::api::EventHub::default(),
        );
        app.state.new_shell = true;
        let now = Instant::now();
        let interval = Duration::from_millis(17);
        app.state
            .new_transitions
            .set(crate::ui::motion::Transition::new(
                crate::ui::motion::UiRegion::Sidebar,
                now,
                Duration::from_millis(100),
                0.0,
                1.0,
                crate::ui::motion::Easing::linear,
                crate::ui::motion::InterruptionPolicy::Retarget,
            ));

        app.sync_animation_timer_with_interval(now, interval);

        assert_eq!(app.next_animation_tick, Some(now + interval));
        assert_eq!(
            app.next_headless_loop_deadline_with_git_refresh(now, false, false),
            Some(now + interval)
        );
    }

"""
replace("src/app/runtime.rs", marker, test + marker)
