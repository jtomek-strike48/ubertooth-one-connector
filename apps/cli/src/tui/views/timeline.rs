//! Packet timeline visualization.
//!
//! Built as part of Phase 3.2 - Ready for future TUI integration.

#![allow(dead_code)]

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use ubertooth_platform::PacketData;

/// Timeline view configuration
#[derive(Debug, Clone)]
pub struct TimelineConfig {
    /// Time window in seconds
    pub time_window_secs: u64,
    /// Number of time bins
    pub time_bins: usize,
    /// Show packet details
    pub show_details: bool,
    /// Group by channel
    pub group_by_channel: bool,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            time_window_secs: 60,
            time_bins: 30,
            show_details: true,
            group_by_channel: false,
        }
    }
}

/// Timeline data structure
pub struct TimelineData {
    /// Time bins with packet counts
    bins: Vec<TimelineBin>,
    /// Total packets
    total_packets: usize,
    /// Start time label
    start_time: String,
    /// End time label
    end_time: String,
}

/// Single time bin
#[derive(Debug, Clone)]
pub struct TimelineBin {
    /// Time label
    pub label: String,
    /// Packet count
    pub count: u64,
    /// Channel distribution
    pub channels: std::collections::HashMap<u8, u32>,
    /// Average RSSI
    pub avg_rssi: Option<f32>,
}

impl TimelineData {
    /// Create timeline from packets
    pub fn from_packets(packets: &[PacketData], config: &TimelineConfig) -> Self {
        if packets.is_empty() {
            return Self {
                bins: Vec::new(),
                total_packets: 0,
                start_time: "N/A".to_string(),
                end_time: "N/A".to_string(),
            };
        }

        let first_time = packets.first().unwrap().timestamp;
        let last_time = packets.last().unwrap().timestamp;
        let duration_ms = (last_time - first_time).num_milliseconds().max(1);
        let bin_duration_ms = duration_ms / config.time_bins as i64;

        let mut bins = vec![
            TimelineBin {
                label: String::new(),
                count: 0,
                channels: std::collections::HashMap::new(),
                avg_rssi: None,
            };
            config.time_bins
        ];

        // Populate bins
        for packet in packets {
            let elapsed_ms = (packet.timestamp - first_time).num_milliseconds();
            let bin_idx = ((elapsed_ms / bin_duration_ms) as usize).min(config.time_bins - 1);

            bins[bin_idx].count += 1;
            *bins[bin_idx].channels.entry(packet.channel).or_insert(0) += 1;

            if let Some(rssi) = packet.rssi {
                let count = bins[bin_idx].count as f32;
                let current_avg = bins[bin_idx].avg_rssi.get_or_insert(0.0);
                *current_avg = (*current_avg * (count - 1.0) + rssi as f32) / count;
            }
        }

        // Generate labels
        for (i, bin) in bins.iter_mut().enumerate() {
            let secs = (i * bin_duration_ms as usize) / 1000;
            bin.label = format!("{}s", secs);
        }

        Self {
            bins,
            total_packets: packets.len(),
            start_time: first_time.format("%H:%M:%S").to_string(),
            end_time: last_time.format("%H:%M:%S").to_string(),
        }
    }

    /// Get maximum count for scaling
    pub fn max_count(&self) -> u64 {
        self.bins.iter().map(|b| b.count).max().unwrap_or(0)
    }

    /// Get bin at index
    pub fn bin_at(&self, index: usize) -> Option<&TimelineBin> {
        self.bins.get(index)
    }

    /// Get statistics
    pub fn stats(&self) -> TimelineStats {
        let max_count = self.max_count();
        let avg_count = if !self.bins.is_empty() {
            self.total_packets as f64 / self.bins.len() as f64
        } else {
            0.0
        };

        let all_channels: std::collections::HashSet<u8> = self
            .bins
            .iter()
            .flat_map(|b| b.channels.keys().copied())
            .collect();

        let avg_rssi = {
            let rssi_values: Vec<f32> = self.bins.iter().filter_map(|b| b.avg_rssi).collect();

            if !rssi_values.is_empty() {
                Some(rssi_values.iter().sum::<f32>() / rssi_values.len() as f32)
            } else {
                None
            }
        };

        TimelineStats {
            total_packets: self.total_packets,
            max_bin_count: max_count,
            avg_bin_count: avg_count,
            unique_channels: all_channels.len(),
            avg_rssi,
        }
    }
}

/// Timeline statistics
#[derive(Debug, Clone)]
pub struct TimelineStats {
    pub total_packets: usize,
    pub max_bin_count: u64,
    pub avg_bin_count: f64,
    pub unique_channels: usize,
    pub avg_rssi: Option<f32>,
}

/// Render packet timeline
pub fn render_timeline(
    frame: &mut Frame,
    area: Rect,
    packets: &[PacketData],
    config: &TimelineConfig,
    title: &str,
) {
    let timeline = TimelineData::from_packets(packets, config);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(12),   // Chart
            Constraint::Length(6), // Stats
        ])
        .split(area);

    // Render header
    render_timeline_header(frame, chunks[0], title, &timeline);

    // Render chart
    render_timeline_chart(frame, chunks[1], &timeline, config);

    // Render stats
    render_timeline_stats(frame, chunks[2], &timeline);
}

/// Render timeline header
fn render_timeline_header(frame: &mut Frame, area: Rect, title: &str, timeline: &TimelineData) {
    let header_text = vec![
        Line::from(vec![Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::raw("Period: "),
            Span::styled(
                format!("{} - {}", timeline.start_time, timeline.end_time),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  Total: "),
            Span::styled(
                timeline.total_packets.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    let header =
        Paragraph::new(header_text).block(Block::default().borders(Borders::ALL).title("Timeline"));

    frame.render_widget(header, area);
}

/// Render timeline bar chart
fn render_timeline_chart(
    frame: &mut Frame,
    area: Rect,
    timeline: &TimelineData,
    config: &TimelineConfig,
) {
    let available_width = area.width.saturating_sub(2) as usize;
    let bins_to_display = timeline.bins.len().min(available_width / 3); // 3 chars per bar
    let bin_skip = if bins_to_display > 0 {
        timeline.bins.len() / bins_to_display
    } else {
        1
    }
    .max(1);

    // Prepare bar chart data
    let bars: Vec<(&str, u64)> = timeline
        .bins
        .iter()
        .step_by(bin_skip)
        .map(|bin| (bin.label.as_str(), bin.count))
        .collect();

    let max_value = timeline.max_count();

    // Convert to Bar structs
    let bar_data: Vec<Bar> = bars
        .iter()
        .map(|(label, value)| {
            let color = if *value > max_value * 3 / 4 {
                Color::Red
            } else if *value > max_value / 2 {
                Color::Yellow
            } else if *value > max_value / 4 {
                Color::Green
            } else {
                Color::Blue
            };

            Bar::default()
                .label(Line::from(*label))
                .value(*value)
                .style(Style::default().fg(color))
        })
        .collect();

    let chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Packets Over Time"),
        )
        .bar_width(2)
        .bar_gap(1)
        .data(BarGroup::default().bars(&bar_data))
        .max(max_value.max(1));

    frame.render_widget(chart, area);
}

/// Render timeline statistics
fn render_timeline_stats(frame: &mut Frame, area: Rect, timeline: &TimelineData) {
    let stats = timeline.stats();

    let stats_text = vec![
        Line::from(vec![
            Span::raw("Total Packets: "),
            Span::styled(
                stats.total_packets.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("Peak Rate: "),
            Span::styled(
                format!("{} pkts/bin", stats.max_bin_count),
                Style::default().fg(Color::Red),
            ),
            Span::raw("  Average: "),
            Span::styled(
                format!("{:.1} pkts/bin", stats.avg_bin_count),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::raw("Unique Channels: "),
            Span::styled(
                stats.unique_channels.to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::raw("Average RSSI: "),
            Span::styled(
                if let Some(rssi) = stats.avg_rssi {
                    format!("{:.1} dBm", rssi)
                } else {
                    "N/A".to_string()
                },
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Statistics"));

    frame.render_widget(stats_widget, area);
}

/// Render packet details list (for detail view)
pub fn render_packet_details(
    frame: &mut Frame,
    area: Rect,
    packets: &[PacketData],
    selected_index: usize,
) {
    let items: Vec<ListItem> = packets
        .iter()
        .enumerate()
        .map(|(i, packet)| {
            let style = if i == selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let rssi_str = packet
                .rssi
                .map(|r| format!("{:4} dBm", r))
                .unwrap_or_else(|| "  N/A   ".to_string());

            let line = Line::from(vec![
                Span::styled(format!("{:5} ", packet.sequence), style),
                Span::styled(packet.timestamp.format("%H:%M:%S.%3f").to_string(), style),
                Span::styled(format!(" Ch{:3} ", packet.channel), style),
                Span::styled(rssi_str, style),
                Span::styled(format!(" {}", packet.packet_type), style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Packets"))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_timeline_creation() {
        let packets = vec![
            PacketData {
                sequence: 1,
                timestamp: Utc::now(),
                data: vec![],
                packet_type: "TEST".to_string(),
                channel: 37,
                rssi: Some(-50),
                metadata: serde_json::json!({}),
            },
            PacketData {
                sequence: 2,
                timestamp: Utc::now(),
                data: vec![],
                packet_type: "TEST".to_string(),
                channel: 38,
                rssi: Some(-55),
                metadata: serde_json::json!({}),
            },
        ];

        let config = TimelineConfig::default();
        let timeline = TimelineData::from_packets(&packets, &config);

        assert_eq!(timeline.total_packets, 2);
        assert!(timeline.max_count() > 0);
    }

    #[test]
    fn test_timeline_stats() {
        let packets = vec![
            PacketData {
                sequence: 1,
                timestamp: Utc::now(),
                data: vec![],
                packet_type: "TEST".to_string(),
                channel: 10,
                rssi: Some(-50),
                metadata: serde_json::json!({}),
            },
            PacketData {
                sequence: 2,
                timestamp: Utc::now(),
                data: vec![],
                packet_type: "TEST".to_string(),
                channel: 20,
                rssi: Some(-60),
                metadata: serde_json::json!({}),
            },
        ];

        let config = TimelineConfig::default();
        let timeline = TimelineData::from_packets(&packets, &config);
        let stats = timeline.stats();

        assert_eq!(stats.total_packets, 2);
        assert_eq!(stats.unique_channels, 2);
        assert!(stats.avg_rssi.is_some());
    }
}
