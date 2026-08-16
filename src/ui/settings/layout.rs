use ratatui::layout::{Constraint, Layout, Rect};

use crate::app::{state::SettingsSection, AppState};

use super::{
    rows::section_rows,
    spinner::{active_spinner_category, SPINNER_CATEGORIES},
};

pub(crate) const SETTINGS_POPUP_WIDTH: u16 = 96;
pub(crate) const SETTINGS_POPUP_HEIGHT: u16 = 30;
pub(crate) const SETTINGS_NAV_WIDTH: u16 = 22;
pub(crate) const SETTINGS_HEADER_ROWS: u16 = 3;
pub(crate) const SETTINGS_FOOTER_ROWS: u16 = 2;
pub(crate) const SETTINGS_ROW_HEIGHT: u16 = 1;
pub(crate) const SETTINGS_SECTION_DESC_ROWS: u16 = 2;
pub(crate) const SETTINGS_SECTION_GAP_ROWS: u16 = 1;
pub(crate) const SETTINGS_SPINNER_CATEGORY_ROWS: u16 = 1;
pub(crate) const SETTINGS_SPINNER_HERO_ROWS: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SettingsLayout {
    pub popup: Rect,
    pub inner: Rect,
    pub title: Rect,
    pub search: Rect,
    pub body: Rect,
    pub nav: Rect,
    pub content: Rect,
    pub footer_hints: Rect,
    pub footer_buttons: Rect,
}

impl SettingsLayout {
    pub(crate) fn compute(area: Rect, app: &AppState) -> Option<Self> {
        let popup = super::super::widgets::centered_popup_rect(
            area,
            SETTINGS_POPUP_WIDTH,
            settings_popup_height(app),
        )?;
        let inner = Rect {
            x: popup.x + 1,
            y: popup.y + 1,
            width: popup.width.saturating_sub(2),
            height: popup.height.saturating_sub(2),
        };
        if inner.width < 20 || inner.height < 10 {
            return None;
        }

        let [header, body, footer] = Layout::vertical([
            Constraint::Length(SETTINGS_HEADER_ROWS),
            Constraint::Min(0),
            Constraint::Length(SETTINGS_FOOTER_ROWS),
        ])
        .areas::<3>(inner);

        let [title, _, search] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas::<3>(header);

        let [nav, content] = Layout::horizontal([
            Constraint::Length(SETTINGS_NAV_WIDTH.min(body.width.saturating_sub(24))),
            Constraint::Min(0),
        ])
        .areas::<2>(body);

        let [footer_hints, footer_buttons] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas::<2>(footer);

        Some(Self {
            popup,
            inner,
            title,
            search,
            body,
            nav,
            content,
            footer_hints,
            footer_buttons,
        })
    }

    pub(crate) fn nav_item_rect(&self, index: usize) -> Option<Rect> {
        SettingsSection::ALL.get(index).map(|_| {
            Rect::new(
                self.nav.x,
                self.nav.y + index as u16,
                self.nav.width,
                SETTINGS_ROW_HEIGHT,
            )
        })
    }

    pub(crate) fn nav_index_at(&self, col: u16, row: u16) -> Option<usize> {
        if col < self.nav.x
            || col >= self.nav.x + self.nav.width
            || row < self.nav.y
            || row >= self.nav.y + SettingsSection::ALL.len() as u16
        {
            return None;
        }
        Some((row - self.nav.y) as usize)
    }

    pub(crate) fn search_rect(&self) -> Rect {
        self.search
    }

    pub(crate) fn search_index_at(&self, col: u16, row: u16) -> bool {
        let rect = self.search_rect();
        col >= rect.x && col < rect.x + rect.width && row == rect.y
    }

    pub(crate) fn content_list_area(&self, app: &AppState) -> Rect {
        let mut y = self.content.y;
        let height = self.content.height;
        y += SETTINGS_SECTION_DESC_ROWS + SETTINGS_SECTION_GAP_ROWS;
        if app.settings.section == SettingsSection::Ui {
            y += SETTINGS_SPINNER_CATEGORY_ROWS
                + SETTINGS_SECTION_GAP_ROWS
                + SETTINGS_SPINNER_HERO_ROWS
                + SETTINGS_SECTION_GAP_ROWS;
        }
        Rect {
            x: self.content.x,
            y,
            width: self.content.width,
            height: height.saturating_sub(y - self.content.y),
        }
    }

    pub(crate) fn spinner_category_rect(&self, app: &AppState) -> Option<Rect> {
        if app.settings.section != SettingsSection::Ui {
            return None;
        }
        let y = self.content.y + SETTINGS_SECTION_DESC_ROWS + SETTINGS_SECTION_GAP_ROWS;
        Some(Rect::new(
            self.content.x,
            y,
            self.content.width,
            SETTINGS_SPINNER_CATEGORY_ROWS,
        ))
    }

    pub(crate) fn spinner_hero_rect(&self, app: &AppState) -> Option<Rect> {
        if app.settings.section != SettingsSection::Ui {
            return None;
        }
        let y = self.content.y
            + SETTINGS_SECTION_DESC_ROWS
            + SETTINGS_SECTION_GAP_ROWS
            + SETTINGS_SPINNER_CATEGORY_ROWS
            + SETTINGS_SECTION_GAP_ROWS;
        Some(Rect::new(
            self.content.x,
            y,
            self.content.width,
            SETTINGS_SPINNER_HERO_ROWS,
        ))
    }

    pub(crate) fn spinner_category_index_at(
        &self,
        app: &AppState,
        col: u16,
        row: u16,
    ) -> Option<usize> {
        let rect = self.spinner_category_rect(app)?;
        if row != rect.y || col < rect.x || col >= rect.x + rect.width {
            return None;
        }
        let rel = (col - rect.x) as usize;
        let mut x = 1usize;
        for (idx, category) in SPINNER_CATEGORIES.iter().enumerate() {
            let width = category.label.len() + 3;
            if rel >= x && rel < x + width {
                return Some(idx);
            }
            x += width;
        }
        None
    }

    pub(crate) fn visible_row_range(&self, app: &AppState) -> (usize, usize) {
        let rows = section_rows(app, app.settings.section);
        let list_area = self.content_list_area(app);
        let visible = list_area.height.max(1) as usize;
        let selected = app.settings.list.selected.min(rows.len().saturating_sub(1));
        let scroll = if selected >= visible {
            selected - visible + 1
        } else {
            0
        };
        (scroll, visible)
    }

    pub(crate) fn content_row_rect(&self, app: &AppState, row_index: usize) -> Option<Rect> {
        let rows = section_rows(app, app.settings.section);
        rows.get(row_index)?;
        let list_area = self.content_list_area(app);
        let (scroll, visible) = self.visible_row_range(app);
        let visible_idx = row_index.saturating_sub(scroll);
        if visible_idx >= visible {
            return None;
        }

        Some(Rect::new(
            list_area.x,
            list_area.y + visible_idx as u16,
            list_area.width,
            SETTINGS_ROW_HEIGHT,
        ))
    }

    pub(crate) fn content_index_at(&self, app: &AppState, col: u16, row: u16) -> Option<usize> {
        if self.spinner_category_index_at(app, col, row).is_some() {
            return None;
        }

        let rows = section_rows(app, app.settings.section);
        let list_area = self.content_list_area(app);
        if col < list_area.x
            || col >= list_area.x + list_area.width
            || row < list_area.y
            || row >= list_area.y + list_area.height
        {
            return None;
        }

        let (scroll, visible) = self.visible_row_range(app);
        let visible_idx = (row - list_area.y) as usize;
        if visible_idx >= visible {
            return None;
        }
        let row_index = scroll + visible_idx;
        rows.get(row_index).map(|_| row_index)
    }
}

pub(crate) fn settings_popup_height(app: &AppState) -> u16 {
    let base = SETTINGS_POPUP_HEIGHT;
    match app.settings.section {
        SettingsSection::Integrations => {
            let integrations = app.integration_recommendations.len().max(2) as u16;
            let installed = super::catalog::installed_plugins_sorted(app).len();
            let catalog = super::catalog::catalog_entries_available(app).len();
            let plugins = installed.saturating_add(catalog).saturating_add(2) as u16;
            base.saturating_add(integrations.min(8)).saturating_add(plugins.min(10))
        }
        _ => base,
    }
}

pub(crate) fn settings_button_rects(
    layout: &SettingsLayout,
    app: &AppState,
    show_primary: bool,
) -> (Option<Rect>, Rect) {
    use super::super::widgets::{action_button_row_rects, ActionButtonSpec};

    let inner = layout.inner;
    let row_y = inner.height.saturating_sub(1);
    if !show_primary {
        let rects = action_button_row_rects(
            inner,
            &[ActionButtonSpec {
                hint: Some("esc"),
                label: "close",
            }],
            2,
            row_y,
        );
        return (None, rects[0]);
    }

    let rects = action_button_row_rects(
        inner,
        &[
            ActionButtonSpec {
                hint: Some("↵"),
                label: settings_primary_button_label(app),
            },
            ActionButtonSpec {
                hint: Some("esc"),
                label: "close",
            },
        ],
        2,
        row_y,
    );
    (Some(rects[0]), rects[1])
}

pub(crate) fn settings_primary_button_label(app: &AppState) -> &'static str {
    match app.settings.section {
        SettingsSection::Theme => "apply",
        SettingsSection::Integrations => {
            if app
                .integration_recommendations
                .iter()
                .any(crate::integration::IntegrationRecommendation::needs_install)
            {
                "install"
            } else {
                "refresh"
            }
        }
        _ => "done",
    }
}

pub(crate) fn settings_show_primary_action(app: &AppState) -> bool {
    match app.settings.section {
        SettingsSection::Theme => true,
        SettingsSection::Integrations => {
            let needs_install = app
                .integration_recommendations
                .iter()
                .any(crate::integration::IntegrationRecommendation::needs_install);
            needs_install || !super::catalog::installed_plugins_sorted(app).is_empty()
        }
        _ => false,
    }
}

pub(crate) fn spinner_category_labels() -> impl Iterator<Item = &'static str> {
    SPINNER_CATEGORIES.iter().map(|category| category.label)
}

pub(crate) fn active_spinner_styles(app: &AppState) -> &'static [crate::config::SpinnerStyle] {
    active_spinner_category(app.settings.spinner_category).styles
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{state::SettingsSection, AppState, Mode};

    fn layout_for_section(section: SettingsSection) -> SettingsLayout {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = section;
        SettingsLayout::compute(Rect::new(0, 0, 120, 40), &app).expect("layout should compute")
    }

    #[test]
    fn nav_index_round_trips_item_rect() {
        let layout = layout_for_section(SettingsSection::Templates);
        for idx in 0..SettingsSection::ALL.len() {
            let rect = layout.nav_item_rect(idx).expect("nav item rect");
            assert_eq!(layout.nav_index_at(rect.x + 1, rect.y), Some(idx));
        }
    }

    #[test]
    fn search_rect_matches_search_index_at() {
        let layout = layout_for_section(SettingsSection::Ui);
        let rect = layout.search_rect();
        assert!(layout.search_index_at(rect.x + 2, rect.y));
        assert!(!layout.search_index_at(rect.x, rect.y.saturating_sub(1)));
    }

    #[test]
    fn content_row_rect_matches_content_index_at_for_template_rows() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Templates;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 120, 40), &app).expect("layout");
        // Row 0 is the "templates" header; the first selectable row is index 1.
        let row_rect = layout
            .content_row_rect(&app, 1)
            .expect("first toggle row should have geometry");
        assert_eq!(
            layout.content_index_at(&app, row_rect.x + 1, row_rect.y),
            Some(1)
        );
    }

    #[test]
    fn content_index_at_returns_header_row_index() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Ui;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 120, 40), &app).expect("layout");
        let header_rect = layout
            .content_row_rect(&app, 0)
            .expect("ui header should have geometry");
        assert_eq!(
            layout.content_index_at(&app, header_rect.x + 1, header_rect.y),
            Some(0)
        );
    }

    #[test]
    fn layout_template_rows_are_plain_content_rows() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Templates;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 120, 40), &app).expect("layout");
        let rows = section_rows(&app, SettingsSection::Templates);
        let template_idx = rows
            .iter()
            .position(|row| {
                matches!(
                    row.kind,
                    crate::ui::settings::rows::SettingsRowKind::Template
                )
            })
            .expect("template row");
        let rect = layout
            .content_row_rect(&app, template_idx)
            .expect("template should use normal row geometry");
        assert_eq!(
            layout.content_index_at(&app, rect.x + 1, rect.y),
            Some(template_idx)
        );
    }

    #[test]
    fn spinner_category_hit_matches_category_rect_row() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Ui;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 120, 40), &app).expect("layout");
        let rect = layout
            .spinner_category_rect(&app)
            .expect("ui should expose category row");
        assert_eq!(
            layout.spinner_category_index_at(&app, rect.x + 2, rect.y),
            Some(0)
        );
    }

    #[test]
    fn integrations_section_shows_primary_action() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Integrations;
        // No plugins installed and nothing to install: no primary action.
        assert!(!settings_show_primary_action(&app));
        // With a demo plugin installed the primary action refreshes.
        app.installed_plugins.insert(
            "com.test.demo".to_string(),
            crate::api::schema::InstalledPluginInfo {
                plugin_id: "com.test.demo".to_string(),
                name: "demo".to_string(),
                version: "0.1.0".to_string(),
                min_herdr_version: String::new(),
                description: None,
                manifest_path: "/tmp/herdr-plugin.toml".to_string(),
                plugin_root: "/tmp".to_string(),
                enabled: true,
                platforms: None,
                build: Vec::new(),
                startup: Vec::new(),
                actions: Vec::new(),
                events: Vec::new(),
                panes: Vec::new(),
                link_handlers: Vec::new(),
                source: Default::default(),
                warnings: Vec::new(),
            },
        );
        assert!(settings_show_primary_action(&app));
        assert_eq!(settings_primary_button_label(&app), "refresh");
    }

    #[test]
    fn button_rects_align_with_footer_buttons_area() {
        let app = AppState::test_new();
        let layout = layout_for_section(SettingsSection::Theme);
        let (apply, close) = settings_button_rects(&layout, &app, true);
        assert!(apply.is_some());
        assert!(close.y >= layout.footer_buttons.y);
    }
}
