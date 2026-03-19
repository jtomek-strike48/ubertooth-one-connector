//! Python sidecar manager for wrapping ubertooth-tools.

mod capture;
mod config;
mod device;
mod parsing;
mod tools;
mod types;
mod validation;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use ubertooth_core::error::{Result, UbertoothError};

use crate::backend::UbertoothBackendProvider;
use crate::capture_store::{CaptureMetadata, CaptureStore};
use crate::config_store::ConfigStore;

pub use validation::check_ubertooth_installed;

/// Python sidecar process manager.
///
/// The sidecar wraps the ubertooth-* command-line tools and provides a
/// simple interface for executing commands.
pub struct SidecarManager {
    #[allow(dead_code)]
    process: Arc<Mutex<Option<Child>>>,
}

impl SidecarManager {
    /// Create a new sidecar manager (not started yet).
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            process: Arc::new(Mutex::new(None)),
        })
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
    ///
    /// Filters out benign API version mismatch warnings from stderr.
    async fn execute_ubertooth_command(&self, tool: &str, args: &[&str]) -> Result<String> {
        tracing::debug!("Executing: {} {:?}", tool, args);

        let output = Command::new(tool)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                UbertoothError::BackendError(format!("Failed to execute {}: {}", tool, e))
            })?;

        // Filter stderr to remove benign API version warnings
        let stderr = String::from_utf8_lossy(&output.stderr);
        let filtered_stderr = stderr
            .lines()
            .filter(|line| {
                // Filter out API version mismatch warnings (firmware newer than libubertooth)
                !line.contains("API version")
                    && !line.contains("newer than that supported")
                    && !line.contains("Things will still work")
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Log filtered warnings at debug level for troubleshooting
        if stderr.contains("API version") {
            tracing::debug!("Filtered benign API version warning from {}", tool);
        }

        if !output.status.success() {
            // Only return actual errors, not filtered warnings
            if !filtered_stderr.trim().is_empty() {
                return Err(UbertoothError::CommandFailed(format!(
                    "{} failed: {}",
                    tool, filtered_stderr
                )));
            } else {
                return Err(UbertoothError::CommandFailed(format!(
                    "{} failed with exit code: {}",
                    tool, output.status
                )));
            }
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
        let result = Command::new("ubertooth-util").arg("-V").output();

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




    /// AFH analysis implementation.
    ///
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




    async fn pcap_merge(&self, _params: Value) -> Result<Value> {
        let capture_ids = _params
            .get("capture_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                UbertoothError::InvalidParameter("Missing 'capture_ids' array".to_string())
            })?;

        if capture_ids.len() < 2 {
            return Err(UbertoothError::InvalidParameter(
                "At least 2 captures required for merge".to_string(),
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
        let capinfos_output = self
            .execute_ubertooth_command("capinfos", &[output_path.to_str().unwrap()])
            .await
            .unwrap_or_default();

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
            category: None,
            notes: None,
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


    // Phase 2 Week 6: Attack operations (all require authorization)






    async fn ubertooth_raw(&self, _params: Value) -> Result<Value> {
        let command = _params
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'command'".to_string()))?;

        let args_array = _params.get("args").and_then(|v| v.as_array());

        tracing::warn!(
            "ubertooth_raw - WARNING: Direct hardware access to {}",
            command
        );

        // Build command arguments
        let mut cmd_args = vec![command];
        let arg_strings: Vec<String>;
        if let Some(args) = args_array {
            arg_strings = args
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            cmd_args.extend(arg_strings.iter().map(|s| s.as_str()));
        }

        // Execute raw ubertooth command
        let output = self
            .execute_ubertooth_command("ubertooth-util", &cmd_args)
            .await?;

        // Parse response
        let response_hex = output
            .lines()
            .find(|line| {
                line.contains("0x")
                    || line
                        .chars()
                        .all(|c| c.is_ascii_hexdigit() || c.is_whitespace())
            })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pcap_ble_rf() {
        // Initialize logging for test
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let pcap_path = format!(
            "{}/.ubertooth/captures/cap-btle-06b8b707-431f-4b7c-8eda-fb02b7e253d3.pcap",
            std::env::var("HOME").unwrap()
        );

        if !std::path::Path::new(&pcap_path).exists() {
            println!("PCAP file not found, skipping test: {}", pcap_path);
            return;
        }

        println!("Parsing PCAP: {}", pcap_path);
        let result = SidecarManager::parse_pcap(&pcap_path);

        match result {
            Ok(analysis) => {
                println!("Packets: {}", analysis.packet_count);
                println!("Devices found: {}", analysis.devices.len());
                println!("Duration: {:.2}s", analysis.duration_sec);
                println!("\nDevices:");
                for device in &analysis.devices {
                    println!(
                        "  - {} ({}) RSSI: {}",
                        device.mac_address,
                        device.name.as_deref().unwrap_or("Unknown"),
                        device.rssi
                    );
                }

                // Verify we found devices
                assert!(
                    analysis.devices.len() > 0,
                    "Expected to find devices in PCAP but found none"
                );

                // Verify specific device from tshark output
                let expected_mac = "88:6B:0F:B2:2D:C2";
                let found = analysis
                    .devices
                    .iter()
                    .any(|d| d.mac_address == expected_mac);
                assert!(found, "Expected to find device {} but didn't", expected_mac);

                println!("\n✅ Test passed: Found {} devices", analysis.devices.len());
            }
            Err(e) => {
                panic!("Failed to parse PCAP: {:?}", e);
            }
        }
    }
}
