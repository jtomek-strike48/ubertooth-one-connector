//! UI rendering modules
//!
//! This module organizes the TUI rendering logic into focused sub-modules.

mod core;
mod forms;
mod live;
mod menu;
mod overlays;
mod packets;
mod session;
mod utils;

// Re-export main render function
pub use core::render;
