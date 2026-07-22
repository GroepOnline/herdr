use super::App;

impl App {
    pub(super) fn update_config_file<F>(&mut self, error_context: &str, update: F) -> bool
    where
        F: FnOnce(&str) -> String,
    {
        #[cfg(test)]
        if std::env::var_os(crate::config::CONFIG_PATH_ENV_VAR).is_none() {
            return false;
        }

        let path = crate::config::config_path();
        if let Some(parent) = path.parent() {
            if let Err(err) = std::fs::create_dir_all(parent) {
                crate::logging::config_write_failed(&path, error_context, &err.to_string());
                self.state.config_diagnostic =
                    Some(format!("failed to save {error_context}: {err}"));
                self.config_diagnostic_deadline =
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(5));
                return false;
            }
        }

        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let new_content = update(&content);
        if let Err(err) = std::fs::write(&path, new_content) {
            crate::logging::config_write_failed(&path, error_context, &err.to_string());
            self.state.config_diagnostic = Some(format!("failed to save {error_context}: {err}"));
            self.config_diagnostic_deadline =
                Some(std::time::Instant::now() + std::time::Duration::from_secs(5));
            return false;
        }

        true
    }

    pub(super) fn mark_onboarding_complete(&mut self) {
        self.update_config_file("onboarding setting", |content| {
            crate::config::upsert_top_level_bool(content, "onboarding", false)
        });
    }

    pub(super) fn save_theme(&mut self, name: &str) {
        if self.update_config_file("theme", |content| {
            let content = crate::config::upsert_section_value(
                content,
                "theme",
                "name",
                &format!("\"{name}\""),
            );
            crate::config::upsert_section_bool(&content, "theme", "auto_switch", false)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_sound(&mut self, enabled: bool) {
        if self.update_config_file("sound setting", |content| {
            crate::config::upsert_section_bool(content, "ui.sound", "enabled", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_toast_delivery(&mut self, delivery: crate::config::ToastDelivery) {
        let value = match delivery {
            crate::config::ToastDelivery::Off => "\"off\"",
            crate::config::ToastDelivery::Herdr => "\"herdr\"",
            crate::config::ToastDelivery::Terminal => "\"terminal\"",
            crate::config::ToastDelivery::System => "\"system\"",
        };
        if self.update_config_file("toast setting", |content| {
            let content =
                crate::config::upsert_section_value(content, "ui.toast", "delivery", value);
            crate::config::remove_section_key(&content, "ui.toast", "enabled")
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_agent_border_labels(&mut self, enabled: bool) {
        if self.update_config_file("agent border labels", |content| {
            crate::config::upsert_section_bool(
                content,
                "ui",
                "show_agent_labels_on_pane_borders",
                enabled,
            )
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_pane_borders(&mut self, enabled: bool) {
        if self.update_config_file("pane borders", |content| {
            crate::config::upsert_section_bool(content, "ui", "pane_borders", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_pane_gaps(&mut self, enabled: bool) {
        if self.update_config_file("pane gaps", |content| {
            crate::config::upsert_section_bool(content, "ui", "pane_gaps", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_hide_tab_bar_when_single_tab(&mut self, enabled: bool) {
        if self.update_config_file("hide tab bar when single tab", |content| {
            crate::config::upsert_section_bool(
                content,
                "ui",
                "hide_tab_bar_when_single_tab",
                enabled,
            )
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_pane_history_persistence(&mut self, enabled: bool) {
        if self.update_config_file("pane screen history", |content| {
            crate::config::upsert_section_bool(content, "experimental", "pane_history", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_switch_ascii_input_source_in_prefix(&mut self, enabled: bool) {
        if self.update_config_file("prefix ascii input source", |content| {
            crate::config::upsert_section_bool(
                content,
                "experimental",
                "switch_ascii_input_source_in_prefix",
                enabled,
            )
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_spinner_style(&mut self, style: crate::config::SpinnerStyle) {
        let value = match style {
            crate::config::SpinnerStyle::Dots => "\"dots\"",
            crate::config::SpinnerStyle::DotsFull => "\"dots-full\"",
            crate::config::SpinnerStyle::DotsCorner => "\"dots-corner\"",
            crate::config::SpinnerStyle::Arc => "\"arc\"",
            crate::config::SpinnerStyle::Circle => "\"circle\"",
            crate::config::SpinnerStyle::CircleQuarters => "\"circle-quarters\"",
            crate::config::SpinnerStyle::CircleHalves => "\"circle-halves\"",
            crate::config::SpinnerStyle::SquareCorners => "\"square-corners\"",
            crate::config::SpinnerStyle::Triangle => "\"triangle\"",
            crate::config::SpinnerStyle::Star => "\"star\"",
            crate::config::SpinnerStyle::Star2 => "\"star2\"",
            crate::config::SpinnerStyle::Arrow => "\"arrow\"",
            crate::config::SpinnerStyle::Arrow3 => "\"arrow3\"",
            crate::config::SpinnerStyle::Bounce => "\"bounce\"",
            crate::config::SpinnerStyle::BoxBounce => "\"box-bounce\"",
            crate::config::SpinnerStyle::Pipe => "\"pipe\"",
            crate::config::SpinnerStyle::Noise => "\"noise\"",
            crate::config::SpinnerStyle::Aesthetic => "\"aesthetic\"",
            crate::config::SpinnerStyle::GrowVertical => "\"grow-vertical\"",
            crate::config::SpinnerStyle::GrowHorizontal => "\"grow-horizontal\"",
            crate::config::SpinnerStyle::Point => "\"point\"",
            crate::config::SpinnerStyle::BetaWave => "\"beta-wave\"",
            crate::config::SpinnerStyle::Layer => "\"layer\"",
            crate::config::SpinnerStyle::Liquid => "\"liquid\"",
            crate::config::SpinnerStyle::Crystal => "\"crystal\"",
            crate::config::SpinnerStyle::Galaxy => "\"galaxy\"",
            crate::config::SpinnerStyle::Vortex => "\"vortex\"",
            crate::config::SpinnerStyle::Toggle => "\"toggle\"",
            crate::config::SpinnerStyle::Flip => "\"flip\"",
            crate::config::SpinnerStyle::Sandwich => "\"sandwich\"",
            crate::config::SpinnerStyle::BouncingBar => "\"bouncing-bar\"",
            crate::config::SpinnerStyle::BouncingBall => "\"bouncing-ball\"",
            crate::config::SpinnerStyle::Pong => "\"pong\"",
            crate::config::SpinnerStyle::Shark => "\"shark\"",
            crate::config::SpinnerStyle::Fish => "\"fish\"",
            crate::config::SpinnerStyle::Binary => "\"binary\"",
            crate::config::SpinnerStyle::DotsCircle => "\"dots-circle\"",
            crate::config::SpinnerStyle::Sand => "\"sand\"",
            crate::config::SpinnerStyle::Dots8Bit => "\"dots8-bit\"",
            crate::config::SpinnerStyle::Moon => "\"moon\"",
            crate::config::SpinnerStyle::Clock => "\"clock\"",
            crate::config::SpinnerStyle::Earth => "\"earth\"",
            crate::config::SpinnerStyle::Weather => "\"weather\"",
            crate::config::SpinnerStyle::Hearts => "\"hearts\"",
            crate::config::SpinnerStyle::Balloon => "\"balloon\"",
            crate::config::SpinnerStyle::Grenade => "\"grenade\"",
            crate::config::SpinnerStyle::FingerDance => "\"finger-dance\"",
            crate::config::SpinnerStyle::FistBump => "\"fist-bump\"",
            crate::config::SpinnerStyle::Smiley => "\"smiley\"",
            crate::config::SpinnerStyle::Monkey => "\"monkey\"",
            crate::config::SpinnerStyle::Speaker => "\"speaker\"",
            crate::config::SpinnerStyle::Runner => "\"runner\"",
            crate::config::SpinnerStyle::SoccerHeader => "\"soccer-header\"",
            crate::config::SpinnerStyle::Mindblown => "\"mindblown\"",
            crate::config::SpinnerStyle::OrangePulse => "\"orange-pulse\"",
            crate::config::SpinnerStyle::BluePulse => "\"blue-pulse\"",
            crate::config::SpinnerStyle::OrangeBluePulse => "\"orange-blue-pulse\"",
            crate::config::SpinnerStyle::TimeTravel => "\"time-travel\"",
            crate::config::SpinnerStyle::Christmas => "\"christmas\"",
            crate::config::SpinnerStyle::Flame => "\"flame\"",
            crate::config::SpinnerStyle::Pizza => "\"pizza\"",
            crate::config::SpinnerStyle::Dizzy => "\"dizzy\"",
            crate::config::SpinnerStyle::Ninja => "\"ninja\"",
            crate::config::SpinnerStyle::Magic => "\"magic\"",
            crate::config::SpinnerStyle::Robot => "\"robot\"",
            crate::config::SpinnerStyle::Boom => "\"boom\"",
            crate::config::SpinnerStyle::Unicorn => "\"unicorn\"",
            crate::config::SpinnerStyle::Bee => "\"bee\"",
            crate::config::SpinnerStyle::Dragon => "\"dragon\"",
            crate::config::SpinnerStyle::Ghost => "\"ghost\"",
            crate::config::SpinnerStyle::Pumpkin => "\"pumpkin\"",
            crate::config::SpinnerStyle::Wizard => "\"wizard\"",
            crate::config::SpinnerStyle::Crown => "\"crown\"",
            crate::config::SpinnerStyle::Diamond => "\"diamond\"",
            crate::config::SpinnerStyle::Fire => "\"fire\"",
            crate::config::SpinnerStyle::Rocket => "\"rocket\"",
            crate::config::SpinnerStyle::StarSpin => "\"star-spin\"",
            crate::config::SpinnerStyle::Confetti => "\"confetti\"",
            crate::config::SpinnerStyle::Cthulhu => "\"cthulhu\"",
            crate::config::SpinnerStyle::DwarfFortress => "\"dwarf-fortress\"",
        };
        self.state.spinner_style = style;
        if self.update_config_file("spinner style", |content| {
            crate::config::upsert_section_value(content, "ui", "spinner_style", value)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_agent_panel_sort(&mut self, sort: crate::app::state::AgentPanelSort) {
        let value = match sort {
            crate::app::state::AgentPanelSort::Spaces => {
                crate::config::AgentPanelSortConfig::Spaces.as_str()
            }
            crate::app::state::AgentPanelSort::Priority => {
                crate::config::AgentPanelSortConfig::Priority.as_str()
            }
        };
        if self.update_config_file("agent panel sort", |content| {
            crate::config::upsert_section_value(
                content,
                "ui",
                "agent_panel_sort",
                &format!("\"{value}\""),
            )
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_mouse_capture(&mut self, enabled: bool) {
        if self.update_config_file("mouse capture", |content| {
            crate::config::upsert_section_bool(content, "ui", "mouse_capture", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_copy_on_select(&mut self, enabled: bool) {
        if self.update_config_file("copy on select", |content| {
            crate::config::upsert_section_bool(content, "ui", "copy_on_select", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_confirm_close(&mut self, enabled: bool) {
        if self.update_config_file("confirm close", |content| {
            crate::config::upsert_section_bool(content, "ui", "confirm_close", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_prompt_new_tab_name(&mut self, enabled: bool) {
        if self.update_config_file("prompt new tab name", |content| {
            crate::config::upsert_section_bool(content, "ui", "prompt_new_tab_name", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_prompt_new_workspace_name(&mut self, enabled: bool) {
        if self.update_config_file("prompt new workspace name", |content| {
            crate::config::upsert_section_bool(content, "ui", "prompt_new_workspace_name", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_redraw_on_focus_gained(&mut self, enabled: bool) {
        if self.update_config_file("redraw on focus gained", |content| {
            crate::config::upsert_section_bool(content, "ui", "redraw_on_focus_gained", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_host_cursor(&mut self, mode: crate::config::HostCursorModeConfig) {
        let value = match mode {
            crate::config::HostCursorModeConfig::Auto => "\"auto\"",
            crate::config::HostCursorModeConfig::Native => "\"native\"",
            crate::config::HostCursorModeConfig::Drawn => "\"drawn\"",
        };
        if self.update_config_file("host cursor", |content| {
            crate::config::upsert_section_value(content, "ui", "host_cursor", value)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_sidebar_collapsed_mode(
        &mut self,
        mode: crate::config::SidebarCollapsedModeConfig,
    ) {
        let value = match mode {
            crate::config::SidebarCollapsedModeConfig::Compact => "\"compact\"",
            crate::config::SidebarCollapsedModeConfig::Hidden => "\"hidden\"",
        };
        if self.update_config_file("sidebar collapsed mode", |content| {
            crate::config::upsert_section_value(content, "ui", "sidebar_collapsed_mode", value)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_shell_mode(&mut self, mode: crate::config::ShellModeConfig) {
        let value = match mode {
            crate::config::ShellModeConfig::Auto => "\"auto\"",
            crate::config::ShellModeConfig::Login => "\"login\"",
            crate::config::ShellModeConfig::NonLogin => "\"non_login\"",
        };
        if self.update_config_file("shell mode", |content| {
            crate::config::upsert_section_value(content, "terminal", "shell_mode", value)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_default_shell(&mut self, shell: &str) {
        let shell = shell.to_string();
        if self.update_config_file("default shell", |content| {
            if shell.is_empty() {
                crate::config::remove_section_key(content, "terminal", "default_shell")
            } else {
                crate::config::upsert_section_value(
                    content,
                    "terminal",
                    "default_shell",
                    &format!("\"{shell}\""),
                )
            }
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_new_terminal_cwd(&mut self, cwd: crate::config::NewTerminalCwdConfig) {
        let value = match cwd {
            crate::config::NewTerminalCwdConfig::Follow => "\"follow\"",
            crate::config::NewTerminalCwdConfig::Home => "\"home\"",
            crate::config::NewTerminalCwdConfig::Current => "\"current\"",
            crate::config::NewTerminalCwdConfig::Path(path) => {
                return self.save_new_terminal_cwd_path(&path);
            }
        };
        if self.update_config_file("new terminal cwd", |content| {
            crate::config::upsert_section_value(content, "terminal", "new_cwd", value)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    fn save_new_terminal_cwd_path(&mut self, path: &str) -> bool {
        let path = path.to_string();
        self.update_config_file("new terminal cwd", |content| {
            crate::config::upsert_section_value(
                content,
                "terminal",
                "new_cwd",
                &format!("\"{path}\""),
            )
        })
        .then(|| {
            self.apply_config_from_disk(false);
            true
        })
        .unwrap_or(false)
    }

    pub(super) fn save_scrollback_limit_bytes(&mut self, bytes: usize) {
        if self.update_config_file("scrollback limit", |content| {
            crate::config::upsert_section_value(
                content,
                "advanced",
                "scrollback_limit_bytes",
                &bytes.to_string(),
            )
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_toast_delay_seconds(&mut self, seconds: u64) {
        if self.update_config_file("toast delay", |content| {
            crate::config::upsert_section_value(
                content,
                "ui.toast",
                "delay_seconds",
                &seconds.to_string(),
            )
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_toast_herdr_position(
        &mut self,
        position: crate::config::ToastHerdrPosition,
    ) {
        let value = match position {
            crate::config::ToastHerdrPosition::TopLeft => "\"top-left\"",
            crate::config::ToastHerdrPosition::TopRight => "\"top-right\"",
            crate::config::ToastHerdrPosition::BottomLeft => "\"bottom-left\"",
            crate::config::ToastHerdrPosition::BottomRight => "\"bottom-right\"",
        };
        if self.update_config_file("toast position", |content| {
            crate::config::upsert_section_value(content, "ui.toast.herdr", "position", value)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_clipboard_toast_enabled(&mut self, enabled: bool) {
        if self.update_config_file("clipboard toast", |content| {
            crate::config::upsert_section_bool(content, "ui.toast.clipboard", "enabled", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_clipboard_toast_position(
        &mut self,
        position: crate::config::ToastClipboardPosition,
    ) {
        let value = match position {
            crate::config::ToastClipboardPosition::TopLeft => "\"top-left\"",
            crate::config::ToastClipboardPosition::TopCenter => "\"top-center\"",
            crate::config::ToastClipboardPosition::TopRight => "\"top-right\"",
            crate::config::ToastClipboardPosition::BottomLeft => "\"bottom-left\"",
            crate::config::ToastClipboardPosition::BottomCenter => "\"bottom-center\"",
            crate::config::ToastClipboardPosition::BottomRight => "\"bottom-right\"",
        };
        if self.update_config_file("clipboard toast position", |content| {
            crate::config::upsert_section_value(content, "ui.toast.clipboard", "position", value)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_update_channel(&mut self, channel: crate::config::UpdateChannelConfig) {
        if self.update_config_file("update channel", |content| {
            crate::config::upsert_section_value(
                content,
                "update",
                "channel",
                &format!("\"{}\"", channel.as_str()),
            )
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_version_check(&mut self, enabled: bool) {
        if self.update_config_file("version check", |content| {
            crate::config::upsert_section_bool(content, "update", "version_check", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_manifest_check(&mut self, enabled: bool) {
        if self.update_config_file("manifest check", |content| {
            crate::config::upsert_section_bool(content, "update", "manifest_check", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_resume_agents_on_restore(&mut self, enabled: bool) {
        if self.update_config_file("resume agents on restore", |content| {
            crate::config::upsert_section_bool(content, "session", "resume_agents_on_restore", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_manage_ssh_config(&mut self, enabled: bool) {
        if self.update_config_file("manage ssh config", |content| {
            crate::config::upsert_section_bool(content, "advanced", "manage_ssh_config", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_clipboard_history_enabled(&mut self, enabled: bool) {
        if self.update_config_file("clipboard history", |content| {
            crate::config::upsert_section_bool(content, "clipboard", "history_enabled", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_allow_nested(&mut self, enabled: bool) {
        if self.update_config_file("allow nested", |content| {
            crate::config::upsert_section_bool(content, "experimental", "allow_nested", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_kitty_graphics(&mut self, enabled: bool) {
        if self.update_config_file("kitty graphics", |content| {
            crate::config::upsert_section_bool(content, "experimental", "kitty_graphics", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_reveal_hidden_cursor_for_cjk_ime(&mut self, enabled: bool) {
        if self.update_config_file("reveal hidden cursor for cjk ime", |content| {
            crate::config::upsert_section_bool(
                content,
                "experimental",
                "reveal_hidden_cursor_for_cjk_ime",
                enabled,
            )
        }) {
            self.apply_config_from_disk(false);
        }
    }

    pub(super) fn save_theme_auto_switch(&mut self, enabled: bool) {
        if self.update_config_file("theme auto switch", |content| {
            crate::config::upsert_section_bool(content, "theme", "auto_switch", enabled)
        }) {
            self.apply_config_from_disk(false);
        }
    }

    /// Apply a pane layout template to the current tab by performing a sequence
    /// of splits. Leaves settings mode first so the user sees the result.
    pub(super) fn apply_pane_template(&mut self, template: crate::pane_template::PaneTemplateId) {
        use crate::api::schema::SplitDirection;
        use crate::layout::NavDirection;
        use crate::pane_template::PaneTemplateId as T;

        // Leave settings overlay so the user sees the layout change.
        if self.state.active.is_some() {
            self.state.mode = crate::app::Mode::Terminal;
        } else {
            self.state.mode = crate::app::Mode::Navigate;
        }

        match template {
            T::Single => {
                // Close every other pane in the active tab so the result matches
                // "one pane, no splits" even when the tab already had splits.
                let Some(ws_idx) = self.state.active else {
                    return;
                };
                let Some(focused) = self
                    .state
                    .workspaces
                    .get(ws_idx)
                    .and_then(|ws| ws.focused_pane_id())
                else {
                    return;
                };
                let Some(tab) = self.state.workspaces[ws_idx].active_tab() else {
                    return;
                };
                let to_close: Vec<String> = tab
                    .layout
                    .pane_ids()
                    .into_iter()
                    .filter(|pane_id| *pane_id != focused)
                    .filter_map(|pane_id| self.public_pane_id(ws_idx, pane_id))
                    .collect();
                for pane_id in to_close {
                    self.runtime_pane_close("tui.pane.close", pane_id);
                }
            }
            T::HorizontalSplit => {
                self.split_focused_pane_via_api(SplitDirection::Right);
            }
            T::VerticalSplit => {
                self.split_focused_pane_via_api(SplitDirection::Down);
            }
            T::Quad => {
                self.split_focused_pane_via_api(SplitDirection::Right);
                self.focus_pane_direction_via_api(NavDirection::Left);
                self.split_focused_pane_via_api(SplitDirection::Down);
                self.focus_pane_direction_via_api(NavDirection::Right);
                self.split_focused_pane_via_api(SplitDirection::Down);
            }
            T::TripleHorizontal => {
                self.split_focused_pane_via_api(SplitDirection::Right);
                self.focus_pane_direction_via_api(NavDirection::Left);
                self.split_focused_pane_via_api(SplitDirection::Right);
            }
            T::MainSidebar => {
                self.runtime_pane_split(
                    "tui.pane.split",
                    crate::api::schema::PaneSplitParams {
                        workspace_id: None,
                        target_pane_id: None,
                        direction: SplitDirection::Right,
                        ratio: Some(0.7),
                        cwd: None,
                        focus: true,
                        env: Default::default(),
                        command: None,
                    },
                );
            }
        }
    }
}
