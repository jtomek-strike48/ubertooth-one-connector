//! Native Rust USB backend (Phase 3).
//!
//! This module will contain the direct libusb implementation for high-performance
//! Ubertooth operations. Currently a placeholder for Phase 3 development.

use async_trait::async_trait;
use serde_json::Value;
use ubertooth_core::error::{Result, UbertoothError};

use crate::backend::UbertoothBackendProvider;

/// Native Rust USB backend.
///
/// Phase 3: This will provide direct libusb access for 100-200x performance
/// improvement over the Python backend.
pub struct RustUsbBackend {
    // TODO Phase 3: Add rusb::DeviceHandle and connection state
}

impl RustUsbBackend {
    /// Create a new Rust USB backend.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for RustUsbBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UbertoothBackendProvider for RustUsbBackend {
    async fn call(&self, method: &str, _params: Value) -> Result<Value> {
        Err(UbertoothError::BackendError(format!(
            "Rust USB backend not implemented yet (Phase 3). Method: {}",
            method
        )))
    }

    async fn is_alive(&self) -> bool {
        false
    }

    async fn restart(&self) -> Result<()> {
        Ok(())
    }

    fn backend_type(&self) -> &str {
        "rust"
    }
}
