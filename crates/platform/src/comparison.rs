//! Multi-capture comparison and analysis.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use ubertooth_core::error::Result;

/// Comparison result for multiple captures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Capture IDs being compared
    pub capture_ids: Vec<String>,
    /// Device presence analysis
    pub device_presence: DevicePresenceComparison,
    /// Traffic pattern analysis
    pub traffic_patterns: TrafficPatternComparison,
    /// Packet statistics comparison
    pub statistics: Vec<CaptureStatistics>,
    /// Similarity matrix (pairwise similarity scores)
    pub similarity_matrix: Vec<Vec<f64>>,
    /// Summary insights
    pub summary: ComparisonSummary,
}

/// Device presence across captures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePresenceComparison {
    /// Devices present in all captures
    pub common_devices: Vec<DeviceInfo>,
    /// Devices unique to specific captures
    pub unique_devices: HashMap<String, Vec<DeviceInfo>>,
    /// Total unique devices across all captures
    pub total_unique_devices: usize,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    /// MAC address
    pub mac_address: String,
    /// Device name (if available)
    pub name: Option<String>,
    /// Packet count
    pub packet_count: usize,
    /// Average RSSI
    pub avg_rssi: Option<f32>,
}

impl std::hash::Hash for DeviceInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.mac_address.hash(state);
    }
}

impl Eq for DeviceInfo {}

/// Traffic pattern comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficPatternComparison {
    /// Channel usage comparison
    pub channel_usage: HashMap<String, Vec<ChannelUsage>>,
    /// Packet type distribution
    pub packet_types: HashMap<String, Vec<PacketTypeCount>>,
    /// Temporal patterns
    pub temporal_patterns: Vec<TemporalPattern>,
}

/// Channel usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelUsage {
    pub channel: u8,
    pub packet_count: usize,
    pub percentage: f64,
}

/// Packet type count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketTypeCount {
    pub packet_type: String,
    pub count: usize,
    pub percentage: f64,
}

/// Temporal pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPattern {
    pub capture_id: String,
    pub duration_seconds: f64,
    pub packets_per_second: f64,
    pub peak_rate: f64,
    pub average_interval_ms: f64,
}

/// Capture statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureStatistics {
    pub capture_id: String,
    pub total_packets: usize,
    pub unique_devices: usize,
    pub channels_used: usize,
    pub packet_types: usize,
    pub duration_seconds: f64,
    pub avg_rssi: Option<f32>,
}

/// Comparison summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSummary {
    /// Key differences between captures
    pub key_differences: Vec<String>,
    /// Similarity assessment
    pub overall_similarity: SimilarityLevel,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Similarity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SimilarityLevel {
    Identical,      // >95%
    VerySimilar,    // 80-95%
    Similar,        // 60-80%
    Somewhat,       // 40-60%
    Different,      // 20-40%
    VeryDifferent,  // <20%
}

impl SimilarityLevel {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.95 {
            Self::Identical
        } else if score >= 0.80 {
            Self::VerySimilar
        } else if score >= 0.60 {
            Self::Similar
        } else if score >= 0.40 {
            Self::Somewhat
        } else if score >= 0.20 {
            Self::Different
        } else {
            Self::VeryDifferent
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Identical => "Captures are virtually identical",
            Self::VerySimilar => "Captures are very similar",
            Self::Similar => "Captures share common patterns",
            Self::Somewhat => "Captures have some similarities",
            Self::Different => "Captures are quite different",
            Self::VeryDifferent => "Captures are very different",
        }
    }
}

/// Multi-capture comparison engine
pub struct ComparisonEngine;

impl ComparisonEngine {
    /// Compare multiple captures
    pub fn compare_captures(
        capture_data: Vec<CaptureData>,
    ) -> Result<ComparisonResult> {
        let capture_ids: Vec<String> = capture_data.iter()
            .map(|c| c.capture_id.clone())
            .collect();

        // Analyze device presence
        let device_presence = Self::analyze_device_presence(&capture_data);

        // Analyze traffic patterns
        let traffic_patterns = Self::analyze_traffic_patterns(&capture_data);

        // Calculate statistics for each capture
        let statistics: Vec<CaptureStatistics> = capture_data.iter()
            .map(|data| Self::calculate_statistics(data))
            .collect();

        // Build similarity matrix
        let similarity_matrix = Self::build_similarity_matrix(&capture_data);

        // Generate summary
        let summary = Self::generate_summary(
            &device_presence,
            &traffic_patterns,
            &statistics,
            &similarity_matrix,
        );

        Ok(ComparisonResult {
            capture_ids,
            device_presence,
            traffic_patterns,
            statistics,
            similarity_matrix,
            summary,
        })
    }

    /// Analyze device presence across captures
    fn analyze_device_presence(captures: &[CaptureData]) -> DevicePresenceComparison {
        let mut all_devices: HashMap<String, Vec<(usize, DeviceInfo)>> = HashMap::new();

        // Collect devices from each capture
        for (idx, capture) in captures.iter().enumerate() {
            for device in &capture.devices {
                all_devices.entry(device.mac_address.clone())
                    .or_insert_with(Vec::new)
                    .push((idx, device.clone()));
            }
        }

        // Identify common devices (present in all captures)
        let common_devices: Vec<DeviceInfo> = all_devices.iter()
            .filter(|(_, occurrences)| occurrences.len() == captures.len())
            .map(|(_, occurrences)| occurrences[0].1.clone())
            .collect();

        // Identify unique devices per capture
        let mut unique_devices: HashMap<String, Vec<DeviceInfo>> = HashMap::new();
        for (mac, occurrences) in &all_devices {
            if occurrences.len() == 1 {
                let (capture_idx, device) = &occurrences[0];
                let capture_id = captures[*capture_idx].capture_id.clone();
                unique_devices.entry(capture_id)
                    .or_insert_with(Vec::new)
                    .push(device.clone());
            }
        }

        DevicePresenceComparison {
            common_devices,
            unique_devices,
            total_unique_devices: all_devices.len(),
        }
    }

    /// Analyze traffic patterns
    fn analyze_traffic_patterns(captures: &[CaptureData]) -> TrafficPatternComparison {
        let mut channel_usage: HashMap<String, Vec<ChannelUsage>> = HashMap::new();
        let mut packet_types: HashMap<String, Vec<PacketTypeCount>> = HashMap::new();
        let mut temporal_patterns = Vec::new();

        for capture in captures {
            // Channel usage
            let total_packets = capture.packets.len();
            let mut channel_counts: HashMap<u8, usize> = HashMap::new();
            for packet in &capture.packets {
                *channel_counts.entry(packet.channel).or_insert(0) += 1;
            }

            let mut usage: Vec<ChannelUsage> = channel_counts.iter()
                .map(|(&channel, &count)| ChannelUsage {
                    channel,
                    packet_count: count,
                    percentage: (count as f64 / total_packets as f64) * 100.0,
                })
                .collect();
            usage.sort_by_key(|u| u.channel);
            channel_usage.insert(capture.capture_id.clone(), usage);

            // Packet type distribution
            let mut type_counts: HashMap<String, usize> = HashMap::new();
            for packet in &capture.packets {
                *type_counts.entry(packet.packet_type.clone()).or_insert(0) += 1;
            }

            let mut types: Vec<PacketTypeCount> = type_counts.iter()
                .map(|(packet_type, &count)| PacketTypeCount {
                    packet_type: packet_type.clone(),
                    count,
                    percentage: (count as f64 / total_packets as f64) * 100.0,
                })
                .collect();
            types.sort_by(|a, b| b.count.cmp(&a.count));
            packet_types.insert(capture.capture_id.clone(), types);

            // Temporal patterns
            if !capture.packets.is_empty() {
                let first_time = capture.packets.first().unwrap().timestamp_ms;
                let last_time = capture.packets.last().unwrap().timestamp_ms;
                let duration = (last_time - first_time) as f64 / 1000.0;
                let pps = if duration > 0.0 {
                    total_packets as f64 / duration
                } else {
                    0.0
                };

                // Calculate peak rate (max packets in any 1-second window)
                let peak_rate = Self::calculate_peak_rate(&capture.packets);

                // Calculate average interval
                let avg_interval = if capture.packets.len() > 1 {
                    duration * 1000.0 / (capture.packets.len() - 1) as f64
                } else {
                    0.0
                };

                temporal_patterns.push(TemporalPattern {
                    capture_id: capture.capture_id.clone(),
                    duration_seconds: duration,
                    packets_per_second: pps,
                    peak_rate,
                    average_interval_ms: avg_interval,
                });
            }
        }

        TrafficPatternComparison {
            channel_usage,
            packet_types,
            temporal_patterns,
        }
    }

    /// Calculate peak packet rate
    fn calculate_peak_rate(packets: &[PacketInfo]) -> f64 {
        if packets.is_empty() {
            return 0.0;
        }

        let first_time = packets.first().unwrap().timestamp_ms;
        let last_time = packets.last().unwrap().timestamp_ms;
        let duration_ms = last_time - first_time;

        if duration_ms == 0 {
            return packets.len() as f64;
        }

        // Use 1-second sliding window
        let window_ms = 1000;
        let mut max_count = 0;

        for window_start in (first_time..=last_time).step_by(100) {
            let window_end = window_start + window_ms;
            let count = packets.iter()
                .filter(|p| p.timestamp_ms >= window_start && p.timestamp_ms < window_end)
                .count();
            max_count = max_count.max(count);
        }

        max_count as f64
    }

    /// Calculate statistics for a capture
    fn calculate_statistics(capture: &CaptureData) -> CaptureStatistics {
        let unique_devices = capture.devices.len();
        let channels_used: HashSet<u8> = capture.packets.iter()
            .map(|p| p.channel)
            .collect();
        let packet_types: HashSet<String> = capture.packets.iter()
            .map(|p| p.packet_type.clone())
            .collect();

        let duration = if !capture.packets.is_empty() {
            let first = capture.packets.first().unwrap().timestamp_ms;
            let last = capture.packets.last().unwrap().timestamp_ms;
            (last - first) as f64 / 1000.0
        } else {
            0.0
        };

        let avg_rssi = if !capture.packets.is_empty() {
            let rssi_sum: i32 = capture.packets.iter()
                .filter_map(|p| p.rssi)
                .map(|r| r as i32)
                .sum();
            let rssi_count = capture.packets.iter().filter(|p| p.rssi.is_some()).count();
            if rssi_count > 0 {
                Some(rssi_sum as f32 / rssi_count as f32)
            } else {
                None
            }
        } else {
            None
        };

        CaptureStatistics {
            capture_id: capture.capture_id.clone(),
            total_packets: capture.packets.len(),
            unique_devices,
            channels_used: channels_used.len(),
            packet_types: packet_types.len(),
            duration_seconds: duration,
            avg_rssi,
        }
    }

    /// Build pairwise similarity matrix
    fn build_similarity_matrix(captures: &[CaptureData]) -> Vec<Vec<f64>> {
        let n = captures.len();
        let mut matrix = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    matrix[i][j] = 1.0;
                } else if i < j {
                    let similarity = Self::calculate_similarity(&captures[i], &captures[j]);
                    matrix[i][j] = similarity;
                    matrix[j][i] = similarity; // Symmetric
                }
            }
        }

        matrix
    }

    /// Calculate similarity between two captures
    fn calculate_similarity(a: &CaptureData, b: &CaptureData) -> f64 {
        let mut scores = Vec::new();

        // Device overlap score
        let devices_a: HashSet<String> = a.devices.iter()
            .map(|d| d.mac_address.clone())
            .collect();
        let devices_b: HashSet<String> = b.devices.iter()
            .map(|d| d.mac_address.clone())
            .collect();
        let common = devices_a.intersection(&devices_b).count();
        let total = devices_a.union(&devices_b).count();
        if total > 0 {
            scores.push(common as f64 / total as f64);
        }

        // Channel overlap score
        let channels_a: HashSet<u8> = a.packets.iter().map(|p| p.channel).collect();
        let channels_b: HashSet<u8> = b.packets.iter().map(|p| p.channel).collect();
        let common_ch = channels_a.intersection(&channels_b).count();
        let total_ch = channels_a.union(&channels_b).count();
        if total_ch > 0 {
            scores.push(common_ch as f64 / total_ch as f64);
        }

        // Packet type overlap
        let types_a: HashSet<String> = a.packets.iter()
            .map(|p| p.packet_type.clone())
            .collect();
        let types_b: HashSet<String> = b.packets.iter()
            .map(|p| p.packet_type.clone())
            .collect();
        let common_types = types_a.intersection(&types_b).count();
        let total_types = types_a.union(&types_b).count();
        if total_types > 0 {
            scores.push(common_types as f64 / total_types as f64);
        }

        // Average similarity score
        if !scores.is_empty() {
            scores.iter().sum::<f64>() / scores.len() as f64
        } else {
            0.0
        }
    }

    /// Generate comparison summary
    fn generate_summary(
        device_presence: &DevicePresenceComparison,
        traffic_patterns: &TrafficPatternComparison,
        statistics: &[CaptureStatistics],
        similarity_matrix: &[Vec<f64>],
    ) -> ComparisonSummary {
        let mut key_differences = Vec::new();
        let mut recommendations = Vec::new();

        // Analyze device differences
        if device_presence.common_devices.is_empty() {
            key_differences.push("No common devices found across captures".to_string());
            recommendations.push("Check if captures are from different environments".to_string());
        } else {
            key_differences.push(format!(
                "{} common devices found across all captures",
                device_presence.common_devices.len()
            ));
        }

        // Analyze unique devices
        for (capture_id, devices) in &device_presence.unique_devices {
            if !devices.is_empty() {
                key_differences.push(format!(
                    "{} unique device(s) in capture {}",
                    devices.len(),
                    capture_id
                ));
            }
        }

        // Analyze packet counts
        let min_packets = statistics.iter().map(|s| s.total_packets).min().unwrap_or(0);
        let max_packets = statistics.iter().map(|s| s.total_packets).max().unwrap_or(0);
        if max_packets > 0 && (max_packets - min_packets) as f64 / max_packets as f64 > 0.5 {
            key_differences.push(format!(
                "Significant packet count variation: {} - {}",
                min_packets, max_packets
            ));
            recommendations.push("Consider normalizing capture durations for better comparison".to_string());
        }

        // Calculate overall similarity
        let avg_similarity = if similarity_matrix.len() > 1 {
            let mut sum = 0.0;
            let mut count = 0;
            for i in 0..similarity_matrix.len() {
                for j in (i + 1)..similarity_matrix[i].len() {
                    sum += similarity_matrix[i][j];
                    count += 1;
                }
            }
            if count > 0 {
                sum / count as f64
            } else {
                0.0
            }
        } else {
            1.0
        };

        let overall_similarity = SimilarityLevel::from_score(avg_similarity);

        // Generate recommendations based on similarity
        match overall_similarity {
            SimilarityLevel::Identical | SimilarityLevel::VerySimilar => {
                recommendations.push("Captures are highly similar - potential replay or identical scenarios".to_string());
            }
            SimilarityLevel::VeryDifferent => {
                recommendations.push("Captures are very different - analyze unique devices and patterns".to_string());
            }
            _ => {
                recommendations.push("Captures show moderate differences - review device presence and traffic patterns".to_string());
            }
        }

        ComparisonSummary {
            key_differences,
            overall_similarity,
            recommendations,
        }
    }
}

/// Capture data for comparison
#[derive(Debug, Clone)]
pub struct CaptureData {
    pub capture_id: String,
    pub packets: Vec<PacketInfo>,
    pub devices: Vec<DeviceInfo>,
}

/// Packet information
#[derive(Debug, Clone)]
pub struct PacketInfo {
    pub timestamp_ms: u64,
    pub channel: u8,
    pub rssi: Option<i8>,
    pub packet_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_capture(id: &str, device_macs: Vec<&str>, packet_count: usize) -> CaptureData {
        let devices = device_macs.iter().enumerate().map(|(i, &mac)| DeviceInfo {
            mac_address: mac.to_string(),
            name: Some(format!("Device {}", i)),
            packet_count: packet_count / device_macs.len(),
            avg_rssi: Some(-50.0),
        }).collect();

        let packets = (0..packet_count).map(|i| PacketInfo {
            timestamp_ms: i as u64 * 100,
            channel: (i % 3 + 37) as u8, // Channels 37, 38, 39
            rssi: Some(-50),
            packet_type: "LE_ADV".to_string(),
        }).collect();

        CaptureData {
            capture_id: id.to_string(),
            packets,
            devices,
        }
    }

    #[test]
    fn test_device_presence_common() {
        let captures = vec![
            create_test_capture("cap1", vec!["AA:BB:CC:DD:EE:FF", "11:22:33:44:55:66"], 100),
            create_test_capture("cap2", vec!["AA:BB:CC:DD:EE:FF", "11:22:33:44:55:66"], 100),
        ];

        let result = ComparisonEngine::compare_captures(captures).unwrap();
        assert_eq!(result.device_presence.common_devices.len(), 2);
        assert_eq!(result.device_presence.total_unique_devices, 2);
    }

    #[test]
    fn test_device_presence_unique() {
        let captures = vec![
            create_test_capture("cap1", vec!["AA:BB:CC:DD:EE:FF"], 100),
            create_test_capture("cap2", vec!["11:22:33:44:55:66"], 100),
        ];

        let result = ComparisonEngine::compare_captures(captures).unwrap();
        assert_eq!(result.device_presence.common_devices.len(), 0);
        assert_eq!(result.device_presence.total_unique_devices, 2);
        assert_eq!(result.device_presence.unique_devices.len(), 2);
    }

    #[test]
    fn test_similarity_calculation() {
        let captures = vec![
            create_test_capture("cap1", vec!["AA:BB:CC:DD:EE:FF"], 100),
            create_test_capture("cap2", vec!["AA:BB:CC:DD:EE:FF"], 100),
        ];

        let result = ComparisonEngine::compare_captures(captures).unwrap();
        assert!(result.similarity_matrix[0][1] > 0.8); // High similarity
    }

    #[test]
    fn test_similarity_level() {
        assert_eq!(SimilarityLevel::from_score(0.99), SimilarityLevel::Identical);
        assert_eq!(SimilarityLevel::from_score(0.85), SimilarityLevel::VerySimilar);
        assert_eq!(SimilarityLevel::from_score(0.10), SimilarityLevel::VeryDifferent);
    }

    #[test]
    fn test_traffic_pattern_analysis() {
        let captures = vec![
            create_test_capture("cap1", vec!["AA:BB:CC:DD:EE:FF"], 100),
        ];

        let result = ComparisonEngine::compare_captures(captures).unwrap();
        assert!(!result.traffic_patterns.channel_usage.is_empty());
        assert!(!result.traffic_patterns.packet_types.is_empty());
        assert_eq!(result.traffic_patterns.temporal_patterns.len(), 1);
    }
}
