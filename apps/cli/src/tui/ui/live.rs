//! Live capture view rendering

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::sync::Arc;



/// Render help overlay with keyboard shortcuts
pub(crate) fn render_live_capture(
    f: &mut Frame,
    area: Rect,
    buffer: &Arc<ubertooth_platform::StreamingBuffer>,
    stats: &ubertooth_platform::BufferStats,
    paused: bool,
    limits: &ubertooth_platform::CaptureLimits,
    selected_index: usize,
    scroll_offset: usize,
    tool_name: &str,
) {
    // Split into header, packets, and stats
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header with stats
            Constraint::Min(10),   // Packet list
            Constraint::Length(8), // Statistics panel
        ])
        .split(area);

    // Render header with capture stats
    render_live_capture_header(f, chunks[0], stats, paused, limits, tool_name);

    // Render packet list
    render_live_packet_list(f, chunks[1], buffer, selected_index, scroll_offset);

    // Render live statistics
    render_live_statistics(f, chunks[2], stats, limits);
}


/// Render live capture header
pub(crate) fn render_live_capture_header(
    f: &mut Frame,
    area: Rect,
    stats: &ubertooth_platform::BufferStats,
    paused: bool,
    limits: &ubertooth_platform::CaptureLimits,
    tool_name: &str,
) {
    let status_text = if paused { "PAUSED" } else { "CAPTURING" };
    let status_color = if paused { Color::Yellow } else { Color::Green };

    let duration = stats.duration_seconds();
    let pps = stats.packets_per_second;

    let header_text = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::White)),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("Tool: {}", tool_name),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("Packets: {} ", stats.total_packets),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("| Buffered: {} ", stats.buffered_packets),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("| Dropped: {} ", stats.dropped_packets),
                Style::default().fg(Color::Red),
            ),
            Span::styled(
                format!("| Rate: {:.1} p/s ", pps),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("| Duration: {:.1}s", duration),
                Style::default().fg(Color::Gray),
            ),
        ]),
    ];

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Live Capture "),
    );
    f.render_widget(header, area);
}


/// Render live packet list
pub(crate) fn render_live_packet_list(
    f: &mut Frame,
    area: Rect,
    buffer: &Arc<ubertooth_platform::StreamingBuffer>,
    selected_index: usize,
    scroll_offset: usize,
) {
    // Get recent packets from buffer
    let packets = buffer.get_recent_packets(100).unwrap_or_default();

    if packets.is_empty() {
        let empty = Paragraph::new("No packets captured yet...")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title(" Packets "))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    // Build packet list items
    let items: Vec<ListItem> = packets
        .iter()
        .skip(scroll_offset)
        .take(area.height as usize - 2)
        .map(|pkt| {
            let timestamp = pkt.timestamp.format("%H:%M:%S%.3f");
            let rssi_str = pkt.rssi.map_or("N/A".to_string(), |r| format!("{:3}", r));

            let line = Line::from(vec![
                Span::styled(
                    format!("{:8} ", pkt.sequence),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(format!("{} ", timestamp), Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("Ch{:2} ", pkt.channel),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("RSSI:{} ", rssi_str),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(
                    format!("{:12} ", pkt.packet_type),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!("{} bytes", pkt.data.len()),
                    Style::default().fg(Color::White),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Packets ({} total) ", packets.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_widget(list, area);
}


/// Render live statistics panel
pub(crate) fn render_live_statistics(
    f: &mut Frame,
    area: Rect,
    stats: &ubertooth_platform::BufferStats,
    limits: &ubertooth_platform::CaptureLimits,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left side: Throughput stats
    let throughput_text = vec![
        Line::from(Span::styled(
            "Throughput",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Packets/sec: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.1}", stats.packets_per_second),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Avg bytes/pkt: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.0}", stats.avg_bytes_per_packet()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Total data: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.2} MB", stats.total_bytes as f64 / 1_048_576.0),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let throughput = Paragraph::new(throughput_text)
        .block(Block::default().borders(Borders::ALL).title(" Throughput "));
    f.render_widget(throughput, chunks[0]);

    // Right side: Buffer usage
    let buffer_percent = (stats.buffered_packets as f64 / limits.max_packets as f64) * 100.0;
    let memory_percent = (stats.memory_usage as f64 / limits.max_memory_bytes as f64) * 100.0;

    let buffer_color = if buffer_percent > 90.0 {
        Color::Red
    } else if buffer_percent > 70.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let memory_color = if memory_percent > 90.0 {
        Color::Red
    } else if memory_percent > 70.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let buffer_text = vec![
        Line::from(Span::styled(
            "Buffer Usage",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Packets: ", Style::default().fg(Color::White)),
            Span::styled(
                format!(
                    "{}/{} ({:.1}%)",
                    stats.buffered_packets, limits.max_packets, buffer_percent
                ),
                Style::default().fg(buffer_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("Memory: ", Style::default().fg(Color::White)),
            Span::styled(
                format!(
                    "{:.1}/{:.1} MB ({:.1}%)",
                    stats.memory_usage as f64 / 1_048_576.0,
                    limits.max_memory_bytes as f64 / 1_048_576.0,
                    memory_percent
                ),
                Style::default().fg(memory_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("Dropped: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", stats.dropped_packets),
                Style::default().fg(Color::Red),
            ),
        ]),
    ];

    let buffer_panel =
        Paragraph::new(buffer_text).block(Block::default().borders(Borders::ALL).title(" Buffer "));
    f.render_widget(buffer_panel, chunks[1]);
}

