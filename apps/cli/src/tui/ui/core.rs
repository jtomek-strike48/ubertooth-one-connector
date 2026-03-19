//! Core UI rendering logic

use super::forms::{render_export_menu, render_filter_dialog, render_tool_form};
use super::live::{
    render_live_capture, render_live_capture_header, render_live_packet_list,
    render_live_statistics,
};
use super::menu::{render_main_menu, render_tool_category, render_tool_hotkeys};
use super::overlays::{
    render_confirmation, render_dialog, render_help_overlay, render_notification,
    render_theme_selector,
};
use super::packets::comparison::render_packet_comparison;
use super::packets::decoded::render_decoded_packets;
use super::packets::list::render_packet_list;
use super::packets::statistics::render_packet_statistics;
use super::packets::timeline::render_packet_timeline;
use super::session::{
    render_session_list, render_session_loading, render_session_manager, render_session_save,
};
use super::utils::{categorize_error, centered_rect};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::sync::Arc;
use ubertooth_core::ToolRegistry;

use crate::tui::app::{AppState, DeviceStatus, Notification, TextInputDialog};
use crate::tui::themes::Theme;
use crate::tui::views::{Category, FieldInputMode, FieldType};

/// Render the entire UI
pub fn render(
    f: &mut Frame,
    state: &AppState,
    registry: &Arc<ToolRegistry>,
    device_status: &DeviceStatus,
    notification: &Option<Notification>,
    frame_count: u64,
    dialog: &Option<TextInputDialog>,
    theme: &Theme,
) {
    // Main layout: header + content + footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(f.size());

    render_header(f, chunks[0], device_status, theme);
    render_content(
        f,
        chunks[1],
        state,
        registry,
        device_status,
        frame_count,
        theme,
    );
    render_footer(f, chunks[2], state);

    // Render notification on top if present
    if let Some(notif) = notification {
        render_notification(f, f.size(), notif);
    }

    // Render dialog overlay on top if present
    if let Some(dlg) = dialog {
        render_dialog(f, f.size(), dlg);
    }
}

/// Render header with device status
fn render_header(f: &mut Frame, area: Rect, device_status: &DeviceStatus, theme: &Theme) {
    // Build status string
    let device_str = if device_status.connected {
        if let Some(fw) = &device_status.firmware {
            format!("Device: Connected ({})", fw)
        } else {
            "Device: Connected".to_string()
        }
    } else {
        "Device: Not Connected".to_string()
    };

    let backend_str = "Backend: Python";
    let strike48_str = "Strike48: Not Connected";

    let title = format!("{} | {} | {}", device_str, backend_str, strike48_str);

    let header = Paragraph::new("Ubertooth CLI")
        .style(
            Style::default()
                .fg(theme.colors.title.to_color())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(header, area);
}

/// Render main content based on state
fn render_content(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    registry: &Arc<ToolRegistry>,
    device_status: &DeviceStatus,
    frame_count: u64,
    theme: &Theme,
) {
    match state {
        AppState::MainMenu { selected_index } => {
            render_main_menu(f, area, *selected_index, device_status);
        }
        AppState::ToolCategory {
            category,
            selected_index,
        } => {
            render_tool_category(f, area, category, *selected_index, registry, device_status);
        }
        AppState::ToolForm {
            form,
            error,
            hotkey_mode,
        } => {
            if *hotkey_mode {
                render_tool_hotkeys(f, area, form.as_ref(), error.as_deref());
            } else {
                render_tool_form(f, area, form.as_ref(), error.as_deref());
            }
        }
        AppState::Executing { tool_name, .. } => {
            render_executing(f, area, tool_name, frame_count);
        }
        AppState::Results {
            tool_name,
            output,
            success,
            selected_capture,
            packet_list_state,
            analysis_view_state,
            ..
        } => {
            render_results(
                f,
                area,
                tool_name,
                output,
                *success,
                *selected_capture,
                packet_list_state.as_ref(),
                analysis_view_state.as_ref(),
            );
        }
        AppState::Settings { selected_index } => {
            render_settings(f, area, *selected_index);
        }
        AppState::Confirmation { message, .. } => {
            render_confirmation(f, area, message);
        }
        AppState::ExportMenu {
            selected_index,
            packets,
            packet_list_state,
            ..
        } => {
            render_export_menu(f, area, *selected_index, packets.len(), packet_list_state);
        }
        AppState::FilterDialog {
            selected_section,
            selected_packet_type,
            packet_type_selections,
            mac_filter,
            rssi_min,
            rssi_max,
            ..
        } => {
            render_filter_dialog(
                f,
                area,
                *selected_section,
                *selected_packet_type,
                packet_type_selections,
                mac_filter,
                rssi_min,
                rssi_max,
            );
        }
        AppState::HelpOverlay { scroll_offset, .. } => {
            render_help_overlay(f, area, *scroll_offset, theme);
        }
        AppState::ThemeSelector {
            selected_index,
            themes,
        } => {
            render_theme_selector(f, area, *selected_index, themes, theme);
        }
        AppState::LiveCapture {
            buffer,
            stats,
            paused,
            limits,
            selected_index,
            scroll_offset,
            tool_name,
        } => {
            render_live_capture(
                f,
                area,
                buffer,
                stats,
                *paused,
                limits,
                *selected_index,
                *scroll_offset,
                tool_name,
            );
        }
        AppState::SessionManager {
            selected_index,
            sessions,
            mode,
            session_name,
        } => {
            render_session_manager(f, area, *selected_index, sessions, mode, session_name);
        }
    }
}

/// Render analysis results in readable format
fn render_analysis_results(
    f: &mut Frame,
    area: Rect,
    output: &serde_json::Value,
    state: Option<&crate::tui::app::AnalysisViewState>,
) {
    use crate::tui::app::AnalysisViewMode;

    // Get state or use default
    let default_state = crate::tui::app::AnalysisViewState::new();
    let state = state.unwrap_or(&default_state);

    // Dispatch based on view mode
    match state.view_mode {
        AnalysisViewMode::Overview => render_analysis_overview(f, area, output),
        AnalysisViewMode::Devices => render_analysis_devices(f, area, output, state),
        AnalysisViewMode::Security => render_analysis_security(f, area, output, state),
        AnalysisViewMode::Timing => render_analysis_timing(f, area, output),
    }
}

/// Render capture comparison results
fn render_comparison_results(f: &mut Frame, area: Rect, output: &serde_json::Value) {
    use ratatui::layout::{Constraint, Direction, Layout};

    if let Some(comparison) = output.get("comparison").and_then(|c| c.as_object()) {
        // Split into left and right panels
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let mut lines_left = Vec::new();
        let mut lines_right = Vec::new();

        // Header
        lines_left.push(Line::from(Span::styled(
            "Capture A",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines_right.push(Line::from(Span::styled(
            "Capture B",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

        lines_left.push(Line::from(""));
        lines_right.push(Line::from(""));

        // Get capture IDs from output root
        let capture_a = output
            .get("capture_id_a")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");
        let capture_b = output
            .get("capture_id_b")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");

        lines_left.push(Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::Gray)),
            Span::styled(capture_a, Style::default().fg(Color::White)),
        ]));
        lines_right.push(Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::Gray)),
            Span::styled(capture_b, Style::default().fg(Color::White)),
        ]));

        lines_left.push(Line::from(""));
        lines_right.push(Line::from(""));

        // Similarity score (in both panels for emphasis)
        if let Some(similarity) = comparison
            .get("similarity_percent")
            .and_then(|s| s.as_f64())
        {
            let similarity_color = if similarity > 80.0 {
                Color::Green
            } else if similarity > 50.0 {
                Color::Yellow
            } else {
                Color::Red
            };

            lines_left.push(Line::from(Span::styled(
                format!("Similarity: {:.1}%", similarity),
                Style::default()
                    .fg(similarity_color)
                    .add_modifier(Modifier::BOLD),
            )));
            lines_right.push(Line::from(Span::styled(
                format!("Similarity: {:.1}%", similarity),
                Style::default()
                    .fg(similarity_color)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        lines_left.push(Line::from(""));
        lines_right.push(Line::from(""));

        // Statistics
        if let Some(common) = comparison.get("common_packets").and_then(|c| c.as_u64()) {
            lines_left.push(Line::from(vec![
                Span::styled("Common packets: ", Style::default().fg(Color::Gray)),
                Span::styled(common.to_string(), Style::default().fg(Color::Green)),
            ]));
            lines_right.push(Line::from(vec![
                Span::styled("Common packets: ", Style::default().fg(Color::Gray)),
                Span::styled(common.to_string(), Style::default().fg(Color::Green)),
            ]));
        }

        if let Some(unique_a) = comparison.get("unique_to_a").and_then(|u| u.as_u64()) {
            lines_left.push(Line::from(vec![
                Span::styled("Unique packets: ", Style::default().fg(Color::Gray)),
                Span::styled(unique_a.to_string(), Style::default().fg(Color::Yellow)),
            ]));
        }

        if let Some(unique_b) = comparison.get("unique_to_b").and_then(|u| u.as_u64()) {
            lines_right.push(Line::from(vec![
                Span::styled("Unique packets: ", Style::default().fg(Color::Gray)),
                Span::styled(unique_b.to_string(), Style::default().fg(Color::Yellow)),
            ]));
        }

        lines_left.push(Line::from(""));
        lines_right.push(Line::from(""));

        // Differences
        if let Some(diffs) = comparison.get("differences").and_then(|d| d.as_array()) {
            if !diffs.is_empty() {
                lines_left.push(Line::from(Span::styled(
                    "Differences Detected",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
                lines_right.push(Line::from(Span::styled(
                    "Differences Detected",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));

                for diff in diffs.iter().take(10) {
                    // Show first 10 differences
                    if let Some(diff_str) = diff.as_str() {
                        lines_left.push(Line::from(vec![
                            Span::styled("• ", Style::default().fg(Color::Red)),
                            Span::styled(diff_str, Style::default().fg(Color::White)),
                        ]));
                        lines_right.push(Line::from(""));
                    } else if let Some(diff_obj) = diff.as_object() {
                        // Structured difference
                        let field = diff_obj
                            .get("field")
                            .and_then(|f| f.as_str())
                            .unwrap_or("unknown");
                        let val_a = diff_obj
                            .get("value_a")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A");
                        let val_b = diff_obj
                            .get("value_b")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A");

                        lines_left.push(Line::from(vec![
                            Span::styled(format!("{}: ", field), Style::default().fg(Color::Gray)),
                            Span::styled(val_a, Style::default().fg(Color::Yellow)),
                        ]));
                        lines_right.push(Line::from(vec![
                            Span::styled(format!("{}: ", field), Style::default().fg(Color::Gray)),
                            Span::styled(val_b, Style::default().fg(Color::Yellow)),
                        ]));
                    }
                }

                if diffs.len() > 10 {
                    lines_left.push(Line::from(Span::styled(
                        format!("... and {} more differences", diffs.len() - 10),
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    )));
                }
            } else {
                lines_left.push(Line::from(Span::styled(
                    "✓ No significant differences",
                    Style::default().fg(Color::Green),
                )));
                lines_right.push(Line::from(Span::styled(
                    "✓ No significant differences",
                    Style::default().fg(Color::Green),
                )));
            }
        }

        // Render left panel (Capture A)
        let left_content = Paragraph::new(Text::from(lines_left))
            .block(Block::default().borders(Borders::ALL).title(" Capture A "))
            .wrap(Wrap { trim: false });
        f.render_widget(left_content, chunks[0]);

        // Render right panel (Capture B)
        let right_content = Paragraph::new(Text::from(lines_right))
            .block(Block::default().borders(Borders::ALL).title(" Capture B "))
            .wrap(Wrap { trim: false });
        f.render_widget(right_content, chunks[1]);
    } else {
        // Fallback if comparison object is missing
        let error_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Error: Invalid comparison result",
                Style::default().fg(Color::Red),
            )),
        ];
        let content = Paragraph::new(Text::from(error_text)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Comparison Error "),
        );
        f.render_widget(content, area);
    }
}

/// Render analysis overview (summary of all sections)
fn render_analysis_overview(f: &mut Frame, area: Rect, output: &serde_json::Value) {
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    if let Some(analysis) = output.get("analysis").and_then(|a| a.as_object()) {
        // Protocol Summary
        if let Some(proto) = analysis.get("protocol_summary").and_then(|p| p.as_object()) {
            lines.push(Line::from(Span::styled(
                "Protocol Summary",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));

            if let Some(ptype) = proto.get("type").and_then(|t| t.as_str()) {
                lines.push(Line::from(vec![
                    Span::raw("  Type: "),
                    Span::styled(ptype, Style::default().fg(Color::Cyan)),
                ]));
            }
            if let Some(count) = proto.get("packet_count").and_then(|c| c.as_u64()) {
                lines.push(Line::from(vec![
                    Span::raw("  Packets: "),
                    Span::styled(count.to_string(), Style::default().fg(Color::Green)),
                ]));
            }
            if let Some(devices) = proto.get("unique_devices").and_then(|d| d.as_u64()) {
                lines.push(Line::from(vec![
                    Span::raw("  Devices: "),
                    Span::styled(devices.to_string(), Style::default().fg(Color::Cyan)),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Quick stats
        if let Some(devices) = analysis.get("devices").and_then(|d| d.as_array()) {
            lines.push(Line::from(vec![
                Span::styled("📱 Devices: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{} found", devices.len()),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  "),
                Span::styled(
                    "(press 'd' for details)",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]));
        }

        if let Some(security) = analysis
            .get("security_observations")
            .and_then(|s| s.as_array())
        {
            if !security.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("🔒 Security: ", Style::default().fg(Color::Red)),
                    Span::styled(
                        format!("{} observations", security.len()),
                        Style::default().fg(Color::White),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        "(press 's' for details)",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
        }

        if let Some(timing) = analysis.get("timing_analysis").and_then(|t| t.as_object()) {
            if let Some(avg) = timing.get("avg_interval_ms").and_then(|a| a.as_f64()) {
                if avg > 0.0 {
                    lines.push(Line::from(vec![
                        Span::styled("⏱️  Timing: ", Style::default().fg(Color::Blue)),
                        Span::styled(
                            format!("{:.2}ms avg", avg),
                            Style::default().fg(Color::White),
                        ),
                        Span::raw("  "),
                        Span::styled(
                            "(press 't' for details)",
                            Style::default()
                                .fg(Color::Gray)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ]));
                }
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));

        // Navigation hint
        lines.push(Line::from(vec![
            Span::styled("Navigation: ", Style::default().fg(Color::Yellow)),
            Span::styled("o", Style::default().fg(Color::Cyan)),
            Span::raw(" overview  "),
            Span::styled("d", Style::default().fg(Color::Cyan)),
            Span::raw(" devices  "),
            Span::styled("s", Style::default().fg(Color::Cyan)),
            Span::raw(" security  "),
            Span::styled("t", Style::default().fg(Color::Cyan)),
            Span::raw(" timing"),
        ]));
    }

    let content = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Analysis Overview "),
    );
    f.render_widget(content, area);
}

/// Render devices view with interactive list
fn render_analysis_devices(
    f: &mut Frame,
    area: Rect,
    output: &serde_json::Value,
    state: &crate::tui::app::AnalysisViewState,
) {
    let mut lines = Vec::new();

    if let Some(analysis) = output.get("analysis").and_then(|a| a.as_object()) {
        if let Some(devices) = analysis.get("devices").and_then(|d| d.as_array()) {
            if devices.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "No devices detected in this capture",
                    Style::default().fg(Color::Gray),
                )));
            } else {
                for (idx, device) in devices.iter().enumerate() {
                    let is_selected = idx == state.selected_index;
                    let is_expanded = state.is_expanded(idx);

                    let expand_icon = if is_expanded { "▼" } else { "▶" };
                    let mac = device
                        .get("mac_address")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown");
                    let name = device
                        .get("device_name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Unknown");
                    let pkts = device
                        .get("packet_count")
                        .and_then(|p| p.as_u64())
                        .unwrap_or(0);

                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    lines.push(Line::from(vec![
                        Span::styled(expand_icon, style),
                        Span::raw(" "),
                        Span::styled(mac, style.fg(Color::Cyan)),
                        Span::raw(" - "),
                        Span::styled(name, style),
                        Span::styled(
                            format!(" ({} pkts)", pkts),
                            Style::default().fg(Color::Gray),
                        ),
                    ]));

                    if is_expanded {
                        // Show device details
                        if let Some(rssi) = device.get("rssi").and_then(|r| r.as_i64()) {
                            lines.push(Line::from(vec![
                                Span::raw("  │ "),
                                Span::styled("RSSI: ", Style::default().fg(Color::Gray)),
                                Span::styled(
                                    format!("{} dBm", rssi),
                                    Style::default().fg(Color::White),
                                ),
                            ]));
                        }
                        if let Some(pdu) = device.get("pdu_type").and_then(|p| p.as_str()) {
                            lines.push(Line::from(vec![
                                Span::raw("  │ "),
                                Span::styled("PDU Type: ", Style::default().fg(Color::Gray)),
                                Span::styled(pdu, Style::default().fg(Color::White)),
                            ]));
                        }
                        if let Some(first) = device.get("first_seen").and_then(|f| f.as_f64()) {
                            if let Some(last) = device.get("last_seen").and_then(|l| l.as_f64()) {
                                let duration = last - first;
                                lines.push(Line::from(vec![
                                    Span::raw("  │ "),
                                    Span::styled(
                                        "Active Duration: ",
                                        Style::default().fg(Color::Gray),
                                    ),
                                    Span::styled(
                                        format!("{:.2}s", duration),
                                        Style::default().fg(Color::White),
                                    ),
                                ]));
                            }
                        }
                        lines.push(Line::from("  │"));
                    }
                }

                // Footer
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
                    Span::raw(" navigate  "),
                    Span::styled("Enter", Style::default().fg(Color::Cyan)),
                    Span::raw(" expand/collapse  "),
                    Span::styled("o/d/s/t", Style::default().fg(Color::Cyan)),
                    Span::raw(" change view"),
                ]));
            }
        }
    }

    let content =
        Paragraph::new(Text::from(lines)).block(Block::default().borders(Borders::ALL).title(
            format!(" Devices ({} total) ",
            output.get("analysis")
                .and_then(|a| a.get("devices"))
                .and_then(|d| d.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0)
        ),
        ));
    f.render_widget(content, area);
}

/// Render security observations view
fn render_analysis_security(
    f: &mut Frame,
    area: Rect,
    output: &serde_json::Value,
    state: &crate::tui::app::AnalysisViewState,
) {
    let mut lines = Vec::new();

    if let Some(analysis) = output.get("analysis").and_then(|a| a.as_object()) {
        // Security summary
        if let Some(summary) = analysis.get("security_summary").and_then(|s| s.as_object()) {
            lines.push(Line::from(Span::styled(
                "Security Summary",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));

            if let Some(privacy) = summary
                .get("privacy_enabled_devices")
                .and_then(|p| p.as_u64())
            {
                lines.push(Line::from(vec![
                    Span::raw("  Privacy-enabled devices: "),
                    Span::styled(privacy.to_string(), Style::default().fg(Color::Green)),
                ]));
            }
            if let Some(public) = summary
                .get("public_address_devices")
                .and_then(|p| p.as_u64())
            {
                lines.push(Line::from(vec![
                    Span::raw("  Public address devices: "),
                    Span::styled(public.to_string(), Style::default().fg(Color::Yellow)),
                ]));
            }
            if let Some(conn) = summary.get("connection_requests").and_then(|c| c.as_u64()) {
                lines.push(Line::from(vec![
                    Span::raw("  Connection requests: "),
                    Span::styled(conn.to_string(), Style::default().fg(Color::Cyan)),
                ]));
            }
            if let Some(scan) = summary.get("scan_requests").and_then(|s| s.as_u64()) {
                lines.push(Line::from(vec![
                    Span::raw("  Scan requests: "),
                    Span::styled(scan.to_string(), Style::default().fg(Color::Cyan)),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Detailed observations
        if let Some(observations) = analysis
            .get("security_observations")
            .and_then(|s| s.as_array())
        {
            if observations.is_empty() {
                lines.push(Line::from(Span::styled(
                    "No security observations",
                    Style::default().fg(Color::Green),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "Observations",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));

                for (idx, obs) in observations.iter().enumerate() {
                    let is_selected = idx == state.selected_index;
                    let is_expanded = state.is_expanded(idx);

                    let expand_icon = if is_expanded { "▼" } else { "▶" };

                    // Handle both string and object observations
                    let (obs_type, description, severity) = if let Some(obj) = obs.as_object() {
                        (
                            obj.get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("Unknown"),
                            obj.get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or(""),
                            obj.get("severity")
                                .and_then(|s| s.as_str())
                                .unwrap_or("info"),
                        )
                    } else if let Some(text) = obs.as_str() {
                        ("Observation", text, "info")
                    } else {
                        ("Unknown", "", "info")
                    };

                    let severity_color = match severity {
                        "critical" => Color::Red,
                        "high" => Color::LightRed,
                        "medium" => Color::Yellow,
                        "low" => Color::Blue,
                        _ => Color::Gray,
                    };

                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    lines.push(Line::from(vec![
                        Span::styled(expand_icon, style),
                        Span::raw(" "),
                        Span::styled(
                            format!("[{}] ", severity.to_uppercase()),
                            Style::default()
                                .fg(severity_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(obs_type, style),
                    ]));

                    if is_expanded {
                        lines.push(Line::from(vec![
                            Span::raw("  │ "),
                            Span::styled(description, Style::default().fg(Color::White)),
                        ]));
                        if let Some(obj) = obs.as_object() {
                            if let Some(device) =
                                obj.get("affected_device").and_then(|d| d.as_str())
                            {
                                lines.push(Line::from(vec![
                                    Span::raw("  │ "),
                                    Span::styled("Affected: ", Style::default().fg(Color::Gray)),
                                    Span::styled(device, Style::default().fg(Color::Cyan)),
                                ]));
                            }
                        }
                        lines.push(Line::from("  │"));
                    }
                }

                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
                    Span::raw(" navigate  "),
                    Span::styled("Enter", Style::default().fg(Color::Cyan)),
                    Span::raw(" expand/collapse"),
                ]));
            }
        }
    }

    let content = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Security Analysis "),
    );
    f.render_widget(content, area);
}

/// Render timing analysis view
fn render_analysis_timing(f: &mut Frame, area: Rect, output: &serde_json::Value) {
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    if let Some(analysis) = output.get("analysis").and_then(|a| a.as_object()) {
        if let Some(timing) = analysis.get("timing_analysis").and_then(|t| t.as_object()) {
            lines.push(Line::from(Span::styled(
                "Timing Analysis",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            if let Some(duration) = timing.get("duration_sec").and_then(|d| d.as_f64()) {
                lines.push(Line::from(vec![
                    Span::styled("Capture Duration: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{:.2}s", duration),
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            if let Some(pps) = timing.get("packets_per_sec").and_then(|p| p.as_f64()) {
                lines.push(Line::from(vec![
                    Span::styled("Packets per Second: ", Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{:.2}", pps), Style::default().fg(Color::White)),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Inter-Packet Intervals",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));

            if let Some(avg) = timing.get("avg_interval_ms").and_then(|a| a.as_f64()) {
                lines.push(Line::from(vec![
                    Span::styled("  Average: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:.2}ms", avg), Style::default().fg(Color::Green)),
                ]));
            }

            if let Some(min) = timing.get("min_interval_ms").and_then(|m| m.as_f64()) {
                lines.push(Line::from(vec![
                    Span::styled("  Minimum: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:.2}ms", min), Style::default().fg(Color::Blue)),
                ]));
            }

            if let Some(max) = timing.get("max_interval_ms").and_then(|m| m.as_f64()) {
                lines.push(Line::from(vec![
                    Span::styled("  Maximum: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:.2}ms", max), Style::default().fg(Color::Red)),
                ]));
            }

            if let Some(count) = timing.get("intervals_calculated").and_then(|c| c.as_u64()) {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Intervals analyzed: ", Style::default().fg(Color::Gray)),
                    Span::styled(count.to_string(), Style::default().fg(Color::White)),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("o/d/s/t", Style::default().fg(Color::Cyan)),
                Span::styled(" to change view", Style::default().fg(Color::Gray)),
            ]));
        }
    }

    let content = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Timing Analysis "),
    );
    f.render_widget(content, area);
}

/// Render capture details in readable format
fn render_capture_details(f: &mut Frame, area: Rect, output: &serde_json::Value) {
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    if let Some(obj) = output.as_object() {
        // Capture ID
        if let Some(id) = obj.get("capture_id").and_then(|v| v.as_str()) {
            lines.push(Line::from(vec![
                Span::styled(
                    "  Capture ID: ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(id, Style::default().fg(Color::Cyan)),
            ]));
        }

        // Type
        if let Some(cap_type) = obj.get("type").and_then(|v| v.as_str()) {
            lines.push(Line::from(vec![
                Span::raw("  Type: "),
                Span::styled(cap_type, Style::default().fg(Color::White)),
            ]));
        }

        // Timestamp
        if let Some(timestamp) = obj.get("timestamp").and_then(|v| v.as_str()) {
            lines.push(Line::from(vec![
                Span::raw("  Timestamp: "),
                Span::styled(timestamp, Style::default().fg(Color::Gray)),
            ]));
        }
        lines.push(Line::from(""));

        // Stats
        lines.push(Line::from(Span::styled(
            "  Statistics",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));

        if let Some(packets) = obj.get("packet_count").and_then(|v| v.as_u64()) {
            lines.push(Line::from(vec![
                Span::raw("    Packets: "),
                Span::styled(packets.to_string(), Style::default().fg(Color::Green)),
            ]));
        }

        if let Some(duration) = obj.get("duration_sec").and_then(|v| v.as_u64()) {
            lines.push(Line::from(vec![
                Span::raw("    Duration: "),
                Span::styled(format!("{}s", duration), Style::default().fg(Color::Blue)),
            ]));
        }

        if let Some(size) = obj.get("file_size_bytes").and_then(|v| v.as_u64()) {
            let size_kb = size as f64 / 1024.0;
            lines.push(Line::from(vec![
                Span::raw("    File Size: "),
                Span::styled(
                    format!("{:.2} KB", size_kb),
                    Style::default().fg(Color::Magenta),
                ),
            ]));
        }
        lines.push(Line::from(""));

        // Tags
        if let Some(tags) = obj.get("tags").and_then(|v| v.as_array()) {
            if !tags.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Tags",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                let tag_str = tags
                    .iter()
                    .filter_map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                lines.push(Line::from(format!("    {}", tag_str)));
                lines.push(Line::from(""));
            }
        }

        // Description
        if let Some(desc) = obj.get("description").and_then(|v| v.as_str()) {
            if !desc.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Description",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(format!("    {}", desc)));
                lines.push(Line::from(""));
            }
        }

        // PCAP Path
        if let Some(path) = obj.get("pcap_path").and_then(|v| v.as_str()) {
            lines.push(Line::from(vec![
                Span::styled("  PCAP File: ", Style::default().fg(Color::Gray)),
                Span::styled(path, Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let content = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Capture Details"),
    );
    f.render_widget(content, area);
}

/// Render capture list as a formatted table
fn render_capture_list_table(
    f: &mut Frame,
    area: Rect,
    captures: &[serde_json::Value],
    selected_index: Option<usize>,
) {
    let mut lines = Vec::new();

    // Header line
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  Found {} capture(s)", captures.len()),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));

    // Show navigation hint if captures available
    if !captures.is_empty() {
        lines.push(Line::from(Span::styled(
            "  [↑/↓] Navigate  [Enter] Analyze  [V] View  [D] Delete  [E] Export  [T] Tag  [Esc] Back",
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines.push(Line::from(""));

    // If no captures, show message
    if captures.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No captures found. Run a scan to create captures.",
            Style::default().fg(Color::Gray),
        )));
    } else {
        // Table header
        lines.push(Line::from(vec![
            Span::styled(
                "  ID                  ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Type           ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Pkts    ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Duration    ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Timestamp              ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Description",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(Span::raw(format!("  {}", "-".repeat(120)))));

        // Table rows
        for (idx, capture) in captures.iter().enumerate() {
            let is_selected = selected_index == Some(idx);
            let id = capture
                .get("capture_id")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .split('-')
                .nth(2)
                .unwrap_or("?")
                .chars()
                .take(16)
                .collect::<String>();

            let cap_type = capture.get("type").and_then(|v| v.as_str()).unwrap_or("?");

            let packet_count = capture
                .get("packet_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let duration = capture
                .get("duration_sec")
                .and_then(|v| v.as_u64())
                .map(|d| format!("{}s", d))
                .unwrap_or_else(|| "N/A".to_string());

            let timestamp = capture
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .chars()
                .take(19)
                .collect::<String>();

            let description = capture
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let tags = capture
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();

            // Main row - highlight if selected
            let row_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let prefix = if is_selected { "> " } else { "  " };

            lines.push(Line::from(vec![
                Span::styled(format!("{}{:<18}  ", prefix, id), row_style.fg(Color::Cyan)),
                Span::styled(format!("{:<13}  ", cap_type), row_style.fg(Color::White)),
                Span::styled(format!("{:<6}  ", packet_count), row_style.fg(Color::Green)),
                Span::styled(format!("{:<10}  ", duration), row_style.fg(Color::Blue)),
                Span::styled(format!("{:<21}  ", timestamp), row_style.fg(Color::Gray)),
                Span::styled(description, row_style.fg(Color::White)),
            ]));

            // Tags row if present
            if !tags.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("Tags: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(tags, Style::default().fg(Color::DarkGray)),
                ]));
            }

            lines.push(Line::from(""));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Esc] Back to menu",
        Style::default().fg(Color::DarkGray),
    )));

    let content = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Capture List")
            .title_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(content, area);
}

/// Render tool execution progress
fn render_executing(f: &mut Frame, area: Rect, tool_name: &str, frame_count: u64) {
    // Animated spinner frames
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = spinner_frames[(frame_count / 2) as usize % spinner_frames.len()];

    let text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(spinner, Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled("Executing: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                tool_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Please wait...",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This may take a few seconds depending on the tool.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Executing"));

    f.render_widget(paragraph, area);
}

/// Render tool results
fn render_results(
    f: &mut Frame,
    area: Rect,
    tool_name: &str,
    output: &serde_json::Value,
    success: bool,
    selected_capture: Option<usize>,
    packet_list_state: Option<&crate::tui::app::PacketListState>,
    analysis_view_state: Option<&crate::tui::app::AnalysisViewState>,
) {
    // Split into header and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Header with status
    let status_symbol = if success { "[OK]" } else { "[FAILED]" };
    let status_text = if success { "Success" } else { "Failed" };
    let status_color = if success { Color::Green } else { Color::Red };

    let header_text = format!("{} {}\n\nTool: {}", status_symbol, status_text, tool_name);
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Execution Result"),
        );
    f.render_widget(header, chunks[0]);

    // Special formatting for capture_list - show as table
    if tool_name == "capture_list" && success {
        if let Some(captures_array) = output.get("captures").and_then(|v| v.as_array()) {
            render_capture_list_table(f, chunks[1], captures_array, selected_capture);
            return;
        }
    }

    // Special formatting for bt_compare - show comparison view
    if tool_name == "bt_compare" && success {
        render_comparison_results(f, chunks[1], output);
        return;
    }

    // Special formatting for bt_analyze - show analysis results
    if tool_name == "bt_analyze" && success {
        render_analysis_results(f, chunks[1], output, analysis_view_state);
        return;
    }

    // Special formatting for bt_decode - show packet list
    if tool_name == "bt_decode" && success {
        render_decoded_packets(f, chunks[1], output, packet_list_state);
        return;
    }

    // Special formatting for capture_get - show capture details
    if tool_name == "capture_get" && success {
        render_capture_details(f, chunks[1], output);
        return;
    }

    // Special formatting for errors - show clean error message
    if !success {
        if let Some(error_msg) = output.get("error").and_then(|v| v.as_str()) {
            render_error_message(f, chunks[1], tool_name, error_msg);
            return;
        }
    }

    // Content - format JSON nicely
    let result_json = serde_json::to_string_pretty(output).unwrap_or_else(|_| "{}".to_string());

    // Highlight specific fields for better readability
    let formatted_output = if let Some(obj) = output.as_object() {
        let mut lines = Vec::new();

        // Show important fields first
        if let Some(capture_id) = obj.get("capture_id").and_then(|v| v.as_str()) {
            lines.push(Line::from(vec![
                Span::styled("Capture ID: ", Style::default().fg(Color::Cyan)),
                Span::styled(capture_id, Style::default().fg(Color::White)),
            ]));
        }

        if let Some(packets) = obj.get("packets_captured").and_then(|v| v.as_u64()) {
            lines.push(Line::from(vec![
                Span::styled("Packets: ", Style::default().fg(Color::Cyan)),
                Span::styled(packets.to_string(), Style::default().fg(Color::White)),
            ]));
        }

        if let Some(devices) = obj.get("devices_found").and_then(|v| v.as_u64()) {
            lines.push(Line::from(vec![
                Span::styled("Devices: ", Style::default().fg(Color::Cyan)),
                Span::styled(devices.to_string(), Style::default().fg(Color::White)),
            ]));
        }

        if let Some(duration) = obj.get("duration").and_then(|v| v.as_f64()) {
            lines.push(Line::from(vec![
                Span::styled("Duration: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:.1}s", duration),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Full Output:",
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));

        // Add full JSON
        for line in result_json.lines() {
            lines.push(Line::from(line.to_string()));
        }

        Text::from(lines)
    } else {
        Text::from(result_json)
    };

    let content = Paragraph::new(formatted_output).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Output")
            .title_style(Style::default().fg(Color::Gray)),
    );

    f.render_widget(content, chunks[1]);
}

/// Render settings page
fn render_settings(f: &mut Frame, area: Rect, selected_index: usize) {
    let settings_items = vec![
        ("View Tool History", "Show recently used tools"),
        ("View Favorites", "Show bookmarked tools"),
        (
            "View Recent MAC Addresses",
            "MAC filter helper for analysis",
        ),
        ("Backend Info", "View backend configuration"),
        ("Strike48 Connection", "Configure cloud connection"),
        ("About", "Version and system information"),
    ];

    let items: Vec<ListItem> = settings_items
        .iter()
        .enumerate()
        .map(|(i, (title, desc))| {
            let style = if i == selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = vec![
                Line::from(Span::styled(format!("{}. {}", i + 1, title), style)),
                Line::from(Span::styled(
                    format!("   {}", desc),
                    Style::default().fg(Color::Gray),
                )),
                Line::from(""),
            ];

            ListItem::new(Text::from(content))
        })
        .collect();

    let settings_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Settings")
            .title_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(settings_list, area);
}

/// Render footer with keyboard shortcuts
fn render_footer(f: &mut Frame, area: Rect, state: &AppState) {
    let shortcuts = match state {
        AppState::MainMenu { .. } => {
            "[0-6] Quick Select  [↑/↓] Navigate  [Enter] Select  [?] Help  [q] Quit"
        }
        AppState::ToolCategory { category, .. } => {
            if matches!(category, Category::DeviceManagement) {
                "[1-9] Quick Select  [→] Device Status  [Enter] Select  [?] Help  [Esc] Back"
            } else {
                "[1-9] Quick Select  [↑/↓] Navigate  [Enter] Select  [?] Help  [Esc] Back"
            }
        }
        AppState::ToolForm { hotkey_mode, .. } => {
            if *hotkey_mode {
                "[1-9] Set Value  [Enter] Execute  [?] Help  [Esc] Back"
            } else {
                "[Tab] Next  [Enter] Execute  [?] Help  [Esc] Cancel"
            }
        }
        AppState::Executing { .. } => {
            "Executing... please wait"
        }
        AppState::Results { tool_name, .. } => {
            match tool_name.as_str() {
                "capture_list" => "[↑/↓] Navigate  [Enter] Analyze  [V] View  [D] Delete  [E] Export  [T] Tag  [?] Help",
                "bt_decode" => "[↑/↓] Navigate  [Enter] Expand  [b] Bookmark  [m] Mark  [/] Filter  [e] Export  [?] Help",
                "bt_analyze" => "[o/d/s/t] View Modes  [↑/↓] Navigate  [Enter] Expand  [?] Help  [Esc] Back",
                "bt_compare" => "[?] Help  [Esc] Back to Menu - Side-by-side capture comparison",
                _ => "[?] Help  [Esc] Back to Menu"
            }
        }
        AppState::Settings { .. } => {
            "[?] Help  [Esc] Back to Menu"
        }
        AppState::Confirmation { .. } => {
            "[Y] Confirm  [N] Cancel"
        }
        AppState::ExportMenu { .. } => {
            "[↑/↓] Navigate  [Enter] Export  [?] Help  [Esc] Cancel"
        }
        AppState::FilterDialog { .. } => {
            "[↑/↓] Navigate  [←/→] Select  [Space] Toggle  [Enter] Apply  [C] Clear  [?] Help  [Esc] Cancel"
        }
        AppState::HelpOverlay { .. } => {
            "[↑/↓/PgUp/PgDn] Scroll  [Esc/?/q] Close Help"
        }
        AppState::ThemeSelector { .. } => {
            "[↑/↓] Navigate  [Enter] Apply Theme  [1-5] Quick Select  [Esc] Cancel"
        }
        AppState::LiveCapture { paused, .. } => {
            if *paused {
                "[Space] Resume  [s] Save  [c] Clear  [Esc] Stop  [?] Help"
            } else {
                "[Space] Pause  [s] Save  [c] Clear  [Esc] Stop  [?] Help"
            }
        }
        AppState::SessionManager { mode, .. } => {
            use crate::tui::app::SessionMode;
            match mode {
                SessionMode::List => "[↑/↓] Navigate  [Enter] Load  [s] Save  [d] Delete  [Esc] Back",
                SessionMode::Save => "[Type name]  [Enter] Save  [Esc] Cancel",
                SessionMode::Load => "Loading session...",
            }
        }
    };

    let footer = Paragraph::new(shortcuts)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

/// Render notification as a centered popup at the bottom of the screen

/// Render confirmation dialog

/// Render error message with helpful formatting
fn render_error_message(f: &mut Frame, area: Rect, tool_name: &str, error_msg: &str) {
    // Parse common error patterns and provide helpful context
    let (category, message, suggestion) = categorize_error(error_msg);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Error Category: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                category,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled("Details:", Style::default().fg(Color::Yellow))),
        Line::from(""),
    ];

    // Word-wrap the error message
    for chunk in message.chars().collect::<Vec<_>>().chunks(60) {
        let chunk_str: String = chunk.iter().collect();
        lines.push(Line::from(Span::styled(
            format!("  {}", chunk_str),
            Style::default().fg(Color::White),
        )));
    }

    lines.push(Line::from(""));

    if let Some(suggestion_text) = suggestion {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Suggestion:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {}", suggestion_text),
            Style::default().fg(Color::Green),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Esc] Back to menu",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Error: {}", tool_name))
                .title_style(Style::default().fg(Color::Red)),
        );

    f.render_widget(paragraph, area);
}

