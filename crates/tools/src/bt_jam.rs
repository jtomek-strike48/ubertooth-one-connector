//! Bluetooth jamming tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for Bluetooth jamming (denial of service).
///
/// **HIGHLY REGULATED:** Illegal in most jurisdictions without authorization.
/// **STRICTLY REQUIRED:** Authorization level REQUIRED.
pub struct BtJamTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtJamTool {
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtJamTool {
    fn name(&self) -> &str {
        "bt_jam"
    }

    fn category(&self) -> &str {
        "bt-attack"
    }

    fn description(&self) -> &str {
        "Jam Bluetooth frequencies (denial of service) - HIGHLY REGULATED"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "jam_mode": {
                    "type": "string",
                    "enum": ["none", "once", "continuous"],
                    "default": "continuous"
                },
                "channel": {
                    "type": ["integer", "null"],
                    "minimum": 0,
                    "maximum": 78
                },
                "duration_sec": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 60,
                    "default": 10
                }
            }
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "jam_mode": { "type": "string" },
                "duration_sec": { "type": "integer" },
                "channels_jammed": { "type": "integer" },
                "message": { "type": "string" }
            }
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::warn!("Executing bt_jam - HIGHLY REGULATED OPERATION");
        let result = self.backend.call("bt_jam", params).await?;
        Ok(result)
    }

    fn requires_authorization(&self) -> bool {
        true
    }

    fn authorization_category(&self) -> &str {
        "bt-attack-jam"
    }
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
            if method == "bt_jam" {
                Ok(json!({"success": true, "jam_mode": "continuous", "duration_sec": 10, "channels_jammed": 79, "message": "Jamming completed"}))
            } else {
                Err(UbertoothError::BackendError("Unexpected method".to_string()))
            }
        }
        async fn is_alive(&self) -> bool { true }
        async fn restart(&self) -> Result<()> { Ok(()) }
        fn backend_type(&self) -> &str { "mock" }
    }

    #[tokio::test]
    async fn test_bt_jam() {
        let backend = Arc::new(MockBackend);
        let tool = BtJamTool::new(backend);
        let result = tool.execute(json!({"jam_mode": "continuous"})).await.unwrap();
        assert_eq!(result["success"], true);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtJamTool::new(backend);
        assert_eq!(tool.name(), "bt_jam");
        assert_eq!(tool.requires_authorization(), true);
    }
}
