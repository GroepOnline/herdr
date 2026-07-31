from pathlib import Path

path = Path("src/ui/shell/mod.rs")
text = path.read_text()
old = '''    // ── Phase 7: Sidebar expand/collapse transition ─────────────────
    {
        let region = crate::ui::motion::UiRegion::Sidebar;
        let target_progress = if app.new_sidebar_collapsed { 0.0 } else { 1.0 };
        let current = app
            .new_transitions
            .sample_scalar(region, now)
            .unwrap_or(target_progress);
        let target_changed = app
            .new_transitions
            .scalar_target(region)
            .is_some_and(|target| (target - target_progress).abs() > 0.01);
        if target_changed {
            app.new_transitions.set(crate::ui::motion::Transition::new(
                region,
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

'''
new = '''    // ── Phase 7: Sidebar expand/collapse transition ─────────────────
    {
        let region = crate::ui::motion::UiRegion::Sidebar;
        let target_progress = if app.new_sidebar_collapsed { 0.0 } else { 1.0 };
        let previous_progress = app
            .new_shell_layout
            .as_ref()
            .map(|layout| if layout.sidebar.collapsed { 0.0 } else { 1.0 })
            .unwrap_or(target_progress);
        let current = app
            .new_transitions
            .sample_scalar(region, now)
            .unwrap_or(previous_progress);
        let should_retarget = app.new_transitions.scalar_target(region).map_or(
            (previous_progress - target_progress).abs() > 0.01,
            |target| (target - target_progress).abs() > 0.01,
        );
        if should_retarget {
            app.new_transitions.set(crate::ui::motion::Transition::new(
                region,
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

'''
if text.count(old) != 1:
    raise SystemExit("generated transition block not found exactly once")
text = text.replace(old, new)
old_condition = "    let (sidebar_rect, main_rect) = if mode.sidebar_visible() && !sidebar_collapsed {"
new_condition = "    let (sidebar_rect, main_rect) = if mode.sidebar_visible() && (!sidebar_collapsed || progress > 0.0) {"
if text.count(old_condition) != 1:
    raise SystemExit("sidebar animation condition not found exactly once")
path.write_text(text.replace(old_condition, new_condition))
