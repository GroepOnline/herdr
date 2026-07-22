use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{
    app::{state::SettingsSection, AppState},
    pane_template::PaneTemplateId,
};

use super::{
    layout::{
        active_spinner_styles, spinner_category_labels, spinner_preview_frame, template_card_rect,
        SettingsLayout, SETTINGS_SECTION_DESC_ROWS, SETTINGS_SECTION_GAP_ROWS,
        SETTINGS_SPINNER_CATEGORY_ROWS,
    },
    rows::{
        row_choice_selected, row_spinner_current, row_theme_current, row_toggle_checked,
        section_rows, SettingsRowKind,
    },
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
    }

    let rows = section_rows(app, section);
    let list_area = layout.content_list_area(app);
    let (scroll, visible) = layout.visible_row_range(app);
    let selected = app.settings.list.selected.min(rows.len().saturating_sub(1));

    for visible_idx in 0..visible {
        let row_index = scroll + visible_idx;
        let Some(row) = rows.get(row_index) else {
            break;
        };

        if row.kind == SettingsRowKind::Template {
            continue;
        }

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
            SettingsRowKind::Integration => integration_marker(app, row.payload),
            SettingsRowKind::Note => "·",
            SettingsRowKind::Template => " ",
        };

        let mut spans = vec![
            Span::styled(format!(" {marker} "), row_style),
            Span::styled(row.label.clone(), row_style),
        ];

        if row.kind == SettingsRowKind::Spinner {
            let styles = active_spinner_styles(app);
            if let Some(style) = styles.get(row.payload) {
                let tick = app.settings.preview_tick;
                let frame_char = style.frames()
                    [(tick as usize / style.speed_divisor() as usize) % style.frames().len()];
                spans.push(Span::styled(
                    format!("  {frame_char} "),
                    Style::default().fg(p.yellow),
                ));
            }
        }

        if row.kind == SettingsRowKind::Note && row.label == "spinner preview" {
            spans.push(Span::styled(
                format!("  {} ", spinner_preview_frame(app)),
                Style::default().fg(p.yellow),
            ));
        }

        if let Some(detail) = &row.detail {
            spans.push(Span::styled(
                format!("  — {detail}"),
                Style::default().fg(p.overlay1),
            ));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), rect);
    }

    if section == SettingsSection::Layout {
        let template_area = super::layout::template_list_area(
            list_area,
            super::layout::layout_non_template_count(app),
        );
        for row in rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.kind == SettingsRowKind::Template)
        {
            render_template_card(app, frame, template_area, row.1.payload, row.0 == selected);
        }
    }

    if section == SettingsSection::Agents {
        render_agents_footer(app, frame, layout);
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

    let sep_y = rect.y + SETTINGS_SPINNER_CATEGORY_ROWS + SETTINGS_SECTION_GAP_ROWS - 1;
    if sep_y >= layout.content.y && sep_y < layout.content.y + layout.content.height {
        let sep = "─".repeat(layout.content.width as usize);
        frame.render_widget(
            Paragraph::new(Span::styled(sep, Style::default().fg(p.surface0))),
            Rect::new(layout.content.x, sep_y, layout.content.width, 1),
        );
    }
}

fn render_template_card(
    app: &AppState,
    frame: &mut Frame,
    list_area: Rect,
    template_idx: usize,
    is_sel: bool,
) {
    let p = &app.palette;
    let Some(card) = template_card_rect(list_area, template_idx) else {
        return;
    };
    let id = PaneTemplateId::ALL[template_idx];
    let tmpl = id.template();
    let row_style = if is_sel {
        Style::default()
            .bg(p.surface0)
            .fg(p.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.subtext0)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  ", row_style),
            Span::styled(tmpl.name, row_style),
        ])),
        Rect::new(card.x, card.y, card.width, 1),
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("  {}", tmpl.description),
            Style::default().fg(p.overlay1),
        )),
        Rect::new(card.x, card.y + 1, card.width, 1),
    );
    for (line_idx, preview_line) in tmpl.preview.lines().enumerate() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("  {}", preview_line),
                Style::default().fg(p.subtext0),
            )),
            Rect::new(card.x, card.y + 2 + line_idx as u16, card.width, 1),
        );
    }
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
