use ratatui::layout::{Constraint, Layout, Rect};

use crate::app::{state::SettingsSection, AppState};

use super::{
    rows::{
        appearance_theme_labels, row_shown_in_scroll_list, scroll_list_row_indices, section_rows,
    },
    spinner::{active_spinner_category, SPINNER_CATEGORIES},
    widgets::{
        chip_row_index_at, chip_row_rects, chip_wrap_index_at, chip_wrap_row_count,
        CHIP_HORIZONTAL_GAP, SETTING_BLOCK_HEADER_ROWS,
    },
};

/// Preferred / maximum CONFIG MENU size. Real size tracks the terminal and
/// fills most of the screen (see [`settings_popup_size`]) — much larger than
/// the old ~100×32 island.
pub(crate) const SETTINGS_POPUP_WIDTH: u16 = 180;
pub(crate) const SETTINGS_POPUP_HEIGHT: u16 = 56;
/// Horizontal / vertical fill of the host terminal (percent).
pub(crate) const SETTINGS_FILL_WIDTH_PCT: u16 = 96;
pub(crate) const SETTINGS_FILL_HEIGHT_PCT: u16 = 94;
pub(crate) const SETTINGS_MARGIN_X: u16 = 1;
pub(crate) const SETTINGS_MARGIN_Y: u16 = 1;
pub(crate) const SETTINGS_MIN_WIDTH: u16 = 72;
pub(crate) const SETTINGS_MIN_HEIGHT: u16 = 22;
pub(crate) const SETTINGS_NAV_WIDTH: u16 = 28;
pub(crate) const SETTINGS_HEADER_ROWS: u16 = 4;
pub(crate) const SETTINGS_FOOTER_ROWS: u16 = 3;
pub(crate) const SETTINGS_ROW_HEIGHT: u16 = 1;
pub(crate) const SETTINGS_NAV_ACCENT_COLS: u16 = 1;
pub(crate) const SETTINGS_SECTION_DESC_ROWS: u16 = 2;
pub(crate) const SETTINGS_SECTION_GAP_ROWS: u16 = 1;
pub(crate) const SETTINGS_SPINNER_CATEGORY_ROWS: u16 = 1;
pub(crate) const SETTINGS_SPINNER_HERO_ROWS: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SettingsFooterButtons {
    pub tertiary: Option<Rect>,
    pub save: Rect,
    pub cancel: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SettingsLayout {
    pub popup: Rect,
    pub inner: Rect,
    pub title: Rect,
    pub subtitle: Rect,
    pub search: Rect,
    pub body: Rect,
    pub nav: Rect,
    pub content: Rect,
    pub footer_hints: Rect,
    pub footer_buttons: Rect,
}

/// Near-fullscreen CONFIG MENU size for `area`, clamped to target max.
pub(crate) fn settings_popup_size(area: Rect, app: &AppState) -> (u16, u16) {
    let max_w = area
        .width
        .saturating_sub(SETTINGS_MARGIN_X.saturating_mul(2));
    let max_h = area
        .height
        .saturating_sub(SETTINGS_MARGIN_Y.saturating_mul(2));
    let fill_w = area
        .width
        .saturating_mul(SETTINGS_FILL_WIDTH_PCT)
        .saturating_div(100)
        .max(SETTINGS_MIN_WIDTH.min(max_w));
    let fill_h = area
        .height
        .saturating_mul(SETTINGS_FILL_HEIGHT_PCT)
        .saturating_div(100)
        .max(SETTINGS_MIN_HEIGHT.min(max_h));
    let width = fill_w
        .min(max_w)
        .min(SETTINGS_POPUP_WIDTH)
        .max(SETTINGS_MIN_WIDTH.min(max_w));
    let height = fill_h
        .min(max_h)
        .min(settings_popup_height(app))
        .max(SETTINGS_MIN_HEIGHT.min(max_h));
    (width.max(4), height.max(4))
}

impl SettingsLayout {
    pub(crate) fn compute(area: Rect, app: &AppState) -> Option<Self> {
        let (popup_w, popup_h) = settings_popup_size(area, app);
        let popup = super::super::widgets::centered_popup_rect(area, popup_w, popup_h)?;
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

        let [title, subtitle, search] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas::<3>(header);

        let [nav, content] = Layout::horizontal([
            Constraint::Length(SETTINGS_NAV_WIDTH.min(body.width.saturating_sub(40))),
            Constraint::Min(0),
        ])
        .areas::<2>(body);

        // Taller footer: breathing room above buttons on the large shell.
        let [footer_hints, _, footer_buttons] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas::<3>(footer);

        Some(Self {
            popup,
            inner,
            title,
            subtitle,
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

    pub(crate) fn content_body_y(&self, _app: &AppState) -> u16 {
        self.content.y + SETTINGS_SECTION_DESC_ROWS + SETTINGS_SECTION_GAP_ROWS
    }

    fn appearance_theme_chip_rows(&self, app: &AppState) -> u16 {
        let labels = appearance_theme_labels(app);
        if labels.is_empty() {
            return 0;
        }
        chip_wrap_row_count(self.content.width, &labels, CHIP_HORIZONTAL_GAP).max(1)
    }

    pub(crate) fn appearance_block_header_y(&self, app: &AppState) -> u16 {
        self.content_body_y(app)
    }

    pub(crate) fn appearance_auto_switch_y(&self, app: &AppState) -> u16 {
        self.appearance_block_header_y(app) + SETTING_BLOCK_HEADER_ROWS
    }

    pub(crate) fn appearance_theme_chips_rect(&self, app: &AppState) -> Rect {
        let y = self.appearance_auto_switch_y(app) + SETTINGS_ROW_HEIGHT;
        let rows = self.appearance_theme_chip_rows(app);
        Rect::new(self.content.x, y, self.content.width, rows)
    }

    pub(crate) fn appearance_spinner_header_y(&self, app: &AppState) -> u16 {
        self.appearance_theme_chips_rect(app).y
            + self.appearance_theme_chips_rect(app).height
            + SETTINGS_SECTION_GAP_ROWS
    }

    pub(crate) fn spinner_category_rect(&self, app: &AppState) -> Option<Rect> {
        if app.settings.section != SettingsSection::Appearance {
            return None;
        }
        let y = self.appearance_spinner_header_y(app) + SETTING_BLOCK_HEADER_ROWS;
        Some(Rect::new(
            self.content.x,
            y,
            self.content.width,
            SETTINGS_SPINNER_CATEGORY_ROWS,
        ))
    }

    pub(crate) fn spinner_hero_rect(&self, app: &AppState) -> Option<Rect> {
        if app.settings.section != SettingsSection::Appearance {
            return None;
        }
        let y = self
            .spinner_category_rect(app)?
            .y
            .saturating_add(SETTINGS_SPINNER_CATEGORY_ROWS)
            .saturating_add(SETTINGS_SECTION_GAP_ROWS);
        Some(Rect::new(
            self.content.x,
            y,
            self.content.width,
            SETTINGS_SPINNER_HERO_ROWS,
        ))
    }

    #[allow(dead_code)]
    pub(crate) fn chip_row_rect(&self, y: u16, labels: &[&str]) -> Vec<Rect> {
        chip_row_rects(
            Rect::new(self.content.x, y, self.content.width, 1),
            labels,
            CHIP_HORIZONTAL_GAP,
        )
    }

    #[allow(dead_code)]
    pub(crate) fn chip_row_index_at(
        &self,
        y: u16,
        labels: &[&str],
        col: u16,
        row: u16,
    ) -> Option<usize> {
        super::widgets::chip_row_index_at(
            Rect::new(self.content.x, y, self.content.width, 1),
            labels,
            CHIP_HORIZONTAL_GAP,
            col,
            row,
        )
    }

    #[allow(dead_code)]
    pub(crate) fn checklist_group_areas(&self, y: u16, item_count: usize) -> (Rect, Rect) {
        let area = Rect::new(
            self.content.x,
            y,
            self.content.width,
            self.content.height.saturating_sub(y - self.content.y),
        );
        super::widgets::checklist_group_areas(area, item_count)
    }

    #[allow(dead_code)]
    pub(crate) fn checklist_item_index_at(
        &self,
        items_area: Rect,
        item_count: usize,
        col: u16,
        row: u16,
    ) -> Option<usize> {
        super::widgets::checklist_item_index_at(items_area, item_count, col, row)
    }

    pub(crate) fn setting_block_header_rect(&self, y: u16) -> Rect {
        super::widgets::setting_block_header_rect(Rect::new(
            self.content.x,
            y,
            self.content.width,
            1,
        ))
    }

    pub(crate) fn search_rect(&self) -> Rect {
        self.search
    }

    #[allow(dead_code)]
    pub(crate) fn subtitle_rect(&self) -> Rect {
        self.subtitle
    }

    pub(crate) fn search_index_at(&self, col: u16, row: u16) -> bool {
        let rect = self.search_rect();
        col >= rect.x && col < rect.x + rect.width && row == rect.y
    }

    pub(crate) fn content_list_area(&self, app: &AppState) -> Rect {
        if super::content::uses_block_layout(app.settings.section) {
            let top = super::content::content_list_top(self, app);
            return Rect {
                x: self.content.x,
                y: top,
                width: self.content.width,
                height: self.content.height.saturating_sub(top - self.content.y),
            };
        }
        let y = if app.settings.section == SettingsSection::Appearance {
            self.spinner_hero_rect(app)
                .map(|rect| {
                    rect.y
                        .saturating_add(rect.height)
                        .saturating_add(SETTINGS_SECTION_GAP_ROWS)
                })
                .unwrap_or_else(|| self.content_body_y(app))
        } else {
            self.content_body_y(app)
        };
        Rect {
            x: self.content.x,
            y,
            width: self.content.width,
            height: self.content.height.saturating_sub(y - self.content.y),
        }
    }

    pub(crate) fn appearance_theme_chip_index_at(
        &self,
        app: &AppState,
        col: u16,
        row: u16,
    ) -> Option<usize> {
        let rect = self.appearance_theme_chips_rect(app);
        let labels = appearance_theme_labels(app);
        chip_wrap_index_at(rect, &labels, CHIP_HORIZONTAL_GAP, col, row)
    }

    pub(crate) fn spinner_category_index_at(
        &self,
        app: &AppState,
        col: u16,
        row: u16,
    ) -> Option<usize> {
        let rect = self.spinner_category_rect(app)?;
        let labels: Vec<&str> = SPINNER_CATEGORIES.iter().map(|c| c.label).collect();
        chip_row_index_at(rect, &labels, CHIP_HORIZONTAL_GAP, col, row)
    }

    pub(crate) fn visible_row_range(&self, app: &AppState) -> (usize, usize) {
        if super::content::uses_block_layout(app.settings.section) {
            let list_area = self.content_list_area(app);
            return (0, list_area.height.max(1) as usize);
        }
        let indices = scroll_list_row_indices(app, app.settings.section);
        let list_area = self.content_list_area(app);
        let visible = list_area.height.max(1) as usize;
        let selected = app.settings.list.selected;
        let selected_pos = indices.iter().position(|idx| *idx == selected).unwrap_or(0);
        let scroll = if selected_pos >= visible {
            selected_pos - visible + 1
        } else {
            0
        };
        (scroll, visible)
    }

    pub(crate) fn content_row_rect(&self, app: &AppState, row_index: usize) -> Option<Rect> {
        if super::content::uses_block_layout(app.settings.section) {
            return super::content::content_row_rect(app, self, row_index);
        }
        let rows = section_rows(app, app.settings.section);
        let row = rows.get(row_index)?;
        if !row_shown_in_scroll_list(app.settings.section, row) {
            return None;
        }
        let indices = scroll_list_row_indices(app, app.settings.section);
        let list_pos = indices.iter().position(|idx| *idx == row_index)?;
        let list_area = self.content_list_area(app);
        let (scroll, visible) = self.visible_row_range(app);
        let visible_idx = list_pos.saturating_sub(scroll);
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
        if app.settings.section == SettingsSection::Appearance
            && (self.spinner_category_index_at(app, col, row).is_some()
                || self.appearance_theme_chip_index_at(app, col, row).is_some())
        {
            return None;
        }

        if super::content::uses_block_layout(app.settings.section) {
            return super::content::content_index_at(app, self, col, row);
        }

        let indices = scroll_list_row_indices(app, app.settings.section);
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
        let list_pos = scroll + visible_idx;
        indices.get(list_pos).copied()
    }
}

pub(crate) fn settings_popup_height(app: &AppState) -> u16 {
    let base = SETTINGS_POPUP_HEIGHT;
    match app.settings.section {
        SettingsSection::Agents => {
            let extra = app.integration_recommendations.len().max(2) as u16;
            base.saturating_add(extra.min(12))
        }
        SettingsSection::Plugins => {
            let installed = super::catalog::installed_plugins_sorted(app).len();
            let catalog = super::catalog::catalog_entries_available(app).len();
            let extra = installed.saturating_add(catalog).saturating_add(2) as u16;
            base.saturating_add(extra.min(14))
        }
        _ => base,
    }
}

pub(crate) fn settings_button_rects(
    layout: &SettingsLayout,
    section: SettingsSection,
    show_tertiary: bool,
) -> SettingsFooterButtons {
    use super::widgets::right_aligned_button_row;

    let mut specs: Vec<(&str, &str)> = Vec::new();
    if show_tertiary {
        let tertiary_hint = match section {
            SettingsSection::Agents => "i",
            _ => "",
        };
        specs.push((tertiary_hint, settings_tertiary_button_label(section)));
    }
    // Enter commits the draft; Space activates the selected row.
    specs.push(("↵", "save"));
    specs.push(("esc", "cancel"));

    let rects = right_aligned_button_row(layout.footer_buttons, &specs, 2);
    let cancel = *rects.last().expect("cancel button rect");
    let save = rects[rects.len() - 2];
    let tertiary = if show_tertiary { Some(rects[0]) } else { None };

    SettingsFooterButtons {
        tertiary,
        save,
        cancel,
    }
}

pub(crate) fn settings_tertiary_button_label(section: SettingsSection) -> &'static str {
    match section {
        SettingsSection::Agents => "install",
        SettingsSection::Plugins => "refresh",
        _ => "apply",
    }
}

#[allow(dead_code)]
pub(crate) fn settings_primary_button_label(section: SettingsSection) -> &'static str {
    settings_tertiary_button_label(section)
}

pub(crate) fn settings_show_tertiary_action(app: &AppState) -> bool {
    match app.settings.section {
        SettingsSection::Agents => app
            .integration_recommendations
            .iter()
            .any(crate::integration::IntegrationRecommendation::needs_install),
        SettingsSection::Plugins => true,
        _ => false,
    }
}

#[allow(dead_code)]
pub(crate) fn settings_show_primary_action(app: &AppState) -> bool {
    settings_show_tertiary_action(app) || app.settings.section == SettingsSection::Appearance
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
        SettingsLayout::compute(Rect::new(0, 0, 160, 50), &app).expect("layout should compute")
    }

    #[test]
    fn settings_popup_fills_most_of_a_large_terminal() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        let area = Rect::new(0, 0, 200, 60);
        let layout = SettingsLayout::compute(area, &app).expect("layout");
        // Much larger than the historical ~100×32 island.
        assert!(
            layout.popup.width >= 180,
            "expected near-fullscreen width, got {}",
            layout.popup.width
        );
        assert!(
            layout.popup.height >= 50,
            "expected near-fullscreen height, got {}",
            layout.popup.height
        );
        assert!(layout.popup.width >= area.width.saturating_mul(90) / 100);
        assert!(layout.popup.height >= area.height.saturating_mul(90) / 100);
        assert!(layout.nav.width >= 24);
        assert!(layout.content.width > layout.nav.width);
    }

    #[test]
    fn settings_popup_still_fits_compact_terminals() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        let area = Rect::new(0, 0, 80, 24);
        let layout = SettingsLayout::compute(area, &app).expect("layout");
        assert!(layout.popup.width <= area.width);
        assert!(layout.popup.height <= area.height);
        assert!(layout.popup.width >= 70);
        assert!(layout.popup.height >= 20);
    }

    #[test]
    fn nav_index_round_trips_item_rect() {
        let layout = layout_for_section(SettingsSection::Layout);
        for idx in 0..SettingsSection::ALL.len() {
            let rect = layout.nav_item_rect(idx).expect("nav item rect");
            assert_eq!(layout.nav_index_at(rect.x + 1, rect.y), Some(idx));
        }
    }

    #[test]
    fn search_rect_matches_search_index_at() {
        let layout = layout_for_section(SettingsSection::Appearance);
        let rect = layout.search_rect();
        assert!(layout.search_index_at(rect.x + 2, rect.y));
        assert!(!layout.search_index_at(rect.x, rect.y.saturating_sub(1)));
    }

    #[test]
    fn content_row_rect_matches_content_index_at_for_layout_toggles() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Layout;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 160, 50), &app).expect("layout");
        let row_rect = layout
            .content_row_rect(&app, 0)
            .expect("first layout row should have geometry");
        assert_eq!(
            layout.content_index_at(&app, row_rect.x + 1, row_rect.y),
            Some(0)
        );
    }

    #[test]
    fn layout_template_rows_are_plain_content_rows() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Layout;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 160, 50), &app).expect("layout");
        let rows = section_rows(&app, SettingsSection::Layout);
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
        app.settings.section = SettingsSection::Appearance;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 160, 50), &app).expect("layout");
        let rect = layout
            .spinner_category_rect(&app)
            .expect("appearance should expose category row");
        assert_eq!(
            layout.spinner_category_index_at(&app, rect.x + 2, rect.y),
            Some(0)
        );
    }

    #[test]
    fn plugins_section_shows_refresh_tertiary_action() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Plugins;
        assert!(settings_show_tertiary_action(&app));
        assert_eq!(
            settings_tertiary_button_label(SettingsSection::Plugins),
            "refresh"
        );
    }

    #[test]
    fn save_cancel_button_rects_align_with_footer_buttons_area() {
        let layout = layout_for_section(SettingsSection::Layout);
        let buttons = settings_button_rects(&layout, SettingsSection::Layout, false);
        assert!(buttons.tertiary.is_none());
        assert!(buttons.save.y >= layout.footer_buttons.y);
        assert!(buttons.cancel.y >= layout.footer_buttons.y);
        assert!(buttons.save.x < buttons.cancel.x);
    }

    #[test]
    fn tertiary_button_rect_sits_left_of_save_cancel() {
        let layout = layout_for_section(SettingsSection::Plugins);
        let buttons = settings_button_rects(&layout, SettingsSection::Plugins, true);
        let tertiary = buttons.tertiary.expect("plugins tertiary");
        assert!(tertiary.x < buttons.save.x);
        assert!(buttons.save.x < buttons.cancel.x);
    }

    #[test]
    fn block_section_content_row_rect_matches_index_at() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Input;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 160, 50), &app).expect("layout");
        let row_rect = layout
            .content_row_rect(&app, 0)
            .expect("first input row should have geometry");
        assert_eq!(
            layout.content_index_at(&app, row_rect.x + 2, row_rect.y),
            Some(0)
        );
    }

    #[test]
    fn chip_row_layout_helpers_match_widget_geometry() {
        let layout = layout_for_section(SettingsSection::Appearance);
        let labels = ["auto", "dark", "light"];
        let y = layout.content.y + 8;
        let rects = layout.chip_row_rect(y, &labels);
        assert_eq!(rects.len(), 3);
        for (idx, rect) in rects.iter().enumerate() {
            assert_eq!(
                layout.chip_row_index_at(y, &labels, rect.x + 1, rect.y),
                Some(idx)
            );
        }
    }

    #[test]
    fn checklist_layout_helpers_match_widget_geometry() {
        let layout = layout_for_section(SettingsSection::Layout);
        let y = layout.content.y + 4;
        let (_header, items) = layout.checklist_group_areas(y, 2);
        let rects = super::super::widgets::checklist_item_rects(items, 2);
        assert_eq!(rects.len(), 2);
        for (idx, rect) in rects.iter().enumerate() {
            assert_eq!(
                layout.checklist_item_index_at(items, 2, rect.x + 2, rect.y),
                Some(idx)
            );
        }
    }

    #[test]
    fn host_cursor_chips_all_have_distinct_hit_geometry() {
        use crate::ui::settings::catalog::SettingsItemId;

        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Input;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 160, 50), &app).expect("layout");
        let rows = section_rows(&app, SettingsSection::Input);
        let host_rows: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter_map(|(idx, row)| {
                matches!(row.id, SettingsItemId::HostCursor { .. }).then_some(idx)
            })
            .collect();
        assert_eq!(host_rows.len(), 3);
        let mut seen = Vec::new();
        for idx in host_rows {
            let rect = layout
                .content_row_rect(&app, idx)
                .expect("host cursor chip rect");
            assert_eq!(layout.content_index_at(&app, rect.x + 1, rect.y), Some(idx));
            assert!(!seen.contains(&(rect.x, rect.y, rect.width)));
            seen.push((rect.x, rect.y, rect.width));
        }
    }

    #[test]
    fn compact_terminal_keeps_content_between_header_and_footer() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Advanced;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 80, 24), &app).expect("layout");
        assert!(layout.content.y > layout.header.y);
        assert!(layout.content.y + layout.content.height <= layout.footer_hints.y);
        assert!(layout.footer_buttons.y >= layout.footer_hints.y);
    }

    #[test]
    fn search_filtered_block_geometry_stays_inside_content() {
        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Input;
        app.settings.search = "mouse".to_string();
        let layout = SettingsLayout::compute(Rect::new(0, 0, 160, 50), &app).expect("layout");
        let rows = section_rows(&app, SettingsSection::Input);
        assert!(!rows.is_empty());
        for idx in 0..rows.len() {
            if let Some(rect) = layout.content_row_rect(&app, idx) {
                assert!(rect.y >= layout.content.y);
                assert!(rect.y < layout.content.y + layout.content.height);
            }
        }
    }

    #[test]
    fn multi_chip_content_index_at_honors_chip_bounds() {
        use crate::ui::settings::catalog::SettingsItemId;

        let mut app = AppState::test_new();
        app.mode = Mode::Settings;
        app.settings.section = SettingsSection::Notifications;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 160, 50), &app).expect("layout");
        let rows = section_rows(&app, SettingsSection::Notifications);
        let toast_rows: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter_map(|(idx, row)| {
                if matches!(row.id, SettingsItemId::ToastDelivery { .. }) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
        assert!(toast_rows.len() >= 3, "expected toast delivery chip rows");

        let second_rect = layout
            .content_row_rect(&app, toast_rows[1])
            .expect("second toast chip rect");
        let third_rect = layout
            .content_row_rect(&app, toast_rows[2])
            .expect("third toast chip rect");

        assert_eq!(
            layout.content_index_at(&app, second_rect.x + 1, second_rect.y),
            Some(toast_rows[1])
        );
        assert_eq!(
            layout.content_index_at(&app, third_rect.x + 1, third_rect.y),
            Some(toast_rows[2])
        );

        let gap_col = second_rect.x.saturating_sub(1);
        if gap_col > layout.content.x {
            assert_ne!(
                layout.content_index_at(&app, gap_col, second_rect.y),
                Some(toast_rows[0])
            );
        }

        app.settings.section = SettingsSection::Terminal;
        let layout = SettingsLayout::compute(Rect::new(0, 0, 160, 50), &app).expect("layout");
        let rows = section_rows(&app, SettingsSection::Terminal);
        let scrollback_rows: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter_map(|(idx, row)| {
                if matches!(row.id, SettingsItemId::ScrollbackPreset { .. }) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
        assert!(scrollback_rows.len() >= 2, "expected scrollback chip rows");

        let later_rect = layout
            .content_row_rect(&app, scrollback_rows[1])
            .expect("second scrollback chip rect");
        assert_eq!(
            layout.content_index_at(&app, later_rect.x + 1, later_rect.y),
            Some(scrollback_rows[1])
        );
    }
}
