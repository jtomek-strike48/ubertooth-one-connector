//! PCAP merge tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for merging multiple PCAP captures into one.
///
/// Combines multiple capture files with optional timestamp sorting.
/// Useful for consolidating related capture sessions.
pub struct PcapMergeTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl PcapMergeTool {
    /// Create a new pcap_merge tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for PcapMergeTool {
    fn name(&self) -> &str {
        "pcap_merge"
    }

    fn category(&self) -> &str {
        "bt-analysis"
    }

    fn description(&self) -> &str {
        "Merge multiple captures into a single PCAP file"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "capture_ids": {
                    "type": "array",
                    "description": "List of capture IDs to merge",
                    "items": { "type": "string" },
                    "minItems": 2
                },
                "output_name": {
                    "type": "string",
                    "description": "Name for merged capture",
                    "default": "merged_capture"
                },
                "sort_by_timestamp": {
                    "type": "boolean",
                    "description": "Sort packets by timestamp",
                    "default": true
                }
            },
            "required": ["capture_ids"]
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
                "source_captures": {
                    "type": "integer"
                },
                "total_packets": {
                    "type": "integer"
                },
                "pcap_path": {
                    "type": "string"
                }
            },
            "required": ["success", "capture_id", "source_captures", "total_packets"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing pcap_merge");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("pcap_merge", params).await?;

        tracing::info!("pcap_merge completed successfully");
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
            if method == "pcap_merge" {
                Ok(json!({
                    "success": true,
                    "capture_id": "cap-merged-test",
                    "source_captures": 3,
                    "total_packets": 450,
                    "pcap_path": "/home/user/.ubertooth/captures/cap-merged-test.pcap"
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
    async fn test_pcap_merge() {
        let backend = Arc::new(MockBackend);
        let tool = PcapMergeTool::new(backend);

        let result = tool.execute(json!({
            "capture_ids": ["cap-1", "cap-2", "cap-3"],
            "output_name": "merged",
            "sort_by_timestamp": true
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["source_captures"], 3);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = PcapMergeTool::new(backend);

        assert_eq!(tool.name(), "pcap_merge");
        assert_eq!(tool.category(), "bt-analysis");
    }
}
