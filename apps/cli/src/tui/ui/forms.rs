//! Form and dialog rendering functions

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::views::{FieldInputMode, FieldType};


/// Render tool parameter form
pub(crate) fn render_tool_form(
    f: &mut Frame,
    area: Rect,
    form: &crate::tui::views::ToolForm,
    error: Option<&str>,
) {
    // Split into sections: header, fields, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                                   // Header
            Constraint::Min(0),                                      // Fields
            Constraint::Length(if error.is_some() { 5 } else { 3 }), // Footer/error
        ])
        .split(area);

    // Header with tool name and description
    let header_text = format!("{}\n{}", form.tool_name(), form.tool_description());
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Tool Parameters"),
        );
    f.render_widget(header, chunks[0]);

    // Form fields
    let fields = form.fields();
    let inputs = form.inputs();
    let input_modes = form.input_modes();
    let focused = form.focused_index();

    if fields.is_empty() {
        let no_params =
            Paragraph::new("This tool has no parameters.\n\nPress Ctrl+Enter to execute.")
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

            let label = Paragraph::new(label_text).style(Style::default().fg(if i == focused {
                Color::Yellow
            } else {
                Color::White
            }));
            f.render_widget(label, field_chunks[label_idx]);

            // Input field - render dropdown or text input
            match &input_modes[i] {
                FieldInputMode::Dropdown { selected_index } => {
                    // Render dropdown
                    if let Some(options) = &field.dropdown_options {
                        let selected_value =
                            options.get(*selected_index).cloned().unwrap_or_default();
                        let display = format!("[ {} ]  (use Up/Down to change)", selected_value);

                        let dropdown_widget = Paragraph::new(display)
                            .style(Style::default().fg(if i == focused {
                                Color::Cyan
                            } else {
                                Color::White
                            }))
                            .block(Block::default().borders(Borders::ALL).border_style(
                                Style::default().fg(if i == focused {
                                    Color::Yellow
                                } else {
                                    Color::Gray
                                }),
                            ));

                        f.render_widget(dropdown_widget, field_chunks[input_idx]);
                    }
                }
                FieldInputMode::Text => {
                    // Render text input
                    let input_text = inputs[i].lines().join("\n");
                    let input_widget = Paragraph::new(input_text).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(if i == focused {
                                Color::Yellow
                            } else {
                                Color::Gray
                            })),
                    );

                    f.render_widget(input_widget, field_chunks[input_idx]);
                }
            }
        }
    }

    // Footer with instructions or error
    if let Some(err) = error {
        let error_text = format!(
            "Error: {}\n\n[Tab] Next field  [Enter] Execute  [Esc] Back",
            err
        );
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


/// Render text input dialog overlay

/// Render export menu
pub(crate) fn render_export_menu(
    f: &mut Frame,
    area: Rect,
    selected_index: usize,
    packet_count: usize,
    state: &crate::tui::app::PacketListState,
) {
    use crate::tui::app::ExportOption;

    let options = ExportOption::all();

    // Build menu items with availability info
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, opt)| {
            let is_available = match opt {
                ExportOption::BookmarkedPackets => !state.bookmarks.is_empty(),
                ExportOption::FilteredPackets => state.follow_mac.is_some(),
                ExportOption::ComparisonReport => state.comparison_marks.len() == 2,
                _ => true,
            };

            let count_info = match opt {
                ExportOption::BookmarkedPackets => {
                    format!(" ({} bookmarks)", state.bookmarks.len())
                }
                ExportOption::FilteredPackets => {
                    if let Some(ref mac) = state.follow_mac {
                        format!(" (following {})", mac)
                    } else {
                        " (no filter active)".to_string()
                    }
                }
                ExportOption::ComparisonReport => {
                    format!(" ({}/2 marked)", state.comparison_marks.len())
                }
                _ => format!(" ({} packets)", packet_count),
            };

            let label = format!("{}  {}", opt.label(), count_info);
            let description = opt.description();

            let style = if idx == selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if !is_available {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            let content = vec![
                Line::from(Span::styled(label, style)),
                Line::from(Span::styled(
                    format!("  {}", description),
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                )),
                Line::from(""),
            ];

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Export Menu "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    // Split into list and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(20),   // Export options list
            Constraint::Length(5), // Help
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
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .alignment(Alignment::Center);

    f.render_widget(help, chunks[1]);
}


/// Render filter dialog
pub(crate) fn render_filter_dialog(
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
            Constraint::Length(8), // Packet types
            Constraint::Length(5), // MAC filter
            Constraint::Length(5), // RSSI range
            Constraint::Length(5), // Actions
            Constraint::Min(1),    // Spacer
        ])
        .split(area);

    // Section 1: Packet Types (multi-select)
    let mut packet_type_lines = vec![
        Line::from(Span::styled(
            "Filter by Packet Type:",
            if selected_section == 0 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
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
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
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

    let packet_type_widget = Paragraph::new(packet_type_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Packet Types (Space to toggle) "),
    );
    f.render_widget(packet_type_widget, chunks[0]);

    // Section 2: MAC Address Filter
    let mac_lines = vec![
        Line::from(Span::styled(
            "Filter by MAC Address:",
            if selected_section == 1 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
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

    let mac_widget = Paragraph::new(mac_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" MAC Address Filter "),
    );
    f.render_widget(mac_widget, chunks[1]);

    // Section 3: RSSI Range
    let rssi_lines = vec![
        Line::from(Span::styled(
            "Filter by RSSI Range:",
            if selected_section == 2 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  Min: {} dBm | Max: {} dBm",
                if rssi_min.is_empty() {
                    "(none)"
                } else {
                    rssi_min
                },
                if rssi_max.is_empty() {
                    "(none)"
                } else {
                    rssi_max
                }
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
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                },
            ),
            Span::raw("Apply Filters    "),
            Span::styled("[C] ", Style::default().fg(Color::Red)),
            Span::raw("Clear All"),
        ]),
    ];

    let actions_widget = Paragraph::new(actions_lines)
        .block(Block::default().borders(Borders::ALL).title(" Actions "));
    f.render_widget(actions_widget, chunks[3]);
}

