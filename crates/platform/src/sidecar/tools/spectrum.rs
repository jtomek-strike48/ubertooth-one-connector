//! Spectrum analysis methods for SidecarManager.

use crate::capture_store::{CaptureMetadata, CaptureStore};
use chrono::Utc;
use serde_json::{json, Value};
use ubertooth_core::error::{Result, UbertoothError};

use super::super::SidecarManager;

impl SidecarManager {
    /// Spectrum analysis implementation.
    pub(in crate::sidecar) async fn bt_specan(&self, params: Value) -> Result<Value> {
        // Parse parameters
        let low_freq = params
            .get("low_freq")
            .and_then(|v| v.as_u64())
            .unwrap_or(2402);

        let high_freq = params
            .get("high_freq")
            .and_then(|v| v.as_u64())
            .unwrap_or(2480);

        let duration_sec = params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        tracing::info!(
            "Starting spectrum scan: {}-{} MHz, duration={}s",
            low_freq,
            high_freq,
            duration_sec
        );

        // Create capture store
        let store = CaptureStore::new()?;

        // Generate capture ID
        let capture_id = CaptureStore::generate_capture_id("specan");

        // Prepare output file path
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));
        let pcap_path_str = pcap_path
            .to_str()
            .ok_or_else(|| UbertoothError::BackendError("Invalid path".to_string()))?;

        // Build ubertooth-specan command
        // -l: low frequency
        // -u: high (upper) frequency
        // -t: timeout duration (estimated based on range)
        let low_str = low_freq.to_string();
        let high_str = high_freq.to_string();
        let args = vec!["-l", low_str.as_str(), "-u", high_str.as_str()];

        tracing::debug!("Executing: ubertooth-specan {:?}", args);

        // Execute ubertooth-specan (with timeout)
        // Note: ubertooth-specan outputs to stdout, we'll capture it
        let output = tokio::time::timeout(
            tokio::time::Duration::from_secs(duration_sec + 5),
            self.execute_ubertooth_command("ubertooth-specan", &args),
        )
        .await
        .map_err(|_| UbertoothError::BackendError("Spectrum scan timed out".to_string()))??;

        tracing::debug!("ubertooth-specan output length: {} bytes", output.len());

        // Parse output for RSSI data
        // ubertooth-specan outputs frequency and RSSI values
        let mut scan_results = Vec::new();
        for line in output.lines().take(100) {
            // Limit to first 100 lines for Phase 1
            if let Some((freq_str, rssi_str)) = line.split_once(',') {
                if let (Ok(freq), Ok(rssi)) = (
                    freq_str.trim().parse::<i32>(),
                    rssi_str.trim().parse::<i32>(),
                ) {
                    let channel = (freq - 2402).max(0);
                    scan_results.push(json!({
                        "frequency_mhz": freq,
                        "channel": channel,
                        "rssi_avg": rssi,
                        "rssi_max": rssi,
                        "rssi_min": rssi,
                        "activity_percent": if rssi > -80 { 50.0 } else { 0.0 }
                    }));
                }
            }
        }

        // Identify hotspots (frequencies with high RSSI)
        let mut hotspots = Vec::new();
        for result in &scan_results {
            if let Some(rssi) = result.get("rssi_max").and_then(|v| v.as_i64()) {
                if rssi > -70 {
                    hotspots.push(json!({
                        "frequency_mhz": result["frequency_mhz"],
                        "rssi_max": rssi,
                        "interpretation": "High activity detected"
                    }));
                }
            }
        }

        // Create capture metadata
        let file_size = std::fs::metadata(&pcap_path).map(|m| m.len()).unwrap_or(0);

        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "specan".to_string(),
            packet_count: scan_results.len(),
            duration_sec: Some(duration_sec),
            file_size_bytes: file_size,
            pcap_path: pcap_path_str.to_string(),
            tags: vec![
                "specan".to_string(),
                format!("{}-{}_MHz", low_freq, high_freq),
            ],
            description: format!("Spectrum scan {}-{} MHz", low_freq, high_freq),
            category: None,
            notes: None,
        };

        // Save metadata
        store.save_metadata(&metadata)?;

        tracing::info!(
            "Spectrum scan complete: {} frequency points",
            scan_results.len()
        );

        // Return result
        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "frequency_range": [low_freq, high_freq],
            "duration_sec": duration_sec,
            "scan_results": scan_results,
            "hotspots": hotspots
        }))
    }
}
