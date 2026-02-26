//! Protocol decoding tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for decoding Bluetooth protocol layers.
///
/// Parses BLE/BR packets and extracts structured protocol information
/// (L2CAP, ATT, SMP, GATT, etc.).
pub struct BtDecodeTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtDecodeTool {
    /// Create a new bt_decode tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtDecodeTool {
    fn name(&self) -> &str {
        "bt_decode"
    }

    fn category(&self) -> &str {
        "bt-analysis"
    }

    fn description(&self) -> &str {
        "Decode specific Bluetooth packet types (L2CAP, ATT, SMP, etc.)"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "capture_id": {
                    "type": "string",
                    "description": "Capture ID to decode"
                },
                "packet_index": {
                    "type": ["integer", "null"],
                    "description": "Specific packet index, or null to decode all"
                },
                "protocol_layer": {
                    "type": "string",
                    "description": "Protocol layer to decode",
                    "enum": ["auto", "l2cap", "att", "smp", "gatt"],
                    "default": "auto"
                }
            },
            "required": ["capture_id"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean"
                },
                "decoded_packets": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "index": { "type": "integer" },
                            "timestamp": { "type": "string" },
                            "layers": { "type": "object" },
                            "interpretation": { "type": "string" }
                        }
                    }
                }
            },
            "required": ["success", "decoded_packets"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing bt_decode");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("bt_decode", params).await?;

        tracing::info!("bt_decode completed successfully");
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
            if method == "bt_decode" {
                Ok(json!({
                    "success": true,
                    "decoded_packets": []
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
    async fn test_bt_decode() {
        let backend = Arc::new(MockBackend);
        let tool = BtDecodeTool::new(backend);

        let result = tool.execute(json!({
            "capture_id": "cap-test-123",
            "protocol_layer": "auto"
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert!(result["decoded_packets"].is_array());
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtDecodeTool::new(backend);

        assert_eq!(tool.name(), "bt_decode");
        assert_eq!(tool.category(), "bt-analysis");
    }
}
