//! Bluetooth device spoofing tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Device address spoofing. Requires authorization.
pub struct BtSpoofTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtSpoofTool {
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtSpoofTool {
    fn name(&self) -> &str { "bt_spoof" }
    fn category(&self) -> &str { "bt-attack" }
    fn description(&self) -> &str { "Spoof a Bluetooth device identity" }

    fn input_schema(&self) -> Value {
        json!({"type": "object", "properties": {
            "spoof_mac": {"type": "string", "pattern": "^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$"},
            "device_name": {"type": "string"},
            "class_of_device": {"type": "string", "pattern": "^0x[0-9A-Fa-f]{6}$"},
            "duration_sec": {"type": "integer", "default": 60, "minimum": 10, "maximum": 600}
        }, "required": ["spoof_mac"]})
    }

    fn output_schema(&self) -> Value {
        json!({"type": "object", "properties": {
            "success": {"type": "boolean"},
            "spoof_mac": {"type": "string"},
            "duration_sec": {"type": "integer"},
            "message": {"type": "string"}
        }})
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        self.backend.call("bt_spoof", params).await
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
            if method == "bt_spoof" { Ok(json!({"success": true, "spoof_mac": "AA:BB:CC:DD:EE:FF", "duration_sec": 60, "message": "Device identity spoofed"})) }
            else { Err(UbertoothError::BackendError("Unexpected".to_string())) }
        }
        async fn is_alive(&self) -> bool { true }
        async fn restart(&self) -> Result<()> { Ok(()) }
        fn backend_type(&self) -> &str { "mock" }
    }
    #[tokio::test]
    async fn test_bt_spoof() {
        let tool = BtSpoofTool::new(Arc::new(MockBackend));
        let result = tool.execute(json!({"spoof_mac": "AA:BB:CC:DD:EE:FF"})).await.unwrap();
        assert_eq!(result["success"], true);
    }
    #[test]
    fn test_tool_metadata() {
        let tool = BtSpoofTool::new(Arc::new(MockBackend));
        assert_eq!(tool.name(), "bt_spoof");
        assert_eq!(tool.requires_authorization(), true);
    }
}
