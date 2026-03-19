//! UI rendering modules
//!
//! This module organizes the TUI rendering logic into focused sub-modules.

mod core;
mod overlays;
mod utils;

// Re-export main render function
pub use core::render;

// Re-export utility functions for internal use
use overlays::{
    render_confirmation, render_dialog, render_help_overlay, render_notification,
    render_theme_selector,
};
use utils::{categorize_error, centered_rect};
