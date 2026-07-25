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
pub fn build_settings_items(_category: SettingsCategory, _app: &crate::app::state::AppState) -> Vec<SettingsItem> {
    Vec::new()
}
