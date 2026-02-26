//! Backend provider trait for Ubertooth One operations.

use async_trait::async_trait;
use serde_json::Value;
use ubertooth_core::error::Result;

/// Trait for backend providers (Python sidecar or Rust USB).
///
/// This abstraction allows tools to work with either backend transparently.
#[async_trait]
pub trait UbertoothBackendProvider: Send + Sync {
    /// Call a backend method with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `method` - The method name (e.g., "device_connect", "btle_scan")
    /// * `params` - JSON parameters for the method
    ///
    /// # Returns
    ///
    /// JSON result from the backend
    async fn call(&self, method: &str, params: Value) -> Result<Value>;

    /// Check if the backend is alive and responsive.
    async fn is_alive(&self) -> bool;

    /// Restart the backend (for Python sidecar recovery).
    async fn restart(&self) -> Result<()>;

    /// Get backend type identifier.
    fn backend_type(&self) -> &str;
}
