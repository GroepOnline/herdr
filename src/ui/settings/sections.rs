use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{state::SettingsSection, AppState};

use super::{
    catalog::{catalog_plugin_index, integration_index, spinner_index},
    layout::{
        active_spinner_styles, spinner_category_labels, SettingsLayout, SETTINGS_SECTION_DESC_ROWS,
        SETTINGS_SECTION_GAP_ROWS,
    },
    rows::{
        row_choice_selected, row_spinner_current, row_theme_current, row_toggle_checked,
        section_rows, SettingsRowKind,
    },
    spinner::{active_spinner_category, spinner_frame_at, spinner_hero_strip},
};

pub(crate) fn render_settings_content(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    let section = app.settings.section;

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                section.title(),
                Style::default().fg(p.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ·  {}", section.description()),
                Style::default().fg(p.overlay1),
            ),
        ])),
        Rect::new(
            layout.content.x,
            layout.content.y,
            layout.content.width,
            SETTINGS_SECTION_DESC_ROWS,
        ),
    );

    if section == SettingsSection::Appearance {
        render_spinner_categories(app, frame, layout);
        render_spinner_hero(app, frame, layout);
    }

    let rows = section_rows(app, section);
    let (scroll, visible) = layout.visible_row_range(app);
    let selected = app.settings.list.selected.min(rows.len().saturating_sub(1));

    for visible_idx in 0..visible {
        let row_index = scroll + visible_idx;
        let Some(row) = rows.get(row_index) else {
            break;
        };

        let Some(rect) = layout.content_row_rect(app, row_index) else {
            continue;
        };

        let is_sel = row_index == selected;
        let row_style = if is_sel {
            Style::default()
                .bg(p.surface0)
                .fg(p.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.subtext0)
        };

        let marker = match row.kind {
            SettingsRowKind::Toggle => {
                if row_toggle_checked(app, section, row) {
                    "[✓]"
                } else {
                    "[ ]"
                }
            }
            SettingsRowKind::Choice => {
                if row_choice_selected(app, section, row) {
                    "●"
                } else {
                    "○"
                }
            }
            SettingsRowKind::Theme => {
                if row_theme_current(app, row) {
                    "✓"
                } else {
                    " "
                }
            }
            SettingsRowKind::Spinner => {
                if row_spinner_current(app, row) {
                    "✓"
                } else {
                    " "
                }
            }
            SettingsRowKind::Integration => {
                if catalog_plugin_index(row.id).is_some() {
                    "+"
                } else {
                    integration_marker(app, integration_index(row.id).unwrap_or_default())
                }
            }
            SettingsRowKind::Note => "·",
            // Compact apply-row — no ASCII wireframe cards.
            SettingsRowKind::Template => "▸",
        };

        let mut spans = vec![
            Span::styled(format!(" {marker} "), row_style),
            Span::styled(row.label.clone(), row_style),
        ];

        if row.kind == SettingsRowKind::Spinner {
            let styles = active_spinner_styles(app);
            if let Some(idx) = spinner_index(row.id)
                && let Some(style) = styles.get(idx)
            {
                let frame_char = spinner_frame_at(*style, app.settings.preview_tick);
                spans.push(Span::styled(
                    format!("  {frame_char} "),
                    Style::default().fg(p.yellow),
                ));
            }
        }

        if let Some(detail) = &row.detail {
            spans.push(Span::styled(
                format!("  — {detail}"),
                Style::default().fg(p.overlay1),
            ));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), rect);
    }

    if section == SettingsSection::Agents {
        render_agents_footer(app, frame, layout);
    }
    if section == SettingsSection::Plugins {
        render_plugins_footer(app, frame, layout);
    }
}

fn render_spinner_categories(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    let Some(rect) = layout.spinner_category_rect(app) else {
        return;
    };
    let mut spans = Vec::new();
    for (idx, label) in spinner_category_labels().enumerate() {
        let active = idx == app.settings.spinner_category;
        let style = if active {
            Style::default()
                .fg(super::super::widgets::panel_contrast_fg(p))
                .bg(p.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.overlay1)
        };
        if idx > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(format!(" {label} "), style));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), rect);
}

fn render_spinner_hero(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    let Some(rect) = layout.spinner_hero_rect(app) else {
        return;
    };
    let style = focused_spinner_style(app);
    let category = active_spinner_category(app.settings.spinner_category).label;
    let tick = app.settings.preview_tick;
    let strip = spinner_hero_strip(style, tick, rect.width.saturating_sub(4) as usize);
    let frame_char = spinner_frame_at(style, tick);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" {frame_char}  "),
                Style::default().fg(p.yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                style.label(),
                Style::default()
                    .fg(p.text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ·  {category}"),
                Style::default().fg(p.overlay1),
            ),
        ])),
        Rect::new(rect.x, rect.y, rect.width, 1),
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!(" {strip}"),
            Style::default().fg(p.yellow),
        )),
        Rect::new(rect.x, rect.y + 1, rect.width, 1),
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            "  [ and ] cycle packs · enter picks this style",
            Style::default().fg(p.overlay0),
        )),
        Rect::new(rect.x, rect.y + 2, rect.width, 1),
    );
}

fn focused_spinner_style(app: &AppState) -> crate::config::SpinnerStyle {
    let rows = section_rows(app, SettingsSection::Appearance);
    if let Some(row) = rows.get(app.settings.list.selected)
        && row.kind == SettingsRowKind::Spinner
        && let Some(idx) = spinner_index(row.id)
        && let Some(style) = active_spinner_category(app.settings.spinner_category)
            .styles
            .get(idx)
            .copied()
    {
        return style;
    }
    app.spinner_style
}

fn integration_marker(app: &AppState, idx: usize) -> &'static str {
    let Some(item) = app.integration_recommendations.get(idx) else {
        return " ";
    };
    match item.state {
        crate::integration::IntegrationStatusKind::Current => "✓",
        crate::integration::IntegrationStatusKind::Outdated => "↻",
        crate::integration::IntegrationStatusKind::NotInstalled if item.available => "+",
        crate::integration::IntegrationStatusKind::NotInstalled => "–",
    }
}

fn render_agents_footer(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    let y = layout.content.y + layout.content.height.saturating_sub(1);
    if y <= layout.content.y {
        return;
    }
    let hint = if !app.integration_install_messages.is_empty() {
        app.integration_install_messages.join(" · ")
    } else if app
        .integration_recommendations
        .iter()
        .any(crate::integration::IntegrationRecommendation::needs_install)
    {
        "press install to add available or outdated integrations".to_string()
    } else if app.integration_recommendations.iter().any(|item| {
        item.available || item.state != crate::integration::IntegrationStatusKind::NotInstalled
    }) {
        "all detected integrations are installed".to_string()
    } else {
        "no supported agent CLIs found on PATH".to_string()
    };
    frame.render_widget(
        Paragraph::new(Span::styled(hint, Style::default().fg(p.overlay1))),
        Rect::new(layout.content.x, y, layout.content.width, 1),
    );
}

fn render_plugins_footer(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    let y = layout.content.y + layout.content.height.saturating_sub(1);
    if y <= layout.content.y {
        return;
    }
    let hint = if !app.plugin_install_messages.is_empty() {
        app.plugin_install_messages.join(" · ")
    } else if super::catalog::catalog_entries_available(app).is_empty() {
        "you're caught up — every listed plugin is installed".to_string()
    } else {
        "enter installs · space toggles on/off".to_string()
    };
    frame.render_widget(
        Paragraph::new(Span::styled(hint, Style::default().fg(p.overlay1))),
        Rect::new(layout.content.x, y, layout.content.width, 1),
    );
}

pub(crate) fn render_settings_nav(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    for (idx, section) in SettingsSection::ALL.iter().enumerate() {
        let Some(rect) = layout.nav_item_rect(idx) else {
            continue;
        };
        let active = *section == app.settings.section;
        let style = if active {
            Style::default()
                .fg(super::super::widgets::panel_contrast_fg(p))
                .bg(p.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.overlay1)
        };
        let badge = if app.settings_section_has_badge(*section) {
            " ●"
        } else {
            ""
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                format!(" {}{}", section.label(), badge),
                style,
            )])),
            rect,
        );
    }

    let sep_x = layout.nav.x + layout.nav.width;
    let sep = "│";
    for y in layout.nav.y..layout.nav.y + layout.nav.height {
        frame.render_widget(
            Paragraph::new(Span::styled(sep, Style::default().fg(p.surface0))),
            Rect::new(sep_x, y, 1, 1),
        );
    }
}

pub(crate) fn render_settings_header(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            " customize herdr",
            Style::default().fg(p.text).add_modifier(Modifier::BOLD),
        )])),
        layout.title,
    );

    let filter = &app.settings.search;
    let placeholder = if filter.is_empty() {
        " search settings…"
    } else {
        filter.as_str()
    };
    frame.render_widget(
        Paragraph::new(Span::styled(
            placeholder,
            Style::default().fg(if filter.is_empty() {
                p.overlay0
            } else {
                p.text
            }),
        )),
        layout.search,
    );

    let sep = "─".repeat(layout.inner.width as usize);
    frame.render_widget(
        Paragraph::new(Span::styled(&sep, Style::default().fg(p.surface0))),
        Rect::new(
            layout.inner.x,
            layout.search.y.saturating_add(1),
            layout.inner.width,
            1,
        ),
    );
}

pub(crate) fn render_settings_footer(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ↑↓", Style::default().fg(p.overlay0)),
            Span::styled(" select  ", Style::default().fg(p.overlay1)),
            Span::styled("tab", Style::default().fg(p.overlay0)),
            Span::styled(" section  ", Style::default().fg(p.overlay1)),
            Span::styled("[", Style::default().fg(p.overlay0)),
            Span::styled("/", Style::default().fg(p.overlay0)),
            Span::styled("]", Style::default().fg(p.overlay0)),
            Span::styled(" search", Style::default().fg(p.overlay1)),
        ])),
        layout.footer_hints,
    );

    let show_primary = super::layout::settings_show_primary_action(app);
    let (apply_rect, close_rect) =
        super::layout::settings_button_rects(layout, app.settings.section, show_primary);
    if let Some(apply_rect) = apply_rect {
        super::super::widgets::render_action_button(
            frame,
            apply_rect,
            Some("↵"),
            super::layout::settings_primary_button_label(app.settings.section),
            Style::default()
                .fg(super::super::widgets::panel_contrast_fg(p))
                .bg(p.accent)
                .add_modifier(Modifier::BOLD),
        );
    }
    super::super::widgets::render_action_button(
        frame,
        close_rect,
        Some("esc"),
        "close",
        Style::default()
            .fg(p.text)
            .bg(p.surface0)
            .add_modifier(Modifier::BOLD),
    );
}
