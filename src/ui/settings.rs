use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};

use super::widgets::{
    action_button_row_rects, centered_popup_rect, modal_stack_areas, panel_contrast_fg,
    render_action_button, render_panel_shell, ActionButtonSpec,
};
use crate::{
    app::{
        state::{ExperimentSetting, UI_SPINNER_OFFSET},
        AppState,
    },
    config::ToastDelivery,
};

pub(crate) const SETTINGS_POPUP_WIDTH: u16 = 76;
pub(crate) const SETTINGS_POPUP_BASE_HEIGHT: u16 = 22;

pub(crate) const SETTINGS_DESC_ROWS: u16 = 2;
pub(crate) const SETTINGS_SPACER_ROWS: u16 = 1;

pub(crate) fn settings_list_area(content: Rect) -> Rect {
    let header_rows = SETTINGS_DESC_ROWS + SETTINGS_SPACER_ROWS;
    Rect {
        x: content.x,
        y: content.y + header_rows,
        width: content.width,
        height: content.height.saturating_sub(header_rows),
    }
}

pub(crate) fn settings_sound_index_at(list_area: Rect, col: u16, row: u16) -> Option<usize> {
    if col < list_area.x || col >= list_area.x + list_area.width || row < list_area.y {
        return None;
    }
    match row - list_area.y {
        0 => Some(0),
        1 | 2 => None,
        r if (3..=6).contains(&r) => Some((r - 2) as usize),
        _ => None,
    }
}

pub(crate) fn settings_ui_index_at(
    list_area: Rect,
    col: u16,
    row: u16,
    selected: usize,
) -> Option<usize> {
    use crate::app::state::{UI_SPINNER_OFFSET, UI_TOGGLE_COUNT};

    if col < list_area.x || col >= list_area.x + list_area.width || row < list_area.y {
        return None;
    }
    let row_idx = (row - list_area.y) as usize;
    if row_idx < UI_TOGGLE_COUNT {
        return Some(row_idx);
    }
    if row_idx == UI_TOGGLE_COUNT {
        return None;
    }
    let grid_row = row_idx - UI_TOGGLE_COUNT - 1;
    let col_width = (list_area.width as usize / 2).max(20);
    let col_idx = if (col - list_area.x) as usize >= col_width {
        1
    } else {
        0
    };
    let grid_height = list_area.height.saturating_sub(UI_TOGGLE_COUNT as u16 + 1) as usize;
    let spinner_selected = selected.saturating_sub(UI_SPINNER_OFFSET);
    let selected_row = spinner_selected / 2;
    let scroll = if selected_row >= grid_height {
        selected_row - grid_height + 1
    } else {
        0
    };
    let spinner_idx = (grid_row + scroll) * 2 + col_idx;
    (spinner_idx < crate::config::SpinnerStyle::ALL.len())
        .then_some(UI_SPINNER_OFFSET + spinner_idx)
}

pub(crate) fn template_card_height(template: &crate::pane_template::PaneTemplate) -> u16 {
    2 + template.preview.lines().count() as u16
}

pub(crate) fn settings_template_card_rect(list_area: Rect, idx: usize) -> Option<Rect> {
    use crate::pane_template::PaneTemplateId;

    let id = *PaneTemplateId::ALL.get(idx)?;
    let col_width = (list_area.width as usize / 2).max(20);
    let col = idx % 2;
    let grid_row = idx / 2;
    let mut y = list_area.y;
    for row in 0..grid_row {
        let left_idx = row * 2;
        let left_h = PaneTemplateId::ALL
            .get(left_idx)
            .map(|id| template_card_height(&id.template()))
            .unwrap_or(0);
        let right_h = PaneTemplateId::ALL
            .get(left_idx + 1)
            .map(|id| template_card_height(&id.template()))
            .unwrap_or(0);
        y = y.saturating_add(left_h.max(right_h));
    }
    let height = template_card_height(&id.template());
    Some(Rect {
        x: list_area.x + col as u16 * col_width as u16,
        y,
        width: col_width as u16,
        height,
    })
}

pub(crate) fn settings_template_index_at(list_area: Rect, col: u16, row: u16) -> Option<usize> {
    use crate::pane_template::PaneTemplateId;

    for idx in 0..PaneTemplateId::ALL.len() {
        let card = settings_template_card_rect(list_area, idx)?;
        if col >= card.x && col < card.x + card.width && row >= card.y && row < card.y + card.height
        {
            return Some(idx);
        }
    }
    None
}

pub(crate) fn settings_popup_height(app: &AppState) -> u16 {
    use crate::app::state::SettingsSection;
    match app.settings.section {
        SettingsSection::Ui => 30, // taller for toggles + 2-column spinner grid
        SettingsSection::Templates => 28, // taller for template previews
        SettingsSection::Integrations => {
            let list_rows = app.integration_recommendations.len().max(1) as u16;
            let footer_rows = integrations_footer_height(app, SETTINGS_POPUP_WIDTH - 2);
            // borders 2 + header 3 + stack gaps 2 + modal footer 2
            // + section title 1 + description 2 + spacers 2
            (14 + list_rows + footer_rows).max(SETTINGS_POPUP_BASE_HEIGHT)
        }
        _ => SETTINGS_POPUP_BASE_HEIGHT,
    }
}

pub(super) fn render_settings_overlay(app: &AppState, frame: &mut Frame, area: Rect) {
    use crate::app::state::SettingsSection;

    let p = &app.palette;
    let Some(popup) = centered_popup_rect(area, SETTINGS_POPUP_WIDTH, settings_popup_height(app))
    else {
        return;
    };

    super::dim_background(frame, area);

    let Some(inner) = render_panel_shell(frame, popup, p.accent, p.panel_bg) else {
        return;
    };
    if inner.height < 4 || inner.width < 10 {
        return;
    }

    let stack = modal_stack_areas(inner, 3, 2, 0, 1);
    let header_rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas::<3>(stack.header);

    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            " settings",
            Style::default().fg(p.text).add_modifier(Modifier::BOLD),
        )])),
        header_rows[0],
    );

    let tab_labels = SettingsSection::ALL.iter().map(|section| {
        if app.settings_section_has_badge(*section) {
            Line::from(vec![
                Span::styled(
                    "● ",
                    Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
                ),
                Span::raw(section.label()),
            ])
        } else {
            Line::from(section.label())
        }
    });
    let tabs = Tabs::new(tab_labels)
        .select(
            SettingsSection::ALL
                .iter()
                .position(|section| *section == app.settings.section)
                .unwrap_or(0),
        )
        .style(Style::default().fg(p.overlay1))
        .highlight_style(
            Style::default()
                .fg(panel_contrast_fg(p))
                .bg(p.accent)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" ")
        .padding(" ", " ");
    frame.render_widget(tabs, header_rows[1]);

    let sep = "─".repeat(inner.width as usize);
    frame.render_widget(
        Paragraph::new(Span::styled(&sep, Style::default().fg(p.surface0))),
        header_rows[2],
    );

    let content_area = stack.content;

    match app.settings.section {
        SettingsSection::Theme => {
            render_settings_theme(app, frame, content_area);
        }
        SettingsSection::Ui => {
            render_settings_ui(app, frame, content_area);
        }
        SettingsSection::Sound => {
            render_settings_sound(app, frame, content_area);
        }
        SettingsSection::System => {
            render_settings_system(app, frame, content_area);
        }
        SettingsSection::Templates => {
            render_settings_templates(app, frame, content_area);
        }
        SettingsSection::Integrations => {
            render_settings_integrations(app, frame, content_area);
        }
    }

    if let Some(footer_area) = stack.footer {
        let footer_rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)])
            .areas::<2>(footer_area);
        let primary_label = settings_primary_button_label(app.settings.section);
        let show_primary = settings_show_primary_action(app);
        let (apply_rect, close_rect) =
            settings_button_rects(inner, app.settings.section, show_primary);
        if let Some(apply_rect) = apply_rect {
            render_action_button(
                frame,
                apply_rect,
                Some("↵"),
                primary_label,
                Style::default()
                    .fg(panel_contrast_fg(p))
                    .bg(p.accent)
                    .add_modifier(Modifier::BOLD),
            );
        }
        render_action_button(
            frame,
            close_rect,
            Some("esc"),
            "close",
            Style::default()
                .fg(p.text)
                .bg(p.surface0)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" ↑↓", Style::default().fg(p.overlay0)),
                Span::styled(" select  ", Style::default().fg(p.overlay1)),
                Span::styled("tab", Style::default().fg(p.overlay0)),
                Span::styled(" section", Style::default().fg(p.overlay1)),
            ])),
            footer_rows[0],
        );
    }
}

pub(crate) fn settings_primary_button_label(
    section: crate::app::state::SettingsSection,
) -> &'static str {
    match section {
        crate::app::state::SettingsSection::Integrations => "install",
        _ => "apply",
    }
}

pub(crate) fn settings_show_primary_action(app: &AppState) -> bool {
    match app.settings.section {
        crate::app::state::SettingsSection::Integrations => app
            .integration_recommendations
            .iter()
            .any(crate::integration::IntegrationRecommendation::needs_install),
        _ => true,
    }
}

pub(crate) fn settings_button_rects(
    inner: Rect,
    section: crate::app::state::SettingsSection,
    show_primary: bool,
) -> (Option<Rect>, Rect) {
    if !show_primary {
        let rects = action_button_row_rects(
            inner,
            &[ActionButtonSpec {
                hint: Some("esc"),
                label: "close",
            }],
            2,
            inner.height.saturating_sub(1),
        );
        return (None, rects[0]);
    }

    let rects = action_button_row_rects(
        inner,
        &[
            ActionButtonSpec {
                hint: Some("↵"),
                label: settings_primary_button_label(section),
            },
            ActionButtonSpec {
                hint: Some("esc"),
                label: "close",
            },
        ],
        2,
        inner.height.saturating_sub(1),
    );
    (Some(rects[0]), rects[1])
}

fn integrations_footer_paragraph(app: &AppState) -> Paragraph<'static> {
    let p = &app.palette;
    let mut footer_lines = Vec::new();
    if !app.integration_install_messages.is_empty() {
        for message in &app.integration_install_messages {
            footer_lines.push(Line::from(Span::styled(
                format!(" {message}"),
                Style::default().fg(p.overlay1),
            )));
        }
    } else {
        let found_any = app.integration_recommendations.iter().any(|item| {
            item.available || item.state != crate::integration::IntegrationStatusKind::NotInstalled
        });
        let hint = if app
            .integration_recommendations
            .iter()
            .any(crate::integration::IntegrationRecommendation::needs_install)
        {
            " press install to add available or outdated integrations"
        } else if found_any {
            " all detected integrations are installed"
        } else {
            " no supported agent CLIs found on PATH"
        };
        footer_lines.push(Line::from(Span::styled(
            hint.to_string(),
            Style::default().fg(p.overlay1),
        )));
    }
    Paragraph::new(footer_lines).wrap(ratatui::widgets::Wrap { trim: false })
}

fn integrations_footer_height(app: &AppState, width: u16) -> u16 {
    (integrations_footer_paragraph(app).line_count(width) as u16).min(6)
}

fn render_settings_integrations(app: &AppState, frame: &mut Frame, area: Rect) {
    let p = &app.palette;

    let footer = integrations_footer_paragraph(app);
    let footer_height = integrations_footer_height(app, area.width);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(footer_height),
    ])
    .areas::<6>(area);

    frame.render_widget(
        Paragraph::new("agent integrations")
            .style(Style::default().fg(p.text).add_modifier(Modifier::BOLD)),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(
            "let agents report state directly instead of relying only on process detection",
        )
        .style(Style::default().fg(p.overlay1))
        .wrap(ratatui::widgets::Wrap { trim: false }),
        rows[1],
    );

    let mut lines = Vec::new();
    for item in &app.integration_recommendations {
        let marker = match item.state {
            crate::integration::IntegrationStatusKind::Current => "✓",
            crate::integration::IntegrationStatusKind::Outdated => "↻",
            crate::integration::IntegrationStatusKind::NotInstalled if item.available => "+",
            crate::integration::IntegrationStatusKind::NotInstalled => "–",
        };
        let marker_style = match item.state {
            crate::integration::IntegrationStatusKind::Current => Style::default().fg(p.green),
            crate::integration::IntegrationStatusKind::Outdated => Style::default().fg(p.yellow),
            crate::integration::IntegrationStatusKind::NotInstalled if item.available => {
                Style::default().fg(p.accent)
            }
            crate::integration::IntegrationStatusKind::NotInstalled => {
                Style::default().fg(p.overlay0)
            }
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {marker} "), marker_style),
            Span::styled(
                format!("{:<9}", item.label),
                Style::default().fg(p.subtext0),
            ),
            Span::styled(item.status_label(), Style::default().fg(p.overlay1)),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " no integration targets available",
            Style::default().fg(p.overlay1),
        )));
    }

    frame.render_widget(Paragraph::new(lines), rows[3]);
    frame.render_widget(footer, rows[5]);
}

fn render_settings_theme(app: &AppState, frame: &mut Frame, area: Rect) {
    use crate::app::state::THEME_NAMES;

    let p = &app.palette;
    let items: Vec<ListItem> = THEME_NAMES
        .iter()
        .map(|name| {
            let is_current = name.to_lowercase().replace([' ', '_'], "-")
                == app.theme_name.to_lowercase().replace([' ', '_'], "-");
            let marker = if is_current { " ✓" } else { "" };
            ListItem::new(Line::from(vec![
                Span::styled(*name, Style::default().fg(p.subtext0)),
                Span::styled(marker, Style::default().fg(p.green)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(p.surface0)
                .fg(p.text)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▸ ")
        .style(Style::default().fg(p.subtext0));

    let mut state = ListState::default().with_selected(Some(app.settings.list.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

/// Ui tab: toggle rows + spinner grid.
fn render_settings_ui(app: &AppState, frame: &mut Frame, area: Rect) {
    use crate::config::SpinnerStyle;
    let p = &app.palette;

    let [desc_area, _, _] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .areas::<3>(area);

    super::widgets::render_modal_description(
        frame,
        desc_area,
        "ui options and spinner animation style",
        Style::default().fg(p.overlay1),
    );

    let list_area = settings_list_area(area);
    let selected = app.settings.list.selected;
    let ui_toggles: &[(bool, &str, &str)] = &[
        (
            app.pane_borders_enabled(),
            "pane borders",
            "draw borders around split panes",
        ),
        (
            app.pane_gaps_enabled(),
            "pane gaps",
            "keep split panes visually separated",
        ),
        (
            app.agent_border_labels_enabled(),
            "agent labels",
            "show agent names in pane borders",
        ),
        (
            app.hide_tab_bar_when_single_tab_enabled(),
            "hide tab bar",
            "hide tab row when only one tab",
        ),
    ];

    // Render toggle rows.
    for (idx, (enabled, title, desc)) in ui_toggles.iter().enumerate() {
        let is_sel = selected == idx;
        let marker = if *enabled { "[✓]" } else { "[ ]" };
        let row_style = if is_sel {
            Style::default()
                .bg(p.surface0)
                .fg(p.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.subtext0)
        };
        let line = Line::from(vec![
            Span::styled(format!(" {marker} "), row_style),
            Span::styled(*title, row_style),
            Span::styled(format!("  — {desc}"), Style::default().fg(p.overlay1)),
        ]);
        frame.render_widget(
            Paragraph::new(line),
            Rect::new(list_area.x, list_area.y + idx as u16, list_area.width, 1),
        );
    }

    // Separator.
    let sep_y = list_area.y + ui_toggles.len() as u16;
    let sep = "─".repeat(list_area.width as usize);
    frame.render_widget(
        Paragraph::new(Span::styled(&sep, Style::default().fg(p.surface0))),
        Rect::new(list_area.x, sep_y, list_area.width, 1),
    );

    // Spinner grid below the toggles.
    let grid_area = Rect::new(
        list_area.x,
        sep_y + 1,
        list_area.width,
        list_area.height.saturating_sub(ui_toggles.len() as u16 + 1),
    );

    let all = SpinnerStyle::ALL;
    let spinner_selected = selected.saturating_sub(UI_SPINNER_OFFSET);
    let current_idx = all
        .iter()
        .position(|&s| s == app.spinner_style)
        .unwrap_or(0);

    let visible_rows = grid_area.height as usize;
    let selected_row = spinner_selected / 2;
    let scroll = if selected_row >= visible_rows {
        selected_row - visible_rows + 1
    } else {
        0
    };

    let col_width = (grid_area.width as usize / 2).max(20);
    let tick = app.settings.preview_tick;

    for (idx, &style) in all.iter().enumerate() {
        let visible_row = idx / 2;
        let visible_idx = visible_row.saturating_sub(scroll);
        if visible_idx >= visible_rows {
            break;
        }
        let col = idx % 2;
        let x = grid_area.x + col as u16 * col_width as u16;
        let y = grid_area.y + visible_idx as u16;
        let is_selected = idx == spinner_selected;
        let is_current = idx == current_idx;
        let preview_frame =
            style.frames()[(tick as usize / style.speed_divisor() as usize) % style.frames().len()];

        let marker = if is_current { "✓" } else { " " };
        let style_fg = if is_selected { p.text } else { p.subtext0 };
        let row_style = if is_selected {
            Style::default()
                .bg(p.surface0)
                .fg(p.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(style_fg)
        };

        let line = Line::from(vec![
            Span::styled(format!(" {marker} "), row_style),
            Span::styled(preview_frame.to_string(), Style::default().fg(p.yellow)),
            Span::styled("  ", row_style),
            Span::styled(style.label(), row_style),
        ]);
        frame.render_widget(Paragraph::new(line), Rect::new(x, y, col_width as u16, 1));
    }
}

/// Sound tab: sound toggle + toast delivery options.
fn render_settings_sound(app: &AppState, frame: &mut Frame, area: Rect) {
    let p = &app.palette;
    let [desc_area, _, _] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .areas::<3>(area);

    super::widgets::render_modal_description(
        frame,
        desc_area,
        "sound alerts and notification popups",
        Style::default().fg(p.overlay1),
    );

    let list_area = settings_list_area(area);
    let selected = app.settings.list.selected;

    // Sound toggle.
    let sound_on = app.sound_enabled();
    let sound_marker = if sound_on { "[✓]" } else { "[ ]" };
    let sound_style = if selected == 0 {
        Style::default()
            .bg(p.surface0)
            .fg(p.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.subtext0)
    };
    frame.render_widget(
        Paragraph::new(format!(" {sound_marker} sound alerts")).style(sound_style),
        Rect::new(list_area.x, list_area.y, list_area.width, 1),
    );

    // Separator.
    let sep = "─".repeat(list_area.width as usize);
    frame.render_widget(
        Paragraph::new(Span::styled(&sep, Style::default().fg(p.surface0))),
        Rect::new(list_area.x, list_area.y + 1, list_area.width, 1),
    );

    // Toast delivery options.
    let toast_label = "notification popups";
    frame.render_widget(
        Paragraph::new(Span::styled(toast_label, Style::default().fg(p.overlay1))),
        Rect::new(list_area.x, list_area.y + 2, list_area.width, 1),
    );

    let toast_options = [
        ("off", ToastDelivery::Off),
        ("inside herdr", ToastDelivery::Herdr),
        ("via terminal", ToastDelivery::Terminal),
        ("via system", ToastDelivery::System),
    ];
    let current_delivery = app.toast_delivery();
    for (idx, (label, delivery)) in toast_options.iter().enumerate() {
        let list_idx = 1 + idx;
        let is_sel = selected == list_idx;
        let is_current = *delivery == current_delivery;
        let marker = if is_current { "✓" } else { " " };
        let row_style = if is_sel {
            Style::default()
                .bg(p.surface0)
                .fg(p.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.subtext0)
        };
        frame.render_widget(
            Paragraph::new(format!(" {marker} {label}")).style(row_style),
            Rect::new(
                list_area.x,
                list_area.y + 3 + idx as u16,
                list_area.width,
                1,
            ),
        );
    }
}

/// System tab: experiments + fleet/plugins info.
fn render_settings_system(app: &AppState, frame: &mut Frame, area: Rect) {
    let p = &app.palette;
    let [desc_area, _, list_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .areas::<3>(area);

    super::widgets::render_modal_description(
        frame,
        desc_area,
        "experiments and system info",
        Style::default().fg(p.overlay1),
    );

    let selected = app.settings.list.selected;

    // Experiment toggles.
    for (idx, setting) in ExperimentSetting::ALL.iter().copied().enumerate() {
        let marker = if setting.enabled(app) { "[✓]" } else { "[ ]" };
        let style = if selected == idx {
            Style::default()
                .bg(p.surface0)
                .fg(p.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.subtext0)
        };
        let row = Rect::new(list_area.x, list_area.y + idx as u16, list_area.width, 1);
        frame.render_widget(
            Paragraph::new(format!(" {} {marker}", setting.label())).style(style),
            row,
        );
    }

    // Separator + info text.
    let info_y = list_area.y + ExperimentSetting::ALL.len() as u16 + 1;
    let sep = "─".repeat(list_area.width as usize);
    frame.render_widget(
        Paragraph::new(Span::styled(&sep, Style::default().fg(p.surface0))),
        Rect::new(list_area.x, info_y, list_area.width, 1),
    );

    let info_lines = vec![
        Line::from(Span::styled(
            "CHEF Fleet Operations",
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "fleet ops bar shows per-pane agent metadata",
            Style::default().fg(p.subtext0),
        )),
        Line::from(Span::styled(
            "plugins: linear, github, fleet-health, cloudflare",
            Style::default().fg(p.subtext0),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Plugin Marketplace",
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "install: herdr plugin install <id>",
            Style::default().fg(p.subtext0),
        )),
    ];
    frame.render_widget(
        Paragraph::new(info_lines),
        Rect::new(
            list_area.x,
            info_y + 1,
            list_area.width,
            list_area
                .height
                .saturating_sub(ExperimentSetting::ALL.len() as u16 + 2),
        ),
    );
}

/// Templates tab: pane layout templates.
fn render_settings_templates(app: &AppState, frame: &mut Frame, area: Rect) {
    let p = &app.palette;
    let [desc_area, _, _] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .areas::<3>(area);

    super::widgets::render_modal_description(
        frame,
        desc_area,
        "apply a pane layout template to the current tab",
        Style::default().fg(p.overlay1),
    );

    let list_area = settings_list_area(area);
    let selected = app.settings.list.selected;

    for idx in 0..crate::pane_template::PaneTemplateId::ALL.len() {
        let Some(card) = settings_template_card_rect(list_area, idx) else {
            continue;
        };
        let id = crate::pane_template::PaneTemplateId::ALL[idx];
        let tmpl = id.template();
        let is_sel = idx == selected;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{state::SettingsSection, Mode};
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn experiments_pane_history_uses_settings_checkmark_marker() {
        let mut app = AppState::test_new();
        app.pane_history_persistence = true;
        app.settings.section = SettingsSection::System;
        app.settings.list.selected = 0;
        app.mode = Mode::Settings;

        let mut terminal =
            Terminal::new(TestBackend::new(80, 24)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 80, 24)))
            .expect("settings overlay should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("pane screen history [✓]"));
        assert!(!rendered.contains("[x]"));
    }

    #[test]
    fn experiments_pane_history_keeps_empty_checkbox_marker_when_disabled() {
        let mut app = AppState::test_new();
        app.pane_history_persistence = false;
        app.settings.section = SettingsSection::System;
        app.settings.list.selected = 0;
        app.mode = Mode::Settings;

        let mut terminal =
            Terminal::new(TestBackend::new(80, 24)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 80, 24)))
            .expect("settings overlay should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("pane screen history [ ]"));
    }

    #[test]
    fn experiments_renders_switch_ascii_input_source_row() {
        let mut app = AppState::test_new();
        app.switch_ascii_input_source_in_prefix = true;
        app.settings.section = SettingsSection::System;
        app.settings.list.selected = 1;
        app.mode = Mode::Settings;

        let mut terminal =
            Terminal::new(TestBackend::new(80, 24)).expect("test terminal should initialize");
        terminal
            .draw(|frame| render_settings_overlay(&app, frame, Rect::new(0, 0, 80, 24)))
            .expect("settings overlay should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("switch to ascii input source in prefix (macOS) [✓]"));
    }
}
