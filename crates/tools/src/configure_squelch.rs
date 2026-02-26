//! RSSI squelch configuration tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for configuring RSSI squelch threshold.
///
/// Sets the minimum signal strength (RSSI) for packets to be captured.
/// Helps filter out weak signals and reduce noise.
pub struct ConfigureSquelchTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl ConfigureSquelchTool {
    /// Create a new configure_squelch tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for ConfigureSquelchTool {
    fn name(&self) -> &str {
        "configure_squelch"
    }

    fn category(&self) -> &str {
        "bt-config"
    }

    fn description(&self) -> &str {
        "Set RSSI squelch threshold to filter weak signals"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "squelch_level": {
                    "type": "integer",
                    "description": "RSSI threshold in dBm",
                    "minimum": -128,
                    "maximum": 0,
                    "default": -90
                }
            },
            "required": ["squelch_level"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean"
                },
                "squelch_level": {
                    "type": "integer"
                },
                "message": {
                    "type": "string"
                }
            },
            "required": ["success", "squelch_level", "message"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing configure_squelch");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("configure_squelch", params).await?;

        tracing::info!("configure_squelch completed successfully");
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
            if method == "configure_squelch" {
                let level = params["squelch_level"].as_i64().unwrap_or(-90);
                Ok(json!({
                    "success": true,
                    "squelch_level": level,
                    "message": format!("Squelch set to {} dBm", level)
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
    async fn test_configure_squelch() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigureSquelchTool::new(backend);

        let result = tool.execute(json!({ "squelch_level": -90 })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["squelch_level"], -90);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigureSquelchTool::new(backend);

        assert_eq!(tool.name(), "configure_squelch");
        assert_eq!(tool.category(), "bt-config");
    }
}
