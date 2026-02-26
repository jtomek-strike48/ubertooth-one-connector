//! Python sidecar manager for wrapping ubertooth-tools.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use ubertooth_core::error::{Result, UbertoothError};

use crate::backend::UbertoothBackendProvider;

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
            "device_status" => self.device_status().await,
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
}
