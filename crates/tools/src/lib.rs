//! Tool implementations for Ubertooth One operations.

mod device_connect;
mod device_disconnect;
mod device_status;

use std::sync::Arc;
use ubertooth_core::tools::ToolRegistry;
use ubertooth_platform::UbertoothBackendProvider;

pub use device_connect::DeviceConnectTool;
pub use device_disconnect::DeviceDisconnectTool;
pub use device_status::DeviceStatusTool;

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

    // Phase 1 tools - bt-device
    registry.register(Arc::new(DeviceConnectTool::new(backend.clone())));
    registry.register(Arc::new(DeviceDisconnectTool::new(backend.clone())));
    registry.register(Arc::new(DeviceStatusTool::new(backend.clone())));

    // Phase 1 tools (remaining) - to be added
    // registry.register(Arc::new(SessionContextTool::new(backend.clone())));
    // registry.register(Arc::new(BtleScanTool::new(backend.clone())));
    // registry.register(Arc::new(BtSpecanTool::new(backend.clone())));
    // ... etc

    registry
}
