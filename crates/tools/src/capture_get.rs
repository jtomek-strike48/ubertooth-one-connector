//! Capture get tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for retrieving packet data from a capture.
pub struct CaptureGetTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl CaptureGetTool {
    /// Create a new capture get tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for CaptureGetTool {
    fn name(&self) -> &str {
        "capture_get"
    }

    fn category(&self) -> &str {
        "bt-capture"
    }

    fn description(&self) -> &str {
        "Retrieve packet data from a capture with pagination"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "capture_id": {
                    "type": "string",
                    "description": "Capture identifier"
                },
                "offset": {
                    "type": "integer",
                    "description": "Packet offset",
                    "default": 0,
                    "minimum": 0
                },
                "limit": {
                    "type": "integer",
                    "description": "Max packets to return",
                    "default": 100,
                    "minimum": 1,
                    "maximum": 1000
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "hex"],
                    "default": "json"
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
                "capture_id": {
                    "type": "string"
                },
                "offset": {
                    "type": "integer"
                },
                "limit": {
                    "type": "integer"
                },
                "packet_count": {
                    "type": "integer"
                },
                "packets": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "index": { "type": "integer" },
                            "timestamp": { "type": "string" },
                            "channel": { "type": "integer" },
                            "rssi": { "type": "integer" },
                            "data_hex": { "type": "string" }
                        }
                    }
                },
                "has_more": {
                    "type": "boolean"
                }
            },
            "required": ["success", "capture_id"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing capture_get");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("capture_get", params).await?;

        tracing::info!("capture_get completed successfully");
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
            if method == "capture_get" {
                Ok(json!({
                    "success": true,
                    "capture_id": "cap-test-123",
                    "offset": 0,
                    "limit": 100,
                    "packet_count": 50,
                    "packets": [],
                    "has_more": false
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
    async fn test_capture_get() {
        let backend = Arc::new(MockBackend);
        let tool = CaptureGetTool::new(backend);

        let result = tool.execute(json!({ "capture_id": "cap-test-123" })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["capture_id"], "cap-test-123");
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = CaptureGetTool::new(backend);

        assert_eq!(tool.name(), "capture_get");
        assert_eq!(tool.category(), "bt-capture");
    }
}
