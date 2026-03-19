//! Packet statistics rendering

use ratatui::{
    layout::Rect,
    Frame,
};

/// Render packet statistics view
pub(crate) fn render_packet_statistics(
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
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
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
            Span::styled(
                format!(" {} ({:.1}%)", count, percentage),
                Style::default().fg(Color::Gray),
            ),
        ]));
    }

    let type_block = Paragraph::new(type_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Packet Types "),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(type_block, chunks[0]);

    // Render channel and RSSI stats
    let mut stats_lines = vec![
        Line::from(vec![
            Span::styled("Total Packets: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                packet_count.to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Unique MACs:   ", Style::default().fg(Color::Yellow)),
            Span::styled(
                mac_addresses.len().to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("RSSI Stats:    ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("Avg: {:.1} dBm", avg_rssi),
                Style::default().fg(Color::White),
            ),
            Span::raw("  "),
            Span::styled(
                format!("Min: {} dBm", min_rssi),
                Style::default().fg(Color::Red),
            ),
            Span::raw("  "),
            Span::styled(
                format!("Max: {} dBm", max_rssi),
                Style::default().fg(Color::Green),
            ),
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
            Span::styled(
                format!("  Ch {:2}:", ch),
                Style::default().fg(Color::Magenta),
            ),
            Span::styled(
                format!(" {} packets ({:.1}%)", count, percentage),
                Style::default().fg(Color::White),
            ),
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
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
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
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    mac_lines.push(Line::from(""));
    mac_lines.push(Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::Gray)),
        Span::styled("l", Style::default().fg(Color::Cyan)),
        Span::styled(
            " to return to packet list view",
            Style::default().fg(Color::Gray),
        ),
    ]));

    let mac_block = Paragraph::new(mac_lines)
        .block(Block::default().borders(Borders::ALL).title(" Devices "))
        .wrap(Wrap { trim: false });
    f.render_widget(mac_block, chunks[2]);
}

