//! BLE MITM attack tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// BLE Man-in-the-Middle attack. STRICTLY REQUIRED authorization.
pub struct BtleMitmTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtleMitmTool {
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtleMitmTool {
    fn name(&self) -> &str { "btle_mitm" }
    fn category(&self) -> &str { "bt-attack" }
    fn description(&self) -> &str { "Perform Man-in-the-Middle attack on BLE connection" }

    fn input_schema(&self) -> Value {
        json!({"type": "object", "properties": {
            "target_mac": {"type": "string", "pattern": "^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$"},
            "access_address": {"type": "string", "pattern": "^0x[0-9A-Fa-f]{8}$"},
            "duration_sec": {"type": "integer", "default": 60, "minimum": 10, "maximum": 600},
            "intercept_mode": {"type": "string", "enum": ["passive", "active"], "default": "passive"}
        }, "required": ["target_mac", "access_address"]})
    }

    fn output_schema(&self) -> Value {
        json!({"type": "object", "properties": {
            "success": {"type": "boolean"},
            "capture_id": {"type": "string"},
            "target_mac": {"type": "string"},
            "packets_intercepted": {"type": "integer"},
            "packets_injected": {"type": "integer"},
            "connection_disrupted": {"type": "boolean"}
        }})
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::warn!("Executing btle_mitm - ACTIVE ATTACK");
        self.backend.call("btle_mitm", params).await
    }

    fn requires_authorization(&self) -> bool { true }
    fn authorization_category(&self) -> &str { "bt-attack-mitm" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use ubertooth_core::error::{Result, UbertoothError};
    use ubertooth_platform::UbertoothBackendProvider;
    struct MockBackend;
    #[async_trait]
    impl UbertoothBackendProvider for MockBackend {
        async fn call(&self, method: &str, _params: Value) -> Result<Value> {
            if method == "btle_mitm" { Ok(json!({"success": true, "capture_id": "cap-mitm-test", "target_mac": "AA:BB:CC:DD:EE:FF", "packets_intercepted": 350, "packets_injected": 0, "connection_disrupted": false})) }
            else { Err(UbertoothError::BackendError("Unexpected".to_string())) }
        }
        async fn is_alive(&self) -> bool { true }
        async fn restart(&self) -> Result<()> { Ok(()) }
        fn backend_type(&self) -> &str { "mock" }
    }
    #[tokio::test]
    async fn test_btle_mitm() {
        let tool = BtleMitmTool::new(Arc::new(MockBackend));
        let result = tool.execute(json!({"target_mac": "AA:BB:CC:DD:EE:FF", "access_address": "0x8E89BED6"})).await.unwrap();
        assert_eq!(result["success"], true);
    }
    #[test]
    fn test_tool_metadata() {
        let tool = BtleMitmTool::new(Arc::new(MockBackend));
        assert_eq!(tool.name(), "btle_mitm");
        assert_eq!(tool.requires_authorization(), true);
    }
}
