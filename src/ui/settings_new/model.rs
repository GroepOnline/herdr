//! View model for the rebuilt settings overlay.

/// A settings category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    General,
    Appearance,
    Keybinds,
    Integrations,
    Advanced,
}

impl SettingsCategory {
    pub fn label(self) -> &'static str {
        match self {
            SettingsCategory::General => "General",
            SettingsCategory::Appearance => "Appearance",
            SettingsCategory::Keybinds => "Keybinds",
            SettingsCategory::Integrations => "Integrations",
            SettingsCategory::Advanced => "Advanced",
        }
    }

    /// All categories in presentation order.
    pub fn all() -> [SettingsCategory; 5] {
        [
            SettingsCategory::General,
            SettingsCategory::Appearance,
            SettingsCategory::Keybinds,
            SettingsCategory::Integrations,
            SettingsCategory::Advanced,
        ]
    }
}

/// A single settings row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsItem {
    pub id: String,
    pub label: String,
    pub value: String,
    pub pending: Option<String>,
    pub description: String,
}

/// Build settings items for a category.
pub fn build_settings_items(
    _category: SettingsCategory,
    _app: &crate::app::state::AppState,
) -> Vec<SettingsItem> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_labels() {
        assert_eq!(SettingsCategory::General.label(), "General");
        assert_eq!(SettingsCategory::Appearance.label(), "Appearance");
        assert_eq!(SettingsCategory::Keybinds.label(), "Keybinds");
        assert_eq!(SettingsCategory::Integrations.label(), "Integrations");
        assert_eq!(SettingsCategory::Advanced.label(), "Advanced");
    }

    #[test]
    fn all_categories_has_five_entries() {
        let all = SettingsCategory::all();
        assert_eq!(all.len(), 5);
        // Ensure no duplicates.
        let mut seen = std::collections::HashSet::new();
        for cat in &all {
            assert!(seen.insert(*cat), "duplicate category: {cat:?}");
        }
    }

    #[test]
    fn category_eq_and_clone() {
        let cat = SettingsCategory::Appearance;
        assert_eq!(cat, SettingsCategory::Appearance);
        assert_ne!(cat, SettingsCategory::General);
        let cloned = cat;
        assert_eq!(cat, cloned);
    }

    #[test]
    fn build_settings_items_returns_empty_by_default() {
        let app = crate::app::state::AppState::test_new();
        let items = build_settings_items(SettingsCategory::Appearance, &app);
        assert!(items.is_empty());
        let items = build_settings_items(SettingsCategory::General, &app);
        assert!(items.is_empty());
    }

    #[test]
    fn settings_item_clone_and_eq() {
        let item = SettingsItem {
            id: "test".into(),
            label: "Test".into(),
            value: "val".into(),
            pending: Some("new".into()),
            description: "desc".into(),
        };
        let item2 = item.clone();
        assert_eq!(item, item2);
        assert_eq!(item.id, "test");
        assert_eq!(item.pending, Some("new".to_string()));
    }

    #[test]
    fn settings_item_no_pending() {
        let item = SettingsItem {
            id: "id".into(),
            label: "L".into(),
            value: "v".into(),
            pending: None,
            description: "d".into(),
        };
        assert!(item.pending.is_none());
    }
}
