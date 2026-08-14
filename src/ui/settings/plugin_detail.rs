//! Plugin detail view rendered inside the Plugins settings section.
//!
//! When `AppState.settings.plugin_detail` is `Some`, the content area switches
//! from the flat plugin list to a focused view of a single installed plugin:
//! metadata, an enable toggle, and its invocable actions.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{api::schema::InstalledPluginInfo, app::AppState};

use super::{catalog::installed_plugins_sorted, layout::SettingsLayout};

/// Vertical offset (in rows) within `layout.content` of the first selectable
/// row (the enable toggle).
const DETAIL_TOGGLE_OFFSET: u16 = 5;
/// Vertical offset of the first action row. The `actions` header sits one row
/// above this.
pub(crate) const DETAIL_ACTIONS_OFFSET: u16 = 7;

/// The installed plugin currently shown in the detail view, if any.
pub(crate) fn detail_plugin(app: &AppState) -> Option<&InstalledPluginInfo> {
    let index = app.settings.plugin_detail?;
    installed_plugins_sorted(app).get(index).copied()
}

/// Number of selectable rows in the detail view: the enable toggle plus every
/// declared action.
pub(crate) fn selectable_count(app: &AppState) -> usize {
    detail_plugin(app)
        .map(|plugin| 1 + plugin.actions.len())
        .unwrap_or(0)
}

/// Resolve a mouse cell into a selectable row index in the detail view.
/// Index `0` is the enable toggle; `1..` are actions in declaration order.
pub(crate) fn index_at(
    layout: &SettingsLayout,
    app: &AppState,
    col: u16,
    row: u16,
) -> Option<usize> {
    let content = layout.content;
    if col < content.x || col >= content.x + content.width {
        return None;
    }
    let plugin = detail_plugin(app)?;
    if row == content.y + DETAIL_TOGGLE_OFFSET {
        return Some(0);
    }
    let first_action_y = content.y + DETAIL_ACTIONS_OFFSET;
    if row < first_action_y {
        return None;
    }
    let scroll = app.settings.plugin_detail_scroll as usize;
    let rel = (row - first_action_y) as usize;
    let action_idx = rel + scroll;
    if action_idx < plugin.actions.len() {
        Some(action_idx + 1)
    } else {
        None
    }
}

fn source_label(plugin: &InstalledPluginInfo) -> String {
    match (&plugin.source.owner, &plugin.source.repo) {
        (Some(owner), Some(repo)) => format!("{owner}/{repo}"),
        (_, Some(repo)) => repo.clone(),
        _ => plugin
            .source
            .managed_path
            .clone()
            .unwrap_or_else(|| "local".to_string()),
    }
}

pub(crate) fn render(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    let Some(plugin) = detail_plugin(app) else {
        return;
    };
    let content = layout.content;
    let cursor = app.settings.plugin_detail_cursor;
    let dim = Style::default().fg(p.overlay1);
    let bold = Style::default().fg(p.text).add_modifier(Modifier::BOLD);

    // Metadata block.
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(plugin.name.clone(), bold),
            Span::styled(format!("  ({})", plugin.plugin_id), dim),
        ])),
        Rect::new(content.x, content.y, content.width, 1),
    );

    let version = if plugin.min_herdr_version.is_empty() {
        format!("v{}", plugin.version)
    } else {
        format!(
            "v{} · requires herdr ≥ {}",
            plugin.version, plugin.min_herdr_version
        )
    };
    frame.render_widget(
        Paragraph::new(Span::styled(version, dim)),
        Rect::new(content.x, content.y + 1, content.width, 1),
    );

    let description = plugin
        .description
        .clone()
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| "no description".to_string());
    frame.render_widget(
        Paragraph::new(Span::styled(description, Style::default().fg(p.subtext0))),
        Rect::new(content.x, content.y + 2, content.width, 1),
    );

    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("source: {}", source_label(plugin)),
            dim,
        )),
        Rect::new(content.x, content.y + 3, content.width, 1),
    );

    if let Some(warning) = plugin.warnings.first() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("⚠ {warning}"),
                Style::default().fg(p.yellow),
            )),
            Rect::new(content.x, content.y + 4, content.width, 1),
        );
    }

    // Enable toggle (selectable row 0).
    let enabled = plugin.enabled;
    let toggle_style = if cursor == 0 {
        Style::default()
            .bg(p.surface0)
            .fg(p.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.subtext0)
    };
    let marker = if enabled { "[✓]" } else { "[ ]" };
    let state = if enabled { "enabled" } else { "disabled" };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!(" {marker} "), toggle_style),
            Span::styled(state, toggle_style),
        ])),
        Rect::new(
            content.x,
            content.y + DETAIL_TOGGLE_OFFSET,
            content.width,
            1,
        ),
    );

    // Actions header + rows.
    frame.render_widget(
        Paragraph::new(Span::styled(
            "actions",
            Style::default().fg(p.overlay0).add_modifier(Modifier::BOLD),
        )),
        Rect::new(
            content.x,
            content.y + DETAIL_ACTIONS_OFFSET - 1,
            content.width,
            1,
        ),
    );

    let scroll = app.settings.plugin_detail_scroll as usize;
    let visible_height = content.height.saturating_sub(DETAIL_ACTIONS_OFFSET + 1) as usize;
    let visible_end = scroll + visible_height;

    for (idx, action) in plugin.actions.iter().enumerate() {
        if idx < scroll || idx >= visible_end {
            continue;
        }
        let visible_idx = idx - scroll;
        let selected = cursor == idx + 1;
        let style = if selected {
            Style::default()
                .bg(p.surface0)
                .fg(p.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.subtext0)
        };
        let mut spans = vec![
            Span::styled(" ▸ ", style),
            Span::styled(action.title.clone(), style),
        ];
        if let Some(description) = &action.description {
            if !description.is_empty() {
                spans.push(Span::styled(
                    format!("  —  {description}"),
                    Style::default().fg(p.overlay1),
                ));
            }
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(
                content.x,
                content.y + DETAIL_ACTIONS_OFFSET + visible_idx as u16,
                content.width,
                1,
            ),
        );
    }
}
