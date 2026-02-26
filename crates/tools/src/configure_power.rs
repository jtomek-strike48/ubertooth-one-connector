//! Power configuration tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for setting TX power level and amplifier settings.
pub struct ConfigurePowerTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl ConfigurePowerTool {
    /// Create a new configure power tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for ConfigurePowerTool {
    fn name(&self) -> &str {
        "configure_power"
    }

    fn category(&self) -> &str {
        "bt-config"
    }

    fn description(&self) -> &str {
        "Set TX power level and amplifier settings"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "power_level": {
                    "type": "integer",
                    "description": "TX power level (0-7, where 7 is maximum)",
                    "minimum": 0,
                    "maximum": 7
                },
                "paen": {
                    "type": "boolean",
                    "description": "Power amplifier enable",
                    "default": true
                },
                "hgm": {
                    "type": "boolean",
                    "description": "High gain mode for RX",
                    "default": false
                }
            },
            "required": ["power_level"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean",
                    "description": "Whether power was configured successfully"
                },
                "power_level": {
                    "type": "integer",
                    "description": "Power level set"
                },
                "paen": {
                    "type": "boolean",
                    "description": "Power amplifier enabled"
                },
                "hgm": {
                    "type": "boolean",
                    "description": "High gain mode enabled"
                },
                "estimated_power_dbm": {
                    "type": "integer",
                    "description": "Approximate TX power in dBm"
                },
                "message": {
                    "type": "string",
                    "description": "Status message"
                }
            },
            "required": ["success", "power_level"]
        })
    }

    fn requires_authorization(&self) -> bool {
        // WARNING level - modifying TX power
        false // Phase 1: no enforcement yet
    }

    fn authorization_category(&self) -> &str {
        "bt-config-power"
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing configure_power");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("configure_power", params).await?;

        tracing::info!("configure_power completed successfully");
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
            if method == "configure_power" {
                let power_level = params["power_level"].as_u64().unwrap_or(7);
                let paen = params.get("paen").and_then(|v| v.as_bool()).unwrap_or(true);
                let hgm = params.get("hgm").and_then(|v| v.as_bool()).unwrap_or(false);

                let estimated_dbm = if paen { 10 + (power_level * 2) as i64 } else { power_level as i64 };

                Ok(json!({
                    "success": true,
                    "power_level": power_level,
                    "paen": paen,
                    "hgm": hgm,
                    "estimated_power_dbm": estimated_dbm,
                    "message": format!("Power configured: Level {} with PA {}", power_level, if paen { "enabled" } else { "disabled" })
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
    async fn test_configure_power() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigurePowerTool::new(backend);

        let result = tool.execute(json!({
            "power_level": 7,
            "paen": true,
            "hgm": false
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["power_level"], 7);
        assert_eq!(result["paen"], true);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigurePowerTool::new(backend);

        assert_eq!(tool.name(), "configure_power");
        assert_eq!(tool.category(), "bt-config");
        assert!(!tool.requires_authorization()); // Phase 1: no enforcement
    }
}
