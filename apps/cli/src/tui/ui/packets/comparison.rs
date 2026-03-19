//! Packet comparison rendering

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render side-by-side packet comparison view
pub(crate) fn render_packet_comparison(
    f: &mut Frame,
    area: Rect,
    packets: &[serde_json::Value],
    state: &crate::tui::app::PacketListState,
) {
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
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::Gray)),
                Span::raw("Select a packet and press "),
                Span::styled(
                    "m",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to mark it"),
            ]),
            Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::Gray)),
                Span::raw("Select another packet and press "),
                Span::styled(
                    "m",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" again"),
            ]),
            Line::from(vec![
                Span::styled("3. ", Style::default().fg(Color::Gray)),
                Span::raw("Press "),
                Span::styled(
                    "c",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to open comparison view"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Currently marked: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{} packet(s)", state.comparison_marks.len()),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("l", Style::default().fg(Color::Cyan)),
                Span::styled(" to return to list view", Style::default().fg(Color::Gray)),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Packet Comparison "),
            )
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
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Helper function to extract field with default
    let get_field = |pkt: &serde_json::Value, field: &str, default: &str| {
        pkt.get(field)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
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

        let diff_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let same_style = Style::default().fg(Color::White);

        let mut lines = vec![
            Line::from(Span::styled(
                format!("Packet #{} (Index: {})", frame_num, idx),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Timestamp:    ", Style::default().fg(Color::Gray)),
                Span::styled(timestamp, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Channel:      ", Style::default().fg(Color::Gray)),
                Span::styled(
                    channel,
                    if differ_channel {
                        diff_style
                    } else {
                        same_style
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("RSSI:         ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} dBm", rssi),
                    if differ_rssi { diff_style } else { same_style },
                ),
            ]),
            Line::from(vec![
                Span::styled("Type:         ", Style::default().fg(Color::Gray)),
                Span::styled(
                    packet_type.clone(),
                    if differ_type { diff_style } else { same_style },
                ),
            ]),
            Line::from(vec![
                Span::styled("Protocol:     ", Style::default().fg(Color::Gray)),
                Span::styled(
                    protocol,
                    if differ_protocol {
                        diff_style
                    } else {
                        same_style
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "MAC Address:  ",
                Style::default().fg(Color::Gray),
            )]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    mac_address,
                    if differ_mac { diff_style } else { same_style },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Access Addr:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    access_addr,
                    if differ_access {
                        diff_style
                    } else {
                        same_style
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Summary:      ",
                Style::default().fg(Color::Gray),
            )]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    summary,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]),
        ];

        // Add annotation if present
        if let Some(note) = state.get_annotation(idx) {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Note:         ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    note.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Differences highlighted in yellow",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
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

