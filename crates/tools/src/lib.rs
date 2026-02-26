//! Tool implementations for Ubertooth One operations.

mod device_connect;

use std::sync::Arc;
use ubertooth_core::tools::ToolRegistry;
use ubertooth_platform::UbertoothBackendProvider;

pub use device_connect::DeviceConnectTool;

/// Create and populate the tool registry with all available tools.
///
/// # Arguments
///
/// * `backend` - The backend provider (Python sidecar or Rust USB)
///
/// # Returns
///
/// A ToolRegistry with all tools registered
pub fn create_tool_registry(backend: Arc<dyn UbertoothBackendProvider>) -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // Phase 1 tools (v0.0.1)
    registry.register(Arc::new(DeviceConnectTool::new(backend.clone())));

    // Phase 1 tools (v0.1.0) - to be added
    // registry.register(Arc::new(DeviceDisconnectTool::new(backend.clone())));
    // registry.register(Arc::new(DeviceStatusTool::new(backend.clone())));
    // ... etc

    registry
}
