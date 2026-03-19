//! Connection following methods for SidecarManager.

use crate::capture_store::{CaptureMetadata, CaptureStore};
use chrono::Utc;
use serde_json::{json, Value};
use ubertooth_core::error::{Result, UbertoothError};

use super::super::SidecarManager;

impl SidecarManager {
    /// Follow Bluetooth connection implementation.
    ///
    /// Phase 2 Week 3: Wraps ubertooth-follow for targeted connection monitoring.
    pub(in crate::sidecar) async fn bt_follow(&self, params: Value) -> Result<Value> {
        let bd_addr = params
            .get("bd_addr")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'bd_addr'".to_string()))?;

        let duration_sec = params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);

        tracing::info!("Following BT connection: {} for {}s", bd_addr, duration_sec);

        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("follow");
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Execute ubertooth-follow with BD_ADDR and output to PCAP
        let duration_str = duration_sec.to_string();
        let pcap_str = pcap_path.to_string_lossy().to_string();

        let output = self
            .execute_ubertooth_command(
                "ubertooth-follow",
                &[
                    "-t",
                    bd_addr,
                    "-r",
                    pcap_str.as_str(),
                    "-d",
                    duration_str.as_str(),
                ],
            )
            .await?;

        // Parse output for connection info and packet count
        let connection_found = output.contains("Following") || output.contains("Connection");
        let packet_count = output
            .lines()
            .filter(|line| line.contains("packet"))
            .count();

        // Parse channel usage from output
        let mut channels_used = Vec::new();
        for line in output.lines() {
            if line.contains("channel") {
                // Extract channel numbers (0-78)
                for word in line.split_whitespace() {
                    if let Ok(ch) = word.parse::<u8>() {
                        if ch <= 78 && !channels_used.contains(&ch) {
                            channels_used.push(ch);
                        }
                    }
                }
            }
        }

        // Save metadata
        let file_size_bytes = if pcap_path.exists() {
            std::fs::metadata(&pcap_path)?.len()
        } else {
            0
        };

        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "bt_follow".to_string(),
            duration_sec: Some(duration_sec),
            packet_count,
            file_size_bytes,
            pcap_path: pcap_path.to_string_lossy().to_string(),
            tags: vec![format!("bd_addr:{}", bd_addr)],
            description: format!("Following Bluetooth connection {}", bd_addr),
            category: None,
            notes: None,
        };
        store.save_metadata(&metadata)?;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "bd_addr": bd_addr,
            "connection_found": connection_found,
            "packet_count": packet_count,
            "duration_sec": duration_sec,
            "channels_used": channels_used,
            "pcap_path": pcap_path.to_string_lossy()
        }))
    }

    /// BLE connection following implementation.
    pub(in crate::sidecar) async fn btle_follow(&self, params: Value) -> Result<Value> {
        let access_address = params
            .get("access_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                UbertoothError::InvalidParameter("Missing 'access_address'".to_string())
            })?;

        let duration_sec = params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);

        tracing::info!(
            "Following BLE connection: {} for {}s",
            access_address,
            duration_sec
        );

        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("btlefollow");
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Execute ubertooth-btle with -f (follow) and -a (access address)
        let duration_str = duration_sec.to_string();
        let pcap_str = pcap_path.to_string_lossy().to_string();

        let output = self
            .execute_ubertooth_command(
                "ubertooth-btle",
                &[
                    "-f",
                    "-a",
                    access_address,
                    "-r",
                    pcap_str.as_str(),
                    "-d",
                    duration_str.as_str(),
                ],
            )
            .await?;

        // Parse output for connection info
        let mut packets_captured = 0;
        let mut connection_events = 0;
        let mut crc_valid = 0;
        let mut crc_total = 0;

        for line in output.lines() {
            if line.contains("data:") || line.contains("Data packet") {
                packets_captured += 1;
            }
            if line.contains("connection event") || line.contains("CE:") {
                connection_events += 1;
            }
            if line.contains("CRC") {
                crc_total += 1;
                if line.contains("OK") || line.contains("valid") || line.contains("pass") {
                    crc_valid += 1;
                }
            }
        }

        let crc_valid_percent = if crc_total > 0 {
            (crc_valid as f64 / crc_total as f64) * 100.0
        } else {
            0.0
        };

        // Save metadata
        let file_size_bytes = if pcap_path.exists() {
            std::fs::metadata(&pcap_path)?.len()
        } else {
            0
        };

        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "btle_follow".to_string(),
            duration_sec: Some(duration_sec),
            packet_count: packets_captured,
            file_size_bytes,
            pcap_path: pcap_path.to_string_lossy().to_string(),
            tags: vec![format!("access_address:{}", access_address)],
            description: format!("Following BLE connection {}", access_address),
            category: None,
            notes: None,
        };
        store.save_metadata(&metadata)?;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "access_address": access_address,
            "packets_captured": packets_captured,
            "connection_events": connection_events,
            "crc_valid_percent": crc_valid_percent,
            "pcap_path": pcap_path.to_string_lossy()
        }))
    }
}
