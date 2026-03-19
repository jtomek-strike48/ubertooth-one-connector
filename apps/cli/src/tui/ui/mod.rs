//! UI rendering modules
//!
//! This module organizes the TUI rendering logic into focused sub-modules.

mod core;
mod forms;
mod menu;
mod overlays;
mod utils;

// Re-export main render function
pub use core::render;
