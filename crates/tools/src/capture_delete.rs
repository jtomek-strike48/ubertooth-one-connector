//! Capture delete tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for deleting a stored capture.
pub struct CaptureDeleteTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl CaptureDeleteTool {
    /// Create a new capture delete tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for CaptureDeleteTool {
    fn name(&self) -> &str {
        "capture_delete"
    }

    fn category(&self) -> &str {
        "bt-capture"
    }

    fn description(&self) -> &str {
        "Delete a stored capture"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "capture_id": {
                    "type": "string",
                    "description": "Capture identifier to delete"
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
                "message": {
                    "type": "string"
                }
            },
            "required": ["success", "message"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing capture_delete");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("capture_delete", params).await?;

        tracing::info!("capture_delete completed successfully");
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
            if method == "capture_delete" {
                let capture_id = params["capture_id"].as_str().unwrap_or("unknown");
                Ok(json!({
                    "success": true,
                    "message": format!("Capture '{}' deleted", capture_id)
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
    async fn test_capture_delete() {
        let backend = Arc::new(MockBackend);
        let tool = CaptureDeleteTool::new(backend);

        let result = tool.execute(json!({ "capture_id": "cap-test-123" })).await.unwrap();

        assert_eq!(result["success"], true);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = CaptureDeleteTool::new(backend);

        assert_eq!(tool.name(), "capture_delete");
        assert_eq!(tool.category(), "bt-capture");
    }
}
