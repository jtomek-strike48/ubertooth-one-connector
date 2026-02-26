//! Capture comparison tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for comparing two captures to find differences.
///
/// Useful for replay attack detection and protocol analysis.
/// Compares packets, devices, and timing between two captures.
pub struct BtCompareTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtCompareTool {
    /// Create a new bt_compare tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtCompareTool {
    fn name(&self) -> &str {
        "bt_compare"
    }

    fn category(&self) -> &str {
        "bt-analysis"
    }

    fn description(&self) -> &str {
        "Compare two captures to find differences (useful for replay attacks)"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "capture_id_a": {
                    "type": "string",
                    "description": "First capture ID"
                },
                "capture_id_b": {
                    "type": "string",
                    "description": "Second capture ID"
                },
                "compare_mode": {
                    "type": "string",
                    "description": "Comparison mode",
                    "enum": ["packets", "devices", "timing"],
                    "default": "packets"
                }
            },
            "required": ["capture_id_a", "capture_id_b"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean"
                },
                "comparison": {
                    "type": "object",
                    "properties": {
                        "mode": { "type": "string" },
                        "similarity_percent": { "type": "number" },
                        "differences": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "type": { "type": "string" },
                                    "packet_index_a": { "type": "integer" },
                                    "packet_index_b": { "type": "integer" },
                                    "field": { "type": "string" },
                                    "value_a": { "type": "string" },
                                    "value_b": { "type": "string" },
                                    "interpretation": { "type": "string" }
                                }
                            }
                        },
                        "unique_to_a": { "type": "integer" },
                        "unique_to_b": { "type": "integer" },
                        "common_packets": { "type": "integer" }
                    }
                }
            },
            "required": ["success", "comparison"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing bt_compare");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("bt_compare", params).await?;

        tracing::info!("bt_compare completed successfully");
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
            if method == "bt_compare" {
                Ok(json!({
                    "success": true,
                    "comparison": {
                        "mode": "packets",
                        "similarity_percent": 87.5,
                        "differences": [],
                        "unique_to_a": 12,
                        "unique_to_b": 8,
                        "common_packets": 122
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
    async fn test_bt_compare() {
        let backend = Arc::new(MockBackend);
        let tool = BtCompareTool::new(backend);

        let result = tool.execute(json!({
            "capture_id_a": "cap-1",
            "capture_id_b": "cap-2",
            "compare_mode": "packets"
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert!(result["comparison"].is_object());
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtCompareTool::new(backend);

        assert_eq!(tool.name(), "bt_compare");
        assert_eq!(tool.category(), "bt-analysis");
    }
}
