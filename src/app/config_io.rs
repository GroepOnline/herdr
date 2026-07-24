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

    pub(super) fn save_fleet_ops_bar(&mut self, enabled: bool) {
        if self.update_config_file("fleet ops bar", |content| {
            crate::config::upsert_section_bool(content, "ui", "fleet_ops_bar", enabled)
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
