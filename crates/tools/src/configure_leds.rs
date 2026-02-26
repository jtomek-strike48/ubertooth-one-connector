//! LED configuration tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for controlling LED indicators on Ubertooth device.
///
/// Controls user LED, RX activity LED, and TX activity LED.
/// Useful for visual feedback during headless operation.
pub struct ConfigureLedsTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl ConfigureLedsTool {
    /// Create a new configure_leds tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for ConfigureLedsTool {
    fn name(&self) -> &str {
        "configure_leds"
    }

    fn category(&self) -> &str {
        "bt-config"
    }

    fn description(&self) -> &str {
        "Control LED indicators (useful for headless operation)"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "usr_led": {
                    "type": "boolean",
                    "description": "User LED state",
                    "default": true
                },
                "rx_led": {
                    "type": "boolean",
                    "description": "RX activity LED state",
                    "default": false
                },
                "tx_led": {
                    "type": "boolean",
                    "description": "TX activity LED state",
                    "default": false
                }
            }
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean"
                },
                "leds": {
                    "type": "object",
                    "properties": {
                        "usr": { "type": "boolean" },
                        "rx": { "type": "boolean" },
                        "tx": { "type": "boolean" }
                    }
                }
            },
            "required": ["success", "leds"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing configure_leds");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("configure_leds", params).await?;

        tracing::info!("configure_leds completed successfully");
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
            if method == "configure_leds" {
                let usr = params.get("usr_led").and_then(|v| v.as_bool()).unwrap_or(true);
                let rx = params.get("rx_led").and_then(|v| v.as_bool()).unwrap_or(false);
                let tx = params.get("tx_led").and_then(|v| v.as_bool()).unwrap_or(false);

                Ok(json!({
                    "success": true,
                    "leds": {
                        "usr": usr,
                        "rx": rx,
                        "tx": tx
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
    async fn test_configure_leds() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigureLedsTool::new(backend);

        let result = tool.execute(json!({
            "usr_led": true,
            "rx_led": false,
            "tx_led": false
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["leds"]["usr"], true);
        assert_eq!(result["leds"]["rx"], false);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = ConfigureLedsTool::new(backend);

        assert_eq!(tool.name(), "configure_leds");
        assert_eq!(tool.category(), "bt-config");
    }
}
