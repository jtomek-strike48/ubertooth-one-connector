//! Configuration save tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for saving current radio configuration as a named preset.
///
/// Captures current device settings and stores them to ~/.ubertooth/configs/
/// as a JSON file for later recall.
pub struct BtSaveConfigTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtSaveConfigTool {
    /// Create a new bt_save_config tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtSaveConfigTool {
    fn name(&self) -> &str {
        "bt_save_config"
    }

    fn category(&self) -> &str {
        "bt-config"
    }

    fn description(&self) -> &str {
        "Save current radio configuration as a named preset"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "config_name": {
                    "type": "string",
                    "description": "Unique name for the configuration",
                    "pattern": "^[a-zA-Z0-9_-]+$"
                },
                "description": {
                    "type": "string",
                    "description": "Human-readable description",
                    "default": ""
                },
                "overwrite": {
                    "type": "boolean",
                    "description": "Allow overwriting existing config",
                    "default": false
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
                "config_path": {
                    "type": "string"
                },
                "saved_settings": {
                    "type": "object",
                    "properties": {
                        "channel": { "type": ["integer", "null"] },
                        "modulation": { "type": ["string", "null"] },
                        "power_level": { "type": ["integer", "null"] },
                        "paen": { "type": ["boolean", "null"] },
                        "hgm": { "type": ["boolean", "null"] },
                        "squelch": { "type": ["integer", "null"] }
                    }
                }
            },
            "required": ["success", "config_name", "config_path", "saved_settings"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing bt_save_config");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("bt_save_config", params).await?;

        tracing::info!("bt_save_config completed successfully");
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
            if method == "bt_save_config" {
                let config_name = params["config_name"].as_str().unwrap_or("test_config");
                Ok(json!({
                    "success": true,
                    "config_name": config_name,
                    "config_path": format!("/home/user/.ubertooth/configs/{}.json", config_name),
                    "saved_settings": {
                        "channel": 37,
                        "modulation": "BT_LOW_ENERGY",
                        "power_level": 7,
                        "paen": true,
                        "hgm": false,
                        "squelch": -90
                    }
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
    async fn test_bt_save_config() {
        let backend = Arc::new(MockBackend);
        let tool = BtSaveConfigTool::new(backend);

        let result = tool.execute(json!({
            "config_name": "ble_ch37",
            "description": "BLE channel 37",
            "overwrite": false
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["config_name"], "ble_ch37");
        assert!(result["saved_settings"].is_object());
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtSaveConfigTool::new(backend);

        assert_eq!(tool.name(), "bt_save_config");
        assert_eq!(tool.category(), "bt-config");
    }
}
