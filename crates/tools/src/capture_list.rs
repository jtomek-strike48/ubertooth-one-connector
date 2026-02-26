//! Capture list tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for listing stored packet captures.
pub struct CaptureListTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl CaptureListTool {
    /// Create a new capture list tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for CaptureListTool {
    fn name(&self) -> &str {
        "capture_list"
    }

    fn category(&self) -> &str {
        "bt-capture"
    }

    fn description(&self) -> &str {
        "List stored packet captures with filtering"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "filter_type": {
                    "type": ["string", "null"],
                    "description": "Filter by capture type (btle_sniff, specan, etc.)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum results",
                    "default": 50,
                    "minimum": 1,
                    "maximum": 1000
                },
                "offset": {
                    "type": "integer",
                    "description": "Pagination offset",
                    "default": 0,
                    "minimum": 0
                },
                "sort_by": {
                    "type": "string",
                    "description": "Sort field",
                    "enum": ["timestamp", "size", "packet_count"],
                    "default": "timestamp"
                },
                "sort_order": {
                    "type": "string",
                    "description": "Sort order",
                    "enum": ["asc", "desc"],
                    "default": "desc"
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
                "captures": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "capture_id": { "type": "string" },
                            "timestamp": { "type": "string" },
                            "type": { "type": "string" },
                            "packet_count": { "type": "integer" },
                            "duration_sec": { "type": ["integer", "null"] },
                            "file_size_bytes": { "type": "integer" },
                            "pcap_path": { "type": "string" },
                            "tags": { "type": "array", "items": { "type": "string" } },
                            "description": { "type": "string" }
                        }
                    }
                },
                "total_count": {
                    "type": "integer"
                },
                "offset": {
                    "type": "integer"
                },
                "limit": {
                    "type": "integer"
                }
            },
            "required": ["success", "captures"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing capture_list");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("capture_list", params).await?;

        tracing::info!("capture_list completed successfully");
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
            if method == "capture_list" {
                Ok(json!({
                    "success": true,
                    "captures": [
                        {
                            "capture_id": "cap-test-123",
                            "timestamp": "2026-02-26T15:30:00Z",
                            "type": "btle_sniff",
                            "packet_count": 142,
                            "file_size_bytes": 45320,
                            "tags": ["ble", "scan"]
                        }
                    ],
                    "total_count": 1,
                    "offset": 0,
                    "limit": 50
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
    async fn test_capture_list() {
        let backend = Arc::new(MockBackend);
        let tool = CaptureListTool::new(backend);

        let result = tool.execute(json!({})).await.unwrap();

        assert_eq!(result["success"], true);
        assert!(result["captures"].is_array());
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = CaptureListTool::new(backend);

        assert_eq!(tool.name(), "capture_list");
        assert_eq!(tool.category(), "bt-capture");
    }
}
