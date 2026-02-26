//! Capture export tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for exporting captures to various formats.
///
/// Supports PCAP, PCAPNG, JSON, and CSV export formats.
/// Enables analysis in external tools.
pub struct CaptureExportTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl CaptureExportTool {
    /// Create a new capture_export tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for CaptureExportTool {
    fn name(&self) -> &str {
        "capture_export"
    }

    fn category(&self) -> &str {
        "bt-capture"
    }

    fn description(&self) -> &str {
        "Export capture to standard formats (PCAP, JSON, CSV)"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "capture_id": {
                    "type": "string",
                    "description": "Capture ID to export"
                },
                "format": {
                    "type": "string",
                    "description": "Export format",
                    "enum": ["pcap", "pcapng", "json", "csv"],
                    "default": "pcap"
                },
                "output_path": {
                    "type": ["string", "null"],
                    "description": "Optional output path (defaults to captures dir)"
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
                "export_path": {
                    "type": "string"
                },
                "format": {
                    "type": "string"
                },
                "packet_count": {
                    "type": "integer"
                },
                "file_size_bytes": {
                    "type": "integer"
                }
            },
            "required": ["success", "export_path", "format"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing capture_export");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("capture_export", params).await?;

        tracing::info!("capture_export completed successfully");
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
            if method == "capture_export" {
                Ok(json!({
                    "success": true,
                    "export_path": "/tmp/export.pcap",
                    "format": "pcap",
                    "packet_count": 142,
                    "file_size_bytes": 45320
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
    async fn test_capture_export() {
        let backend = Arc::new(MockBackend);
        let tool = CaptureExportTool::new(backend);

        let result = tool.execute(json!({
            "capture_id": "cap-test-123",
            "format": "pcap"
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["format"], "pcap");
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = CaptureExportTool::new(backend);

        assert_eq!(tool.name(), "capture_export");
        assert_eq!(tool.category(), "bt-capture");
    }
}
