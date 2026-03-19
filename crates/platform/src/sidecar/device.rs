//! Device management methods for SidecarManager.

use serde_json::{json, Value};
use ubertooth_core::error::Result;

use super::{validation::check_ubertooth_installed, SidecarManager};

impl SidecarManager {
    /// Device connect implementation.
    pub(super) async fn device_connect(&self) -> Result<Value> {
        // Check if tools are installed
        check_ubertooth_installed()?;

        // Get device information using ubertooth-util
        let output = self
            .execute_ubertooth_command("ubertooth-util", &["-V"])
            .await?;

        // Parse output
        // Expected format: "ubertooth 2020-12-R1 (mikeryan@steel) Fri Dec 25 13:55:05 PST 2020"
        let firmware_version = output
            .lines()
            .find(|line| line.contains("ubertooth"))
            .and_then(|line| {
                // Extract version number (e.g., "2020-12-R1")
                line.split_whitespace().nth(1).map(|s| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        Ok(json!({
            "success": true,
            "device_id": "ubertooth-001",
            "firmware_version": firmware_version,
            "message": "Connected to Ubertooth One"
        }))
    }

    /// Device disconnect implementation.
    pub(super) async fn device_disconnect(&self) -> Result<Value> {
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
    pub(super) async fn device_status(&self) -> Result<Value> {
        // Get device information
        let output = self
            .execute_ubertooth_command("ubertooth-util", &["-V"])
            .await?;

        let firmware_version = output
            .lines()
            .find(|line| line.contains("ubertooth"))
            .and_then(|line| {
                // Extract version number (e.g., "2020-12-R1")
                line.split_whitespace().nth(1).map(|s| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        Ok(json!({
            "success": true,
            "connected": true,
            "device_id": "ubertooth-001",
            "firmware": firmware_version,
            "current_mode": "idle"
        }))
    }
}
