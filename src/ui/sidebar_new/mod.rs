//! Unified sidebar for the new Herdr shell.
//!
//! This module replaces the two fixed "Spaces" and "Agents" regions with one
//! coherent sidebar that can present Workspaces, Agents, or Attention.  It is
//! intentionally additive: the existing `ui::sidebar` implementation remains in
//! place while the new shell grows alongside it.

pub mod layout;
pub mod model;
pub mod render;
