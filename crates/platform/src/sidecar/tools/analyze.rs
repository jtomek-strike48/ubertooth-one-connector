//! Analysis methods for SidecarManager.

use crate::capture_store::CaptureStore;
use serde_json::{json, Value};
use ubertooth_core::error::{Result, UbertoothError};

use super::super::SidecarManager;

impl SidecarManager {
    /// Analyze captured packets implementation.
    ///
    /// Phase 1: Basic analysis with metadata only
    /// Phase 2: Full PCAP parsing with protocol analysis
    pub(in crate::sidecar) async fn bt_analyze(&self, params: Value) -> Result<Value> {
        let capture_id = params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let analysis_type = params
            .get("analysis_type")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        tracing::info!(
            "Analyzing capture: {} (type: {})",
            capture_id,
            analysis_type
        );

        let store = CaptureStore::new()?;
        let metadata = store.load_metadata(capture_id)?;

        let protocol_type = match metadata.capture_type.as_str() {
            "btle_sniff" => "BLE",
            "specan" => "Spectrum",
            "bt_follow" => "BR/EDR",
            _ => "Unknown",
        };

        // Phase 2 Complete: PCAP parsing + device extraction + timing + security analysis
        let pcap_analysis = Self::parse_pcap(&metadata.pcap_path)?;

        // Build device list for JSON output
        let devices: Vec<Value> = pcap_analysis
            .devices
            .iter()
            .map(|dev| {
                json!({
                    "mac_address": dev.mac_address,
                    "device_name": dev.name.as_deref().unwrap_or("Unknown"),
                    "name": dev.name,  // Keep for backwards compatibility
                    "rssi": dev.rssi,
                    "pdu_type": dev.pdu_type,
                    "first_seen": dev.first_seen,
                    "last_seen": dev.last_seen,
                    "packet_count": dev.packet_count
                })
            })
            .collect();

        // Build security observations for JSON output
        let security_observations: Vec<Value> = pcap_analysis
            .security
            .observations
            .iter()
            .map(|obs| {
                json!({
                    "type": obs.observation_type,
                    "severity": obs.severity,
                    "description": obs.description,
                    "affected_device": obs.affected_device
                })
            })
            .collect();

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "analysis": {
                "protocol_summary": {
                    "type": protocol_type,
                    "packet_count": pcap_analysis.packet_count,
                    "total_bytes": pcap_analysis.total_bytes,
                    "avg_packet_size": pcap_analysis.avg_packet_size,
                    "unique_devices": pcap_analysis.devices.len()
                },
                "devices": devices,
                "timing_analysis": {
                    "duration_sec": pcap_analysis.duration_sec,
                    "packets_per_sec": pcap_analysis.packets_per_sec,
                    "avg_interval_ms": pcap_analysis.timing.avg_interval_ms,
                    "min_interval_ms": pcap_analysis.timing.min_interval_ms,
                    "max_interval_ms": pcap_analysis.timing.max_interval_ms,
                    "intervals_calculated": pcap_analysis.timing.intervals_count
                },
                "security_observations": security_observations,
                "security_summary": {
                    "privacy_enabled_devices": pcap_analysis.security.privacy_enabled_count,
                    "public_address_devices": pcap_analysis.security.public_address_count,
                    "connection_requests": pcap_analysis.security.connection_requests,
                    "scan_requests": pcap_analysis.security.scan_requests,
                    "total_observations": pcap_analysis.security.observations.len()
                },
                "note": "Phase 2 Complete: Full PCAP analysis with packet parsing, device extraction, timing analysis, and security observations."
            }
        }))
    }

    /// AFH analysis implementation.
    ///
    /// Phase 2 Week 3: Wraps ubertooth-afh for channel map analysis.
    pub(in crate::sidecar) async fn afh_analyze(&self, params: Value) -> Result<Value> {
        let bd_addr = params.get("bd_addr").and_then(|v| v.as_str());
        let duration_sec = params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        tracing::info!("Analyzing AFH: {:?} for {}s", bd_addr, duration_sec);

        // Build command arguments
        let duration_str = duration_sec.to_string();
        let mut args = vec!["-d", duration_str.as_str()];
        let bd_addr_str;
        if let Some(addr) = bd_addr {
            bd_addr_str = addr.to_string();
            args.extend_from_slice(&["-t", bd_addr_str.as_str()]);
        }

        let output = self
            .execute_ubertooth_command("ubertooth-afh", &args)
            .await?;

        // Parse AFH channel map from output
        let mut afh_map = "0x0000000000000000000000".to_string();
        let mut channels_used = Vec::new();
        let mut channels_avoided = Vec::new();

        for line in output.lines() {
            if line.contains("AFH map:") || line.contains("Channel map:") {
                // Extract hex map
                if let Some(hex_start) = line.find("0x") {
                    afh_map = line[hex_start..]
                        .split_whitespace()
                        .next()
                        .unwrap_or("0x0000000000000000000000")
                        .to_string();
                }
            }
            if line.contains("Used:") || line.contains("Active:") {
                // Parse channel list
                for word in line.split_whitespace() {
                    if let Ok(ch) = word.trim_matches(|c: char| !c.is_numeric()).parse::<u8>() {
                        if ch <= 78 {
                            channels_used.push(ch);
                        }
                    }
                }
            }
            if line.contains("Avoided:") || line.contains("Disabled:") {
                // Parse avoided channel list
                for word in line.split_whitespace() {
                    if let Ok(ch) = word.trim_matches(|c: char| !c.is_numeric()).parse::<u8>() {
                        if ch <= 78 {
                            channels_avoided.push(ch);
                        }
                    }
                }
            }
        }

        let used_count = channels_used.len();
        let avoided_count = channels_avoided.len();
        let interpretation = if used_count > 0 {
            format!(
                "Device uses {} channels, avoids {} channels (likely due to WiFi interference)",
                used_count, avoided_count
            )
        } else {
            "No AFH data captured yet".to_string()
        };

        Ok(json!({
            "success": true,
            "bd_addr": bd_addr,
            "afh_map": afh_map,
            "channels_used": channels_used,
            "channels_avoided": channels_avoided,
            "used_count": used_count,
            "avoided_count": avoided_count,
            "interpretation": interpretation
        }))
    }
}
