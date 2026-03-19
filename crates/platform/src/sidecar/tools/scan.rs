//! Scanning methods for SidecarManager.

use crate::capture_store::{CaptureMetadata, CaptureStore};
use chrono::Utc;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::time::Duration;
use ubertooth_core::error::{Result, UbertoothError};

use super::super::SidecarManager;

impl SidecarManager {
    /// BLE scan implementation.
    pub(in crate::sidecar) async fn btle_scan(&self, params: Value) -> Result<Value> {
        // Parse parameters
        let total_duration = params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        let _channel = params.get("channel").and_then(|v| v.as_u64()).unwrap_or(37);

        tracing::info!(
            "Starting multi-channel BLE scan: duration={}s (scanning channels 37, 38, 39)",
            total_duration
        );

        // Create capture store
        let store = CaptureStore::new()?;

        // Generate capture ID
        let capture_id = CaptureStore::generate_capture_id("btle");

        // BLE advertising channels
        let channels = vec![37u64, 38u64, 39u64];

        // Split duration across channels
        let duration_per_channel = std::cmp::max(total_duration / channels.len() as u64, 1);

        // Scan each channel and collect results
        let mut channel_pcaps = Vec::new();
        let mut total_packets = 0u64;

        for ch in &channels {
            tracing::info!("Scanning channel {} for {}s...", ch, duration_per_channel);

            // Prepare output file path for this channel
            let channel_pcap_path = store
                .captures_dir()
                .join(format!("{}_ch{}.pcap", capture_id, ch));
            let channel_pcap_str = channel_pcap_path
                .to_str()
                .ok_or_else(|| UbertoothError::BackendError("Invalid path".to_string()))?
                .to_string();

            // Scan single channel
            match self
                .scan_single_channel(*ch, duration_per_channel, &channel_pcap_str)
                .await
            {
                Ok(packet_count) => {
                    total_packets += packet_count;
                    channel_pcaps.push(channel_pcap_str);
                    tracing::info!("Channel {} scan complete: {} packets", ch, packet_count);
                }
                Err(e) => {
                    tracing::warn!("Channel {} scan failed: {}", ch, e);
                    // Continue with other channels
                }
            }
        }

        // Merge all PCAP files into one
        let final_pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));
        let final_pcap_str = final_pcap_path
            .to_str()
            .ok_or_else(|| UbertoothError::BackendError("Invalid path".to_string()))?;

        if !channel_pcaps.is_empty() {
            self.merge_pcap_files(&channel_pcaps, final_pcap_str)
                .await?;

            // Clean up individual channel files
            for pcap in &channel_pcaps {
                let _ = std::fs::remove_file(pcap);
            }
        } else {
            // No captures, create empty PCAP
            tracing::warn!("No packets captured on any channel");
            std::fs::write(&final_pcap_path, []).map_err(|e| {
                UbertoothError::BackendError(format!("Failed to create empty PCAP: {}", e))
            })?;
        }

        // Get final file size
        let file_size = std::fs::metadata(&final_pcap_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Create capture metadata
        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "btle_sniff".to_string(),
            packet_count: total_packets as usize,
            duration_sec: Some(total_duration),
            file_size_bytes: file_size,
            pcap_path: final_pcap_str.to_string(),
            tags: vec!["ble".to_string(), "multi-channel".to_string()],
            description: "Multi-channel BLE scan (channels 37, 38, 39)".to_string(),
            category: None,
            notes: None,
        };

        // Save metadata
        store.save_metadata(&metadata)?;

        tracing::info!(
            "Multi-channel BLE scan complete: {} packets total, {} bytes",
            total_packets,
            file_size
        );

        // Return result
        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "scan_duration_sec": total_duration,
            "channels_scanned": channels,
            "devices_found": [],  // TODO Phase 2: Parse PCAP to extract devices
            "total_packets": total_packets,
            "pcap_path": final_pcap_str,
            "preview": [
                format!("Scanned channels 37, 38, 39 ({}s each)", duration_per_channel),
                format!("Captured {} BLE packets total", total_packets),
                format!("Saved to: {}", final_pcap_str)
            ]
        }))
    }

    /// Scan a single BLE advertising channel
    async fn scan_single_channel(
        &self,
        channel: u64,
        duration_sec: u64,
        pcap_path: &str,
    ) -> Result<u64> {
        // Build ubertooth-btle command
        let channel_str = channel.to_string();
        let args = vec![
            "-n", // Scan mode (don't follow connections)
            "-A",
            channel_str.as_str(),
            "-q",
            pcap_path,
        ];

        tracing::debug!("Executing: ubertooth-btle {:?}", args);

        // Spawn process so we can kill it
        let mut child = tokio::process::Command::new("ubertooth-btle")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                UbertoothError::BackendError(format!("Failed to spawn ubertooth-btle: {}", e))
            })?;

        // Wait for duration, then kill process
        tokio::time::sleep(Duration::from_secs(duration_sec)).await;

        tracing::debug!("Duration elapsed, killing ubertooth-btle process...");
        let _ = child.kill().await;

        let _output_result = child.wait_with_output().await.map_err(|e| {
            UbertoothError::BackendError(format!("Failed to wait for ubertooth-btle: {}", e))
        })?;

        // Count packets from PCAP file size (rough estimate)
        let file_size = std::fs::metadata(pcap_path).map(|m| m.len()).unwrap_or(0);

        // Estimate packet count: PCAP header is 24 bytes, typical BLE packet ~50-100 bytes
        let packet_count = if file_size > 24 {
            (file_size - 24) / 75 // Conservative estimate
        } else {
            0
        };

        Ok(packet_count)
    }

    /// Merge multiple PCAP files into one using mergecap or manual merge
    async fn merge_pcap_files(&self, input_files: &[String], output_file: &str) -> Result<()> {
        // Try using mergecap first (from wireshark-common)
        let mergecap_result = tokio::process::Command::new("mergecap")
            .arg("-w")
            .arg(output_file)
            .args(input_files)
            .output()
            .await;

        match mergecap_result {
            Ok(output) if output.status.success() => {
                tracing::debug!("Merged PCAP files using mergecap");
                Ok(())
            }
            _ => {
                // Fallback: manual merge by copying first file and appending packets from others
                tracing::debug!("mergecap not available, using manual merge");

                if input_files.is_empty() {
                    return Err(UbertoothError::BackendError(
                        "No PCAP files to merge".to_string(),
                    ));
                }

                // Copy first file as base
                std::fs::copy(&input_files[0], output_file).map_err(|e| {
                    UbertoothError::BackendError(format!("Failed to copy base PCAP: {}", e))
                })?;

                // Append packets from other files (skip their headers)
                for input_file in &input_files[1..] {
                    let data = std::fs::read(input_file).map_err(|e| {
                        UbertoothError::BackendError(format!("Failed to read PCAP: {}", e))
                    })?;

                    // Skip PCAP global header (24 bytes) and append packet data
                    if data.len() > 24 {
                        let mut output = std::fs::OpenOptions::new()
                            .append(true)
                            .open(output_file)
                            .map_err(|e| {
                                UbertoothError::BackendError(format!(
                                    "Failed to open output PCAP: {}",
                                    e
                                ))
                            })?;

                        std::io::Write::write_all(&mut output, &data[24..]).map_err(|e| {
                            UbertoothError::BackendError(format!(
                                "Failed to append PCAP data: {}",
                                e
                            ))
                        })?;
                    }
                }

                tracing::debug!("Manual PCAP merge complete");
                Ok(())
            }
        }
    }

    /// Bluetooth Classic scan implementation.
    pub(in crate::sidecar) async fn bt_scan(&self, params: Value) -> Result<Value> {
        let duration_sec = params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        let _extended_inquiry = params
            .get("extended_inquiry")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        tracing::info!("Starting Bluetooth Classic scan: {}s", duration_sec);

        // Create capture store and generate ID
        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("btscan");
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Execute ubertooth-scan with timeout
        // Note: ubertooth-scan does not support PCAP output directly
        let duration_str = duration_sec.to_string();

        let output = self
            .execute_ubertooth_command("ubertooth-scan", &["-t", duration_str.as_str()])
            .await?;

        // Parse output for discovered devices
        let mut devices_found = Vec::new();
        for line in output.lines() {
            // Parse device lines (format: BD_ADDR - Device Name)
            if line.contains(':') && (line.len() > 17) {
                let parts: Vec<&str> = line.splitn(2, " - ").collect();
                if !parts.is_empty() {
                    let bd_addr = parts[0].trim();
                    let name = if parts.len() > 1 {
                        parts[1].trim()
                    } else {
                        "Unknown"
                    };
                    devices_found.push(json!({
                        "bd_addr": bd_addr,
                        "name": name
                    }));
                }
            }
        }

        let total_devices = devices_found.len();

        // Save metadata
        let file_size_bytes = if pcap_path.exists() {
            std::fs::metadata(&pcap_path)?.len()
        } else {
            0
        };

        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "bt_scan".to_string(),
            duration_sec: Some(duration_sec),
            packet_count: total_devices,
            file_size_bytes,
            pcap_path: pcap_path.to_string_lossy().to_string(),
            tags: Vec::new(),
            description: format!("Bluetooth Classic scan, {} devices found", total_devices),
            category: None,
            notes: None,
        };
        store.save_metadata(&metadata)?;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "devices_found": devices_found,
            "total_devices": total_devices,
            "pcap_path": pcap_path.to_string_lossy(),
            "duration_sec": duration_sec
        }))
    }
}
