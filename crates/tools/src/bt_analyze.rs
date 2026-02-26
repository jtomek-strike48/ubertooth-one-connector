//! Bluetooth analysis tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for analyzing captured packets and extracting insights.
///
/// Phase 1: Basic analysis with metadata
/// Phase 2: Full PCAP parsing with protocol analysis
pub struct BtAnalyzeTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtAnalyzeTool {
    /// Create a new bt_analyze tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtAnalyzeTool {
    fn name(&self) -> &str {
        "bt_analyze"
    }

    fn category(&self) -> &str {
        "bt-analysis"
    }

    fn description(&self) -> &str {
        "Analyze captured packets and extract insights"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "capture_id": {
                    "type": "string",
                    "description": "Capture identifier to analyze"
                },
                "analysis_type": {
                    "type": "string",
                    "description": "Analysis type",
                    "enum": ["auto", "protocol", "timing", "security"],
                    "default": "auto"
                },
                "target_mac": {
                    "type": ["string", "null"],
                    "description": "Optional: focus on specific device MAC address"
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
                "analysis": {
                    "type": "object",
                    "properties": {
                        "protocol_summary": {
                            "type": "object",
                            "properties": {
                                "type": { "type": "string" },
                                "pdu_types": { "type": "object" }
                            }
                        },
                        "devices": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "mac_address": { "type": "string" },
                                    "packet_count": { "type": "integer" },
                                    "device_name": { "type": ["string", "null"] }
                                }
                            }
                        },
                        "timing_analysis": {
                            "type": "object",
                            "properties": {
                                "avg_interval_ms": { "type": "number" },
                                "min_interval_ms": { "type": "number" },
                                "max_interval_ms": { "type": "number" }
                            }
                        },
                        "security_observations": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "anomalies": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    }
                }
            },
            "required": ["success", "capture_id"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing bt_analyze");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("bt_analyze", params).await?;

        tracing::info!("bt_analyze completed successfully");
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
            if method == "bt_analyze" {
                let capture_id = params["capture_id"].as_str().unwrap_or("unknown");
                Ok(json!({
                    "success": true,
                    "capture_id": capture_id,
                    "analysis": {
                        "protocol_summary": {
                            "type": "BLE",
                            "pdu_types": {
                                "ADV_IND": 10,
                                "SCAN_REQ": 5
                            }
                        },
                        "devices": [
                            {
                                "mac_address": "AA:BB:CC:DD:EE:FF",
                                "packet_count": 15,
                                "device_name": "Test Device"
                            }
                        ],
                        "timing_analysis": {
                            "avg_interval_ms": 100.0,
                            "min_interval_ms": 95.0,
                            "max_interval_ms": 110.0
                        },
                        "security_observations": ["Address randomization detected"],
                        "anomalies": []
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
    async fn test_bt_analyze() {
        let backend = Arc::new(MockBackend);
        let tool = BtAnalyzeTool::new(backend);

        let result = tool.execute(json!({
            "capture_id": "cap-test-123",
            "analysis_type": "auto"
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["capture_id"], "cap-test-123");
        assert!(result["analysis"].is_object());
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtAnalyzeTool::new(backend);

        assert_eq!(tool.name(), "bt_analyze");
        assert_eq!(tool.category(), "bt-analysis");
    }
}
