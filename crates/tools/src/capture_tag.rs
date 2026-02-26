//! Capture tag tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for adding tags and notes to captures.
pub struct CaptureTagTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl CaptureTagTool {
    /// Create a new capture tag tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for CaptureTagTool {
    fn name(&self) -> &str {
        "capture_tag"
    }

    fn category(&self) -> &str {
        "bt-capture"
    }

    fn description(&self) -> &str {
        "Add tags and notes to a capture"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "capture_id": {
                    "type": "string",
                    "description": "Capture identifier"
                },
                "tags": {
                    "type": "array",
                    "description": "Tags to add",
                    "items": { "type": "string" }
                },
                "description": {
                    "type": "string",
                    "description": "Description text"
                },
                "append_tags": {
                    "type": "boolean",
                    "description": "Append tags vs replace",
                    "default": true
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
                "tags": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "description": {
                    "type": "string"
                }
            },
            "required": ["success", "capture_id"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing capture_tag");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("capture_tag", params).await?;

        tracing::info!("capture_tag completed successfully");
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
            if method == "capture_tag" {
                let capture_id = params["capture_id"].as_str().unwrap_or("unknown");
                let tags = params.get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect::<Vec<_>>())
                    .unwrap_or_default();

                Ok(json!({
                    "success": true,
                    "capture_id": capture_id,
                    "tags": tags,
                    "description": "Test capture"
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
    async fn test_capture_tag() {
        let backend = Arc::new(MockBackend);
        let tool = CaptureTagTool::new(backend);

        let result = tool.execute(json!({
            "capture_id": "cap-test-123",
            "tags": ["ble", "scan"],
            "description": "Test capture"
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["capture_id"], "cap-test-123");
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = CaptureTagTool::new(backend);

        assert_eq!(tool.name(), "capture_tag");
        assert_eq!(tool.category(), "bt-capture");
    }
}
