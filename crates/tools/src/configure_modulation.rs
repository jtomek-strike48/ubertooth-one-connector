//! Modulation configuration tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for setting modulation type.
pub struct ConfigureModulationTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl ConfigureModulationTool {
    /// Create a new configure modulation tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for ConfigureModulationTool {
    fn name(&self) -> &str {
        "configure_modulation"
    }

    fn category(&self) -> &str {
        "bt-config"
    }

    fn description(&self) -> &str {
        "Set modulation type (BT Basic Rate, BT Low Energy, etc.)"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "modulation": {
                    "type": "string",
                    "description": "Modulation type",
                    "enum": ["BT_BASIC_RATE", "BT_LOW_ENERGY", "80211_FHSS", "NONE"]
                }
            },
            "required": ["modulation"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean",
                    "description": "Whether modulation was set successfully"
                },
                "modulation": {
                    "type": "string",
                    "description": "Modulation type set"
                },
                "message": {
                    "type": "string",
                    "description": "Status message"
                }
            },
            "required": ["success", "modulation"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing configure_modulation");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("configure_modulation", params).await?;

        tracing::info!("configure_modulation completed successfully");
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
            if method == "configure_modulation" {
                let modulation = params["modulation"].as_str().unwrap_or("BT_LOW_ENERGY");
                Ok(json!({
                    "success": true,
                    "modulation": modulation,
                    "message": format!("Modulation set to {}", modulation)
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
    async fn test_configure_modulation() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigureModulationTool::new(backend);

        let result = tool.execute(json!({ "modulation": "BT_LOW_ENERGY" })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["modulation"], "BT_LOW_ENERGY");
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigureModulationTool::new(backend);

        assert_eq!(tool.name(), "configure_modulation");
        assert_eq!(tool.category(), "bt-config");
    }
}
