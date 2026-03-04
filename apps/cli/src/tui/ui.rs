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

use super::app::{AppState, DeviceStatus, Notification};
use super::views::{Category, FieldInputMode, FieldType};

/// Render the entire UI
pub fn render(f: &mut Frame, state: &AppState, registry: &Arc<ToolRegistry>, device_status: &DeviceStatus, notification: &Option<Notification>) {
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
    render_content(f, chunks[1], state, registry, device_status);
    render_footer(f, chunks[2], state);

    // Render notification on top if present
    if let Some(notif) = notification {
        render_notification(f, f.size(), notif);
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
fn render_content(f: &mut Frame, area: Rect, state: &AppState, registry: &Arc<ToolRegistry>, device_status: &DeviceStatus) {
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
            render_executing(f, area, tool_name);
        }
        AppState::Results { tool_name, output, success, selected_capture, .. } => {
            render_results(f, area, tool_name, output, *success, *selected_capture);
        }
        AppState::Settings {} => {
            render_settings(f, area);
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
fn render_analysis_results(f: &mut Frame, area: Rect, output: &serde_json::Value) {
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    if let Some(analysis) = output.get("analysis").and_then(|a| a.as_object()) {
        // Protocol Summary
        lines.push(Line::from(Span::styled(
            "  Protocol Summary",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));

        if let Some(proto) = analysis.get("protocol_summary").and_then(|p| p.as_object()) {
            if let Some(ptype) = proto.get("type").and_then(|t| t.as_str()) {
                lines.push(Line::from(vec![
                    Span::raw("    Type: "),
                    Span::styled(ptype, Style::default().fg(Color::Cyan)),
                ]));
            }
        }
        lines.push(Line::from(""));

        // Devices Found
        if let Some(devices) = analysis.get("devices").and_then(|d| d.as_array()) {
            lines.push(Line::from(Span::styled(
                format!("  Devices Found: {}", devices.len()),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));

            if devices.is_empty() {
                lines.push(Line::from(Span::styled(
                    "    No devices detected",
                    Style::default().fg(Color::Gray),
                )));
            } else {
                for device in devices {
                    if let Some(mac) = device.get("mac_address").and_then(|m| m.as_str()) {
                        let name = device.get("device_name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                        let pkts = device.get("packet_count").and_then(|p| p.as_u64()).unwrap_or(0);
                        lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled(mac, Style::default().fg(Color::Cyan)),
                            Span::raw(" - "),
                            Span::styled(name, Style::default().fg(Color::White)),
                            Span::styled(format!(" ({} pkts)", pkts), Style::default().fg(Color::Gray)),
                        ]));
                    }
                }
            }
            lines.push(Line::from(""));
        }

        // Timing Analysis
        if let Some(timing) = analysis.get("timing_analysis").and_then(|t| t.as_object()) {
            lines.push(Line::from(Span::styled(
                "  Timing Analysis",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));

            if let Some(avg) = timing.get("avg_interval_ms").and_then(|a| a.as_f64()) {
                if avg > 0.0 {
                    lines.push(Line::from(format!("    Average Interval: {:.2}ms", avg)));
                }
            }
            lines.push(Line::from(""));
        }

        // Security Observations
        if let Some(security) = analysis.get("security_observations").and_then(|s| s.as_array()) {
            if !security.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Security Observations",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
                for obs in security {
                    if let Some(note) = obs.as_str() {
                        lines.push(Line::from(format!("    • {}", note)));
                    }
                }
                lines.push(Line::from(""));
            }
        }

        // Note (Phase indicator)
        if let Some(note) = analysis.get("note").and_then(|n| n.as_str()) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  {}", note),
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            )));
        }
    }

    let content = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("Analysis Results"));
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
fn render_executing(f: &mut Frame, area: Rect, tool_name: &str) {
    let text = format!(
        "Executing tool: {}\n\n\
        Please wait...\n\n\
        This may take a few seconds depending on the tool.\n\n\
        Working...",
        tool_name
    );
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Executing"));

    f.render_widget(paragraph, area);
}

/// Render tool results
fn render_results(f: &mut Frame, area: Rect, tool_name: &str, output: &serde_json::Value, success: bool, selected_capture: Option<usize>) {
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
        render_analysis_results(f, chunks[1], output);
        return;
    }

    // Special formatting for capture_get - show capture details
    if tool_name == "capture_get" && success {
        render_capture_details(f, chunks[1], output);
        return;
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
fn render_settings(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(Span::styled("Strike48 / Prospector Studio Connection", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Server URL: ", Style::default().fg(Color::Yellow)),
            Span::raw("wss://jt-demo-01.strike48.engineering"),
        ]),
        Line::from(vec![
            Span::styled("Tenant ID:  ", Style::default().fg(Color::Yellow)),
            Span::raw("non-prod"),
        ]),
        Line::from(vec![
            Span::styled("Auth Token: ", Style::default().fg(Color::Yellow)),
            Span::styled("(not configured)", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Backend Configuration", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Backend:    ", Style::default().fg(Color::Yellow)),
            Span::raw("Rust (native USB) with Python fallback"),
        ]),
        Line::from(vec![
            Span::styled("Device:     ", Style::default().fg(Color::Yellow)),
            Span::raw("Auto-detect first Ubertooth"),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("Tip:", Style::default().fg(Color::Blue))),
        Line::from(Span::raw("  Settings are loaded from ~/.ubertooth/config.toml")),
        Line::from(Span::raw("  You can edit this file directly or use the CLI tool.")),
        Line::from(""),
        Line::from(Span::styled("  For Strike48 agent mode:", Style::default().fg(Color::Gray))),
        Line::from(Span::raw("  Use 'ubertooth-agent' instead of 'ubertooth-cli --tui'")),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Settings")
            .title_style(Style::default().fg(Color::Cyan)));

    f.render_widget(paragraph, area);
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
