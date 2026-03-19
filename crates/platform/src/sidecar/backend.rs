//! Backend trait implementation for SidecarManager.

use async_trait::async_trait;
use serde_json::Value;
use std::process::Command;
use ubertooth_core::error::{Result, UbertoothError};

use crate::backend::UbertoothBackendProvider;

use super::SidecarManager;

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
