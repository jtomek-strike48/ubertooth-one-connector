//! Tool implementations for Ubertooth One operations.

mod device_connect;
mod device_disconnect;
mod device_status;
mod btle_scan;
mod bt_specan;
mod configure_channel;
mod configure_modulation;
mod configure_power;
mod capture_list;
mod capture_get;
mod capture_delete;
mod capture_tag;
mod bt_analyze;
mod session_context;

use std::sync::Arc;
use ubertooth_core::tools::ToolRegistry;
use ubertooth_platform::UbertoothBackendProvider;

pub use device_connect::DeviceConnectTool;
pub use device_disconnect::DeviceDisconnectTool;
pub use device_status::DeviceStatusTool;
pub use btle_scan::BtleScanTool;
pub use bt_specan::BtSpecanTool;
pub use configure_channel::ConfigureChannelTool;
pub use configure_modulation::ConfigureModulationTool;
pub use configure_power::ConfigurePowerTool;
pub use capture_list::CaptureListTool;
pub use capture_get::CaptureGetTool;
pub use capture_delete::CaptureDeleteTool;
pub use capture_tag::CaptureTagTool;
pub use bt_analyze::BtAnalyzeTool;
pub use session_context::SessionContextTool;

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

    // Phase 1 tools - bt-recon
    registry.register(Arc::new(BtleScanTool::new(backend.clone())));
    registry.register(Arc::new(BtSpecanTool::new(backend.clone())));

    // Phase 1 tools - bt-config
    registry.register(Arc::new(ConfigureChannelTool::new(backend.clone())));
    registry.register(Arc::new(ConfigureModulationTool::new(backend.clone())));
    registry.register(Arc::new(ConfigurePowerTool::new(backend.clone())));

    // Phase 1 tools - bt-capture
    registry.register(Arc::new(CaptureListTool::new(backend.clone())));
    registry.register(Arc::new(CaptureGetTool::new(backend.clone())));
    registry.register(Arc::new(CaptureDeleteTool::new(backend.clone())));
    registry.register(Arc::new(CaptureTagTool::new(backend.clone())));

    // Phase 1 tools - bt-analysis
    registry.register(Arc::new(BtAnalyzeTool::new(backend.clone())));

    // Phase 1 tools - session context
    registry.register(Arc::new(SessionContextTool::new(backend)));

    registry
}
