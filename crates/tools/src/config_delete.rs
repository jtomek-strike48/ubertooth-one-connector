//! Configuration delete tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for deleting a saved configuration preset.
///
/// Removes a configuration file from ~/.ubertooth/configs/ by name.
pub struct ConfigDeleteTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl ConfigDeleteTool {
    /// Create a new config_delete tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for ConfigDeleteTool {
    fn name(&self) -> &str {
        "config_delete"
    }

    fn category(&self) -> &str {
        "bt-config"
    }

    fn description(&self) -> &str {
        "Delete a saved configuration preset"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "config_name": {
                    "type": "string",
                    "description": "Name of the configuration to delete"
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
                "message": {
                    "type": "string"
                }
            },
            "required": ["success", "message"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing config_delete");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("config_delete", params).await?;

        tracing::info!("config_delete completed successfully");
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
            if method == "config_delete" {
                let config_name = params["config_name"].as_str().unwrap_or("unknown");
                Ok(json!({
                    "success": true,
                    "message": format!("Configuration '{}' deleted", config_name)
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
    async fn test_config_delete() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigDeleteTool::new(backend);

        let result = tool.execute(json!({
            "config_name": "ble_ch37"
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert!(result["message"].as_str().unwrap().contains("deleted"));
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigDeleteTool::new(backend);

        assert_eq!(tool.name(), "config_delete");
        assert_eq!(tool.category(), "bt-config");
    }
}
