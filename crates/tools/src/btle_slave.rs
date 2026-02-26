//! BLE peripheral/slave mode tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// BLE peripheral/slave mode. Requires authorization.
pub struct BtleSlaveTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtleSlaveTool {
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtleSlaveTool {
    fn name(&self) -> &str { "btle_slave" }
    fn category(&self) -> &str { "bt-attack" }
    fn description(&self) -> &str { "Act as a BLE peripheral/slave device" }

    fn input_schema(&self) -> Value {
        json!({"type": "object", "properties": {
            "mac_address": {"type": "string", "pattern": "^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$"},
            "adv_data": {"type": "string", "pattern": "^[0-9A-Fa-f]+$"},
            "adv_interval_ms": {"type": "integer", "default": 100, "minimum": 20, "maximum": 10240},
            "connectable": {"type": "boolean", "default": true}
        }, "required": ["mac_address", "adv_data"]})
    }

    fn output_schema(&self) -> Value {
        json!({"type": "object", "properties": {
            "success": {"type": "boolean"},
            "mac_address": {"type": "string"},
            "advertising": {"type": "boolean"},
            "connections_received": {"type": "integer"},
            "message": {"type": "string"}
        }})
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        self.backend.call("btle_slave", params).await
    }

    fn requires_authorization(&self) -> bool { true }
    fn authorization_category(&self) -> &str { "bt-attack-spoof" }
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
            if method == "btle_slave" { Ok(json!({"success": true, "mac_address": "AA:BB:CC:DD:EE:FF", "advertising": true, "connections_received": 0, "message": "Active"})) }
            else { Err(UbertoothError::BackendError("Unexpected".to_string())) }
        }
        async fn is_alive(&self) -> bool { true }
        async fn restart(&self) -> Result<()> { Ok(()) }
        fn backend_type(&self) -> &str { "mock" }
    }
    #[tokio::test]
    async fn test_btle_slave() {
        let tool = BtleSlaveTool::new(Arc::new(MockBackend));
        let result = tool.execute(json!({"mac_address": "AA:BB:CC:DD:EE:FF", "adv_data": "02010"})).await.unwrap();
        assert_eq!(result["success"], true);
    }
    #[test]
    fn test_tool_metadata() {
        let tool = BtleSlaveTool::new(Arc::new(MockBackend));
        assert_eq!(tool.name(), "btle_slave");
        assert_eq!(tool.requires_authorization(), true);
    }
}
