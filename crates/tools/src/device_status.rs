//! Device status tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for getting current device state and configuration.
pub struct DeviceStatusTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl DeviceStatusTool {
    /// Create a new device status tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for DeviceStatusTool {
    fn name(&self) -> &str {
        "device_status"
    }

    fn category(&self) -> &str {
        "bt-device"
    }

    fn description(&self) -> &str {
        "Get current device state and configuration"
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
                    "description": "Whether the status query succeeded"
                },
                "connected": {
                    "type": "boolean",
                    "description": "Whether device is connected"
                },
                "device_id": {
                    "type": "string",
                    "description": "Unique device identifier"
                },
                "serial": {
                    "type": "string",
                    "description": "Device serial number"
                },
                "firmware": {
                    "type": "string",
                    "description": "Firmware version"
                },
                "board_id": {
                    "type": "integer",
                    "description": "Board ID (0=Zero, 1=One, 2=TC13Badge)"
                },
                "current_mode": {
                    "type": "string",
                    "description": "Current operating mode (idle, rx_symbols, btle_sniff, etc.)"
                },
                "channel": {
                    "type": "integer",
                    "description": "Current channel (0-78)"
                },
                "modulation": {
                    "type": "string",
                    "description": "Current modulation type"
                },
                "power_level": {
                    "type": "integer",
                    "description": "TX power level (0-7)"
                }
            },
            "required": ["success", "connected"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing device_status");
        tracing::debug!("Parameters: {}", params);

        // Call the backend
        let result = self.backend.call("device_status", params).await?;

        tracing::info!("device_status completed successfully");
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
            if method == "device_status" {
                Ok(json!({
                    "success": true,
                    "connected": true,
                    "device_id": "ubertooth-test",
                    "firmware": "2020-12-R1",
                    "current_mode": "idle"
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
    async fn test_device_status() {
        let backend = Arc::new(MockBackend);
        let tool = DeviceStatusTool::new(backend);

        let result = tool.execute(json!({})).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["connected"], true);
        assert_eq!(result["device_id"], "ubertooth-test");
        assert_eq!(result["firmware"], "2020-12-R1");
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = DeviceStatusTool::new(backend);

        assert_eq!(tool.name(), "device_status");
        assert_eq!(tool.category(), "bt-device");
        assert!(!tool.description().is_empty());
    }
}
