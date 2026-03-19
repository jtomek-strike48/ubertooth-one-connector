//! Packet timeline visualization

use ratatui::{
    layout::Rect,
    Frame,
};

/// Render packet timeline view
pub(crate) fn render_packet_timeline(
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

    let packet_count = packets.len();
    if packet_count == 0 {
        return;
    }

    // Split area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Timeline
            Constraint::Length(8), // Legend and stats
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
        Span::styled(
            "Packet Activity Over Time",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("({} packets)", packet_count),
            Style::default().fg(Color::Gray),
        ),
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
        let mut line_spans = vec![Span::styled(
            format!("{:12} ", type_name),
            Style::default().fg(*color),
        )];

        for bucket in &buckets {
            let count_of_type = bucket
                .iter()
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
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
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
    let mut label_line = vec![Span::styled(
        "Time -->     ",
        Style::default().fg(Color::Gray),
    )];
    for i in 0..bucket_count {
        if i % 20 == 0 && i > 0 {
            let packet_num = ((i as f64 / bucket_count as f64) * packet_count as f64) as usize;
            label_line.push(Span::styled(
                format!("{}", packet_num),
                Style::default().fg(Color::Gray),
            ));
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Timeline View "),
        )
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

