//! Unified sidebar for the new Herdr shell.
//!
//! This module replaces the two fixed "Spaces" and "Agents" regions with one
//! coherent sidebar that can present Workspaces, Agents, or Attention.  It is
//! intentionally additive: the existing `ui::sidebar` implementation remains in
//! place while the new shell grows alongside it.

pub mod layout;
pub mod model;
pub mod render;

pub use layout::{layout_sidebar, row_at};
pub use model::{SidebarItem, SidebarItemId, SidebarModel, SidebarRowKind, build_agents, build_attention, build_workspaces};
pub use render::render_sidebar_new;
