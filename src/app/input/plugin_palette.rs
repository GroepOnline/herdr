//! Keyboard and mouse handling for the plugin action palette (`prefix+e`).

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

use crate::{
    app::{
        state::{AppState, PluginPaletteState, ToastKind, ToastNotification},
        App, Mode,
    },
    config::format_key_combo,
    ui::{
        plugin_palette_entries, plugin_palette_entry_index_at, plugin_palette_search_index_at,
        PaletteEntry,
    },
};

impl App {
    pub(crate) fn open_plugin_palette(&mut self) {
        if self.no_session {
            return;
        }
        self.reload_plugins_for_settings();
        self.state.plugin_palette = PluginPaletteState::default();
        self.state.mode = Mode::PluginPalette;
    }

    pub(crate) fn close_plugin_palette(&mut self) {
        self.state.plugin_palette = PluginPaletteState::default();
        super::modal::leave_modal(&mut self.state);
    }

    pub(crate) fn handle_plugin_palette_key(&mut self, key: KeyEvent) {
        if let Some(qualified) = self.state.plugin_palette.recording_keybind.clone() {
            self.finish_plugin_keybind_recording(qualified, key);
            return;
        }

        if self.state.plugin_palette.search_focused {
            self.handle_plugin_palette_search_key(key);
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.close_plugin_palette(),
            KeyCode::Char('/') if key.modifiers.is_empty() => {
                self.state.plugin_palette.search_focused = true;
                self.state.plugin_palette.query.clear();
                self.state.plugin_palette.selected = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.plugin_palette.selected =
                    self.state.plugin_palette.selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = plugin_palette_entries(&self.state).len();
                if count > 0 {
                    self.state.plugin_palette.selected =
                        (self.state.plugin_palette.selected + 1).min(count - 1);
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.run_selected_palette_action(),
            KeyCode::Char('f') => self.toggle_selected_palette_favorite(),
            KeyCode::Char('b') => self.start_selected_palette_keybind_recording(),
            _ => {}
        }
    }

    fn handle_plugin_palette_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.state.plugin_palette.search_focused = false;
                self.state.plugin_palette.query.clear();
            }
            KeyCode::Enter => {
                self.state.plugin_palette.search_focused = false;
            }
            KeyCode::Backspace => {
                self.state.plugin_palette.query.pop();
                self.state.plugin_palette.selected = 0;
            }
            KeyCode::Char(ch) if key.modifiers.is_empty() => {
                self.state.plugin_palette.query.push(ch);
                self.state.plugin_palette.selected = 0;
            }
            _ => {}
        }
    }

    fn selected_palette_entry(&self) -> Option<PaletteEntry> {
        plugin_palette_entries(&self.state)
            .get(self.state.plugin_palette.selected)
            .cloned()
    }

    pub(crate) fn run_selected_palette_action(&mut self) {
        let Some(entry) = self.selected_palette_entry() else {
            return;
        };
        let result = self.invoke_plugin_action(&entry.plugin_id, &entry.action_id);
        if let Err(err) = result {
            self.show_palette_toast("plugin action failed", err);
            return;
        }
        self.close_plugin_palette();
    }

    fn toggle_selected_palette_favorite(&mut self) {
        let Some(entry) = self.selected_palette_entry() else {
            return;
        };
        let qualified = entry.qualified.clone();
        match self.toggle_plugin_favorite(qualified.clone()) {
            Ok(now_favorite) => {
                // Sorting changes the numeric position; retain the action identity.
                if let Some(index) = plugin_palette_entries(&self.state)
                    .iter()
                    .position(|entry| entry.qualified == qualified)
                {
                    self.state.plugin_palette.selected = index;
                }
                if now_favorite {
                    self.show_palette_toast("added favorite", entry.title);
                } else {
                    self.show_palette_toast("removed favorite", entry.title);
                }
            }
            Err(err) => self.show_palette_toast("favorite failed", err),
        }
    }

    fn start_selected_palette_keybind_recording(&mut self) {
        let Some(entry) = self.selected_palette_entry() else {
            return;
        };
        self.state.plugin_palette.recording_keybind = Some(entry.qualified);
    }

    fn finish_plugin_keybind_recording(&mut self, qualified: String, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.state.plugin_palette.recording_keybind = None;
            return;
        }
        if matches!(key.code, KeyCode::Modifier(_)) {
            return;
        }

        let base = format_key_combo((key.code, key.modifiers));
        let binding = if key.modifiers.is_empty() {
            format!("prefix+{base}")
        } else {
            // Control-key combinations must use prefix+ to avoid global shadowing
            if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                || key.modifiers.contains(crossterm::event::KeyModifiers::ALT)
                || key.modifiers.contains(crossterm::event::KeyModifiers::SUPER)
            {
                self.state.plugin_palette.recording_keybind = None;
                self.show_palette_toast(
                    "invalid keybind",
                    "use prefix+key or unmodified keys only".into(),
                );
                return;
            }
            format!("prefix+{base}")
        };
        // The palette opener is hardcoded and runs before custom commands, so a
        // recorded `prefix+e` binding could never fire; refuse it explicitly.
        if binding == "prefix+e" {
            self.state.plugin_palette.recording_keybind = None;
            self.show_palette_toast(
                "keybind reserved",
                "prefix+e opens the plugin palette".into(),
            );
            return;
        }

        let description = plugin_palette_entries(&self.state)
            .iter()
            .find(|entry| entry.qualified == qualified)
            .map(|entry| entry.title.clone())
            .unwrap_or_else(|| qualified.clone());

        let persisted = self.update_config_file("plugin keybind", |content| {
            crate::config::append_keys_plugin_command(content, &binding, &qualified, &description)
        });
        self.state.plugin_palette.recording_keybind = None;
        if persisted {
            self.apply_config_from_disk(false);
            self.show_palette_toast("plugin keybind saved", format!("{binding} → {description}"));
        } else {
            self.show_palette_toast(
                "plugin keybind failed",
                "could not write config.toml".into(),
            );
        }
        self.close_plugin_palette();
    }

    fn show_palette_toast(&mut self, title: &str, context: String) {
        self.state.toast = Some(ToastNotification {
            kind: ToastKind::Finished,
            title: title.to_string(),
            context,
            position: None,
            target: None,
        });
    }
}

impl AppState {
    pub(super) fn handle_plugin_palette_mouse(
        &mut self,
        mouse: MouseEvent,
    ) -> Option<super::mouse::MouseAction> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let area = self.screen_rect();
                if plugin_palette_search_index_at(area, mouse.column, mouse.row) {
                    self.plugin_palette.search_focused = true;
                    return None;
                }
                if let Some(index) =
                    plugin_palette_entry_index_at(self, area, mouse.column, mouse.row)
                {
                    Some(super::mouse::MouseAction::PluginPaletteRun { index })
                } else {
                    Some(super::mouse::MouseAction::PluginPaletteDismiss)
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn test_app() -> App {
        let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
        App::new(
            &crate::config::Config::default(),
            true,
            None,
            api_rx,
            crate::api::EventHub::default(),
        )
    }

    #[test]
    fn recording_prefix_e_is_refused() {
        let mut app = test_app();
        app.state.mode = Mode::PluginPalette;
        app.state.plugin_palette.recording_keybind = Some("com.a.one".into());

        app.handle_plugin_palette_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));

        assert_eq!(app.state.plugin_palette.recording_keybind, None);
        assert_eq!(app.state.mode, Mode::PluginPalette);
        let toast = app.state.toast.as_ref().expect("toast shown");
        assert_eq!(toast.title, "keybind reserved");
    }
}
