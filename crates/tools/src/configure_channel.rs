//! Channel configuration tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for setting Bluetooth channel (0-78).
pub struct ConfigureChannelTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl ConfigureChannelTool {
    /// Create a new configure channel tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for ConfigureChannelTool {
    fn name(&self) -> &str {
        "configure_channel"
    }

    fn category(&self) -> &str {
        "bt-config"
    }

    fn description(&self) -> &str {
        "Set Bluetooth channel (0-78)"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "channel": {
                    "type": "integer",
                    "description": "Channel number (0-78)",
                    "minimum": 0,
                    "maximum": 78
                },
                "validate": {
                    "type": "boolean",
                    "description": "Validate channel range",
                    "default": true
                }
            },
            "required": ["channel"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean",
                    "description": "Whether channel was set successfully"
                },
                "channel": {
                    "type": "integer",
                    "description": "Channel number set"
                },
                "frequency_mhz": {
                    "type": "integer",
                    "description": "Calculated frequency in MHz"
                },
                "message": {
                    "type": "string",
                    "description": "Status message"
                }
            },
            "required": ["success", "channel"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing configure_channel");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("configure_channel", params).await?;

        tracing::info!("configure_channel completed successfully");
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
            if method == "configure_channel" {
                let channel = params["channel"].as_u64().unwrap_or(37);
                Ok(json!({
                    "success": true,
                    "channel": channel,
                    "frequency_mhz": 2402 + channel,
                    "message": format!("Channel set to {}", channel)
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
    async fn test_configure_channel() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigureChannelTool::new(backend);

        let result = tool.execute(json!({ "channel": 37 })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["channel"], 37);
        assert_eq!(result["frequency_mhz"], 2439);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigureChannelTool::new(backend);

        assert_eq!(tool.name(), "configure_channel");
        assert_eq!(tool.category(), "bt-config");
    }
}
