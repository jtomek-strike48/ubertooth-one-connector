//! Configuration list tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for listing all saved configuration presets.
///
/// Returns a list of all configuration files from ~/.ubertooth/configs/
/// with their names, descriptions, and settings preview.
pub struct ConfigListTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl ConfigListTool {
    /// Create a new config_list tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for ConfigListTool {
    fn name(&self) -> &str {
        "config_list"
    }

    fn category(&self) -> &str {
        "bt-config"
    }

    fn description(&self) -> &str {
        "List all saved configuration presets"
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
                    "type": "boolean"
                },
                "configs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "description": { "type": "string" },
                            "created": { "type": "string", "format": "date-time" },
                            "settings_preview": {
                                "type": "object",
                                "properties": {
                                    "channel": { "type": ["integer", "null"] },
                                    "modulation": { "type": ["string", "null"] }
                                }
                            }
                        }
                    }
                },
                "count": {
                    "type": "integer"
                }
            },
            "required": ["success", "configs", "count"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing config_list");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("config_list", params).await?;

        tracing::info!("config_list completed successfully");
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
            if method == "config_list" {
                Ok(json!({
                    "success": true,
                    "configs": [
                        {
                            "name": "ble_adv_ch37",
                            "description": "BLE advertising on channel 37",
                            "created": "2026-02-26T10:00:00Z",
                            "settings_preview": {
                                "channel": 37,
                                "modulation": "BT_LOW_ENERGY"
                            }
                        }
                    ],
                    "count": 1
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
    async fn test_config_list() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigListTool::new(backend);

        let result = tool.execute(json!({})).await.unwrap();

        assert_eq!(result["success"], true);
        assert!(result["configs"].is_array());
        assert_eq!(result["count"], 1);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigListTool::new(backend);

        assert_eq!(tool.name(), "config_list");
        assert_eq!(tool.category(), "bt-config");
    }
}
