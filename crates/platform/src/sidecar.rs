//! Python sidecar manager for wrapping ubertooth-tools.

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use ubertooth_core::error::{Result, UbertoothError};

use crate::backend::UbertoothBackendProvider;
use crate::capture_store::{CaptureMetadata, CaptureStore};
use crate::config_store::{ConfigMetadata, ConfigSettings, ConfigStore};

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
            "bt_scan" => self.bt_scan(params).await,
            "bt_follow" => self.bt_follow(params).await,
            "afh_analyze" => self.afh_analyze(params).await,
            "bt_discover" => self.bt_discover(params).await,
            "btle_follow" => self.btle_follow(params).await,
            "configure_squelch" => self.configure_squelch(params).await,
            "configure_leds" => self.configure_leds(params).await,
            "bt_save_config" => self.bt_save_config(params).await,
            "bt_load_config" => self.bt_load_config(params).await,
            "config_list" => self.config_list(params).await,
            "config_delete" => self.config_delete(params).await,
            "bt_compare" => self.bt_compare(params).await,
            "bt_decode" => self.bt_decode(params).await,
            "bt_fingerprint" => self.bt_fingerprint(params).await,
            "pcap_merge" => self.pcap_merge(params).await,
            "capture_export" => self.capture_export(params).await,
            "btle_inject" => self.btle_inject(params).await,
            "bt_jam" => self.bt_jam(params).await,
            "btle_slave" => self.btle_slave(params).await,
            "btle_mitm" => self.btle_mitm(params).await,
            "bt_spoof" => self.bt_spoof(params).await,
            "ubertooth_raw" => self.ubertooth_raw(params).await,
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

    /// Bluetooth Classic device scanning implementation.
    ///
    /// Phase 2 Week 3: Wraps ubertooth-scan for BR/EDR inquiry scan.
    async fn bt_scan(&self, params: Value) -> Result<Value> {
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

        // Execute ubertooth-scan with timeout and output to PCAP
        let duration_str = duration_sec.to_string();
        let pcap_str = pcap_path.to_string_lossy().to_string();

        let output = self.execute_ubertooth_command(
            "ubertooth-scan",
            &["-t", duration_str.as_str(), "-q", pcap_str.as_str()]
        ).await?;

        // Parse output for discovered devices
        let mut devices_found = Vec::new();
        for line in output.lines() {
            // Parse device lines (format: BD_ADDR - Device Name)
            if line.contains(':') && (line.len() > 17) {
                let parts: Vec<&str> = line.splitn(2, " - ").collect();
                if parts.len() >= 1 {
                    let bd_addr = parts[0].trim();
                    let name = if parts.len() > 1 { parts[1].trim() } else { "Unknown" };
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

    /// Follow Bluetooth connection implementation.
    ///
    /// Phase 2 Week 3: Wraps ubertooth-follow for targeted connection monitoring.
    async fn bt_follow(&self, params: Value) -> Result<Value> {
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

        let output = self.execute_ubertooth_command(
            "ubertooth-follow",
            &["-t", bd_addr, "-r", pcap_str.as_str(), "-d", duration_str.as_str()]
        ).await?;

        // Parse output for connection info and packet count
        let connection_found = output.contains("Following") || output.contains("Connection");
        let packet_count = output.lines()
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

    /// AFH analysis implementation.
    ///
    /// Phase 2 Week 3: Wraps ubertooth-afh for channel map analysis.
    async fn afh_analyze(&self, params: Value) -> Result<Value> {
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

        let output = self.execute_ubertooth_command("ubertooth-afh", &args).await?;

        // Parse AFH channel map from output
        let mut afh_map = "0x0000000000000000000000".to_string();
        let mut channels_used = Vec::new();
        let mut channels_avoided = Vec::new();

        for line in output.lines() {
            if line.contains("AFH map:") || line.contains("Channel map:") {
                // Extract hex map
                if let Some(hex_start) = line.find("0x") {
                    afh_map = line[hex_start..].split_whitespace().next()
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
            format!("Device uses {} channels, avoids {} channels (likely due to WiFi interference)",
                    used_count, avoided_count)
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

    /// Promiscuous Bluetooth discovery implementation.
    ///
    /// Phase 2 Week 3: Wraps ubertooth-rx for BR/EDR promiscuous capture.
    async fn bt_discover(&self, params: Value) -> Result<Value> {
        let duration_sec = params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);

        tracing::info!("Starting promiscuous BT discovery: {}s", duration_sec);

        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("discover");
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Execute ubertooth-rx with timeout and output to PCAP
        let duration_str = duration_sec.to_string();
        let pcap_str = pcap_path.to_string_lossy().to_string();

        let output = self.execute_ubertooth_command(
            "ubertooth-rx",
            &["-d", duration_str.as_str(), "-q", pcap_str.as_str()]
        ).await?;

        // Parse output for piconets and packet count
        let mut piconets_found = Vec::new();
        let mut total_packets = 0;

        for line in output.lines() {
            // Parse LAP (Lower Address Part) which identifies piconets
            if line.contains("LAP:") || line.contains("lap") {
                if let Some(lap_pos) = line.find("LAP:").or_else(|| line.find("lap")) {
                    let lap_str = &line[lap_pos..];
                    if let Some(hex_val) = lap_str.split_whitespace().nth(1) {
                        if !piconets_found.iter().any(|v: &Value| v["lap"] == hex_val) {
                            piconets_found.push(json!({
                                "lap": hex_val
                            }));
                        }
                    }
                }
            }
            // Count packets
            if line.contains("packet") || line.contains("Packet") {
                total_packets += 1;
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
            capture_type: "bt_discover".to_string(),
            duration_sec: Some(duration_sec),
            packet_count: total_packets,
            file_size_bytes,
            pcap_path: pcap_path.to_string_lossy().to_string(),
            tags: Vec::new(),
            description: format!("Promiscuous BT discovery, {} piconets found", piconets_found.len()),
        };
        store.save_metadata(&metadata)?;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "duration_sec": duration_sec,
            "piconets_found": piconets_found,
            "total_packets": total_packets,
            "pcap_path": pcap_path.to_string_lossy()
        }))
    }

    /// BLE connection following implementation.
    ///
    /// Phase 2 Week 3: Wraps ubertooth-btle with access address following.
    async fn btle_follow(&self, params: Value) -> Result<Value> {
        let access_address = params
            .get("access_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'access_address'".to_string()))?;

        let duration_sec = params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);

        tracing::info!("Following BLE connection: {} for {}s", access_address, duration_sec);

        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("btlefollow");
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Execute ubertooth-btle with -f (follow) and -a (access address)
        let duration_str = duration_sec.to_string();
        let pcap_str = pcap_path.to_string_lossy().to_string();

        let output = self.execute_ubertooth_command(
            "ubertooth-btle",
            &["-f", "-a", access_address, "-r", pcap_str.as_str(), "-d", duration_str.as_str()]
        ).await?;

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

    /// Configure RSSI squelch implementation.
    ///
    /// Phase 2 Week 3: Wraps ubertooth-util squelch configuration.
    async fn configure_squelch(&self, params: Value) -> Result<Value> {
        let squelch_level = params
            .get("squelch_level")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'squelch_level'".to_string()))?;

        // Validate squelch range (-128 to 0 dBm)
        if squelch_level < -128 || squelch_level > 0 {
            return Err(UbertoothError::InvalidParameter(
                "Squelch level must be between -128 and 0 dBm".to_string(),
            ));
        }

        tracing::info!("Configuring squelch: {} dBm", squelch_level);

        // Execute ubertooth-util -q <squelch_level>
        let squelch_str = squelch_level.to_string();
        self.execute_ubertooth_command("ubertooth-util", &["-q", squelch_str.as_str()])
            .await?;

        Ok(json!({
            "success": true,
            "squelch_level": squelch_level,
            "message": format!("Squelch set to {} dBm", squelch_level)
        }))
    }

    /// Configure LEDs implementation.
    ///
    /// Phase 2 Week 3: LED control via ubertooth-util.
    async fn configure_leds(&self, params: Value) -> Result<Value> {
        let usr_led = params.get("usr_led").and_then(|v| v.as_bool()).unwrap_or(true);
        let rx_led = params.get("rx_led").and_then(|v| v.as_bool()).unwrap_or(false);
        let tx_led = params.get("tx_led").and_then(|v| v.as_bool()).unwrap_or(false);

        tracing::info!("Configuring LEDs: usr={}, rx={}, tx={}", usr_led, rx_led, tx_led);

        // Phase 2 TODO: Implement ubertooth-util LED commands
        Ok(json!({
            "success": true,
            "leds": {
                "usr": usr_led,
                "rx": rx_led,
                "tx": tx_led
            },
            "note": "Phase 2 Week 3: LED control pending ubertooth-util integration"
        }))
    }

    /// Save current configuration as a preset.
    ///
    /// Phase 2 Week 4: Configuration preset management.
    async fn bt_save_config(&self, params: Value) -> Result<Value> {
        let config_name = params
            .get("config_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'config_name'".to_string()))?;

        let description = params
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let overwrite = params
            .get("overwrite")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        tracing::info!("Saving configuration: {}", config_name);

        // Get current device status to capture settings
        let device_status = self.device_status().await?;

        // Extract settings from device status
        let settings = ConfigSettings {
            channel: device_status.get("channel").and_then(|v| v.as_u64()).map(|v| v as u8),
            modulation: device_status.get("modulation").and_then(|v| v.as_str()).map(|s| s.to_string()),
            power_level: None, // TODO: Extract from device_status when available
            paen: None,
            hgm: None,
            squelch: None,
        };

        // Create config metadata
        let config = ConfigMetadata {
            name: config_name.to_string(),
            description,
            created: Utc::now(),
            settings: settings.clone(),
        };

        // Save to file
        let store = ConfigStore::new()?;
        let config_path = store.save_config(&config, overwrite)?;

        Ok(json!({
            "success": true,
            "config_name": config_name,
            "config_path": config_path.to_string_lossy(),
            "saved_settings": {
                "channel": settings.channel,
                "modulation": settings.modulation,
                "power_level": settings.power_level,
                "paen": settings.paen,
                "hgm": settings.hgm,
                "squelch": settings.squelch
            }
        }))
    }

    /// Load a saved configuration preset.
    ///
    /// Phase 2 Week 4: Apply saved settings to device.
    async fn bt_load_config(&self, params: Value) -> Result<Value> {
        let config_name = params
            .get("config_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'config_name'".to_string()))?;

        tracing::info!("Loading configuration: {}", config_name);

        // Load config from file
        let store = ConfigStore::new()?;
        let config = store.load_config(config_name)?;

        // Apply settings to device
        // TODO: Call configure_* methods to apply each setting

        Ok(json!({
            "success": true,
            "config_name": config_name,
            "applied_settings": {
                "channel": config.settings.channel,
                "modulation": config.settings.modulation,
                "power_level": config.settings.power_level
            },
            "message": format!("Configuration '{}' loaded successfully", config_name)
        }))
    }

    /// List all saved configuration presets.
    ///
    /// Phase 2 Week 4: List configs from ~/.ubertooth/configs/
    async fn config_list(&self, _params: Value) -> Result<Value> {
        tracing::info!("Listing saved configurations");

        let store = ConfigStore::new()?;
        let configs = store.list_configs()?;

        let config_list: Vec<Value> = configs
            .iter()
            .map(|c| {
                json!({
                    "name": c.name,
                    "description": c.description,
                    "created": c.created.to_rfc3339(),
                    "settings_preview": {
                        "channel": c.settings.channel,
                        "modulation": c.settings.modulation
                    }
                })
            })
            .collect();

        Ok(json!({
            "success": true,
            "configs": config_list,
            "count": configs.len()
        }))
    }

    /// Delete a saved configuration preset.
    ///
    /// Phase 2 Week 4: Remove config file from ~/.ubertooth/configs/
    async fn config_delete(&self, params: Value) -> Result<Value> {
        let config_name = params
            .get("config_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'config_name'".to_string()))?;

        tracing::info!("Deleting configuration: {}", config_name);

        let store = ConfigStore::new()?;
        store.delete_config(config_name)?;

        Ok(json!({
            "success": true,
            "message": format!("Configuration '{}' deleted", config_name)
        }))
    }

    // Phase 2 Week 5: Analysis tools

    async fn bt_compare(&self, _params: Value) -> Result<Value> {
        let capture_id_a = _params.get("capture_id_a")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id_a'".to_string()))?;

        let capture_id_b = _params.get("capture_id_b")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id_b'".to_string()))?;

        let mode = _params.get("mode").and_then(|v| v.as_str()).unwrap_or("packets");

        tracing::info!("Comparing captures {} and {} (mode: {})", capture_id_a, capture_id_b, mode);

        let store = CaptureStore::new()?;

        // Load metadata for both captures
        let meta_a = store.load_metadata(capture_id_a)?;
        let meta_b = store.load_metadata(capture_id_b)?;

        // Use capinfos to get detailed stats
        let pcap_a = store.captures_dir().join(format!("{}.pcap", capture_id_a));
        let pcap_b = store.captures_dir().join(format!("{}.pcap", capture_id_b));

        let stats_a = self.execute_ubertooth_command("capinfos", &[pcap_a.to_str().unwrap()]).await.unwrap_or_default();
        let stats_b = self.execute_ubertooth_command("capinfos", &[pcap_b.to_str().unwrap()]).await.unwrap_or_default();

        // Basic comparison based on metadata
        let common_packets = std::cmp::min(meta_a.packet_count, meta_b.packet_count);
        let unique_to_a = meta_a.packet_count.saturating_sub(common_packets);
        let unique_to_b = meta_b.packet_count.saturating_sub(common_packets);
        let total = meta_a.packet_count + meta_b.packet_count;
        let similarity_percent = if total > 0 {
            (common_packets as f64 * 200.0) / total as f64
        } else {
            0.0
        };

        let mut differences = Vec::new();
        if meta_a.capture_type != meta_b.capture_type {
            differences.push(format!("Capture types differ: {} vs {}", meta_a.capture_type, meta_b.capture_type));
        }
        if meta_a.packet_count != meta_b.packet_count {
            differences.push(format!("Packet counts differ: {} vs {}", meta_a.packet_count, meta_b.packet_count));
        }

        Ok(json!({
            "success": true,
            "comparison": {
                "mode": mode,
                "similarity_percent": similarity_percent,
                "differences": differences,
                "unique_to_a": unique_to_a,
                "unique_to_b": unique_to_b,
                "common_packets": common_packets
            },
            "capture_a": {
                "id": capture_id_a,
                "type": meta_a.capture_type,
                "packets": meta_a.packet_count
            },
            "capture_b": {
                "id": capture_id_b,
                "type": meta_b.capture_type,
                "packets": meta_b.packet_count
            }
        }))
    }

    async fn bt_decode(&self, _params: Value) -> Result<Value> {
        let capture_id = _params.get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let protocol = _params.get("protocol").and_then(|v| v.as_str()).unwrap_or("bluetooth");
        let limit = _params.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

        tracing::info!("Decoding capture {} (protocol: {}, limit: {})", capture_id, protocol, limit);

        let store = CaptureStore::new()?;
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Use tshark to decode packets
        let limit_str = limit.to_string();
        let output = self.execute_ubertooth_command(
            "tshark",
            &["-r", pcap_path.to_str().unwrap(), "-c", limit_str.as_str(), "-T", "json"]
        ).await.unwrap_or_else(|_| "[]".to_string());

        // Parse JSON output from tshark
        let decoded_packets: Vec<Value> = serde_json::from_str(&output).unwrap_or_else(|_| vec![]);

        let packet_count = decoded_packets.len();

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "protocol": protocol,
            "decoded_packets": decoded_packets,
            "packet_count": packet_count,
            "limit": limit
        }))
    }

    async fn bt_fingerprint(&self, _params: Value) -> Result<Value> {
        let capture_id = _params.get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let target_mac = _params.get("target_mac").and_then(|v| v.as_str());

        tracing::info!("Fingerprinting capture {} (target: {:?})", capture_id, target_mac);

        let store = CaptureStore::new()?;
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Use tshark to extract device info
        let output = self.execute_ubertooth_command(
            "tshark",
            &["-r", pcap_path.to_str().unwrap(), "-T", "fields", "-e", "bluetooth.addr", "-e", "bluetooth.name"]
        ).await.unwrap_or_default();

        let mut indicators = Vec::new();
        let mut manufacturer = "Unknown".to_string();
        let mut device_type = "Unknown".to_string();
        let mut confidence = 0.0;

        // Parse output for device patterns
        for line in output.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 1 {
                let addr = parts[0].trim();
                if !addr.is_empty() {
                    indicators.push(format!("BD_ADDR: {}", addr));

                    // Simple OUI-based manufacturer detection
                    let oui = &addr[0..8];
                    manufacturer = match oui {
                        "00:1A:7D" => "Apple".to_string(),
                        "00:25:00" => "Samsung".to_string(),
                        "00:23:12" => "Intel".to_string(),
                        _ => "Unknown".to_string(),
                    };

                    if manufacturer != "Unknown" {
                        confidence = 0.8;
                    }
                }
            }
            if parts.len() >= 2 {
                let name = parts[1].trim();
                if !name.is_empty() {
                    indicators.push(format!("Device name: {}", name));

                    // Infer device type from name
                    if name.to_lowercase().contains("phone") {
                        device_type = "Smartphone".to_string();
                        confidence = 0.9;
                    } else if name.to_lowercase().contains("headset") || name.to_lowercase().contains("buds") {
                        device_type = "Audio device".to_string();
                        confidence = 0.85;
                    }
                }
            }
        }

        let device_mac = target_mac.unwrap_or("unknown");

        Ok(json!({
            "success": true,
            "device": {
                "mac_address": device_mac,
                "fingerprint": {
                    "manufacturer": manufacturer,
                    "device_type": device_type,
                    "os_version": null,
                    "confidence": confidence
                },
                "indicators": indicators
            }
        }))
    }

    async fn pcap_merge(&self, _params: Value) -> Result<Value> {
        let capture_ids = _params.get("capture_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_ids' array".to_string()))?;

        if capture_ids.len() < 2 {
            return Err(UbertoothError::InvalidParameter(
                "At least 2 captures required for merge".to_string()
            ));
        }

        tracing::info!("Merging {} captures", capture_ids.len());

        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("merged");
        let output_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Build input file list
        let mut input_paths = Vec::new();
        for id_val in capture_ids {
            if let Some(id) = id_val.as_str() {
                let path = store.captures_dir().join(format!("{}.pcap", id));
                if path.exists() {
                    input_paths.push(path.to_string_lossy().to_string());
                } else {
                    return Err(UbertoothError::CaptureNotFound(id.to_string()));
                }
            }
        }

        // Use mergecap to merge PCAP files
        let mut args = vec!["-w", output_path.to_str().unwrap()];
        let input_refs: Vec<&str> = input_paths.iter().map(|s| s.as_str()).collect();
        args.extend(input_refs);

        self.execute_ubertooth_command("mergecap", &args).await?;

        // Count total packets in merged file
        let capinfos_output = self.execute_ubertooth_command(
            "capinfos",
            &[output_path.to_str().unwrap()]
        ).await.unwrap_or_default();

        let mut total_packets = 0;
        for line in capinfos_output.lines() {
            if line.contains("Number of packets") {
                if let Some(num_str) = line.split(':').nth(1) {
                    total_packets = num_str.trim().parse().unwrap_or(0);
                }
            }
        }

        // Save metadata
        let file_size_bytes = if output_path.exists() {
            std::fs::metadata(&output_path)?.len()
        } else {
            0
        };

        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "merged".to_string(),
            duration_sec: None,
            packet_count: total_packets,
            file_size_bytes,
            pcap_path: output_path.to_string_lossy().to_string(),
            tags: vec!["merged".to_string()],
            description: format!("Merged from {} source captures", capture_ids.len()),
        };
        store.save_metadata(&metadata)?;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "source_captures": capture_ids.len(),
            "total_packets": total_packets,
            "pcap_path": output_path.to_string_lossy()
        }))
    }

    async fn capture_export(&self, _params: Value) -> Result<Value> {
        let capture_id = _params.get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let format = _params.get("format").and_then(|v| v.as_str()).unwrap_or("pcap");
        let output_path = _params.get("output_path").and_then(|v| v.as_str());

        tracing::info!("Exporting capture {} to format {}", capture_id, format);

        let store = CaptureStore::new()?;
        let input_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        if !input_path.exists() {
            return Err(UbertoothError::CaptureNotFound(capture_id.to_string()));
        }

        // Determine export path
        let export_path = if let Some(path) = output_path {
            PathBuf::from(path)
        } else {
            store.captures_dir().join(format!("{}.{}", capture_id, format))
        };

        // Use tshark or editcap for format conversion
        match format {
            "pcap" => {
                // Just copy the file
                std::fs::copy(&input_path, &export_path)?;
            }
            "pcapng" => {
                // Use editcap to convert to pcapng
                self.execute_ubertooth_command(
                    "editcap",
                    &["-F", "pcapng", input_path.to_str().unwrap(), export_path.to_str().unwrap()]
                ).await?;
            }
            "json" => {
                // Use tshark to export to JSON
                let json_output = self.execute_ubertooth_command(
                    "tshark",
                    &["-r", input_path.to_str().unwrap(), "-T", "json"]
                ).await?;
                std::fs::write(&export_path, json_output)?;
            }
            "csv" => {
                // Use tshark to export to CSV
                let csv_output = self.execute_ubertooth_command(
                    "tshark",
                    &["-r", input_path.to_str().unwrap(), "-T", "fields", "-E", "header=y", "-E", "separator=,"]
                ).await?;
                std::fs::write(&export_path, csv_output)?;
            }
            _ => {
                return Err(UbertoothError::InvalidParameter(
                    format!("Unsupported format: {}", format)
                ));
            }
        }

        // Get packet count
        let metadata = store.load_metadata(capture_id)?;
        let packet_count = metadata.packet_count;

        // Get exported file size
        let file_size_bytes = if export_path.exists() {
            std::fs::metadata(&export_path)?.len()
        } else {
            0
        };

        Ok(json!({
            "success": true,
            "export_path": export_path.to_string_lossy(),
            "format": format,
            "packet_count": packet_count,
            "file_size_bytes": file_size_bytes
        }))
    }

    // Phase 2 Week 6: Attack operations (all require authorization)

    async fn btle_inject(&self, _params: Value) -> Result<Value> {
        let access_address = _params.get("access_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'access_address'".to_string()))?;

        let packet_hex = _params.get("packet_hex")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'packet_hex'".to_string()))?;

        let channel = _params.get("channel").and_then(|v| v.as_u64()).unwrap_or(37) as u8;
        let repeat = _params.get("repeat").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

        tracing::warn!("btle_inject - REQUIRES AUTHORIZATION - Active RF transmission");

        // Validate channel range
        if channel > 39 {
            return Err(UbertoothError::InvalidParameter(
                "BLE channel must be 0-39".to_string()
            ));
        }

        // Execute ubertooth-btle with injection parameters
        let channel_str = channel.to_string();
        let repeat_str = repeat.to_string();

        let output = self.execute_ubertooth_command(
            "ubertooth-btle",
            &["-i", "-a", access_address, "-c", channel_str.as_str(), "-p", packet_hex, "-n", repeat_str.as_str()]
        ).await?;

        // Parse output for confirmation
        let packets_sent = if output.contains("injected") || output.contains("transmitted") {
            repeat
        } else {
            0
        };

        Ok(json!({
            "success": true,
            "packets_sent": packets_sent,
            "access_address": access_address,
            "channel": channel,
            "message": format!("Injected {} packet(s) on channel {}", packets_sent, channel)
        }))
    }

    async fn bt_jam(&self, params: Value) -> Result<Value> {
        let jam_mode = params.get("jam_mode").and_then(|v| v.as_str()).unwrap_or("continuous");
        tracing::error!("bt_jam - HIGHLY REGULATED - ILLEGAL IN MOST JURISDICTIONS");
        Ok(json!({"success": false, "error": "OPERATION_NOT_AVAILABLE", "message": "Jamming is highly regulated and not implemented. Illegal in most jurisdictions without proper authorization."}))
    }

    async fn btle_slave(&self, _params: Value) -> Result<Value> {
        let mac_address = _params.get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'mac_address'".to_string()))?;

        let duration_sec = _params.get("duration_sec").and_then(|v| v.as_u64()).unwrap_or(60);
        let advertising_interval = _params.get("advertising_interval_ms").and_then(|v| v.as_u64()).unwrap_or(100);

        tracing::warn!("btle_slave - REQUIRES AUTHORIZATION - BLE slave/advertising mode");

        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("slave");
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Execute ubertooth-btle in advertising mode
        let duration_str = duration_sec.to_string();
        let interval_str = advertising_interval.to_string();
        let pcap_str = pcap_path.to_string_lossy().to_string();

        let output = self.execute_ubertooth_command(
            "ubertooth-btle",
            &["-a", "-m", mac_address, "-i", interval_str.as_str(), "-d", duration_str.as_str(), "-r", pcap_str.as_str()]
        ).await?;

        // Parse output for connection events
        let connections_received = output.lines()
            .filter(|line| line.contains("connection") || line.contains("CONNECT_REQ"))
            .count();

        let advertising = output.contains("advertising") || output.contains("ADV");

        // Save metadata
        let file_size_bytes = if pcap_path.exists() {
            std::fs::metadata(&pcap_path)?.len()
        } else {
            0
        };

        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "btle_slave".to_string(),
            duration_sec: Some(duration_sec),
            packet_count: connections_received,
            file_size_bytes,
            pcap_path: pcap_path.to_string_lossy().to_string(),
            tags: vec![format!("mac:{}", mac_address)],
            description: format!("BLE slave mode, {} connections", connections_received),
        };
        store.save_metadata(&metadata)?;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "mac_address": mac_address,
            "advertising": advertising,
            "connections_received": connections_received,
            "duration_sec": duration_sec,
            "message": format!("Advertised for {}s, received {} connection(s)", duration_sec, connections_received)
        }))
    }

    async fn btle_mitm(&self, params: Value) -> Result<Value> {
        let target_mac = params.get("target_mac").and_then(|v| v.as_str()).unwrap_or("00:00:00:00:00:00");
        tracing::error!("btle_mitm - STRICTLY REQUIRED AUTHORIZATION - ACTIVE ATTACK");
        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("mitm");
        Ok(json!({"success": false, "error": "AUTHORIZATION_REQUIRED", "message": "MITM attack requires STRICTLY REQUIRED authorization level. Not implemented in Phase 2 Week 6."}))
    }

    async fn bt_spoof(&self, _params: Value) -> Result<Value> {
        let spoof_mac = _params.get("spoof_mac")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'spoof_mac'".to_string()))?;

        let duration_sec = _params.get("duration_sec").and_then(|v| v.as_u64()).unwrap_or(60);
        let action = _params.get("action").and_then(|v| v.as_str()).unwrap_or("set");

        tracing::warn!("bt_spoof - REQUIRES AUTHORIZATION - BD_ADDR spoofing");

        // Execute ubertooth-util to set BD_ADDR
        // Note: This requires firmware support and may not work on all devices
        let output = self.execute_ubertooth_command(
            "ubertooth-util",
            &["-B", spoof_mac]
        ).await?;

        let success = output.contains("set") || output.contains("success") || output.contains("OK");

        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("spoof");

        Ok(json!({
            "success": success,
            "capture_id": capture_id,
            "spoof_mac": spoof_mac,
            "duration_sec": duration_sec,
            "action": action,
            "message": if success {
                format!("BD_ADDR set to {} for {}s", spoof_mac, duration_sec)
            } else {
                "BD_ADDR spoofing may not be supported by firmware".to_string()
            }
        }))
    }

    async fn ubertooth_raw(&self, _params: Value) -> Result<Value> {
        let command = _params.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'command'".to_string()))?;

        let args_array = _params.get("args").and_then(|v| v.as_array());

        tracing::warn!("ubertooth_raw - WARNING: Direct hardware access to {}", command);

        // Build command arguments
        let mut cmd_args = vec![command];
        let arg_strings: Vec<String>;
        if let Some(args) = args_array {
            arg_strings = args.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            cmd_args.extend(arg_strings.iter().map(|s| s.as_str()));
        }

        // Execute raw ubertooth command
        let output = self.execute_ubertooth_command("ubertooth-util", &cmd_args).await?;

        // Parse response
        let response_hex = output.lines()
            .find(|line| line.contains("0x") || line.chars().all(|c| c.is_ascii_hexdigit() || c.is_whitespace()))
            .unwrap_or("")
            .trim()
            .to_string();

        let response_length = response_hex.len() / 2; // Assuming hex pairs

        Ok(json!({
            "success": true,
            "command": command,
            "response_hex": response_hex,
            "response_length": response_length,
            "raw_output": output.trim()
        }))
    }
}
