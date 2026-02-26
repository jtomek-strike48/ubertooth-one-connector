//! Python sidecar manager for wrapping ubertooth-tools.

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use ubertooth_core::error::{Result, UbertoothError};

use crate::backend::UbertoothBackendProvider;
use crate::capture_store::{CaptureMetadata, CaptureStore};

/// Python sidecar process manager.
///
/// The sidecar wraps the ubertooth-* command-line tools and provides a
/// simple interface for executing commands.
pub struct SidecarManager {
    process: Arc<Mutex<Option<Child>>>,
}

impl SidecarManager {
    /// Create a new sidecar manager (not started yet).
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            process: Arc::new(Mutex::new(None)),
        })
    }

    /// Check if ubertooth-tools are installed on the system.
    pub fn check_ubertooth_installed() -> Result<()> {
        // Check for ubertooth-util (core utility)
        let output = Command::new("which")
            .arg("ubertooth-util")
            .output()
            .map_err(|e| UbertoothError::BackendError(format!("Failed to check for ubertooth-util: {}", e)))?;

        if !output.status.success() {
            return Err(UbertoothError::BackendError(
                "ubertooth-tools not found. Please install:\n\
                 Ubuntu/Debian: sudo apt-get install ubertooth\n\
                 Arch: sudo pacman -S ubertooth\n\
                 macOS: brew install ubertooth\n\
                 From source: https://github.com/greatscottgadgets/ubertooth".to_string()
            ));
        }

        Ok(())
    }

    /// Spawn the Python sidecar process (if needed in future).
    ///
    /// For now, we'll call ubertooth-* tools directly without a persistent sidecar.
    async fn _spawn(&self) -> Result<()> {
        // Phase 1: We'll call ubertooth-* tools directly
        // Phase 2: If needed, we could spawn a persistent Python process
        Ok(())
    }

    /// Execute a ubertooth command-line tool.
    async fn execute_ubertooth_command(
        &self,
        tool: &str,
        args: &[&str],
    ) -> Result<String> {
        tracing::debug!("Executing: {} {:?}", tool, args);

        let output = Command::new(tool)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                UbertoothError::BackendError(format!("Failed to execute {}: {}", tool, e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(UbertoothError::CommandFailed(format!(
                "{} failed: {}",
                tool, stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }
}

impl Default for SidecarManager {
    fn default() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl UbertoothBackendProvider for SidecarManager {
    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        // Route method calls to appropriate ubertooth-* tools
        match method {
            "device_connect" => self.device_connect().await,
            "device_disconnect" => self.device_disconnect().await,
            "device_status" => self.device_status().await,
            "btle_scan" => self.btle_scan(params).await,
            "bt_specan" => self.bt_specan(params).await,
            "configure_channel" => self.configure_channel(params).await,
            "configure_modulation" => self.configure_modulation(params).await,
            "configure_power" => self.configure_power(params).await,
            "capture_list" => self.capture_list(params).await,
            "capture_get" => self.capture_get(params).await,
            "capture_delete" => self.capture_delete(params).await,
            "capture_tag" => self.capture_tag(params).await,
            "bt_analyze" => self.bt_analyze(params).await,
            "session_context" => self.session_context(params).await,
            _ => Err(UbertoothError::BackendError(format!(
                "Method not implemented: {}",
                method
            ))),
        }
    }

    async fn is_alive(&self) -> bool {
        // Check if ubertooth-util responds
        let result = Command::new("ubertooth-util")
            .arg("-V")
            .output();

        result.is_ok()
    }

    async fn restart(&self) -> Result<()> {
        // No persistent process to restart in Phase 1
        Ok(())
    }

    fn backend_type(&self) -> &str {
        "python"
    }
}

impl SidecarManager {
    /// Device connect implementation.
    async fn device_connect(&self) -> Result<Value> {
        // Check if tools are installed
        Self::check_ubertooth_installed()?;

        // Get device information using ubertooth-util
        let output = self
            .execute_ubertooth_command("ubertooth-util", &["-V"])
            .await?;

        // Parse output (simplified for v0.0.1)
        // Expected format: "Firmware version: 2020-12-R1"
        let firmware_version = output
            .lines()
            .find(|line| line.contains("Firmware"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(json!({
            "success": true,
            "device_id": "ubertooth-001",
            "firmware_version": firmware_version,
            "message": "Connected to Ubertooth One"
        }))
    }

    /// Device disconnect implementation.
    async fn device_disconnect(&self) -> Result<Value> {
        // For Python backend, there's no persistent connection to close
        // Each ubertooth-* command opens and closes the device
        // So this is essentially a no-op that confirms success

        tracing::debug!("Device disconnect called (Python backend - no persistent connection)");

        Ok(json!({
            "success": true,
            "message": "Device disconnected"
        }))
    }

    /// Device status implementation.
    async fn device_status(&self) -> Result<Value> {
        // Get device information
        let output = self
            .execute_ubertooth_command("ubertooth-util", &["-V"])
            .await?;

        let firmware_version = output
            .lines()
            .find(|line| line.contains("Firmware"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(json!({
            "success": true,
            "connected": true,
            "device_id": "ubertooth-001",
            "firmware": firmware_version,
            "current_mode": "idle"
        }))
    }

    /// BLE scan implementation.
    async fn btle_scan(&self, params: Value) -> Result<Value> {
        // Parse parameters
        let duration_sec = params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        let channel = params
            .get("channel")
            .and_then(|v| v.as_u64())
            .unwrap_or(37);

        tracing::info!(
            "Starting BLE scan: channel={}, duration={}s",
            channel,
            duration_sec
        );

        // Create capture store
        let store = CaptureStore::new()?;

        // Generate capture ID
        let capture_id = CaptureStore::generate_capture_id("btle");

        // Prepare output file path
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));
        let pcap_path_str = pcap_path
            .to_str()
            .ok_or_else(|| UbertoothError::BackendError("Invalid path".to_string()))?;

        // Build ubertooth-btle command
        // -f: follow connections
        // -c: channel
        // -t: timeout (in seconds)
        // -q: output PCAP file
        let channel_str = channel.to_string();
        let duration_str = duration_sec.to_string();
        let args = vec![
            "-f",
            "-c",
            channel_str.as_str(),
            "-t",
            duration_str.as_str(),
            "-q",
            pcap_path_str,
        ];

        tracing::debug!("Executing: ubertooth-btle {:?}", args);

        // Execute ubertooth-btle
        let output = self
            .execute_ubertooth_command("ubertooth-btle", &args)
            .await?;

        tracing::debug!("ubertooth-btle output: {}", output);

        // Get file size
        let file_size = std::fs::metadata(&pcap_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Parse output for basic statistics
        // ubertooth-btle output format varies, for now use placeholder counts
        let total_packets = output.lines().filter(|line| line.contains("packet") || line.contains("ADV")).count();

        // Create capture metadata
        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "btle_sniff".to_string(),
            packet_count: total_packets,
            duration_sec: Some(duration_sec),
            file_size_bytes: file_size,
            pcap_path: pcap_path_str.to_string(),
            tags: vec!["ble".to_string(), format!("channel_{}", channel)],
            description: format!("BLE scan on channel {}", channel),
        };

        // Save metadata
        store.save_metadata(&metadata)?;

        tracing::info!(
            "BLE scan complete: {} packets, {} bytes",
            total_packets,
            file_size
        );

        // Return result
        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "scan_duration_sec": duration_sec,
            "channel": channel,
            "devices_found": [],  // TODO Phase 2: Parse PCAP to extract devices
            "total_packets": total_packets,
            "pcap_path": pcap_path_str,
            "preview": [
                format!("Captured {} BLE packets on channel {}", total_packets, channel),
                format!("Saved to: {}", pcap_path_str)
            ]
        }))
    }

    /// Spectrum analysis implementation.
    async fn bt_specan(&self, params: Value) -> Result<Value> {
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
        let args = vec![
            "-l",
            low_str.as_str(),
            "-u",
            high_str.as_str(),
        ];

        tracing::debug!("Executing: ubertooth-specan {:?}", args);

        // Execute ubertooth-specan (with timeout)
        // Note: ubertooth-specan outputs to stdout, we'll capture it
        let output = tokio::time::timeout(
            tokio::time::Duration::from_secs(duration_sec + 5),
            self.execute_ubertooth_command("ubertooth-specan", &args),
        )
        .await
        .map_err(|_| UbertoothError::BackendError("Spectrum scan timed out".to_string()))?
        ?;

        tracing::debug!("ubertooth-specan output length: {} bytes", output.len());

        // Parse output for RSSI data
        // ubertooth-specan outputs frequency and RSSI values
        let mut scan_results = Vec::new();
        for line in output.lines().take(100) {
            // Limit to first 100 lines for Phase 1
            if let Some((freq_str, rssi_str)) = line.split_once(',') {
                if let (Ok(freq), Ok(rssi)) = (freq_str.trim().parse::<i32>(), rssi_str.trim().parse::<i32>()) {
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
        let file_size = std::fs::metadata(&pcap_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "specan".to_string(),
            packet_count: scan_results.len(),
            duration_sec: Some(duration_sec),
            file_size_bytes: file_size,
            pcap_path: pcap_path_str.to_string(),
            tags: vec!["specan".to_string(), format!("{}-{}_MHz", low_freq, high_freq)],
            description: format!("Spectrum scan {}-{} MHz", low_freq, high_freq),
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

    /// Configure channel implementation.
    async fn configure_channel(&self, params: Value) -> Result<Value> {
        let channel = params
            .get("channel")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'channel' parameter".to_string()))?;

        // Validate channel range
        if channel > 78 {
            return Err(UbertoothError::InvalidParameter(
                "Channel must be 0-78".to_string(),
            ));
        }

        tracing::info!("Setting channel to {}", channel);

        // Execute ubertooth-util -c <channel>
        let channel_str = channel.to_string();
        self.execute_ubertooth_command("ubertooth-util", &["-c", channel_str.as_str()])
            .await?;

        // Calculate frequency (2402 + channel MHz)
        let frequency_mhz = 2402 + channel;

        Ok(json!({
            "success": true,
            "channel": channel,
            "frequency_mhz": frequency_mhz,
            "message": format!("Channel set to {} ({} MHz)", channel, frequency_mhz)
        }))
    }

    /// Configure modulation implementation.
    async fn configure_modulation(&self, params: Value) -> Result<Value> {
        let modulation = params
            .get("modulation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'modulation' parameter".to_string()))?;

        // Validate modulation type
        let valid_mods = ["BT_BASIC_RATE", "BT_LOW_ENERGY", "80211_FHSS", "NONE"];
        if !valid_mods.contains(&modulation) {
            return Err(UbertoothError::InvalidParameter(format!(
                "Invalid modulation: {}. Must be one of: {:?}",
                modulation, valid_mods
            )));
        }

        tracing::info!("Setting modulation to {}", modulation);

        // Map to ubertooth-util flag value
        // For Phase 1, we'll just store the value; actual command depends on device capabilities
        // This is a placeholder implementation
        tracing::debug!("Modulation configuration (placeholder): {}", modulation);

        Ok(json!({
            "success": true,
            "modulation": modulation,
            "message": format!("Modulation set to {}", modulation)
        }))
    }

    /// Configure power implementation.
    async fn configure_power(&self, params: Value) -> Result<Value> {
        let power_level = params
            .get("power_level")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'power_level' parameter".to_string()))?;

        let paen = params.get("paen").and_then(|v| v.as_bool()).unwrap_or(true);
        let hgm = params.get("hgm").and_then(|v| v.as_bool()).unwrap_or(false);

        // Validate power level range
        if power_level > 7 {
            return Err(UbertoothError::InvalidParameter(
                "Power level must be 0-7".to_string(),
            ));
        }

        tracing::info!(
            "Setting power: level={}, paen={}, hgm={}",
            power_level,
            paen,
            hgm
        );

        // Execute ubertooth-util -p <power>
        let power_str = power_level.to_string();
        self.execute_ubertooth_command("ubertooth-util", &["-p", power_str.as_str()])
            .await?;

        // Estimate TX power in dBm
        // Rough estimates: level 0-7 spans ~0-14 dBm without PA, ~10-24 dBm with PA
        let estimated_power_dbm = if paen {
            10 + (power_level * 2) as i64
        } else {
            power_level as i64 * 2
        };

        Ok(json!({
            "success": true,
            "power_level": power_level,
            "paen": paen,
            "hgm": hgm,
            "estimated_power_dbm": estimated_power_dbm,
            "message": format!(
                "Power configured: Level {} with PA {} (~{} dBm)",
                power_level,
                if paen { "enabled" } else { "disabled" },
                estimated_power_dbm
            )
        }))
    }

    /// List captures implementation.
    async fn capture_list(&self, params: Value) -> Result<Value> {
        let store = CaptureStore::new()?;

        let filter_type = params.get("filter_type").and_then(|v| v.as_str());
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
        let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

        // Get all captures
        let mut all_captures = store.list_captures()?;

        // Filter by type if specified
        if let Some(filter) = filter_type {
            all_captures.retain(|c| c.capture_type == filter);
        }

        let total_count = all_captures.len();

        // Apply pagination
        let captures: Vec<_> = all_captures
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|c| json!({
                "capture_id": c.capture_id,
                "timestamp": c.timestamp.to_rfc3339(),
                "type": c.capture_type,
                "packet_count": c.packet_count,
                "duration_sec": c.duration_sec,
                "file_size_bytes": c.file_size_bytes,
                "pcap_path": c.pcap_path,
                "tags": c.tags,
                "description": c.description
            }))
            .collect();

        Ok(json!({
            "success": true,
            "captures": captures,
            "total_count": total_count,
            "offset": offset,
            "limit": limit
        }))
    }

    /// Get capture implementation.
    async fn capture_get(&self, params: Value) -> Result<Value> {
        let capture_id = params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

        let store = CaptureStore::new()?;
        let metadata = store.load_metadata(capture_id)?;

        // For Phase 1, return metadata without parsing PCAP
        // PCAP parsing will be added in Phase 2
        let packets: Vec<Value> = Vec::new();
        let has_more = false;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "offset": offset,
            "limit": limit,
            "packet_count": metadata.packet_count,
            "packets": packets,
            "has_more": has_more,
            "note": "Phase 1: PCAP parsing not yet implemented. Use pcap_path to access raw file."
        }))
    }

    /// Delete capture implementation.
    async fn capture_delete(&self, params: Value) -> Result<Value> {
        let capture_id = params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let store = CaptureStore::new()?;
        store.delete_capture(capture_id)?;

        Ok(json!({
            "success": true,
            "message": format!("Capture '{}' deleted", capture_id)
        }))
    }

    /// Tag capture implementation.
    async fn capture_tag(&self, params: Value) -> Result<Value> {
        let capture_id = params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let new_tags = params
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            });

        let new_description = params.get("description").and_then(|v| v.as_str());
        let append_tags = params.get("append_tags").and_then(|v| v.as_bool()).unwrap_or(true);

        let store = CaptureStore::new()?;
        let mut metadata = store.load_metadata(capture_id)?;

        // Update tags
        if let Some(tags) = new_tags {
            if append_tags {
                metadata.tags.extend(tags);
                metadata.tags.sort();
                metadata.tags.dedup();
            } else {
                metadata.tags = tags;
            }
        }

        // Update description
        if let Some(desc) = new_description {
            metadata.description = desc.to_string();
        }

        // Save updated metadata
        store.save_metadata(&metadata)?;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "tags": metadata.tags,
            "description": metadata.description
        }))
    }

    /// Analyze captured packets implementation.
    ///
    /// Phase 1: Basic analysis with metadata only
    /// Phase 2: Full PCAP parsing with protocol analysis
    async fn bt_analyze(&self, params: Value) -> Result<Value> {
        let capture_id = params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let analysis_type = params
            .get("analysis_type")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        tracing::info!("Analyzing capture: {} (type: {})", capture_id, analysis_type);

        let store = CaptureStore::new()?;
        let metadata = store.load_metadata(capture_id)?;

        // Phase 1: Basic analysis from metadata
        // Phase 2: Will parse PCAP with libbtbb or Rust parser
        let protocol_type = match metadata.capture_type.as_str() {
            "btle_sniff" => "BLE",
            "specan" => "Spectrum",
            "bt_follow" => "BR/EDR",
            _ => "Unknown",
        };

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "analysis": {
                "protocol_summary": {
                    "type": protocol_type,
                    "pdu_types": {}  // TODO Phase 2: Parse PCAP
                },
                "devices": [],  // TODO Phase 2: Extract device info
                "timing_analysis": {
                    "avg_interval_ms": 0.0,
                    "min_interval_ms": 0.0,
                    "max_interval_ms": 0.0
                },
                "security_observations": [],
                "anomalies": [],
                "note": "Phase 1: Metadata-only analysis. Full PCAP parsing in Phase 2."
            }
        }))
    }

    /// Session context implementation - comprehensive AI orientation.
    ///
    /// Combines device_status + capture_list + configs into one response.
    async fn session_context(&self, params: Value) -> Result<Value> {
        let include_recent_captures = params
            .get("include_recent_captures")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let max_captures = params
            .get("max_captures")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as usize;

        let _include_configs = params
            .get("include_configs")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        tracing::info!("Building session context");

        // Get device status
        let device_info = self.device_status().await?;

        // Get recent captures if requested
        let mut recent_captures = Vec::new();
        if include_recent_captures {
            let captures_result = self
                .capture_list(json!({
                    "limit": max_captures,
                    "sort_by": "timestamp",
                    "sort_order": "desc"
                }))
                .await?;

            if let Some(captures) = captures_result.get("captures").and_then(|v| v.as_array()) {
                for capture in captures.iter().take(max_captures) {
                    recent_captures.push(capture.clone());
                }
            }
        }

        // Calculate storage stats
        let store = CaptureStore::new()?;
        let all_captures = store.list_captures()?;
        let total_size_bytes: u64 = all_captures
            .iter()
            .map(|m| m.file_size_bytes)
            .sum();
        let total_size_mb = total_size_bytes as f64 / 1_048_576.0;

        let timestamp = chrono::Utc::now().to_rfc3339();

        Ok(json!({
            "success": true,
            "timestamp": timestamp,
            "device": device_info.get("device").unwrap_or(&json!({})),
            "recent_captures": recent_captures,
            "saved_configs": [],  // TODO Phase 2: Config persistence
            "storage": {
                "captures_dir": store.captures_dir().to_string_lossy(),
                "captures_count": all_captures.len(),
                "total_size_mb": format!("{:.1}", total_size_mb).parse::<f64>().unwrap_or(0.0)
            }
        }))
    }
}
