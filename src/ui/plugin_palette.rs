//! Plugin action palette (`prefix+e`): a launcher that lists every installed
//! plugin action, favorites first, with search, favorite toggling, keybind
//! recording, and action invocation.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;

const POPUP_WIDTH: u16 = 80;
const POPUP_HEIGHT: u16 = 24;
/// Rows before the selectable entry list starts (title, search, gap).
const LIST_OFFSET: u16 = 4;

#[derive(Clone)]
pub(crate) struct PaletteEntry {
    pub qualified: String,
    pub plugin_id: String,
    pub action_id: String,
    pub title: String,
    pub description: Option<String>,
    pub favorite: bool,
    pub enabled: bool,
}

/// Every installed plugin action, favorites first, then by qualified id.
pub(crate) fn palette_entries(app: &AppState) -> Vec<PaletteEntry> {
    let mut entries = Vec::new();
    for plugin in app.installed_plugins.values() {
        for action in &plugin.actions {
            let qualified = format!("{}.{}", plugin.plugin_id, action.id);
            entries.push(PaletteEntry {
                favorite: app.plugin_favorites.contains(&qualified),
                qualified,
                plugin_id: plugin.plugin_id.clone(),
                action_id: action.id.clone(),
                title: action.title.clone(),
                description: action.description.clone(),
                enabled: plugin.enabled,
            });
        }
    }
    entries.sort_by(|a, b| {
        b.favorite
            .cmp(&a.favorite)
            .then_with(|| a.qualified.cmp(&b.qualified))
    });
    entries
}

fn matches_query(entry: &PaletteEntry, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let needle = query.to_ascii_lowercase();
    entry.title.to_ascii_lowercase().contains(&needle)
        || entry.qualified.to_ascii_lowercase().contains(&needle)
        || entry.plugin_id.to_ascii_lowercase().contains(&needle)
        || entry
            .description
            .as_deref()
            .is_some_and(|d| d.to_ascii_lowercase().contains(&needle))
}

/// Entries filtered by the active search query.
pub(crate) fn filtered_entries(app: &AppState) -> Vec<PaletteEntry> {
    let query = app.plugin_palette.query.as_str();
    palette_entries(app)
        .into_iter()
        .filter(|entry| matches_query(entry, query))
        .collect()
}

fn popup_rect(area: Rect) -> Option<Rect> {
    super::widgets::centered_popup_rect(area, POPUP_WIDTH, POPUP_HEIGHT)
}

pub(crate) fn search_rect(area: Rect) -> Option<Rect> {
    let popup = popup_rect(area)?;
    Some(Rect::new(
        popup.x + 2,
        popup.y + 2,
        popup.width.saturating_sub(4),
        1,
    ))
}

pub(crate) fn search_index_at(area: Rect, col: u16, row: u16) -> bool {
    let Some(rect) = search_rect(area) else {
        return false;
    };
    col >= rect.x && col < rect.x + rect.width && row == rect.y
}

fn list_area(area: Rect) -> Option<Rect> {
    let popup = popup_rect(area)?;
    let y = popup.y + LIST_OFFSET;
    Some(Rect::new(
        popup.x + 2,
        y,
        popup.width.saturating_sub(4),
        popup.height.saturating_sub(LIST_OFFSET + 2),
    ))
}

pub(crate) fn visible_range(app: &AppState, area: Rect) -> (usize, usize) {
    let entries = filtered_entries(app);
    let Some(list) = list_area(area) else {
        return (0, 0);
    };
    let visible = list.height.max(1) as usize;
    let selected = app
        .plugin_palette
        .selected
        .min(entries.len().saturating_sub(1));
    let scroll = if selected >= visible {
        selected - visible + 1
    } else {
        0
    };
    (scroll, visible)
}

/// Resolve a mouse cell into a selectable entry index.
pub(crate) fn entry_index_at(app: &AppState, area: Rect, col: u16, row: u16) -> Option<usize> {
    let entries = filtered_entries(app);
    let list = list_area(area)?;
    if col < list.x || col >= list.x + list.width || row < list.y || row >= list.y + list.height {
        return None;
    }
    let (scroll, visible) = visible_range(app, area);
    let rel = (row - list.y) as usize;
    if rel >= visible {
        return None;
    }
    let index = scroll + rel;
    entries.get(index).map(|_| index)
}

pub(crate) fn render_plugin_palette_overlay(app: &AppState, frame: &mut Frame, area: Rect) {
    if app.mode != crate::app::Mode::PluginPalette {
        return;
    }
    let p = &app.palette;
    let Some(popup) = popup_rect(area) else {
        return;
    };

    super::dim_background(frame, area);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(p.accent))
            .style(Style::default().bg(p.panel_bg)),
        popup,
    );

    frame.render_widget(
        Paragraph::new(Span::styled(
            " plugin actions",
            Style::default().fg(p.text).add_modifier(Modifier::BOLD),
        )),
        Rect::new(popup.x + 2, popup.y + 1, popup.width.saturating_sub(4), 1),
    );

    let search_focused = app.plugin_palette.search_focused;
    let query = app.plugin_palette.query.as_str();
    let search_style = if search_focused {
        Style::default().fg(p.text)
    } else {
        Style::default().fg(p.overlay0)
    };
    let label = if search_focused && query.is_empty() {
        " search…".to_string()
    } else {
        format!(" {query}")
    };
    frame.render_widget(
        Paragraph::new(Span::styled(label, search_style)),
        search_rect(area).unwrap_or_default(),
    );

    let entries = filtered_entries(app);
    let (scroll, visible) = visible_range(app, area);
    let Some(list) = list_area(area) else {
        return;
    };
    let selected = app
        .plugin_palette
        .selected
        .min(entries.len().saturating_sub(1));

    if entries.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                " no plugin actions (install a plugin first)",
                Style::default().fg(p.overlay1),
            )),
            Rect::new(list.x, list.y, list.width, 1),
        );
    }

    for visible_idx in 0..visible {
        let index = scroll + visible_idx;
        let Some(entry) = entries.get(index) else {
            break;
        };
        let rect = Rect::new(list.x, list.y + visible_idx as u16, list.width, 1);
        let selected_style = if index == selected {
            Style::default()
                .bg(p.surface0)
                .fg(p.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.subtext0)
        };
        let star = if entry.favorite { "★ " } else { "  " };
        let mut spans = vec![
            Span::styled(star, Style::default().fg(p.yellow)),
            Span::styled(format!("{} ", entry.title), selected_style),
            Span::styled(entry.plugin_id.clone(), Style::default().fg(p.overlay1)),
        ];
        if !entry.enabled {
            spans.push(Span::styled(
                "  (disabled)",
                Style::default().fg(p.overlay0),
            ));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), rect);
    }

    let hint = if app.plugin_palette.recording_keybind.is_some() {
        " press the key to bind (esc cancels)".to_string()
    } else {
        "↑↓ select · ↵ run · f favorite · b bind · / search · esc close".to_string()
    };
    frame.render_widget(
        Paragraph::new(Span::styled(hint, Style::default().fg(p.overlay1))),
        Rect::new(
            popup.x + 2,
            popup.y + popup.height.saturating_sub(2),
            popup.width.saturating_sub(4),
            1,
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::schema::{InstalledPluginInfo, PluginManifestAction};
    use crate::app::AppState;

    fn entry_plugin(id: &str, action_id: &str) -> InstalledPluginInfo {
        InstalledPluginInfo {
            plugin_id: id.to_string(),
            name: id.to_string(),
            version: "0.1.0".to_string(),
            min_herdr_version: String::new(),
            description: None,
            manifest_path: "/tmp/x.toml".to_string(),
            plugin_root: "/tmp".to_string(),
            enabled: true,
            platforms: None,
            build: Vec::new(),
            startup: Vec::new(),
            actions: vec![PluginManifestAction {
                id: action_id.to_string(),
                title: format!("{action_id} title"),
                description: None,
                contexts: Vec::new(),
                platforms: None,
                command: vec!["true".to_string()],
            }],
            events: Vec::new(),
            panes: Vec::new(),
            link_handlers: Vec::new(),
            source: Default::default(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn palette_entries_sort_favorites_first() {
        let mut app = AppState::test_new();
        app.installed_plugins
            .insert("com.b".to_string(), entry_plugin("com.b", "two"));
        app.installed_plugins
            .insert("com.a".to_string(), entry_plugin("com.a", "one"));
        app.plugin_favorites = vec!["com.b.two".to_string()];

        let entries = palette_entries(&app);
        assert_eq!(entries.len(), 2);
        assert!(entries[0].favorite);
        assert_eq!(entries[0].qualified, "com.b.two");
        assert_eq!(entries[1].qualified, "com.a.one");
    }

    #[test]
    fn palette_entries_filter_by_query() {
        let mut app = AppState::test_new();
        app.installed_plugins
            .insert("com.a".to_string(), entry_plugin("com.a", "one"));
        app.plugin_palette.query = "one".to_string();

        let entries = filtered_entries(&app);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].qualified, "com.a.one");
    }
}
