//! Sample plugin demonstrating the plugin API.
//!
//! This plugin:
//! - Filters packets by RSSI threshold
//! - Counts packet types
//! - Adds custom metadata
//!
//! Build as dynamic library:
//! ```bash
//! cargo build --example sample_plugin --release
//! ```

use std::collections::HashMap;
use ubertooth_plugin::*;

/// Sample packet filter plugin
pub struct SamplePlugin {
    metadata: PluginMetadata,
    rssi_threshold: i8,
    packet_counts: HashMap<String, usize>,
}

impl SamplePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "sample-plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Sample plugin for packet filtering and analysis".to_string(),
                author: "Ubertooth Team".to_string(),
                api_version: "1.0".to_string(),
            },
            rssi_threshold: -70, // Default threshold
            packet_counts: HashMap::new(),
        }
    }
}

impl Plugin for SamplePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![
            PluginCapability::PacketProcessor,
            PluginCapability::CaptureAnalyzer,
            PluginCapability::Command,
        ]
    }

    fn initialize(&mut self, context: PluginContext) -> Result<(), PluginError> {
        // Read RSSI threshold from config
        if let Some(threshold) = context.get_config("rssi_threshold") {
            if let Some(value) = threshold.as_i64() {
                self.rssi_threshold = value as i8;
            }
        }

        tracing::info!(
            "Sample plugin initialized with RSSI threshold: {}",
            self.rssi_threshold
        );

        Ok(())
    }

    fn process_packet(&mut self, packet: &PacketData) -> Result<ProcessResult, PluginError> {
        // Count packet types
        *self.packet_counts.entry(packet.packet_type.clone()).or_insert(0) += 1;

        let mut result = ProcessResult::default();

        // Filter by RSSI
        if let Some(rssi) = packet.rssi {
            if rssi < self.rssi_threshold {
                result.keep = false;
                result.tags.push("filtered-rssi".to_string());
            } else {
                result.tags.push("good-signal".to_string());
            }

            // Add metadata
            result.metadata.insert(
                "rssi_status".to_string(),
                serde_json::json!({
                    "value": rssi,
                    "threshold": self.rssi_threshold,
                    "passed": rssi >= self.rssi_threshold
                }),
            );
        }

        // Tag by channel
        if packet.channel >= 37 && packet.channel <= 39 {
            result.tags.push("advertising-channel".to_string());
        } else {
            result.tags.push("data-channel".to_string());
        }

        Ok(result)
    }

    fn analyze_capture(
        &mut self,
        capture_id: &str,
        packets: &[PacketData],
    ) -> Result<AnalysisResult, PluginError> {
        let total_packets = packets.len();
        let mut filtered_count = 0;
        let mut avg_rssi_sum = 0i32;
        let mut rssi_count = 0;

        // Reset packet counts for this analysis
        self.packet_counts.clear();

        // Analyze packets
        for packet in packets {
            *self.packet_counts.entry(packet.packet_type.clone()).or_insert(0) += 1;

            if let Some(rssi) = packet.rssi {
                avg_rssi_sum += rssi as i32;
                rssi_count += 1;

                if rssi < self.rssi_threshold {
                    filtered_count += 1;
                }
            }
        }

        let avg_rssi = if rssi_count > 0 {
            avg_rssi_sum as f64 / rssi_count as f64
        } else {
            0.0
        };

        // Create findings
        let mut findings = Vec::new();

        if filtered_count > total_packets / 2 {
            findings.push(Finding {
                severity: Severity::Medium,
                title: "High packet loss".to_string(),
                description: format!(
                    "{} of {} packets ({:.1}%) would be filtered due to weak signal",
                    filtered_count,
                    total_packets,
                    (filtered_count as f64 / total_packets as f64) * 100.0
                ),
                packet_indices: vec![],
            });
        }

        if avg_rssi < -60.0 {
            findings.push(Finding {
                severity: Severity::Low,
                title: "Weak average signal".to_string(),
                description: format!("Average RSSI: {:.1} dBm", avg_rssi),
                packet_indices: vec![],
            });
        }

        // Statistics
        let mut statistics = HashMap::new();
        statistics.insert("total_packets".to_string(), serde_json::json!(total_packets));
        statistics.insert("filtered_count".to_string(), serde_json::json!(filtered_count));
        statistics.insert("avg_rssi".to_string(), serde_json::json!(avg_rssi));
        statistics.insert("packet_types".to_string(), serde_json::json!(self.packet_counts));

        // Recommendations
        let mut recommendations = Vec::new();
        if avg_rssi < -60.0 {
            recommendations.push("Consider moving closer to target device".to_string());
        }
        if filtered_count > 0 {
            recommendations.push(format!(
                "Adjust RSSI threshold (current: {}) to capture more packets",
                self.rssi_threshold
            ));
        }

        Ok(AnalysisResult {
            summary: format!(
                "Analyzed {} packets in capture {}. Average RSSI: {:.1} dBm, {} filtered",
                total_packets, capture_id, avg_rssi, filtered_count
            ),
            findings,
            statistics,
            recommendations,
        })
    }

    fn execute_command(&mut self, command: &str, args: &[String]) -> Result<serde_json::Value, PluginError> {
        match command {
            "get-stats" => {
                Ok(serde_json::json!({
                    "packet_counts": self.packet_counts,
                    "rssi_threshold": self.rssi_threshold
                }))
            }
            "set-threshold" => {
                if args.is_empty() {
                    return Err(PluginError::ExecutionError(
                        "Missing threshold value".to_string(),
                    ));
                }

                let threshold: i8 = args[0].parse().map_err(|_| {
                    PluginError::ExecutionError("Invalid threshold value".to_string())
                })?;

                self.rssi_threshold = threshold;

                Ok(serde_json::json!({
                    "success": true,
                    "new_threshold": threshold
                }))
            }
            "reset" => {
                self.packet_counts.clear();
                Ok(serde_json::json!({ "success": true }))
            }
            _ => Err(PluginError::ExecutionError(format!(
                "Unknown command: {}",
                command
            ))),
        }
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        tracing::info!("Sample plugin shutting down");
        tracing::info!("Final packet counts: {:?}", self.packet_counts);
        Ok(())
    }
}

// Declare plugin (required for dynamic loading)
declare_plugin!(SamplePlugin, SamplePlugin::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = SamplePlugin::new();
        assert_eq!(plugin.metadata().name, "sample-plugin");
    }

    #[test]
    fn test_packet_processing() {
        let mut plugin = SamplePlugin::new();
        let context = PluginContext::new(HashMap::new(), std::path::PathBuf::from("/tmp"));
        plugin.initialize(context).unwrap();

        let packet = PacketData {
            sequence: 1,
            timestamp_ms: 1000,
            data: vec![1, 2, 3],
            packet_type: "LE_ADV".to_string(),
            channel: 37,
            rssi: Some(-50),
            metadata: HashMap::new(),
        };

        let result = plugin.process_packet(&packet).unwrap();
        assert!(result.keep);
        assert!(result.tags.contains(&"good-signal".to_string()));
        assert!(result.tags.contains(&"advertising-channel".to_string()));
    }

    #[test]
    fn test_rssi_filtering() {
        let mut plugin = SamplePlugin::new();
        plugin.rssi_threshold = -60;

        let weak_packet = PacketData {
            sequence: 1,
            timestamp_ms: 1000,
            data: vec![],
            packet_type: "LE_ADV".to_string(),
            channel: 37,
            rssi: Some(-80), // Below threshold
            metadata: HashMap::new(),
        };

        let result = plugin.process_packet(&weak_packet).unwrap();
        assert!(!result.keep);
        assert!(result.tags.contains(&"filtered-rssi".to_string()));
    }

    #[test]
    fn test_capture_analysis() {
        let mut plugin = SamplePlugin::new();
        let context = PluginContext::new(HashMap::new(), std::path::PathBuf::from("/tmp"));
        plugin.initialize(context).unwrap();

        let packets = vec![
            PacketData {
                sequence: 1,
                timestamp_ms: 1000,
                data: vec![],
                packet_type: "LE_ADV".to_string(),
                channel: 37,
                rssi: Some(-50),
                metadata: HashMap::new(),
            },
            PacketData {
                sequence: 2,
                timestamp_ms: 2000,
                data: vec![],
                packet_type: "LE_DATA".to_string(),
                channel: 10,
                rssi: Some(-55),
                metadata: HashMap::new(),
            },
        ];

        let result = plugin.analyze_capture("test-capture", &packets).unwrap();
        assert_eq!(result.findings.len(), 0); // No issues with good signals
        assert!(result.summary.contains("2 packets"));
    }

    #[test]
    fn test_command_execution() {
        let mut plugin = SamplePlugin::new();

        // Get stats
        let result = plugin.execute_command("get-stats", &[]).unwrap();
        assert!(result.is_object());

        // Set threshold
        let result = plugin.execute_command("set-threshold", &["-80".to_string()]).unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(plugin.rssi_threshold, -80);

        // Unknown command
        let result = plugin.execute_command("unknown", &[]);
        assert!(result.is_err());
    }
}
