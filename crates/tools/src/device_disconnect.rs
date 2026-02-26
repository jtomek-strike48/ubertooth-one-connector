//! Device disconnection tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for disconnecting from an Ubertooth One device.
pub struct DeviceDisconnectTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl DeviceDisconnectTool {
    /// Create a new device disconnect tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for DeviceDisconnectTool {
    fn name(&self) -> &str {
        "device_disconnect"
    }

    fn category(&self) -> &str {
        "bt-device"
    }

    fn description(&self) -> &str {
        "Disconnect from Ubertooth One and release USB device"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean",
                    "description": "Whether the disconnection succeeded"
                },
                "message": {
                    "type": "string",
                    "description": "Human-readable status message"
                }
            },
            "required": ["success", "message"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing device_disconnect");
        tracing::debug!("Parameters: {}", params);

        // Call the backend
        let result = self.backend.call("device_disconnect", params).await?;

        tracing::info!("device_disconnect completed successfully");
        Ok(result)
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
            if method == "device_disconnect" {
                Ok(json!({
                    "success": true,
                    "message": "Device disconnected"
                }))
            } else {
                Err(UbertoothError::BackendError("Unexpected method".to_string()))
            }
        }

        async fn is_alive(&self) -> bool {
            true
        }

        async fn restart(&self) -> Result<()> {
            Ok(())
        }

        fn backend_type(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_device_disconnect() {
        let backend = Arc::new(MockBackend);
        let tool = DeviceDisconnectTool::new(backend);

        let result = tool.execute(json!({})).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["message"], "Device disconnected");
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = DeviceDisconnectTool::new(backend);

        assert_eq!(tool.name(), "device_disconnect");
        assert_eq!(tool.category(), "bt-device");
        assert!(!tool.description().is_empty());
    }
}
