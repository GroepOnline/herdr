//! Pane layout templates.
//!
//! Predefined split structures that can be applied to the current tab to
//! quickly arrange panes for common workflows (dual-agent review, main+sidebar,
//! quad grid, etc.).

/// A predefined pane layout that can be applied to the current tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneTemplate {
    pub id: PaneTemplateId,
    pub name: &'static str,
    pub description: &'static str,
    /// ASCII-art preview shown in the settings picker.
    pub preview: &'static str,
}

/// Identifier for a built-in pane template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneTemplateId {
    /// Single pane, no splits.
    Single,
    /// Two panes side by side.
    HorizontalSplit,
    /// Two panes stacked vertically.
    VerticalSplit,
    /// Four panes in a 2×2 grid.
    Quad,
    /// Three panes side by side.
    TripleHorizontal,
    /// One large pane (70%) + one narrow sidebar (30%).
    MainSidebar,
}

impl PaneTemplateId {
    /// All built-in templates in display order.
    pub const ALL: &[Self] = &[
        Self::Single,
        Self::HorizontalSplit,
        Self::VerticalSplit,
        Self::Quad,
        Self::TripleHorizontal,
        Self::MainSidebar,
    ];

    pub const fn template(self) -> PaneTemplate {
        match self {
            Self::Single => PaneTemplate {
                id: self,
                name: "single",
                description: "one pane, no splits",
                preview: "┌─────────────────┐\n│                 │\n│                 │\n│                 │\n└─────────────────┘",
            },
            Self::HorizontalSplit => PaneTemplate {
                id: self,
                name: "horizontal split",
                description: "two panes side by side",
                preview: "┌────────┬────────┐\n│        │        │\n│        │        │\n│        │        │\n└────────┴────────┘",
            },
            Self::VerticalSplit => PaneTemplate {
                id: self,
                name: "vertical split",
                description: "two panes stacked",
                preview: "┌─────────────────┐\n│                 │\n├─────────────────┤\n│                 │\n└─────────────────┘",
            },
            Self::Quad => PaneTemplate {
                id: self,
                name: "quad",
                description: "four panes in a 2×2 grid",
                preview: "┌────────┬────────┐\n│        │        │\n├────────┼────────┤\n│        │        │\n└────────┴────────┘",
            },
            Self::TripleHorizontal => PaneTemplate {
                id: self,
                name: "triple horizontal",
                description: "three panes side by side",
                preview: "┌──────┬──────┬──────┐\n│      │      │      │\n│      │      │      │\n│      │      │      │\n└──────┴──────┴──────┘",
            },
            Self::MainSidebar => PaneTemplate {
                id: self,
                name: "main + sidebar",
                description: "one large pane + narrow sidebar",
                preview: "┌──────────────┬───┐\n│              │   │\n│    main      │ s │\n│              │   │\n└──────────────┴───┘",
            },
        }
    }
}
