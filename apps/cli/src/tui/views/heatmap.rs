//! Channel activity heatmap visualization.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;
use ubertooth_platform::PacketData;

/// Heatmap view configuration
#[derive(Debug, Clone)]
pub struct HeatmapConfig {
    /// Time window in seconds
    pub time_window_secs: u64,
    /// Number of time buckets
    pub time_buckets: usize,
    /// Channel range to display
    pub channel_range: (u8, u8),
    /// Color scheme
    pub color_scheme: ColorScheme,
}

impl Default for HeatmapConfig {
    fn default() -> Self {
        Self {
            time_window_secs: 60,
            time_buckets: 60,
            channel_range: (0, 78), // BLE channels 0-78 (2400-2478 MHz)
            color_scheme: ColorScheme::Heat,
        }
    }
}

/// Color scheme for heatmap
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorScheme {
    /// Red-yellow heat gradient
    Heat,
    /// Blue-cyan cool gradient
    Cool,
    /// Green-yellow activity gradient
    Activity,
    /// Grayscale
    Grayscale,
}

impl ColorScheme {
    /// Get color for intensity (0.0 - 1.0)
    pub fn color_for_intensity(&self, intensity: f64) -> Color {
        let intensity = intensity.clamp(0.0, 1.0);

        match self {
            Self::Heat => {
                if intensity < 0.25 {
                    Color::Black
                } else if intensity < 0.5 {
                    Color::Blue
                } else if intensity < 0.75 {
                    Color::Yellow
                } else {
                    Color::Red
                }
            }
            Self::Cool => {
                if intensity < 0.25 {
                    Color::Black
                } else if intensity < 0.5 {
                    Color::Blue
                } else if intensity < 0.75 {
                    Color::Cyan
                } else {
                    Color::White
                }
            }
            Self::Activity => {
                if intensity < 0.25 {
                    Color::Black
                } else if intensity < 0.5 {
                    Color::Green
                } else if intensity < 0.75 {
                    Color::Yellow
                } else {
                    Color::LightYellow
                }
            }
            Self::Grayscale => {
                if intensity < 0.25 {
                    Color::Black
                } else if intensity < 0.5 {
                    Color::DarkGray
                } else if intensity < 0.75 {
                    Color::Gray
                } else {
                    Color::White
                }
            }
        }
    }

    /// Get character for intensity (more granular display)
    pub fn char_for_intensity(&self, intensity: f64) -> char {
        let intensity = intensity.clamp(0.0, 1.0);

        if intensity < 0.1 {
            ' '
        } else if intensity < 0.2 {
            '.'
        } else if intensity < 0.3 {
            ':'
        } else if intensity < 0.4 {
            '-'
        } else if intensity < 0.5 {
            '='
        } else if intensity < 0.6 {
            '+'
        } else if intensity < 0.7 {
            '*'
        } else if intensity < 0.8 {
            '#'
        } else if intensity < 0.9 {
            '%'
        } else {
            '@'
        }
    }
}

/// Heatmap data structure
pub struct HeatmapData {
    /// Channel activity counts [time_bucket][channel]
    data: Vec<Vec<u32>>,
    /// Maximum count for normalization
    max_count: u32,
    /// Time bucket labels
    time_labels: Vec<String>,
    /// Channel labels
    channel_labels: Vec<String>,
}

impl HeatmapData {
    /// Create heatmap data from packets
    pub fn from_packets(packets: &[PacketData], config: &HeatmapConfig) -> Self {
        let num_channels = (config.channel_range.1 - config.channel_range.0 + 1) as usize;
        let num_buckets = config.time_buckets;

        // Initialize 2D grid
        let mut data = vec![vec![0u32; num_channels]; num_buckets];

        if packets.is_empty() {
            return Self {
                data,
                max_count: 0,
                time_labels: Vec::new(),
                channel_labels: Vec::new(),
            };
        }

        // Find time range
        let first_time = packets.first().unwrap().timestamp;
        let last_time = packets.last().unwrap().timestamp;
        let duration_ms = (last_time - first_time).num_milliseconds().max(1);
        let bucket_duration_ms = duration_ms / num_buckets as i64;

        // Populate heatmap
        let mut max_count = 0;

        for packet in packets {
            let channel_idx = (packet.channel as i32 - config.channel_range.0 as i32) as usize;
            if channel_idx >= num_channels {
                continue;
            }

            let elapsed_ms = (packet.timestamp - first_time).num_milliseconds();
            let bucket_idx = ((elapsed_ms / bucket_duration_ms) as usize).min(num_buckets - 1);

            data[bucket_idx][channel_idx] += 1;
            max_count = max_count.max(data[bucket_idx][channel_idx]);
        }

        // Generate labels
        let time_labels = (0..num_buckets)
            .map(|i| {
                let secs = (i * bucket_duration_ms as usize) / 1000;
                format!("{}s", secs)
            })
            .collect();

        let channel_labels = (config.channel_range.0..=config.channel_range.1)
            .map(|ch| format!("{}", ch))
            .collect();

        Self {
            data,
            max_count,
            time_labels,
            channel_labels,
        }
    }

    /// Get normalized intensity at position
    pub fn intensity_at(&self, time_bucket: usize, channel: usize) -> f64 {
        if time_bucket >= self.data.len() || channel >= self.data[0].len() {
            return 0.0;
        }

        if self.max_count == 0 {
            return 0.0;
        }

        self.data[time_bucket][channel] as f64 / self.max_count as f64
    }

    /// Get count at position
    pub fn count_at(&self, time_bucket: usize, channel: usize) -> u32 {
        if time_bucket >= self.data.len() || channel >= self.data[0].len() {
            return 0;
        }

        self.data[time_bucket][channel]
    }

    /// Get summary statistics
    pub fn summary(&self) -> HeatmapSummary {
        let mut total_packets = 0;
        let mut active_channels = std::collections::HashSet::new();
        let mut channel_totals = HashMap::new();

        for (bucket_idx, bucket) in self.data.iter().enumerate() {
            for (ch_idx, &count) in bucket.iter().enumerate() {
                if count > 0 {
                    total_packets += count;
                    active_channels.insert(ch_idx);
                    *channel_totals.entry(ch_idx).or_insert(0) += count;
                }
            }
        }

        // Find busiest channel
        let busiest_channel = channel_totals
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&ch, &count)| (ch, count));

        HeatmapSummary {
            total_packets,
            active_channels: active_channels.len(),
            max_intensity: self.max_count,
            busiest_channel,
        }
    }
}

/// Heatmap summary statistics
#[derive(Debug, Clone)]
pub struct HeatmapSummary {
    pub total_packets: u32,
    pub active_channels: usize,
    pub max_intensity: u32,
    pub busiest_channel: Option<(usize, u32)>,
}

/// Render channel activity heatmap
pub fn render_heatmap(
    frame: &mut Frame,
    area: Rect,
    packets: &[PacketData],
    config: &HeatmapConfig,
    title: &str,
) {
    let heatmap = HeatmapData::from_packets(packets, config);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Heatmap
            Constraint::Length(5), // Legend + stats
        ])
        .split(area);

    // Render header
    render_heatmap_header(frame, chunks[0], title, &heatmap);

    // Render heatmap grid
    render_heatmap_grid(frame, chunks[1], &heatmap, config);

    // Render legend and stats
    render_heatmap_legend(frame, chunks[2], &heatmap, config);
}

/// Render heatmap header with title and summary
fn render_heatmap_header(frame: &mut Frame, area: Rect, title: &str, heatmap: &HeatmapData) {
    let summary = heatmap.summary();

    let header_text = vec![
        Line::from(vec![
            Span::styled(title, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Packets: "),
            Span::styled(
                summary.total_packets.to_string(),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  Active Channels: "),
            Span::styled(
                summary.active_channels.to_string(),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  Max: "),
            Span::styled(
                summary.max_intensity.to_string(),
                Style::default().fg(Color::Red),
            ),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Heatmap"));

    frame.render_widget(header, area);
}

/// Render heatmap grid
fn render_heatmap_grid(
    frame: &mut Frame,
    area: Rect,
    heatmap: &HeatmapData,
    config: &HeatmapConfig,
) {
    let available_height = area.height.saturating_sub(2) as usize; // Subtract borders
    let available_width = area.width.saturating_sub(2) as usize;

    let num_channels = heatmap.data.get(0).map(|v| v.len()).unwrap_or(0);
    let num_buckets = heatmap.data.len();

    // Calculate how many channels we can display
    let channels_to_display = num_channels.min(available_height);
    let channel_skip = if channels_to_display > 0 {
        num_channels / channels_to_display
    } else {
        1
    };

    // Calculate how many time buckets we can display
    let buckets_to_display = num_buckets.min(available_width);
    let bucket_skip = if buckets_to_display > 0 {
        num_buckets / buckets_to_display
    } else {
        1
    };

    let mut lines = Vec::new();

    // Render grid (channels are rows, time buckets are columns)
    for ch_idx in (0..num_channels).step_by(channel_skip.max(1)) {
        let mut spans = Vec::new();

        for bucket_idx in (0..num_buckets).step_by(bucket_skip.max(1)) {
            let intensity = heatmap.intensity_at(bucket_idx, ch_idx);
            let color = config.color_scheme.color_for_intensity(intensity);
            let ch = config.color_scheme.char_for_intensity(intensity);

            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(color).bg(Color::Black),
            ));
        }

        lines.push(Line::from(spans));
    }

    let grid = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));

    frame.render_widget(grid, area);
}

/// Render legend and statistics
fn render_heatmap_legend(
    frame: &mut Frame,
    area: Rect,
    heatmap: &HeatmapData,
    config: &HeatmapConfig,
) {
    let summary = heatmap.summary();

    let legend_text = vec![
        Line::from(vec![
            Span::raw("Intensity: "),
            Span::styled("  ", Style::default().bg(Color::Black)),
            Span::raw(" Low  "),
            Span::styled("==", Style::default().fg(Color::Yellow)),
            Span::raw(" Medium  "),
            Span::styled("@@", Style::default().fg(Color::Red)),
            Span::raw(" High"),
        ]),
        Line::from(vec![
            Span::raw("Color Scheme: "),
            Span::styled(
                format!("{:?}", config.color_scheme),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Busiest Channel: "),
            Span::styled(
                if let Some((ch, count)) = summary.busiest_channel {
                    format!("Ch {} ({} packets)", ch + config.channel_range.0 as usize, count)
                } else {
                    "N/A".to_string()
                },
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let legend = Paragraph::new(legend_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Legend & Stats"),
    );

    frame.render_widget(legend, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_color_scheme_intensity() {
        let scheme = ColorScheme::Heat;

        assert_eq!(scheme.color_for_intensity(0.0), Color::Black);
        assert_eq!(scheme.color_for_intensity(0.9), Color::Red);
        assert_eq!(scheme.char_for_intensity(0.0), ' ');
        assert_eq!(scheme.char_for_intensity(0.95), '@');
    }

    #[test]
    fn test_heatmap_data_creation() {
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
                channel: 37,
                rssi: Some(-50),
                metadata: serde_json::json!({}),
            },
        ];

        let config = HeatmapConfig::default();
        let heatmap = HeatmapData::from_packets(&packets, &config);

        assert!(heatmap.max_count > 0);
        assert_eq!(heatmap.data.len(), config.time_buckets);
    }

    #[test]
    fn test_heatmap_summary() {
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
                rssi: Some(-50),
                metadata: serde_json::json!({}),
            },
        ];

        let config = HeatmapConfig::default();
        let heatmap = HeatmapData::from_packets(&packets, &config);
        let summary = heatmap.summary();

        assert_eq!(summary.total_packets, 2);
        assert!(summary.active_channels > 0);
    }
}
