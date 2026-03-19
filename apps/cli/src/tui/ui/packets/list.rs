//! Packet list rendering with filtering

use ratatui::{
    layout::Rect,
    Frame,
};



/// Render packet list view (original table view)
pub(crate) fn render_packet_list(
    f: &mut Frame,
    area: Rect,
    packets: &[serde_json::Value],
    state: &crate::tui::app::PacketListState,
) {
    use ratatui::{
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph, Wrap},
    };

    let packet_count = packets.len();

    // Apply all active filters
    let filtered_packets: Vec<(usize, &serde_json::Value)> = packets
        .iter()
        .enumerate()
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
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Decoded Packets (Filtered)"),
            )
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
        Span::styled(
            " # ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Time          ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Ch ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "RSSI ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Type          ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "MAC Address             ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Proto ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Summary",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
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
            let frame_num = packet
                .get("frame_number")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let timestamp = packet
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let time_short = timestamp.split(',').last().unwrap_or(timestamp).trim();
            let time_display = if time_short.len() > 12 {
                &time_short[time_short.len() - 12..]
            } else {
                time_short
            };

            let channel = packet
                .get("channel")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let rssi = packet.get("rssi").and_then(|v| v.as_str()).unwrap_or("?");
            let packet_type = packet
                .get("packet_type")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let mac_address = packet
                .get("mac_address")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A");
            let protocol = packet
                .get("protocol")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
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
                if is_expanded {
                    "▼"
                } else {
                    "▶"
                }
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
                Span::styled(
                    bookmark_indicator,
                    Style::default().fg(Color::Yellow).bg(bg_color),
                ),
                Span::styled(
                    mark_indicator,
                    Style::default().fg(Color::Magenta).bg(bg_color),
                ),
                Span::styled(
                    expand_indicator,
                    Style::default().fg(Color::Cyan).bg(bg_color),
                ),
                Span::styled(
                    format!("{:3} ", frame_num),
                    Style::default().fg(Color::Gray).bg(bg_color),
                ),
                Span::styled(
                    format!("{:12} ", time_display),
                    Style::default().fg(Color::White).bg(bg_color),
                ),
                Span::styled(
                    format!("{:2} ", channel),
                    Style::default().fg(Color::Magenta).bg(bg_color),
                ),
                Span::styled(
                    format!("{:4} ", rssi),
                    Style::default().fg(Color::Red).bg(bg_color),
                ),
                Span::styled(
                    format!("{:13} ", packet_type),
                    Style::default().fg(type_color).bg(bg_color),
                ),
                Span::styled(
                    format!("{:23} ", mac_address),
                    Style::default().fg(Color::Cyan).bg(bg_color),
                ),
                Span::styled(
                    format!("{:5} ", protocol),
                    Style::default().fg(Color::Blue).bg(bg_color),
                ),
                Span::styled(summary, Style::default().fg(Color::White).bg(bg_color)),
            ]));

            // Expanded view
            if is_expanded {
                let access_addr = packet
                    .get("access_addr")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

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
                                Span::styled(
                                    layer_names.join(" → "),
                                    Style::default().fg(Color::Cyan),
                                ),
                            ]));
                        }
                    }
                }

                // Show annotation if present
                if let Some(note) = state.get_annotation(*original_idx) {
                    lines.push(Line::from(vec![
                        Span::raw("  │ "),
                        Span::styled(
                            "Note: ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            note.clone(),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ]));
                }

                lines.push(Line::from(vec![
                    Span::raw("  │ "),
                    Span::styled(
                        "[Enter: collapse | n: add note | Del: remove note]",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    ),
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
        Span::styled(
            format!(
                "Showing {}-{} of {}",
                start_idx + 1,
                end_idx,
                displayed_count
            ),
            Style::default().fg(Color::Gray),
        ),
        if displayed_count < packet_count {
            Span::styled(
                format!(" (filtered from {})", packet_count),
                Style::default().fg(Color::Yellow),
            )
        } else {
            Span::raw("")
        },
    ]));

    // Follow stream indicator
    if let Some(ref mac) = state.follow_mac {
        lines.push(Line::from(vec![
            Span::styled("Following: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                mac.clone(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "(press 'f' to clear)",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
    }

    // Bookmark indicator
    let bookmark_count = state.bookmarks.len();
    if bookmark_count > 0 {
        lines.push(Line::from(vec![
            Span::styled("★ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{} bookmarked packet{}",
                    bookmark_count,
                    if bookmark_count == 1 { "" } else { "s" }
                ),
                Style::default().fg(Color::White),
            ),
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
            let min = state
                .filters
                .rssi_min
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".to_string());
            let max = state
                .filters
                .rssi_max
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".to_string());
            filter_parts.push(format!("RSSI: {} to {} dBm", min, max));
        }

        lines.push(Line::from(vec![
            Span::styled(
                "🔍 Active Filters: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(filter_parts.join(" | "), Style::default().fg(Color::White)),
            Span::raw("  "),
            Span::styled(
                "(press '/' to modify)",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
    }

    let title = format!(" Decoded Packets ({} total) ", packet_count);
    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

