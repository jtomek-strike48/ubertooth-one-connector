//! Device connection tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for connecting to an Ubertooth One device.
pub struct DeviceConnectTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl DeviceConnectTool {
    /// Create a new device connect tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for DeviceConnectTool {
    fn name(&self) -> &str {
        "device_connect"
    }

    fn category(&self) -> &str {
        "bt-device"
    }

    fn description(&self) -> &str {
        "Connect to an Ubertooth One USB device"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "device_index": {
                    "type": "integer",
                    "description": "Device index if multiple Ubertooth devices connected",
                    "default": 0
                }
            }
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean",
                    "description": "Whether the connection succeeded"
                },
                "device_id": {
                    "type": "string",
                    "description": "Unique device identifier"
                },
                "serial": {
                    "type": "string",
                    "description": "Device serial number"
                },
                "firmware_version": {
                    "type": "string",
                    "description": "Firmware version string"
                },
                "api_version": {
                    "type": "string",
                    "description": "API version"
                },
                "board_id": {
                    "type": "integer",
                    "description": "Board ID (0=Zero, 1=One, 2=TC13Badge)"
                },
                "capabilities": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of supported capabilities"
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
        tracing::info!("Executing device_connect");
        tracing::debug!("Parameters: {}", params);

        // Call the backend
        let result = self.backend.call("device_connect", params).await?;

        tracing::info!("device_connect completed successfully");
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
            if method == "device_connect" {
                Ok(json!({
                    "success": true,
                    "device_id": "ubertooth-test",
                    "firmware_version": "2020-12-R1",
                    "message": "Mock connection successful"
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
    async fn test_device_connect() {
        let backend = Arc::new(MockBackend);
        let tool = DeviceConnectTool::new(backend);

        let result = tool.execute(json!({})).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["device_id"], "ubertooth-test");
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = DeviceConnectTool::new(backend);

        assert_eq!(tool.name(), "device_connect");
        assert_eq!(tool.category(), "bt-device");
        assert!(!tool.description().is_empty());
    }
}
