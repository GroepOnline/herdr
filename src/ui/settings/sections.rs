use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{state::SettingsSection, AppState};
use crate::ui::text::{display_width_u16, truncate_end};

use super::{
    catalog::{catalog_plugin_id, integration_index, spinner_index, theme_index},
    layout::{
        active_spinner_styles, spinner_category_labels, SettingsLayout, SETTINGS_NAV_ACCENT_COLS,
        SETTINGS_SECTION_DESC_ROWS,
    },
    rows::{
        appearance_theme_labels, row_choice_selected, row_spinner_current, row_theme_current,
        row_toggle_checked, scroll_list_row_indices, section_rows, SettingsRowKind,
    },
    spinner::{active_spinner_category, spinner_frame_at, spinner_hero_strip},
    widgets::{render_chip_row, render_setting_block_header, ChipSpec, CHIP_HORIZONTAL_GAP},
};

pub(crate) fn render_settings_content(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    let section = app.settings.section;

    let title = section.title();
    let title_width = display_width_u16(title) as usize;
    let sep = "  ·  ";
    let sep_width = display_width_u16(sep) as usize;
    let desc_budget = (layout.content.width as usize).saturating_sub(title_width + sep_width);
    let description = if desc_budget == 0 {
        String::new()
    } else {
        truncate_end(section.description(), desc_budget)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                title,
                Style::default().fg(p.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{sep}{description}"),
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
        render_appearance_showcase(app, frame, layout);
    }

    if super::content::uses_block_layout(section) {
        super::content::render_block_section(app, frame, layout);
        if section == SettingsSection::Agents {
            render_agents_footer(app, frame, layout);
        }
        if section == SettingsSection::Plugins {
            render_plugins_footer(app, frame, layout);
        }
        return;
    }

    let rows = section_rows(app, section);
    let list_indices = if section == SettingsSection::Appearance {
        scroll_list_row_indices(app, section)
    } else {
        (0..rows.len()).collect()
    };
    let (scroll, visible) = layout.visible_row_range(app);
    let selected = app.settings.list.selected.min(rows.len().saturating_sub(1));

    for visible_idx in 0..visible {
        let list_pos = scroll + visible_idx;
        let Some(&row_index) = list_indices.get(list_pos) else {
            break;
        };
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
                if catalog_plugin_id(row.id).is_some() {
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
            if let Some(idx) = spinner_index(row.id) {
                if let Some(style) = styles.get(idx) {
                    let frame_char = spinner_frame_at(*style, app.settings.preview_tick);
                    spans.push(Span::styled(
                        format!("  {frame_char} "),
                        Style::default().fg(p.yellow),
                    ));
                }
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

fn render_appearance_showcase(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    let rows = section_rows(app, SettingsSection::Appearance);
    let selected = app.settings.list.selected;

    render_setting_block_header(
        frame,
        layout.setting_block_header_rect(layout.appearance_block_header_y(app)),
        "appearance",
        p,
    );

    if let Some(row) = rows
        .iter()
        .find(|row| row.id == super::catalog::SettingsItemId::ThemeAutoSwitch)
    {
        let y = layout.appearance_auto_switch_y(app);
        let is_sel = rows
            .iter()
            .position(|r| r.id == row.id)
            .is_some_and(|idx| idx == selected);
        render_toggle_row(
            app,
            frame,
            row,
            is_sel,
            Rect::new(layout.content.x, y, layout.content.width, 1),
        );
    }

    let theme_rect = layout.appearance_theme_chips_rect(app);
    let theme_labels = appearance_theme_labels(app);
    if !theme_labels.is_empty() {
        let chips: Vec<ChipSpec<'_>> = theme_labels
            .iter()
            .enumerate()
            .map(|(label_idx, label)| {
                let theme_idx = super::rows::appearance_theme_index_at_label_index(app, label_idx)
                    .unwrap_or(label_idx);
                let row_selected = rows
                    .iter()
                    .position(|row| theme_index(row.id) == Some(theme_idx))
                    .is_some_and(|idx| idx == selected);
                let row = rows
                    .iter()
                    .find(|row| theme_index(row.id) == Some(theme_idx));
                ChipSpec {
                    label,
                    selected: row.is_some_and(|row| row_theme_current(app, row)) || row_selected,
                }
            })
            .collect();
        render_chip_row(frame, theme_rect, &chips, CHIP_HORIZONTAL_GAP, p);
    }

    render_setting_block_header(
        frame,
        layout.setting_block_header_rect(layout.appearance_spinner_header_y(app)),
        "spinner",
        p,
    );
    render_spinner_category_chips(app, frame, layout);
    render_spinner_hero(app, frame, layout);
}

fn render_toggle_row(
    app: &AppState,
    frame: &mut Frame,
    row: &super::rows::SettingsRow,
    is_sel: bool,
    rect: Rect,
) {
    let p = &app.palette;
    let section = SettingsSection::Appearance;
    let row_style = if is_sel {
        Style::default()
            .bg(p.surface0)
            .fg(p.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.subtext0)
    };
    let marker = if row_toggle_checked(app, section, row) {
        "[✓]"
    } else {
        "[ ]"
    };
    let mut spans = vec![
        Span::styled(format!(" {marker} "), row_style),
        Span::styled(row.label.clone(), row_style),
    ];
    if let Some(detail) = &row.detail {
        spans.push(Span::styled(
            format!("  ·  {detail}"),
            Style::default().fg(p.overlay1),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), rect);
}

fn render_spinner_category_chips(app: &AppState, frame: &mut Frame, layout: &SettingsLayout) {
    let p = &app.palette;
    let Some(rect) = layout.spinner_category_rect(app) else {
        return;
    };
    let chips: Vec<ChipSpec<'_>> = spinner_category_labels()
        .enumerate()
        .map(|(idx, label)| ChipSpec {
            label,
            selected: idx == app.settings.spinner_category,
        })
        .collect();
    render_chip_row(frame, rect, &chips, CHIP_HORIZONTAL_GAP, p);
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
                Style::default().fg(p.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  ·  {category}"), Style::default().fg(p.overlay1)),
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
    if let Some(row) = rows.get(app.settings.list.selected) {
        if row.kind == SettingsRowKind::Spinner {
            if let Some(idx) = spinner_index(row.id) {
                if let Some(style) = active_spinner_category(app.settings.spinner_category)
                    .styles
                    .get(idx)
                    .copied()
                {
                    return style;
                }
            }
        }
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
    let hint = if let Some(job) = &app.settings.plugin_install_job {
        job.message.clone()
    } else if !app.plugin_install_messages.is_empty() {
        app.plugin_install_messages.join(" · ")
    } else if super::catalog::catalog_entries_available(app).is_empty() {
        "you're caught up — every listed plugin is installed".to_string()
    } else {
        "enter installs · space toggles on/off · ↵ refresh".to_string()
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
        let badge = if app.settings_section_has_badge(*section) {
            " ●"
        } else {
            ""
        };

        if active {
            let accent_rect = Rect::new(rect.x, rect.y, SETTINGS_NAV_ACCENT_COLS, 1);
            frame.render_widget(
                Paragraph::new(Span::styled(
                    "▌",
                    Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
                )),
                accent_rect,
            );
            let label_rect = Rect::new(
                rect.x + SETTINGS_NAV_ACCENT_COLS,
                rect.y,
                rect.width.saturating_sub(SETTINGS_NAV_ACCENT_COLS),
                1,
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!(" {}{}", section.label(), badge),
                    Style::default()
                        .fg(p.text)
                        .bg(p.surface0)
                        .add_modifier(Modifier::BOLD),
                ))),
                label_rect,
            );
        } else {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(
                        format!("{}{}", section.label(), badge),
                        Style::default().fg(p.overlay0),
                    ),
                ])),
                rect,
            );
        }
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
    let section = app.settings.section;

    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            " CONFIG MENU",
            Style::default().fg(p.text).add_modifier(Modifier::BOLD),
        )])),
        layout.title,
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            format!(" {} settings", section.label()),
            Style::default().fg(p.overlay1),
        )])),
        layout.subtitle,
    );

    let filter = &app.settings.search;
    let placeholder = if filter.is_empty() {
        " / search settings…"
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
            Span::styled("space", Style::default().fg(p.overlay0)),
            Span::styled(" toggle  ", Style::default().fg(p.overlay1)),
            Span::styled("tab", Style::default().fg(p.overlay0)),
            Span::styled(" section  ", Style::default().fg(p.overlay1)),
            Span::styled("/", Style::default().fg(p.overlay0)),
            Span::styled(" search", Style::default().fg(p.overlay1)),
        ])),
        layout.footer_hints,
    );

    let show_tertiary = super::layout::settings_show_tertiary_action(app);
    let buttons = super::layout::settings_button_rects(layout, app.settings.section, show_tertiary);

    if let Some(tertiary_rect) = buttons.tertiary {
        let tertiary_hint = match app.settings.section {
            SettingsSection::Agents => Some("i"),
            _ => None,
        };
        super::super::widgets::render_action_button(
            frame,
            tertiary_rect,
            tertiary_hint,
            super::layout::settings_tertiary_button_label(app.settings.section),
            Style::default()
                .fg(p.text)
                .bg(p.surface0)
                .add_modifier(Modifier::BOLD),
        );
    }

    super::super::widgets::render_action_button(
        frame,
        buttons.save,
        Some("↵"),
        "save",
        Style::default()
            .fg(super::super::widgets::panel_contrast_fg(p))
            .bg(p.accent)
            .add_modifier(Modifier::BOLD),
    );
    super::super::widgets::render_action_button(
        frame,
        buttons.cancel,
        Some("esc"),
        "cancel",
        Style::default()
            .fg(p.text)
            .bg(p.surface0)
            .add_modifier(Modifier::BOLD),
    );
}
