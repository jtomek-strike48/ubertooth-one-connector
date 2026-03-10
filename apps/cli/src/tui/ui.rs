//! UI rendering logic

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::sync::Arc;
use ubertooth_core::ToolRegistry;

use super::app::{AppState, DeviceStatus, Notification, TextInputDialog};
use super::views::{Category, FieldInputMode, FieldType};

/// Render the entire UI
pub fn render(f: &mut Frame, state: &AppState, registry: &Arc<ToolRegistry>, device_status: &DeviceStatus, notification: &Option<Notification>, frame_count: u64, dialog: &Option<TextInputDialog>) {
    // Main layout: header + content + footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(f.size());

    render_header(f, chunks[0], device_status);
    render_content(f, chunks[1], state, registry, device_status, frame_count);
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
fn render_header(f: &mut Frame, area: Rect, device_status: &DeviceStatus) {
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
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(header, area);
}

/// Render main content based on state
fn render_content(f: &mut Frame, area: Rect, state: &AppState, registry: &Arc<ToolRegistry>, device_status: &DeviceStatus, frame_count: u64) {
    match state {
        AppState::MainMenu { selected_index } => {
            render_main_menu(f, area, *selected_index, device_status);
        }
        AppState::ToolCategory { category, selected_index } => {
            render_tool_category(f, area, category, *selected_index, registry, device_status);
        }
        AppState::ToolForm { form, error, hotkey_mode } => {
            if *hotkey_mode {
                render_tool_hotkeys(f, area, form.as_ref(), error.as_deref());
            } else {
                render_tool_form(f, area, form.as_ref(), error.as_deref());
            }
        }
        AppState::Executing { tool_name, .. } => {
            render_executing(f, area, tool_name, frame_count);
        }
        AppState::Results { tool_name, output, success, selected_capture, packet_list_state, analysis_view_state, .. } => {
            render_results(f, area, tool_name, output, *success, *selected_capture, packet_list_state.as_ref(), analysis_view_state.as_ref());
        }
        AppState::Settings { selected_index } => {
            render_settings(f, area, *selected_index);
        }
        AppState::Confirmation { message, .. } => {
            render_confirmation(f, area, message);
        }
        AppState::ExportMenu { selected_index, packets, packet_list_state, .. } => {
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
            render_filter_dialog(f, area, *selected_section, *selected_packet_type, packet_type_selections, mac_filter, rssi_min, rssi_max);
        }
    }
}

/// Render main menu with 7 categories
fn render_main_menu(f: &mut Frame, area: Rect, selected_index: usize, device_status: &DeviceStatus) {
    // First item is the dynamic connection toggle
    let connection_label = if device_status.connected {
        "[Disconnect from Ubertooth]"
    } else {
        "[Connect to Ubertooth]"
    };

    let categories = vec![
        (connection_label, "Toggle device connection"),
        ("1. Captures", "Manage all captures with hotkeys"),
        ("2. Reconnaissance (7 tools)", "BLE scan, spectrum analysis, follow connections"),
        ("3. Analysis (5 tools)", "Packet analysis, fingerprinting, comparison"),
        ("4. Attack Operations (5 tools)", "Injection, jamming, MITM (requires authorization)"),
        ("5. Configuration (8 tools)", "Channel, power, modulation, presets"),
        ("6. Advanced (2 tools)", "Raw USB commands, session context"),
    ];

    let items: Vec<ListItem> = categories
        .iter()
        .enumerate()
        .map(|(i, (title, desc))| {
            // Add blank line at top before connection toggle (index 0)
            let mut content = vec![];
            if i == 0 {
                content.push(Line::from("")); // Blank line at top
            }

            // Special styling for connection toggle (index 0)
            if i == 0 {
                let (dot, dot_color, text_color, text_modifier) = if device_status.connected {
                    ("●", Color::Green, Color::Green, Modifier::BOLD)
                } else {
                    ("○", Color::Gray, Color::Gray, Modifier::empty())
                };

                // Override with yellow if selected
                let (final_dot_color, final_text_color, final_modifier) = if i == selected_index {
                    (Color::Yellow, Color::Yellow, Modifier::BOLD)
                } else {
                    (dot_color, text_color, text_modifier)
                };

                let title_line = Line::from(vec![
                    Span::raw(" "),
                    Span::styled(dot, Style::default().fg(final_dot_color)),
                    Span::raw(" "),
                    Span::styled(*title, Style::default().fg(final_text_color).add_modifier(final_modifier)),
                ]);

                content.push(title_line);
            } else {
                // Regular category items
                let style = if i == selected_index {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                content.push(Line::from(Span::styled(*title, style)));
            }

            content.push(Line::from(Span::styled(format!("   {}", desc), Style::default().fg(Color::Gray))));
            content.push(Line::from("")); // Standard spacing after each item

            ListItem::new(Text::from(content))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Select Tool Category"));

    f.render_widget(list, area);
}

/// Render tool category submenu
fn render_tool_category(
    f: &mut Frame,
    area: Rect,
    category: &Category,
    selected_index: usize,
    registry: &Arc<ToolRegistry>,
    device_status: &DeviceStatus,
) {
    // Use filtered tools for DeviceManagement
    let device_connected = if matches!(category, Category::DeviceManagement) {
        Some(device_status.connected)
    } else {
        None
    };
    let tools = category.get_tools_filtered(registry, device_connected);

    let items: Vec<ListItem> = tools
        .iter()
        .enumerate()
        .map(|(i, tool)| {
            let style = if i == selected_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Add numbering to tool names
            let numbered_name = format!("{}. {}", i + 1, tool.name());

            let content = vec![
                Line::from(Span::styled(numbered_name, style)),
                Line::from(Span::styled(format!("   {}", tool.description()), Style::default().fg(Color::Gray))),
                Line::from(""),
            ];

            ListItem::new(Text::from(content))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("{:?} - Select Tool", category)));

    f.render_widget(list, area);
}

/// Render tool with hotkey parameter configuration
fn render_tool_hotkeys(f: &mut Frame, area: Rect, form: &crate::tui::views::ToolForm, error: Option<&str>) {
    // Split into: header, content, hotkey bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(0),      // Content/description
            Constraint::Length(5),  // Hotkey parameter bar
        ])
        .split(area);

    // Header with tool name
    let header_text = format!(
        "{}\n\n{}",
        form.tool_name(),
        form.tool_description()
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Tool"));
    f.render_widget(header, chunks[0]);

    // Content area - show current parameter values
    let mut content_lines = Vec::new();
    content_lines.push(Line::from(""));
    content_lines.push(Line::from(Span::styled(
        "  Current Configuration:",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    content_lines.push(Line::from(""));

    // Show current parameter values
    for field in form.fields() {
        let value_str = form.get_field_value(&field.name).unwrap_or_else(|| "<empty>".to_string());
        content_lines.push(Line::from(vec![
            Span::styled(format!("    {}: ", field.name), Style::default().fg(Color::Gray)),
            Span::styled(value_str, Style::default().fg(Color::White)),
        ]));
    }

    let content = Paragraph::new(Text::from(content_lines))
        .block(Block::default().borders(Borders::ALL).title("Configuration"));
    f.render_widget(content, chunks[1]);

    // Hotkey parameter bar at bottom
    let mut hotkey_lines = Vec::new();
    hotkey_lines.push(Line::from(""));

    // Build hotkey hints from mapping
    let hotkey_mapping = form.get_hotkey_mapping();
    let mut row1 = Vec::new();
    let mut row2 = Vec::new();

    for (idx, (hotkey, field_name, options)) in hotkey_mapping.iter().enumerate() {
        // Format options for display
        let opts_display = if options.len() > 4 {
            format!("{} options", options.len())
        } else if options.len() == 1 && options[0] == "<text input>" {
            "text".to_string()
        } else {
            options.join("/")
        };

        let hint = format!("[{}] {}: {}  ", hotkey, field_name, opts_display);

        // Split into two rows if more than 3 params
        if idx < 3 || hotkey_mapping.len() <= 3 {
            row1.push(hint);
        } else {
            row2.push(hint);
        }
    }

    hotkey_lines.push(Line::from(Span::styled(
        format!("  {}", row1.join("")),
        Style::default().fg(Color::Cyan),
    )));

    if !row2.is_empty() {
        hotkey_lines.push(Line::from(Span::styled(
            format!("  {}", row2.join("")),
            Style::default().fg(Color::Cyan),
        )));
    }

    if let Some(err) = error {
        hotkey_lines.push(Line::from(Span::styled(
            format!("  Error: {}", err),
            Style::default().fg(Color::Red),
        )));
    }

    let hotkeys = Paragraph::new(Text::from(hotkey_lines))
        .block(Block::default().borders(Borders::ALL).title("Parameters"));
    f.render_widget(hotkeys, chunks[2]);
}

/// Render tool parameter form
fn render_tool_form(f: &mut Frame, area: Rect, form: &crate::tui::views::ToolForm, error: Option<&str>) {
    // Split into sections: header, fields, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Fields
            Constraint::Length(if error.is_some() { 5 } else { 3 }), // Footer/error
        ])
        .split(area);

    // Header with tool name and description
    let header_text = format!("{}\n{}", form.tool_name(), form.tool_description());
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Tool Parameters"));
    f.render_widget(header, chunks[0]);

    // Form fields
    let fields = form.fields();
    let inputs = form.inputs();
    let input_modes = form.input_modes();
    let focused = form.focused_index();

    if fields.is_empty() {
        let no_params = Paragraph::new("This tool has no parameters.\n\nPress Ctrl+Enter to execute.")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(no_params, chunks[1]);
    } else {
        // Create layout for each field (label + input)
        let field_constraints: Vec<Constraint> = fields
            .iter()
            .flat_map(|_| vec![Constraint::Length(1), Constraint::Length(3)])
            .collect();

        let field_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(field_constraints)
            .split(chunks[1]);

        for (i, field) in fields.iter().enumerate() {
            let label_idx = i * 2;
            let input_idx = i * 2 + 1;

            // Field label with type and required indicator
            let type_hint = match field.field_type {
                FieldType::String => "text",
                FieldType::Integer => "integer",
                FieldType::Number => "number",
                FieldType::Boolean => "true/false",
                FieldType::Array => "comma-separated",
            };

            let required_marker = if field.required { " *" } else { "" };
            let label_text = format!(
                "{}{} ({}): {}",
                field.name, required_marker, type_hint, field.description
            );

            let label = Paragraph::new(label_text)
                .style(Style::default().fg(if i == focused { Color::Yellow } else { Color::White }));
            f.render_widget(label, field_chunks[label_idx]);

            // Input field - render dropdown or text input
            match &input_modes[i] {
                FieldInputMode::Dropdown { selected_index } => {
                    // Render dropdown
                    if let Some(options) = &field.dropdown_options {
                        let selected_value = options.get(*selected_index).cloned().unwrap_or_default();
                        let display = format!("[ {} ]  (use Up/Down to change)", selected_value);

                        let dropdown_widget = Paragraph::new(display)
                            .style(Style::default().fg(if i == focused { Color::Cyan } else { Color::White }))
                            .block(Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(if i == focused {
                                    Color::Yellow
                                } else {
                                    Color::Gray
                                })));

                        f.render_widget(dropdown_widget, field_chunks[input_idx]);
                    }
                }
                FieldInputMode::Text => {
                    // Render text input
                    let input_text = inputs[i].lines().join("\n");
                    let input_widget = Paragraph::new(input_text)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(if i == focused {
                                Color::Yellow
                            } else {
                                Color::Gray
                            })));

                    f.render_widget(input_widget, field_chunks[input_idx]);
                }
            }
        }
    }

    // Footer with instructions or error
    if let Some(err) = error {
        let error_text = format!("Error: {}\n\n[Tab] Next field  [Enter] Execute  [Esc] Back", err);
        let footer = Paragraph::new(error_text)
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
    } else {
        // Check if focused field is dropdown
        let is_dropdown = matches!(
            input_modes.get(focused),
            Some(FieldInputMode::Dropdown { .. })
        );

        let footer_text = if is_dropdown {
            "[Up/Down] Select  [Tab] Next field  [Enter] Execute  [Esc] Back"
        } else {
            "[Tab] Next field  [Shift+Tab] Previous  [Enter] Execute  [Esc] Back"
        };

        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
    }
}

/// Render analysis results in readable format
fn render_analysis_results(f: &mut Frame, area: Rect, output: &serde_json::Value, state: Option<&crate::tui::app::AnalysisViewState>) {
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

/// Render analysis overview (summary of all sections)
fn render_analysis_overview(f: &mut Frame, area: Rect, output: &serde_json::Value) {
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    if let Some(analysis) = output.get("analysis").and_then(|a| a.as_object()) {
        // Protocol Summary
        if let Some(proto) = analysis.get("protocol_summary").and_then(|p| p.as_object()) {
            lines.push(Line::from(Span::styled(
                "Protocol Summary",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
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
                Span::styled(format!("{} found", devices.len()), Style::default().fg(Color::White)),
                Span::raw("  "),
                Span::styled("(press 'd' for details)", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
            ]));
        }

        if let Some(security) = analysis.get("security_observations").and_then(|s| s.as_array()) {
            if !security.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("🔒 Security: ", Style::default().fg(Color::Red)),
                    Span::styled(format!("{} observations", security.len()), Style::default().fg(Color::White)),
                    Span::raw("  "),
                    Span::styled("(press 's' for details)", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
                ]));
            }
        }

        if let Some(timing) = analysis.get("timing_analysis").and_then(|t| t.as_object()) {
            if let Some(avg) = timing.get("avg_interval_ms").and_then(|a| a.as_f64()) {
                if avg > 0.0 {
                    lines.push(Line::from(vec![
                        Span::styled("⏱️  Timing: ", Style::default().fg(Color::Blue)),
                        Span::styled(format!("{:.2}ms avg", avg), Style::default().fg(Color::White)),
                        Span::raw("  "),
                        Span::styled("(press 't' for details)", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
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

    let content = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title(" Analysis Overview "));
    f.render_widget(content, area);
}

/// Render devices view with interactive list
fn render_analysis_devices(f: &mut Frame, area: Rect, output: &serde_json::Value, state: &crate::tui::app::AnalysisViewState) {
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
                    let mac = device.get("mac_address").and_then(|m| m.as_str()).unwrap_or("Unknown");
                    let name = device.get("device_name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                    let pkts = device.get("packet_count").and_then(|p| p.as_u64()).unwrap_or(0);

                    let style = if is_selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    lines.push(Line::from(vec![
                        Span::styled(expand_icon, style),
                        Span::raw(" "),
                        Span::styled(mac, style.fg(Color::Cyan)),
                        Span::raw(" - "),
                        Span::styled(name, style),
                        Span::styled(format!(" ({} pkts)", pkts), Style::default().fg(Color::Gray)),
                    ]));

                    if is_expanded {
                        // Show device details
                        if let Some(rssi) = device.get("rssi").and_then(|r| r.as_i64()) {
                            lines.push(Line::from(vec![
                                Span::raw("  │ "),
                                Span::styled("RSSI: ", Style::default().fg(Color::Gray)),
                                Span::styled(format!("{} dBm", rssi), Style::default().fg(Color::White)),
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
                                    Span::styled("Active Duration: ", Style::default().fg(Color::Gray)),
                                    Span::styled(format!("{:.2}s", duration), Style::default().fg(Color::White)),
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

    let content = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title(format!(" Devices ({} total) ",
            output.get("analysis")
                .and_then(|a| a.get("devices"))
                .and_then(|d| d.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0)
        )));
    f.render_widget(content, area);
}

/// Render security observations view
fn render_analysis_security(f: &mut Frame, area: Rect, output: &serde_json::Value, state: &crate::tui::app::AnalysisViewState) {
    let mut lines = Vec::new();

    if let Some(analysis) = output.get("analysis").and_then(|a| a.as_object()) {
        // Security summary
        if let Some(summary) = analysis.get("security_summary").and_then(|s| s.as_object()) {
            lines.push(Line::from(Span::styled(
                "Security Summary",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));

            if let Some(privacy) = summary.get("privacy_enabled_devices").and_then(|p| p.as_u64()) {
                lines.push(Line::from(vec![
                    Span::raw("  Privacy-enabled devices: "),
                    Span::styled(privacy.to_string(), Style::default().fg(Color::Green)),
                ]));
            }
            if let Some(public) = summary.get("public_address_devices").and_then(|p| p.as_u64()) {
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
        if let Some(observations) = analysis.get("security_observations").and_then(|s| s.as_array()) {
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
                            obj.get("type").and_then(|t| t.as_str()).unwrap_or("Unknown"),
                            obj.get("description").and_then(|d| d.as_str()).unwrap_or(""),
                            obj.get("severity").and_then(|s| s.as_str()).unwrap_or("info"),
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
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    lines.push(Line::from(vec![
                        Span::styled(expand_icon, style),
                        Span::raw(" "),
                        Span::styled(format!("[{}] ", severity.to_uppercase()), Style::default().fg(severity_color).add_modifier(Modifier::BOLD)),
                        Span::styled(obs_type, style),
                    ]));

                    if is_expanded {
                        lines.push(Line::from(vec![
                            Span::raw("  │ "),
                            Span::styled(description, Style::default().fg(Color::White)),
                        ]));
                        if let Some(obj) = obs.as_object() {
                            if let Some(device) = obj.get("affected_device").and_then(|d| d.as_str()) {
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

    let content = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title(" Security Analysis "));
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
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            if let Some(duration) = timing.get("duration_sec").and_then(|d| d.as_f64()) {
                lines.push(Line::from(vec![
                    Span::styled("Capture Duration: ", Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{:.2}s", duration), Style::default().fg(Color::White)),
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
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
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

    let content = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title(" Timing Analysis "));
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
                Span::styled("  Capture ID: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
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
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
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
                Span::styled(format!("{:.2} KB", size_kb), Style::default().fg(Color::Magenta)),
            ]));
        }
        lines.push(Line::from(""));

        // Tags
        if let Some(tags) = obj.get("tags").and_then(|v| v.as_array()) {
            if !tags.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Tags",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )));
                let tag_str = tags.iter()
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
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
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

    let content = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("Capture Details"));
    f.render_widget(content, area);
}

/// Render capture list as a formatted table
fn render_capture_list_table(f: &mut Frame, area: Rect, captures: &[serde_json::Value], selected_index: Option<usize>) {
    let mut lines = Vec::new();

    // Header line
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  Found {} capture(s)", captures.len()),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
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
            Span::styled("  ID                  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Type           ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Pkts    ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Duration    ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Timestamp              ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Description", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(Span::raw(format!("  {}", "-".repeat(120)))));

        // Table rows
        for (idx, capture) in captures.iter().enumerate() {
            let is_selected = selected_index == Some(idx);
            let id = capture.get("capture_id")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .split('-')
                .nth(2)
                .unwrap_or("?")
                .chars()
                .take(16)
                .collect::<String>();

            let cap_type = capture.get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("?");

            let packet_count = capture.get("packet_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let duration = capture.get("duration_sec")
                .and_then(|v| v.as_u64())
                .map(|d| format!("{}s", d))
                .unwrap_or_else(|| "N/A".to_string());

            let timestamp = capture.get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .chars()
                .take(19)
                .collect::<String>();

            let description = capture.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let tags = capture.get("tags")
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

    let content = Paragraph::new(Text::from(lines))
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Capture List")
            .title_style(Style::default().fg(Color::Cyan)));

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
            Span::styled(tool_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Please wait...", Style::default().fg(Color::Gray))),
        Line::from(""),
        Line::from(Span::styled("This may take a few seconds depending on the tool.", Style::default().fg(Color::DarkGray))),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Executing"));

    f.render_widget(paragraph, area);
}

/// Render tool results
fn render_results(f: &mut Frame, area: Rect, tool_name: &str, output: &serde_json::Value, success: bool, selected_capture: Option<usize>, packet_list_state: Option<&crate::tui::app::PacketListState>, analysis_view_state: Option<&crate::tui::app::AnalysisViewState>) {
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

    let header_text = format!(
        "{} {}\n\nTool: {}",
        status_symbol, status_text, tool_name
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Execution Result"));
    f.render_widget(header, chunks[0]);

    // Special formatting for capture_list - show as table
    if tool_name == "capture_list" && success {
        if let Some(captures_array) = output.get("captures").and_then(|v| v.as_array()) {
            render_capture_list_table(f, chunks[1], captures_array, selected_capture);
            return;
        }
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
    let result_json = serde_json::to_string_pretty(output)
        .unwrap_or_else(|_| "{}".to_string());

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
                Span::styled(format!("{:.1}s", duration), Style::default().fg(Color::White)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Full Output:", Style::default().fg(Color::Gray))));
        lines.push(Line::from(""));

        // Add full JSON
        for line in result_json.lines() {
            lines.push(Line::from(line.to_string()));
        }

        Text::from(lines)
    } else {
        Text::from(result_json)
    };

    let content = Paragraph::new(formatted_output)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Output")
            .title_style(Style::default().fg(Color::Gray)));

    f.render_widget(content, chunks[1]);
}

/// Render settings page
fn render_settings(f: &mut Frame, area: Rect, selected_index: usize) {
    let settings_items = vec![
        ("View Tool History", "Show recently used tools"),
        ("View Favorites", "Show bookmarked tools"),
        ("View Recent MAC Addresses", "MAC filter helper for analysis"),
        ("Backend Info", "View backend configuration"),
        ("Strike48 Connection", "Configure cloud connection"),
        ("About", "Version and system information"),
    ];

    let items: Vec<ListItem> = settings_items
        .iter()
        .enumerate()
        .map(|(i, (title, desc))| {
            let style = if i == selected_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = vec![
                Line::from(Span::styled(format!("{}. {}", i + 1, title), style)),
                Line::from(Span::styled(format!("   {}", desc), Style::default().fg(Color::Gray))),
                Line::from(""),
            ];

            ListItem::new(Text::from(content))
        })
        .collect();

    let settings_list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Settings")
            .title_style(Style::default().fg(Color::Cyan)));

    f.render_widget(settings_list, area);
}

/// Render footer with keyboard shortcuts
fn render_footer(f: &mut Frame, area: Rect, state: &AppState) {
    let shortcuts = match state {
        AppState::MainMenu { .. } => {
            "[0-6] Quick Select  [↑/↓] Navigate  [Enter] Select  [Esc] Back  [q] Quit"
        }
        AppState::ToolCategory { category, .. } => {
            if matches!(category, Category::DeviceManagement) {
                "[1-9] Quick Select  [→] Device Status  [Enter] Select  [Esc] Back"
            } else {
                "[1-9] Quick Select  [↑/↓] Navigate  [Enter] Select  [Esc] Back"
            }
        }
        AppState::ToolForm { hotkey_mode, .. } => {
            if *hotkey_mode {
                "[1-9] Set Value  [Enter] Execute  [Esc] Back"
            } else {
                "[Tab] Next  [Enter] Execute  [Esc] Cancel"
            }
        }
        AppState::Executing { .. } => {
            "Executing... please wait"
        }
        AppState::Results { .. } => {
            "[Esc] Back to Menu"
        }
        AppState::Settings { .. } => {
            "[Esc] Back to Menu"
        }
        AppState::Confirmation { .. } => {
            "[Y] Confirm  [N] Cancel"
        }
        AppState::ExportMenu { .. } => {
            "[↑/↓] Navigate  [Enter] Export  [Esc] Cancel"
        }
        AppState::FilterDialog { .. } => {
            "[↑/↓] Navigate  [←/→] Select  [Space] Toggle  [Enter] Apply  [C] Clear  [Esc] Cancel"
        }
    };

    let footer = Paragraph::new(shortcuts)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

/// Render notification as a centered popup at the bottom of the screen
fn render_notification(f: &mut Frame, area: Rect, notification: &Notification) {
    // Calculate notification size and position
    let notif_width = notification.message.len().min(60) as u16 + 4;
    let notif_height = 3;

    // Position at bottom center
    let notif_x = area.width.saturating_sub(notif_width) / 2;
    let notif_y = area.height.saturating_sub(notif_height + 4); // Above footer

    let notif_area = Rect {
        x: area.x + notif_x,
        y: area.y + notif_y,
        width: notif_width,
        height: notif_height,
    };

    // Choose color based on success/failure
    let (bg_color, fg_color) = if notification.success {
        (Color::Green, Color::Black)
    } else {
        (Color::Red, Color::White)
    };

    let notif_widget = Paragraph::new(notification.message.as_str())
        .style(Style::default().fg(fg_color).bg(bg_color))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(notif_widget, notif_area);
}

/// Render confirmation dialog
fn render_confirmation(f: &mut Frame, area: Rect, message: &str) {
    // Create centered dialog
    let dialog_width = message.len().max(40).min(80) as u16 + 4;
    let dialog_height = 7;

    let dialog_x = area.width.saturating_sub(dialog_width) / 2;
    let dialog_y = area.height.saturating_sub(dialog_height) / 2;

    let dialog_area = Rect {
        x: area.x + dialog_x,
        y: area.y + dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the background
    let clear_widget = Block::default()
        .style(Style::default().bg(Color::Black));
    f.render_widget(clear_widget, area);

    // Build dialog content
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::styled("Press [Y] to confirm or [N] to cancel", Style::default().fg(Color::Gray))),
        Line::from(""),
    ];

    let dialog = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title("Confirmation")
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));

    f.render_widget(dialog, dialog_area);
}

/// Render error message with helpful formatting
fn render_error_message(f: &mut Frame, area: Rect, tool_name: &str, error_msg: &str) {
    // Parse common error patterns and provide helpful context
    let (category, message, suggestion) = categorize_error(error_msg);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Error Category: ", Style::default().fg(Color::DarkGray)),
            Span::styled(category, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
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
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
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
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Error: {}", tool_name))
            .title_style(Style::default().fg(Color::Red)));

    f.render_widget(paragraph, area);
}

/// Categorize error and provide helpful suggestions
fn categorize_error(error_msg: &str) -> (&'static str, &str, Option<&'static str>) {
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
        (
            "General Error",
            error_msg,
            None,
        )
    }
}
/// Render decoded packet list for bt_decode
fn render_decoded_packets(f: &mut Frame, area: Rect, output: &serde_json::Value, packet_list_state: Option<&crate::tui::app::PacketListState>) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph, Wrap},
    };

    let packets = match output.get("decoded_packets").and_then(|p| p.as_array()) {
        Some(p) => p,
        None => {
            let text = vec![
                Line::from(""),
                Line::from("No packets to display"),
                Line::from(""),
                Line::from(Span::styled(
                    "The capture may be empty or the decode limit was 0.",
                    Style::default().fg(Color::Gray),
                )),
            ];
            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Decoded Packets"))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(paragraph, area);
            return;
        }
    };

    let packet_count = packets.len();
    if packet_count == 0 {
        let text = vec![
            Line::from(""),
            Line::from("No packets to display"),
            Line::from(""),
            Line::from(Span::styled(
                "The capture may be empty or the decode limit was 0.",
                Style::default().fg(Color::Gray),
            )),
        ];
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Decoded Packets"))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let default_state = crate::tui::app::PacketListState::new();
    let state = packet_list_state.unwrap_or(&default_state);

    // Dispatch to appropriate view based on view_mode
    match state.view_mode {
        crate::tui::app::PacketViewMode::List => {
            render_packet_list(f, area, packets, state);
        }
        crate::tui::app::PacketViewMode::Statistics => {
            render_packet_statistics(f, area, packets, state);
        }
        crate::tui::app::PacketViewMode::Timeline => {
            render_packet_timeline(f, area, packets, state);
        }
        crate::tui::app::PacketViewMode::Comparison => {
            render_packet_comparison(f, area, packets, state);
        }
    }
}

/// Render packet list view (original table view)
fn render_packet_list(f: &mut Frame, area: Rect, packets: &[serde_json::Value], state: &crate::tui::app::PacketListState) {
    use ratatui::{
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph, Wrap},
    };

    let packet_count = packets.len();

    // Apply all active filters
    let filtered_packets: Vec<(usize, &serde_json::Value)> = packets.iter().enumerate()
        .filter(|(_, pkt)| {
            // Follow stream filter (legacy, keeping for compatibility)
            if let Some(ref mac) = state.follow_mac {
                if let Some(packet_mac) = pkt.get("mac_address").and_then(|m| m.as_str()) {
                    if !packet_mac.contains(mac) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Apply filter rules from state.filters
            state.filters.matches(pkt)
        })
        .collect();

    let displayed_count = filtered_packets.len();
    if displayed_count == 0 {
        let text = vec![
            Line::from(""),
            Line::from("No packets match the current filter"),
            Line::from(""),
            Line::from(Span::styled(
                "Press '/' to modify filters",
                Style::default().fg(Color::Gray),
            )),
        ];
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Decoded Packets (Filtered)"))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    // Calculate visible area
    let visible_height = area.height.saturating_sub(6) as usize; // Account for borders, header, and footer
    let start_idx = state.scroll_offset.min(displayed_count.saturating_sub(1));
    let end_idx = (start_idx + visible_height).min(displayed_count);

    let mut lines = vec![];

    // Header line with indicators
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()), // Space for bookmark/mark indicators
        Span::styled(" # ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Time          ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Ch ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("RSSI ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Type          ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("MAC Address             ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Proto ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Summary", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]));

    lines.push(Line::from(Span::styled(
        "━".repeat(area.width as usize - 2),
        Style::default().fg(Color::DarkGray),
    )));

    // Render visible packets
    for display_idx in start_idx..end_idx {
        if let Some((original_idx, packet)) = filtered_packets.get(display_idx) {
            let is_selected = display_idx == state.selected_index;
            let is_expanded = state.is_expanded(*original_idx);
            let is_bookmarked = state.is_bookmarked(*original_idx);
            let is_marked = state.is_marked_for_comparison(*original_idx);

            // Extract packet fields
            let frame_num = packet.get("frame_number").and_then(|v| v.as_str()).unwrap_or("?");
            let timestamp = packet.get("timestamp").and_then(|v| v.as_str()).unwrap_or("Unknown");
            let time_short = timestamp.split(',').last().unwrap_or(timestamp).trim();
            let time_display = if time_short.len() > 12 {
                &time_short[time_short.len() - 12..]
            } else {
                time_short
            };

            let channel = packet.get("channel").and_then(|v| v.as_str()).unwrap_or("?");
            let rssi = packet.get("rssi").and_then(|v| v.as_str()).unwrap_or("?");
            let packet_type = packet.get("packet_type").and_then(|v| v.as_str()).unwrap_or("?");
            let mac_address = packet.get("mac_address").and_then(|v| v.as_str()).unwrap_or("N/A");
            let protocol = packet.get("protocol").and_then(|v| v.as_str()).unwrap_or("?");
            let summary = packet.get("summary").and_then(|v| v.as_str()).unwrap_or("");

            // Color based on packet type
            let type_color = match packet_type {
                "ADV_IND" | "ADV_NONCONN_IND" | "ADV_SCAN_IND" => Color::Green,
                "SCAN_REQ" | "SCAN_RSP" => Color::Cyan,
                "CONNECT_REQ" => Color::Yellow,
                "DATA" => Color::Blue,
                _ => Color::White,
            };

            // Indicators: bookmark, comparison mark, annotation, expand/collapse
            let bookmark_indicator = if is_bookmarked { "★" } else { " " };
            let mark_indicator = if is_marked { "●" } else { " " };
            let has_annotation = state.has_annotation(*original_idx);
            let annotation_indicator = if has_annotation { "📝" } else { " " };
            let expand_indicator = if is_selected {
                if is_expanded { "▼" } else { "▶" }
            } else {
                " "
            };

            let bg_color = if is_selected {
                Color::DarkGray
            } else {
                Color::Reset
            };

            // Main packet row
            lines.push(Line::from(vec![
                Span::styled(bookmark_indicator, Style::default().fg(Color::Yellow).bg(bg_color)),
                Span::styled(mark_indicator, Style::default().fg(Color::Magenta).bg(bg_color)),
                Span::styled(expand_indicator, Style::default().fg(Color::Cyan).bg(bg_color)),
                Span::styled(format!("{:3} ", frame_num), Style::default().fg(Color::Gray).bg(bg_color)),
                Span::styled(format!("{:12} ", time_display), Style::default().fg(Color::White).bg(bg_color)),
                Span::styled(format!("{:2} ", channel), Style::default().fg(Color::Magenta).bg(bg_color)),
                Span::styled(format!("{:4} ", rssi), Style::default().fg(Color::Red).bg(bg_color)),
                Span::styled(format!("{:13} ", packet_type), Style::default().fg(type_color).bg(bg_color)),
                Span::styled(format!("{:23} ", mac_address), Style::default().fg(Color::Cyan).bg(bg_color)),
                Span::styled(format!("{:5} ", protocol), Style::default().fg(Color::Blue).bg(bg_color)),
                Span::styled(summary, Style::default().fg(Color::White).bg(bg_color)),
            ]));

            // Expanded view
            if is_expanded {
                let access_addr = packet.get("access_addr").and_then(|v| v.as_str()).unwrap_or("Unknown");

                lines.push(Line::from(vec![
                    Span::raw("  │ "),
                    Span::styled("Access Address: ", Style::default().fg(Color::Gray)),
                    Span::styled(access_addr, Style::default().fg(Color::White)),
                ]));

                lines.push(Line::from(vec![
                    Span::raw("  │ "),
                    Span::styled("Full timestamp: ", Style::default().fg(Color::Gray)),
                    Span::styled(timestamp, Style::default().fg(Color::White)),
                ]));

                // Show protocol layers if available
                if let Some(full_packet) = packet.get("full_packet") {
                    if let Some(layers) = full_packet.get("_source").and_then(|s| s.get("layers")) {
                        let layer_names: Vec<String> = if let Some(obj) = layers.as_object() {
                            obj.keys().map(|k| k.to_string()).collect()
                        } else {
                            vec![]
                        };

                        if !layer_names.is_empty() {
                            lines.push(Line::from(vec![
                                Span::raw("  │ "),
                                Span::styled("Layers: ", Style::default().fg(Color::Gray)),
                                Span::styled(layer_names.join(" → "), Style::default().fg(Color::Cyan)),
                            ]));
                        }
                    }
                }

                // Show annotation if present
                if let Some(note) = state.get_annotation(*original_idx) {
                    lines.push(Line::from(vec![
                        Span::raw("  │ "),
                        Span::styled("Note: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::styled(note.clone(), Style::default().fg(Color::White).add_modifier(Modifier::ITALIC)),
                    ]));
                }

                lines.push(Line::from(vec![
                    Span::raw("  │ "),
                    Span::styled("[Enter: collapse | n: add note | Del: remove note]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                ]));

                lines.push(Line::from("  │"));
            }
        }
    }

    // Footer with navigation hints and stats
    lines.push(Line::from(""));

    // Navigation line
    lines.push(Line::from(vec![
        Span::styled("Nav: ", Style::default().fg(Color::Yellow)),
        Span::styled("↑↓", Style::default().fg(Color::Cyan)),
        Span::raw(" scroll  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" expand  "),
        Span::styled("b", Style::default().fg(Color::Cyan)),
        Span::raw(" bookmark  "),
        Span::styled("m", Style::default().fg(Color::Cyan)),
        Span::raw(" mark  "),
        Span::styled("f", Style::default().fg(Color::Cyan)),
        Span::raw(" follow  "),
    ]));

    // View mode line
    lines.push(Line::from(vec![
        Span::styled("Views: ", Style::default().fg(Color::Yellow)),
        Span::styled("l", Style::default().fg(Color::Cyan)),
        Span::raw(" list  "),
        Span::styled("s", Style::default().fg(Color::Cyan)),
        Span::raw(" statistics  "),
        Span::styled("t", Style::default().fg(Color::Cyan)),
        Span::raw(" timeline  "),
        Span::styled("c", Style::default().fg(Color::Cyan)),
        Span::raw(" compare  "),
        Span::styled("n", Style::default().fg(Color::Cyan)),
        Span::raw(" note  "),
        Span::styled("/", Style::default().fg(Color::Cyan)),
        Span::raw(" filter  "),
        Span::styled("e", Style::default().fg(Color::Cyan)),
        Span::raw(" export  "),
        Span::raw(" │ "),
        Span::styled(format!("Showing {}-{} of {}", start_idx + 1, end_idx, displayed_count), Style::default().fg(Color::Gray)),
        if displayed_count < packet_count {
            Span::styled(format!(" (filtered from {})", packet_count), Style::default().fg(Color::Yellow))
        } else {
            Span::raw("")
        },
    ]));

    // Follow stream indicator
    if let Some(ref mac) = state.follow_mac {
        lines.push(Line::from(vec![
            Span::styled("Following: ", Style::default().fg(Color::Yellow)),
            Span::styled(mac.clone(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("(press 'f' to clear)", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
        ]));
    }

    // Bookmark indicator
    let bookmark_count = state.bookmarks.len();
    if bookmark_count > 0 {
        lines.push(Line::from(vec![
            Span::styled("★ ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("{} bookmarked packet{}", bookmark_count, if bookmark_count == 1 { "" } else { "s" }), Style::default().fg(Color::White)),
        ]));
    }

    // Active filters indicator
    if state.filters.is_active() {
        let mut filter_parts = vec![];

        if !state.filters.packet_types.is_empty() {
            filter_parts.push(format!("Types: {}", state.filters.packet_types.join(", ")));
        }
        if let Some(ref mac) = state.filters.mac_address {
            filter_parts.push(format!("MAC: {}", mac));
        }
        if state.filters.rssi_min.is_some() || state.filters.rssi_max.is_some() {
            let min = state.filters.rssi_min.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string());
            let max = state.filters.rssi_max.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string());
            filter_parts.push(format!("RSSI: {} to {} dBm", min, max));
        }

        lines.push(Line::from(vec![
            Span::styled("🔍 Active Filters: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(filter_parts.join(" | "), Style::default().fg(Color::White)),
            Span::raw("  "),
            Span::styled("(press '/' to modify)", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
        ]));
    }

    let title = format!(" Decoded Packets ({} total) ", packet_count);
    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
/// Render packet statistics view
fn render_packet_statistics(f: &mut Frame, area: Rect, packets: &[serde_json::Value], state: &crate::tui::app::PacketListState) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph, Wrap},
    };
    use std::collections::HashMap;

    let packet_count = packets.len();

    // Split area into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // Packet type distribution
            Constraint::Length(8),  // Channel and RSSI stats
            Constraint::Min(8),     // MAC addresses
        ])
        .split(area);

    // 1. Packet Type Distribution
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut total_rssi: i32 = 0;
    let mut rssi_count = 0;
    let mut channel_counts: HashMap<String, usize> = HashMap::new();
    let mut mac_addresses: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut min_rssi = 0i8;
    let mut max_rssi = -128i8;

    for packet in packets {
        // Packet types
        if let Some(ptype) = packet.get("packet_type").and_then(|v| v.as_str()) {
            *type_counts.entry(ptype.to_string()).or_insert(0) += 1;
        }

        // RSSI stats
        if let Some(rssi_str) = packet.get("rssi").and_then(|v| v.as_str()) {
            if let Ok(rssi) = rssi_str.parse::<i8>() {
                total_rssi += rssi as i32;
                rssi_count += 1;
                min_rssi = min_rssi.min(rssi);
                max_rssi = max_rssi.max(rssi);
            }
        }

        // Channel distribution
        if let Some(ch) = packet.get("channel").and_then(|v| v.as_str()) {
            *channel_counts.entry(ch.to_string()).or_insert(0) += 1;
        }

        // MAC addresses
        if let Some(mac) = packet.get("mac_address").and_then(|v| v.as_str()) {
            // Split combined MACs (e.g., "AA ← BB")
            for part in mac.split(&['←', '→'][..]) {
                let cleaned = part.trim();
                if !cleaned.is_empty() && cleaned != "N/A" {
                    mac_addresses.insert(cleaned.to_string());
                }
            }
        }
    }

    let avg_rssi = if rssi_count > 0 {
        total_rssi as f32 / rssi_count as f32
    } else {
        0.0
    };

    // Render packet type distribution
    let mut type_lines = vec![
        Line::from(Span::styled(
            "Packet Type Distribution",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    let mut sorted_types: Vec<_> = type_counts.iter().collect();
    sorted_types.sort_by(|a, b| b.1.cmp(a.1));

    for (ptype, count) in sorted_types.iter().take(8) {
        let percentage = (**count as f32 / packet_count as f32) * 100.0;
        let bar_width = (percentage / 3.0) as usize; // Scale for display
        let bar = "█".repeat(bar_width.max(1));

        let color = match ptype.as_str() {
            "ADV_IND" | "ADV_NONCONN_IND" | "ADV_SCAN_IND" => Color::Green,
            "SCAN_REQ" | "SCAN_RSP" => Color::Cyan,
            "CONNECT_REQ" => Color::Yellow,
            "DATA" => Color::Blue,
            _ => Color::White,
        };

        type_lines.push(Line::from(vec![
            Span::styled(format!("{:15}", ptype), Style::default().fg(Color::White)),
            Span::styled(bar, Style::default().fg(color)),
            Span::styled(format!(" {} ({:.1}%)", count, percentage), Style::default().fg(Color::Gray)),
        ]));
    }

    let type_block = Paragraph::new(type_lines)
        .block(Block::default().borders(Borders::ALL).title(" Packet Types "))
        .wrap(Wrap { trim: false });
    f.render_widget(type_block, chunks[0]);

    // Render channel and RSSI stats
    let mut stats_lines = vec![
        Line::from(vec![
            Span::styled("Total Packets: ", Style::default().fg(Color::Yellow)),
            Span::styled(packet_count.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Unique MACs:   ", Style::default().fg(Color::Yellow)),
            Span::styled(mac_addresses.len().to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("RSSI Stats:    ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("Avg: {:.1} dBm", avg_rssi), Style::default().fg(Color::White)),
            Span::raw("  "),
            Span::styled(format!("Min: {} dBm", min_rssi), Style::default().fg(Color::Red)),
            Span::raw("  "),
            Span::styled(format!("Max: {} dBm", max_rssi), Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
    ];

    // Channel distribution
    stats_lines.push(Line::from(Span::styled(
        "Channel Distribution:",
        Style::default().fg(Color::Yellow),
    )));

    let mut sorted_channels: Vec<_> = channel_counts.iter().collect();
    sorted_channels.sort_by_key(|(ch, _)| ch.parse::<u8>().unwrap_or(255));

    for (ch, count) in sorted_channels {
        let percentage = (*count as f32 / packet_count as f32) * 100.0;
        stats_lines.push(Line::from(vec![
            Span::styled(format!("  Ch {:2}:", ch), Style::default().fg(Color::Magenta)),
            Span::styled(format!(" {} packets ({:.1}%)", count, percentage), Style::default().fg(Color::White)),
        ]));
    }

    let stats_block = Paragraph::new(stats_lines)
        .block(Block::default().borders(Borders::ALL).title(" Statistics "))
        .wrap(Wrap { trim: false });
    f.render_widget(stats_block, chunks[1]);

    // Render unique MAC addresses
    let mut mac_lines = vec![
        Line::from(Span::styled(
            format!("Unique MAC Addresses ({})", mac_addresses.len()),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    let mut sorted_macs: Vec<_> = mac_addresses.iter().collect();
    sorted_macs.sort();

    for (i, mac) in sorted_macs.iter().take(15).enumerate() {
        mac_lines.push(Line::from(vec![
            Span::styled(format!("{:2}. ", i + 1), Style::default().fg(Color::Gray)),
            Span::styled(mac.to_string(), Style::default().fg(Color::Cyan)),
        ]));
    }

    if sorted_macs.len() > 15 {
        mac_lines.push(Line::from(Span::styled(
            format!("... and {} more", sorted_macs.len() - 15),
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
        )));
    }

    mac_lines.push(Line::from(""));
    mac_lines.push(Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::Gray)),
        Span::styled("l", Style::default().fg(Color::Cyan)),
        Span::styled(" to return to packet list view", Style::default().fg(Color::Gray)),
    ]));

    let mac_block = Paragraph::new(mac_lines)
        .block(Block::default().borders(Borders::ALL).title(" Devices "))
        .wrap(Wrap { trim: false });
    f.render_widget(mac_block, chunks[2]);
}
/// Render packet timeline view
fn render_packet_timeline(f: &mut Frame, area: Rect, packets: &[serde_json::Value], state: &crate::tui::app::PacketListState) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph, Wrap},
    };

    let packet_count = packets.len();
    if packet_count == 0 {
        return;
    }

    // Split area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),    // Timeline
            Constraint::Length(8),  // Legend and stats
        ])
        .split(area);

    // Parse timestamps and determine time range
    let mut timestamps: Vec<(usize, f64)> = Vec::new();
    for (idx, packet) in packets.iter().enumerate() {
        if let Some(ts_str) = packet.get("timestamp").and_then(|v| v.as_str()) {
            // Parse timestamp - just use index if parsing fails
            // Format: "Mar  4, 2026 17:07:00.999844945 EST"
            timestamps.push((idx, idx as f64));
        }
    }

    if timestamps.is_empty() {
        return;
    }

    let min_time = 0.0;
    let max_time = packet_count as f64;
    let time_range = max_time - min_time;

    // Calculate timeline width
    let timeline_width = area.width.saturating_sub(6) as usize;
    let timeline_height = chunks[0].height.saturating_sub(4) as usize;

    // Create timeline buckets
    let bucket_count = timeline_width;
    let mut buckets: Vec<Vec<(usize, &serde_json::Value)>> = vec![vec![]; bucket_count];

    for (idx, _) in &timestamps {
        if let Some(packet) = packets.get(*idx) {
            let normalized_time = (*idx as f64 - min_time) / time_range;
            let bucket_idx = (normalized_time * (bucket_count - 1) as f64) as usize;
            if bucket_idx < bucket_count {
                buckets[bucket_idx].push((*idx, packet));
            }
        }
    }

    // Render timeline
    let mut timeline_lines = vec![];

    // Title
    timeline_lines.push(Line::from(vec![
        Span::styled("Packet Activity Over Time", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(format!("({} packets)", packet_count), Style::default().fg(Color::Gray)),
    ]));
    timeline_lines.push(Line::from(""));

    // Draw timeline rows (different packet types stacked)
    let packet_types = vec!["ADV_IND", "SCAN_REQ", "SCAN_RSP", "CONNECT_REQ", "DATA"];
    let type_colors = vec![
        Color::Green,
        Color::Cyan,
        Color::Blue,
        Color::Yellow,
        Color::Magenta,
    ];

    for (type_name, color) in packet_types.iter().zip(type_colors.iter()) {
        let mut line_spans = vec![
            Span::styled(format!("{:12} ", type_name), Style::default().fg(*color)),
        ];

        for bucket in &buckets {
            let count_of_type = bucket.iter()
                .filter(|(_, pkt)| {
                    pkt.get("packet_type")
                        .and_then(|v| v.as_str())
                        .map(|t| t == *type_name)
                        .unwrap_or(false)
                })
                .count();

            let symbol = if count_of_type == 0 {
                "·"
            } else if count_of_type == 1 {
                "▁"
            } else if count_of_type == 2 {
                "▃"
            } else if count_of_type <= 4 {
                "▅"
            } else {
                "█"
            };

            line_spans.push(Span::styled(symbol, Style::default().fg(*color)));
        }

        timeline_lines.push(Line::from(line_spans));
    }

    // Add a separator
    timeline_lines.push(Line::from(""));
    timeline_lines.push(Line::from(Span::styled(
        "─".repeat(timeline_width + 15),
        Style::default().fg(Color::DarkGray),
    )));

    // Density visualization (all packet types combined)
    timeline_lines.push(Line::from(""));
    timeline_lines.push(Line::from(Span::styled(
        "All Packets  ",
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));

    let mut density_line = vec![Span::raw("             ")];
    for bucket in &buckets {
        let total = bucket.len();
        let symbol = if total == 0 {
            " "
        } else if total == 1 {
            "░"
        } else if total <= 3 {
            "▒"
        } else if total <= 6 {
            "▓"
        } else {
            "█"
        };
        density_line.push(Span::styled(symbol, Style::default().fg(Color::White)));
    }
    timeline_lines.push(Line::from(density_line));

    // Time axis markers
    timeline_lines.push(Line::from(""));
    let mut axis_line = vec![Span::raw("             ")];
    for i in 0..bucket_count {
        if i % 10 == 0 {
            axis_line.push(Span::styled("|", Style::default().fg(Color::Gray)));
        } else {
            axis_line.push(Span::raw(" "));
        }
    }
    timeline_lines.push(Line::from(axis_line));

    // Time labels
    let mut label_line = vec![Span::styled("Time -->     ", Style::default().fg(Color::Gray))];
    for i in 0..bucket_count {
        if i % 20 == 0 && i > 0 {
            let packet_num = ((i as f64 / bucket_count as f64) * packet_count as f64) as usize;
            label_line.push(Span::styled(format!("{}", packet_num), Style::default().fg(Color::Gray)));
            // Add spacing
            for _ in 0..format!("{}", packet_num).len() {
                if i + 1 < bucket_count {
                    label_line.push(Span::raw(" "));
                }
            }
        }
    }
    timeline_lines.push(Line::from(label_line));

    let timeline_block = Paragraph::new(timeline_lines)
        .block(Block::default().borders(Borders::ALL).title(" Timeline View "))
        .wrap(Wrap { trim: false });
    f.render_widget(timeline_block, chunks[0]);

    // Legend and navigation
    let legend_lines = vec![
        Line::from(vec![
            Span::styled("Legend: ", Style::default().fg(Color::Yellow)),
            Span::styled("· ", Style::default().fg(Color::DarkGray)),
            Span::raw("none  "),
            Span::styled("▁ ", Style::default().fg(Color::White)),
            Span::raw("1  "),
            Span::styled("▃ ", Style::default().fg(Color::White)),
            Span::raw("2  "),
            Span::styled("▅ ", Style::default().fg(Color::White)),
            Span::raw("3-4  "),
            Span::styled("█ ", Style::default().fg(Color::White)),
            Span::raw("5+  "),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Packet Types: ", Style::default().fg(Color::Yellow)),
            Span::styled("■ ", Style::default().fg(Color::Green)),
            Span::raw("ADV  "),
            Span::styled("■ ", Style::default().fg(Color::Cyan)),
            Span::raw("SCAN_REQ  "),
            Span::styled("■ ", Style::default().fg(Color::Blue)),
            Span::raw("SCAN_RSP  "),
            Span::styled("■ ", Style::default().fg(Color::Yellow)),
            Span::raw("CONNECT  "),
            Span::styled("■ ", Style::default().fg(Color::Magenta)),
            Span::raw("DATA"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation: ", Style::default().fg(Color::Yellow)),
            Span::styled("l", Style::default().fg(Color::Cyan)),
            Span::raw(" list view  "),
            Span::styled("s", Style::default().fg(Color::Cyan)),
            Span::raw(" statistics  "),
            Span::styled("t", Style::default().fg(Color::Cyan)),
            Span::raw(" timeline"),
        ]),
    ];

    let legend_block = Paragraph::new(legend_lines)
        .block(Block::default().borders(Borders::ALL).title(" Legend "))
        .wrap(Wrap { trim: false });
    f.render_widget(legend_block, chunks[1]);
}
/// Render side-by-side packet comparison view
fn render_packet_comparison(f: &mut Frame, area: Rect, packets: &[serde_json::Value], state: &crate::tui::app::PacketListState) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph, Wrap},
    };

    // Check if we have exactly 2 packets marked for comparison
    if state.comparison_marks.len() != 2 {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Please mark exactly 2 packets for comparison",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::Gray)),
                Span::raw("Select a packet and press "),
                Span::styled("m", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" to mark it"),
            ]),
            Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::Gray)),
                Span::raw("Select another packet and press "),
                Span::styled("m", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" again"),
            ]),
            Line::from(vec![
                Span::styled("3. ", Style::default().fg(Color::Gray)),
                Span::raw("Press "),
                Span::styled("c", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" to open comparison view"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Currently marked: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{} packet(s)", state.comparison_marks.len()), Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("l", Style::default().fg(Color::Cyan)),
                Span::styled(" to return to list view", Style::default().fg(Color::Gray)),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Packet Comparison "))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    // Get the two marked packets
    let idx1 = state.comparison_marks[0];
    let idx2 = state.comparison_marks[1];

    let packet1 = packets.get(idx1);
    let packet2 = packets.get(idx2);

    if packet1.is_none() || packet2.is_none() {
        let text = vec![
            Line::from(""),
            Line::from("Error: Could not load marked packets"),
            Line::from(""),
        ];
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Error "))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let packet1 = packet1.unwrap();
    let packet2 = packet2.unwrap();

    // Split screen into two columns
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Helper function to extract field with default
    let get_field = |pkt: &serde_json::Value, field: &str, default: &str| {
        pkt.get(field).and_then(|v| v.as_str()).unwrap_or(default).to_string()
    };

    // Helper function to render one packet
    let render_packet_details = |pkt: &serde_json::Value, idx: usize, area: Rect, f: &mut Frame| {
        let frame_num = get_field(pkt, "frame_number", "?");
        let timestamp = get_field(pkt, "timestamp", "Unknown");
        let channel = get_field(pkt, "channel", "?");
        let rssi = get_field(pkt, "rssi", "?");
        let packet_type = get_field(pkt, "packet_type", "?");
        let mac_address = get_field(pkt, "mac_address", "N/A");
        let protocol = get_field(pkt, "protocol", "?");
        let summary = get_field(pkt, "summary", "");
        let access_addr = get_field(pkt, "access_addr", "Unknown");

        // Determine if fields differ
        let p1_frame = get_field(packet1, "frame_number", "?");
        let p2_frame = get_field(packet2, "frame_number", "?");
        let p1_channel = get_field(packet1, "channel", "?");
        let p2_channel = get_field(packet2, "channel", "?");
        let p1_rssi = get_field(packet1, "rssi", "?");
        let p2_rssi = get_field(packet2, "rssi", "?");
        let p1_type = get_field(packet1, "packet_type", "?");
        let p2_type = get_field(packet2, "packet_type", "?");
        let p1_mac = get_field(packet1, "mac_address", "N/A");
        let p2_mac = get_field(packet2, "mac_address", "N/A");
        let p1_protocol = get_field(packet1, "protocol", "?");
        let p2_protocol = get_field(packet2, "protocol", "?");
        let p1_access = get_field(packet1, "access_addr", "Unknown");
        let p2_access = get_field(packet2, "access_addr", "Unknown");

        let differ_channel = p1_channel != p2_channel;
        let differ_rssi = p1_rssi != p2_rssi;
        let differ_type = p1_type != p2_type;
        let differ_mac = p1_mac != p2_mac;
        let differ_protocol = p1_protocol != p2_protocol;
        let differ_access = p1_access != p2_access;

        let diff_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let same_style = Style::default().fg(Color::White);

        let mut lines = vec![
            Line::from(Span::styled(
                format!("Packet #{} (Index: {})", frame_num, idx),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Timestamp:    ", Style::default().fg(Color::Gray)),
                Span::styled(timestamp, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Channel:      ", Style::default().fg(Color::Gray)),
                Span::styled(channel, if differ_channel { diff_style } else { same_style }),
            ]),
            Line::from(vec![
                Span::styled("RSSI:         ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{} dBm", rssi), if differ_rssi { diff_style } else { same_style }),
            ]),
            Line::from(vec![
                Span::styled("Type:         ", Style::default().fg(Color::Gray)),
                Span::styled(packet_type.clone(), if differ_type { diff_style } else { same_style }),
            ]),
            Line::from(vec![
                Span::styled("Protocol:     ", Style::default().fg(Color::Gray)),
                Span::styled(protocol, if differ_protocol { diff_style } else { same_style }),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("MAC Address:  ", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(mac_address, if differ_mac { diff_style } else { same_style }),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Access Addr:  ", Style::default().fg(Color::Gray)),
                Span::styled(access_addr, if differ_access { diff_style } else { same_style }),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Summary:      ", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(summary, Style::default().fg(Color::White).add_modifier(Modifier::ITALIC)),
            ]),
        ];

        // Add annotation if present
        if let Some(note) = state.get_annotation(idx) {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Note:         ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(note.clone(), Style::default().fg(Color::White).add_modifier(Modifier::ITALIC)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Differences highlighted in yellow",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )));

        let title = format!(" Packet {} ", if idx == idx1 { "A" } else { "B" });
        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    };

    // Render both packets side by side
    render_packet_details(packet1, idx1, chunks[0], f);
    render_packet_details(packet2, idx2, chunks[1], f);
}

/// Render text input dialog overlay
fn render_dialog(f: &mut Frame, area: Rect, dialog: &TextInputDialog) {
    use ratatui::widgets::Clear;

    // Create centered dialog area
    let dialog_width = area.width.saturating_sub(20).min(80);
    let dialog_height = 10;
    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the background
    f.render_widget(Clear, dialog_area);

    // Split dialog into title, input, and help
    let dialog_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(4),     // Text input
            Constraint::Length(2),  // Help text
        ])
        .split(dialog_area);

    // Render title
    let title_text = format!(" {} ", dialog.title);
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));
    f.render_widget(title, dialog_chunks[0]);

    // Render textarea widget
    let widget = dialog.textarea.widget();
    f.render_widget(widget, dialog_chunks[1]);

    // Render help text
    let help_text = vec![
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" to submit  |  "),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" to cancel"),
        ]),
    ];
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT));
    f.render_widget(help, dialog_chunks[2]);
}

/// Render export menu
fn render_export_menu(f: &mut Frame, area: Rect, selected_index: usize, packet_count: usize, state: &crate::tui::app::PacketListState) {
    use crate::tui::app::ExportOption;

    let options = ExportOption::all();

    // Build menu items with availability info
    let items: Vec<ListItem> = options.iter().enumerate().map(|(idx, opt)| {
        let is_available = match opt {
            ExportOption::BookmarkedPackets => !state.bookmarks.is_empty(),
            ExportOption::FilteredPackets => state.follow_mac.is_some(),
            ExportOption::ComparisonReport => state.comparison_marks.len() == 2,
            _ => true,
        };

        let count_info = match opt {
            ExportOption::BookmarkedPackets => format!(" ({} bookmarks)", state.bookmarks.len()),
            ExportOption::FilteredPackets => {
                if let Some(ref mac) = state.follow_mac {
                    format!(" (following {})", mac)
                } else {
                    " (no filter active)".to_string()
                }
            }
            ExportOption::ComparisonReport => format!(" ({}/2 marked)", state.comparison_marks.len()),
            _ => format!(" ({} packets)", packet_count),
        };

        let label = format!("{}  {}", opt.label(), count_info);
        let description = opt.description();

        let style = if idx == selected_index {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if !is_available {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        let content = vec![
            Line::from(Span::styled(label, style)),
            Line::from(Span::styled(
                format!("  {}", description),
                Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
        ];

        ListItem::new(content)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Export Menu "))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    // Split into list and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(20),     // Export options list
            Constraint::Length(5),   // Help
        ])
        .split(area);

    f.render_widget(list, chunks[0]);

    // Help text
    let help_text = vec![
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" Export  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Exports are saved to ~/.ubertooth/exports/",
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .alignment(Alignment::Center);

    f.render_widget(help, chunks[1]);
}

/// Render filter dialog
fn render_filter_dialog(
    f: &mut Frame,
    area: Rect,
    selected_section: usize,
    selected_packet_type: usize,
    packet_type_selections: &std::collections::HashSet<String>,
    mac_filter: &str,
    rssi_min: &str,
    rssi_max: &str,
) {
    let packet_types = vec!["ADV_IND", "SCAN_REQ", "SCAN_RSP", "CONNECT_REQ", "DATA"];

    // Split into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),   // Packet types
            Constraint::Length(5),   // MAC filter
            Constraint::Length(5),   // RSSI range
            Constraint::Length(5),   // Actions
            Constraint::Min(1),      // Spacer
        ])
        .split(area);

    // Section 1: Packet Types (multi-select)
    let mut packet_type_lines = vec![
        Line::from(Span::styled(
            "Filter by Packet Type:",
            if selected_section == 0 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        )),
        Line::from(""),
    ];

    for (idx, pkt_type) in packet_types.iter().enumerate() {
        let is_selected = idx == selected_packet_type && selected_section == 0;
        let is_checked = packet_type_selections.contains(*pkt_type);

        let checkbox = if is_checked { "[✓]" } else { "[ ]" };
        let style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if is_checked {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };

        packet_type_lines.push(Line::from(Span::styled(
            format!("  {} {}", checkbox, pkt_type),
            style,
        )));
    }

    let packet_type_widget = Paragraph::new(packet_type_lines)
        .block(Block::default().borders(Borders::ALL).title(" Packet Types (Space to toggle) "));
    f.render_widget(packet_type_widget, chunks[0]);

    // Section 2: MAC Address Filter
    let mac_lines = vec![
        Line::from(Span::styled(
            "Filter by MAC Address:",
            if selected_section == 1 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        )),
        Line::from(""),
        Line::from(Span::styled(
            if mac_filter.is_empty() {
                "  (empty - no MAC filter)"
            } else {
                mac_filter
            },
            if selected_section == 1 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            },
        )),
    ];

    let mac_widget = Paragraph::new(mac_lines)
        .block(Block::default().borders(Borders::ALL).title(" MAC Address Filter "));
    f.render_widget(mac_widget, chunks[1]);

    // Section 3: RSSI Range
    let rssi_lines = vec![
        Line::from(Span::styled(
            "Filter by RSSI Range:",
            if selected_section == 2 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  Min: {} dBm | Max: {} dBm",
                if rssi_min.is_empty() { "(none)" } else { rssi_min },
                if rssi_max.is_empty() { "(none)" } else { rssi_max }
            ),
            if selected_section == 2 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            },
        )),
    ];

    let rssi_widget = Paragraph::new(rssi_lines)
        .block(Block::default().borders(Borders::ALL).title(" RSSI Range "));
    f.render_widget(rssi_widget, chunks[2]);

    // Section 4: Actions
    let actions_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [Enter] ",
                if selected_section == 3 {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                },
            ),
            Span::raw("Apply Filters    "),
            Span::styled(
                "[C] ",
                Style::default().fg(Color::Red),
            ),
            Span::raw("Clear All"),
        ]),
    ];

    let actions_widget = Paragraph::new(actions_lines)
        .block(Block::default().borders(Borders::ALL).title(" Actions "));
    f.render_widget(actions_widget, chunks[3]);
}
