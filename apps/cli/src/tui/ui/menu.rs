//! Menu rendering functions

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::sync::Arc;
use ubertooth_core::ToolRegistry;

use crate::tui::app::{AppState, DeviceStatus};
use crate::tui::themes::Theme;
use crate::tui::views::Category;


/// Render main menu with 7 categories
pub(crate) fn render_main_menu(
    f: &mut Frame,
    area: Rect,
    selected_index: usize,
    device_status: &DeviceStatus,
) {
    // First item is the dynamic connection toggle
    let connection_label = if device_status.connected {
        "[Disconnect from Ubertooth]"
    } else {
        "[Connect to Ubertooth]"
    };

    let categories = vec![
        (connection_label, "Toggle device connection"),
        ("1. Captures", "Manage all captures with hotkeys"),
        (
            "2. Reconnaissance (7 tools)",
            "BLE scan, spectrum analysis, follow connections",
        ),
        (
            "3. Analysis (5 tools)",
            "Packet analysis, fingerprinting, comparison",
        ),
        (
            "4. Attack Operations (5 tools)",
            "Injection, jamming, MITM (requires authorization)",
        ),
        (
            "5. Configuration (8 tools)",
            "Channel, power, modulation, presets",
        ),
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
                    Span::styled(
                        *title,
                        Style::default()
                            .fg(final_text_color)
                            .add_modifier(final_modifier),
                    ),
                ]);

                content.push(title_line);
            } else {
                // Regular category items
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                content.push(Line::from(Span::styled(*title, style)));
            }

            content.push(Line::from(Span::styled(
                format!("   {}", desc),
                Style::default().fg(Color::Gray),
            )));
            content.push(Line::from("")); // Standard spacing after each item

            ListItem::new(Text::from(content))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Select Tool Category"),
    );

    f.render_widget(list, area);
}


/// Render tool category submenu
pub(crate) fn render_tool_category(
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
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Add numbering to tool names
            let numbered_name = format!("{}. {}", i + 1, tool.name());

            let content = vec![
                Line::from(Span::styled(numbered_name, style)),
                Line::from(Span::styled(
                    format!("   {}", tool.description()),
                    Style::default().fg(Color::Gray),
                )),
                Line::from(""),
            ];

            ListItem::new(Text::from(content))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("{:?} - Select Tool", category)),
    );

    f.render_widget(list, area);
}


/// Render tool with hotkey parameter configuration
pub(crate) fn render_tool_hotkeys(
    f: &mut Frame,
    area: Rect,
    form: &crate::tui::views::ToolForm,
    error: Option<&str>,
) {
    // Split into: header, content, hotkey bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(0),    // Content/description
            Constraint::Length(5), // Hotkey parameter bar
        ])
        .split(area);

    // Header with tool name
    let header_text = format!("{}\n\n{}", form.tool_name(), form.tool_description());
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Tool"));
    f.render_widget(header, chunks[0]);

    // Content area - show current parameter values
    let mut content_lines = Vec::new();
    content_lines.push(Line::from(""));
    content_lines.push(Line::from(Span::styled(
        "  Current Configuration:",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    content_lines.push(Line::from(""));

    // Show current parameter values
    for field in form.fields() {
        let value_str = form
            .get_field_value(&field.name)
            .unwrap_or_else(|| "<empty>".to_string());
        content_lines.push(Line::from(vec![
            Span::styled(
                format!("    {}: ", field.name),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(value_str, Style::default().fg(Color::White)),
        ]));
    }

    let content = Paragraph::new(Text::from(content_lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Configuration"),
    );
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

