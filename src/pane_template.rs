//! Pane layout templates.
//!
//! Predefined split structures that can be applied to the current tab to
//! quickly arrange panes for common workflows (dual-agent review, monitoring,
//! ops dashboards, etc.).

/// A predefined pane layout that can be applied to the current tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneTemplate {
    pub id: PaneTemplateId,
    pub name: &'static str,
    pub description: &'static str,
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
    /// Large agent pane on top, monitor/logs strip below.
    MonitorBottom,
    /// Agent left, stacked monitor panes on the right.
    MonitorSide,
    /// Three equal columns: agent · status · logs.
    OpsTriple,
    /// Wide main + two stacked side panels (review / preview).
    ReviewDeck,
}

impl PaneTemplateId {
    /// All built-in templates in display order.
    pub const ALL: &[Self] = &[
        Self::Single,
        Self::HorizontalSplit,
        Self::VerticalSplit,
        Self::MainSidebar,
        Self::MonitorBottom,
        Self::MonitorSide,
        Self::OpsTriple,
        Self::ReviewDeck,
        Self::TripleHorizontal,
        Self::Quad,
    ];

    pub const fn template(self) -> PaneTemplate {
        match self {
            Self::Single => PaneTemplate {
                id: self,
                name: "focus",
                description: "one pane, full attention",
            },
            Self::HorizontalSplit => PaneTemplate {
                id: self,
                name: "side by side",
                description: "two agents or agent + tool",
            },
            Self::VerticalSplit => PaneTemplate {
                id: self,
                name: "stacked",
                description: "two panes, one above the other",
            },
            Self::MainSidebar => PaneTemplate {
                id: self,
                name: "main + rail",
                description: "wide work area with a narrow side pane",
            },
            Self::MonitorBottom => PaneTemplate {
                id: self,
                name: "monitor strip",
                description: "agent on top, logs/metrics along the bottom",
            },
            Self::MonitorSide => PaneTemplate {
                id: self,
                name: "monitor column",
                description: "agent left, two stacked monitors on the right",
            },
            Self::OpsTriple => PaneTemplate {
                id: self,
                name: "ops board",
                description: "agent · status · logs — three equal columns",
            },
            Self::ReviewDeck => PaneTemplate {
                id: self,
                name: "review deck",
                description: "wide main plus two stacked side panels",
            },
            Self::TripleHorizontal => PaneTemplate {
                id: self,
                name: "three wide",
                description: "three panes across",
            },
            Self::Quad => PaneTemplate {
                id: self,
                name: "quad",
                description: "four panes in a 2×2 grid",
            },
        }
    }
}
