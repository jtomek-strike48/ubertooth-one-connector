//! Python sidecar manager for wrapping ubertooth-tools.

mod backend;
mod capture;
mod config;
mod device;
mod parsing;
mod tools;
mod types;
mod validation;

use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use ubertooth_core::error::{Result, UbertoothError};

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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::sidecar::parsing::pcap::parse_pcap;

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
        let result = parse_pcap(&pcap_path);

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
