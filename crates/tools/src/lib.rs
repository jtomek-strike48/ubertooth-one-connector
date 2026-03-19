//! Tool implementations for Ubertooth One operations.

mod afh_analyze;
mod bt_analyze;
mod bt_compare;
mod bt_decode;
mod bt_discover;
mod bt_fingerprint;
mod bt_follow;
mod bt_jam;
mod bt_load_config;
mod bt_save_config;
mod bt_scan;
mod bt_specan;
mod bt_spoof;
mod btle_follow;
mod btle_inject;
mod btle_mitm;
mod btle_scan;
mod btle_slave;
mod capture_delete;
mod capture_export;
mod capture_get;
mod capture_list;
mod capture_tag;
mod config_delete;
mod config_list;
mod configure_channel;
mod configure_leds;
mod configure_modulation;
mod configure_power;
mod configure_squelch;
mod device_connect;
mod device_disconnect;
mod device_status;
mod pcap_merge;
mod session_context;
mod ubertooth_raw;

use std::sync::Arc;
use ubertooth_core::tools::ToolRegistry;
use ubertooth_platform::UbertoothBackendProvider;

pub use afh_analyze::AfhAnalyzeTool;
pub use bt_analyze::BtAnalyzeTool;
pub use bt_compare::BtCompareTool;
pub use bt_decode::BtDecodeTool;
pub use bt_discover::BtDiscoverTool;
pub use bt_fingerprint::BtFingerprintTool;
pub use bt_follow::BtFollowTool;
pub use bt_jam::BtJamTool;
pub use bt_load_config::BtLoadConfigTool;
pub use bt_save_config::BtSaveConfigTool;
pub use bt_scan::BtScanTool;
pub use bt_specan::BtSpecanTool;
pub use bt_spoof::BtSpoofTool;
pub use btle_follow::BtleFollowTool;
pub use btle_inject::BtleInjectTool;
pub use btle_mitm::BtleMitmTool;
pub use btle_scan::BtleScanTool;
pub use btle_slave::BtleSlaveTool;
pub use capture_delete::CaptureDeleteTool;
pub use capture_export::CaptureExportTool;
pub use capture_get::CaptureGetTool;
pub use capture_list::CaptureListTool;
pub use capture_tag::CaptureTagTool;
pub use config_delete::ConfigDeleteTool;
pub use config_list::ConfigListTool;
pub use configure_channel::ConfigureChannelTool;
pub use configure_leds::ConfigureLedsTool;
pub use configure_modulation::ConfigureModulationTool;
pub use configure_power::ConfigurePowerTool;
pub use configure_squelch::ConfigureSquelchTool;
pub use device_connect::DeviceConnectTool;
pub use device_disconnect::DeviceDisconnectTool;
pub use device_status::DeviceStatusTool;
pub use pcap_merge::PcapMergeTool;
pub use session_context::SessionContextTool;
pub use ubertooth_raw::UbertoothRawTool;

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

    // Phase 2 Week 3 - bt-recon (advanced)
    registry.register(Arc::new(BtScanTool::new(backend.clone())));
    registry.register(Arc::new(BtFollowTool::new(backend.clone())));
    registry.register(Arc::new(AfhAnalyzeTool::new(backend.clone())));
    registry.register(Arc::new(BtDiscoverTool::new(backend.clone())));
    registry.register(Arc::new(BtleFollowTool::new(backend.clone())));

    // Phase 1 tools - bt-config
    registry.register(Arc::new(ConfigureChannelTool::new(backend.clone())));
    registry.register(Arc::new(ConfigureModulationTool::new(backend.clone())));
    registry.register(Arc::new(ConfigurePowerTool::new(backend.clone())));

    // Phase 2 Week 3 - bt-config (advanced)
    registry.register(Arc::new(ConfigureSquelchTool::new(backend.clone())));
    registry.register(Arc::new(ConfigureLedsTool::new(backend.clone())));

    // Phase 2 Week 4 - bt-config (presets)
    registry.register(Arc::new(BtSaveConfigTool::new(backend.clone())));
    registry.register(Arc::new(BtLoadConfigTool::new(backend.clone())));
    registry.register(Arc::new(ConfigListTool::new(backend.clone())));
    registry.register(Arc::new(ConfigDeleteTool::new(backend.clone())));

    // Phase 1 tools - bt-capture
    registry.register(Arc::new(CaptureListTool::new(backend.clone())));
    registry.register(Arc::new(CaptureGetTool::new(backend.clone())));
    registry.register(Arc::new(CaptureDeleteTool::new(backend.clone())));
    registry.register(Arc::new(CaptureTagTool::new(backend.clone())));

    // Phase 1 tools - bt-analysis
    registry.register(Arc::new(BtAnalyzeTool::new(backend.clone())));

    // Phase 2 Week 5 - bt-analysis (advanced)
    registry.register(Arc::new(BtCompareTool::new(backend.clone())));
    registry.register(Arc::new(BtDecodeTool::new(backend.clone())));
    registry.register(Arc::new(BtFingerprintTool::new(backend.clone())));
    registry.register(Arc::new(PcapMergeTool::new(backend.clone())));

    // Phase 2 Week 5 - capture export
    registry.register(Arc::new(CaptureExportTool::new(backend.clone())));

    // Phase 2 Week 6 - bt-attack (requires authorization)
    registry.register(Arc::new(BtleInjectTool::new(backend.clone())));
    registry.register(Arc::new(BtJamTool::new(backend.clone())));
    registry.register(Arc::new(BtleSlaveTool::new(backend.clone())));
    registry.register(Arc::new(BtleMitmTool::new(backend.clone())));
    registry.register(Arc::new(BtSpoofTool::new(backend.clone())));

    // Phase 2 Week 6 - bt-advanced
    registry.register(Arc::new(UbertoothRawTool::new(backend.clone())));

    // Phase 1 tools - session context
    registry.register(Arc::new(SessionContextTool::new(backend)));

    registry
}
