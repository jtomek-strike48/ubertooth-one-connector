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

use super::app::AppState;
use super::views::{Category, FieldType};

/// Render the entire UI
pub fn render(f: &mut Frame, state: &AppState, registry: &Arc<ToolRegistry>) {
    // Main layout: header + content + footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(f.size());

    render_header(f, chunks[0], registry);
    render_content(f, chunks[1], state, registry);
    render_footer(f, chunks[2], state);
}

/// Render header with device status
fn render_header(f: &mut Frame, area: Rect, _registry: &Arc<ToolRegistry>) {
    let header = Paragraph::new("Ubertooth CLI")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Device: Not Connected | Backend: Rust | Strike48: Not Connected"));

    f.render_widget(header, area);
}

/// Render main content based on state
fn render_content(f: &mut Frame, area: Rect, state: &AppState, registry: &Arc<ToolRegistry>) {
    match state {
        AppState::MainMenu { selected_index } => {
            render_main_menu(f, area, *selected_index);
        }
        AppState::ToolCategory { category, selected_index } => {
            render_tool_category(f, area, category, *selected_index, registry);
        }
        AppState::ToolForm { form, error } => {
            render_tool_form(f, area, form.as_ref(), error.as_deref());
        }
        AppState::Executing { tool_name, .. } => {
            render_executing(f, area, tool_name);
        }
        AppState::Results { tool_name, output, success } => {
            render_results(f, area, tool_name, output, *success);
        }
        AppState::Settings {} => {
            render_settings(f, area);
        }
    }
}

/// Render main menu with 7 categories
fn render_main_menu(f: &mut Frame, area: Rect, selected_index: usize) {
    let categories = vec![
        ("1. Device Management (3 tools)", "Connect, status, disconnect"),
        ("2. Reconnaissance (7 tools)", "BLE scan, spectrum analysis, follow connections"),
        ("3. Analysis (5 tools)", "Packet analysis, fingerprinting, comparison"),
        ("4. Capture Management (5 tools)", "List, view, export, tag captures"),
        ("5. Configuration (8 tools)", "Channel, power, modulation, presets"),
        ("6. Attack Operations (5 tools)", "Injection, jamming, MITM (requires authorization)"),
        ("7. Advanced (2 tools)", "Raw USB commands, session context"),
    ];

    let items: Vec<ListItem> = categories
        .iter()
        .enumerate()
        .map(|(i, (title, desc))| {
            let style = if i == selected_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = vec![
                Line::from(Span::styled(*title, style)),
                Line::from(Span::styled(format!("   {}", desc), Style::default().fg(Color::Gray))),
                Line::from(""),
            ];

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
) {
    let tools = category.get_tools(registry);

    let items: Vec<ListItem> = tools
        .iter()
        .enumerate()
        .map(|(i, tool)| {
            let style = if i == selected_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = vec![
                Line::from(Span::styled(tool.name(), style)),
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

            // Input field - render text content in a bordered paragraph
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

    // Footer with instructions or error
    if let Some(err) = error {
        let error_text = format!("Error: {}\n\n[Tab] Next field  [Enter] Execute  [Esc] Back", err);
        let footer = Paragraph::new(error_text)
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
    } else {
        let footer_text = "[Tab] Next field  [Shift+Tab] Previous  [Enter] Execute  [Esc] Back";
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
    }
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
fn render_results(f: &mut Frame, area: Rect, tool_name: &str, output: &serde_json::Value, success: bool) {
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
        AppState::MainMenu { .. } | AppState::ToolCategory { .. } => {
            "[Up/Down] Navigate  [Enter] Select  [Esc] Back  [q] Quit  [s] Settings"
        }
        AppState::ToolForm { .. } => {
            "[Tab] Next  [Enter] Execute  [Esc] Cancel"
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
