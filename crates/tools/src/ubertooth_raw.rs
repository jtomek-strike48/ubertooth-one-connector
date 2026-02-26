//! Raw USB command tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Send raw USB commands. WARNING: Direct hardware access.
pub struct UbertoothRawTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl UbertoothRawTool {
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for UbertoothRawTool {
    fn name(&self) -> &str { "ubertooth_raw" }
    fn category(&self) -> &str { "bt-advanced" }
    fn description(&self) -> &str { "Send raw USB commands to Ubertooth (escape hatch for advanced users)" }

    fn input_schema(&self) -> Value {
        json!({"type": "object", "properties": {
            "command": {"type": "string", "description": "Command name or numeric ID"},
            "command_id": {"type": ["integer", "null"], "minimum": 0, "maximum": 255},
            "data": {"type": "string", "pattern": "^[0-9A-Fa-f]*$", "default": ""},
            "timeout_ms": {"type": "integer", "default": 5000, "minimum": 100, "maximum": 30000}
        }, "required": ["command"]})
    }

    fn output_schema(&self) -> Value {
        json!({"type": "object", "properties": {
            "success": {"type": "boolean"},
            "command": {"type": "string"},
            "response_hex": {"type": "string"},
            "response_length": {"type": "integer"}
        }})
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::warn!("Executing ubertooth_raw - Direct hardware access");
        self.backend.call("ubertooth_raw", params).await
    }

    fn requires_authorization(&self) -> bool { true }
    fn authorization_category(&self) -> &str { "bt-advanced-raw" }
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
            if method == "ubertooth_raw" { Ok(json!({"success": true, "command": "UBERTOOTH_PING", "response_hex": "00", "response_length": 1})) }
            else { Err(UbertoothError::BackendError("Unexpected".to_string())) }
        }
        async fn is_alive(&self) -> bool { true }
        async fn restart(&self) -> Result<()> { Ok(()) }
        fn backend_type(&self) -> &str { "mock" }
    }
    #[tokio::test]
    async fn test_ubertooth_raw() {
        let tool = UbertoothRawTool::new(Arc::new(MockBackend));
        let result = tool.execute(json!({"command": "UBERTOOTH_PING"})).await.unwrap();
        assert_eq!(result["success"], true);
    }
    #[test]
    fn test_tool_metadata() {
        let tool = UbertoothRawTool::new(Arc::new(MockBackend));
        assert_eq!(tool.name(), "ubertooth_raw");
        assert_eq!(tool.requires_authorization(), true);
    }
}
