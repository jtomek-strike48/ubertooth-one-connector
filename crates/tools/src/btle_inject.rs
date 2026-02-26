//! BLE packet injection tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for injecting BLE packets into a connection.
///
/// **WARNING:** Active RF transmission. Requires authorization.
/// Used for testing and security research.
pub struct BtleInjectTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtleInjectTool {
    /// Create a new btle_inject tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtleInjectTool {
    fn name(&self) -> &str {
        "btle_inject"
    }

    fn category(&self) -> &str {
        "bt-attack"
    }

    fn description(&self) -> &str {
        "Inject BLE packets into a connection"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "access_address": {
                    "type": "string",
                    "description": "BLE access address in hex",
                    "pattern": "^0x[0-9A-Fa-f]{8}$"
                },
                "channel": {
                    "type": "integer",
                    "description": "BLE channel (0-39)",
                    "minimum": 0,
                    "maximum": 39,
                    "default": 37
                },
                "packet_hex": {
                    "type": "string",
                    "description": "Raw packet data in hex",
                    "pattern": "^[0-9A-Fa-f]+$"
                },
                "repeat": {
                    "type": "integer",
                    "description": "Number of times to transmit",
                    "minimum": 1,
                    "maximum": 100,
                    "default": 1
                }
            },
            "required": ["access_address", "packet_hex"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean"
                },
                "packets_sent": {
                    "type": "integer"
                },
                "access_address": {
                    "type": "string"
                },
                "channel": {
                    "type": "integer"
                },
                "message": {
                    "type": "string"
                }
            },
            "required": ["success", "packets_sent", "message"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing btle_inject");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("btle_inject", params).await?;

        tracing::info!("btle_inject completed successfully");
        Ok(result)
    }

    fn requires_authorization(&self) -> bool {
        true
    }

    fn authorization_category(&self) -> &str {
        "bt-attack-inject"
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
            if method == "btle_inject" {
                Ok(json!({
                    "success": true,
                    "packets_sent": 1,
                    "access_address": "0x8E89BED6",
                    "channel": 37,
                    "message": "Packet injected successfully"
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
    async fn test_btle_inject() {
        let backend = Arc::new(MockBackend);
        let tool = BtleInjectTool::new(backend);

        let result = tool.execute(json!({
            "access_address": "0x8E89BED6",
            "channel": 37,
            "packet_hex": "0201061AFF4C00"
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["packets_sent"], 1);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtleInjectTool::new(backend);

        assert_eq!(tool.name(), "btle_inject");
        assert_eq!(tool.category(), "bt-attack");
        assert_eq!(tool.requires_authorization(), true);
    }
}
