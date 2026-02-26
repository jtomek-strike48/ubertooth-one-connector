//! Configuration load tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for loading a saved configuration preset.
///
/// Reads configuration from ~/.ubertooth/configs/ and applies all settings
/// to the device (channel, modulation, power, squelch, etc.).
pub struct BtLoadConfigTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtLoadConfigTool {
    /// Create a new bt_load_config tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtLoadConfigTool {
    fn name(&self) -> &str {
        "bt_load_config"
    }

    fn category(&self) -> &str {
        "bt-config"
    }

    fn description(&self) -> &str {
        "Load a saved configuration preset"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "config_name": {
                    "type": "string",
                    "description": "Name of the configuration to load"
                }
            },
            "required": ["config_name"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean"
                },
                "config_name": {
                    "type": "string"
                },
                "applied_settings": {
                    "type": "object",
                    "properties": {
                        "channel": { "type": ["integer", "null"] },
                        "modulation": { "type": ["string", "null"] },
                        "power_level": { "type": ["integer", "null"] }
                    }
                },
                "message": {
                    "type": "string"
                }
            },
            "required": ["success", "config_name", "message"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing bt_load_config");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("bt_load_config", params).await?;

        tracing::info!("bt_load_config completed successfully");
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
        async fn call(&self, method: &str, params: Value) -> Result<Value> {
            if method == "bt_load_config" {
                let config_name = params["config_name"].as_str().unwrap_or("test_config");
                Ok(json!({
                    "success": true,
                    "config_name": config_name,
                    "applied_settings": {
                        "channel": 37,
                        "modulation": "BT_LOW_ENERGY",
                        "power_level": 7
                    },
                    "message": format!("Configuration '{}' loaded successfully", config_name)
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
    async fn test_bt_load_config() {
        let backend = Arc::new(MockBackend);
        let tool = BtLoadConfigTool::new(backend);

        let result = tool.execute(json!({
            "config_name": "ble_ch37"
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["config_name"], "ble_ch37");
        assert!(result["applied_settings"].is_object());
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtLoadConfigTool::new(backend);

        assert_eq!(tool.name(), "bt_load_config");
        assert_eq!(tool.category(), "bt-config");
    }
}
