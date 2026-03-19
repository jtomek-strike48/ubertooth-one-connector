//! Utility functions for UI rendering

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
};

/// Helper to create a centered rectangle for popups/dialogs
pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Categorize error and provide helpful suggestions
pub(crate) fn categorize_error(error_msg: &str) -> (&'static str, &str, Option<&'static str>) {
    let lower = error_msg.to_lowercase();

    if lower.contains("device") && (lower.contains("not found") || lower.contains("connect")) {
        (
            "Device Connection",
            error_msg,
            Some("Make sure Ubertooth One is plugged in and recognized by the system"),
        )
    } else if lower.contains("permission") || lower.contains("access denied") {
        (
            "Permission Error",
            error_msg,
            Some("Try running with sudo or check USB device permissions"),
        )
    } else if lower.contains("timeout") {
        (
            "Timeout",
            error_msg,
            Some("The operation took too long. Try increasing the duration or checking device connection"),
        )
    } else if lower.contains("not found") && !lower.contains("device") {
        (
            "Resource Not Found",
            error_msg,
            Some("Check that the specified resource (capture, file, etc.) exists"),
        )
    } else if lower.contains("invalid") || lower.contains("parse") {
        (
            "Invalid Input",
            error_msg,
            Some("Check the parameter values and format"),
        )
    } else {
        ("General Error", error_msg, None)
    }
}
