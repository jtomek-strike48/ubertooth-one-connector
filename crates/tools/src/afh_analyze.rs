//! AFH (Adaptive Frequency Hopping) analysis tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for analyzing Adaptive Frequency Hopping patterns.
///
/// Analyzes AFH channel usage for a Bluetooth piconet.
/// Helps identify WiFi interference and channel avoidance patterns.
pub struct AfhAnalyzeTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl AfhAnalyzeTool {
    /// Create a new afh_analyze tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for AfhAnalyzeTool {
    fn name(&self) -> &str {
        "afh_analyze"
    }

    fn category(&self) -> &str {
        "bt-recon"
    }

    fn description(&self) -> &str {
        "Analyze Adaptive Frequency Hopping (AFH) channel usage for a Bluetooth piconet"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "bd_addr": {
                    "type": ["string", "null"],
                    "description": "Piconet master address (optional)",
                    "pattern": "^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$"
                },
                "duration_sec": {
                    "type": "integer",
                    "description": "Analysis duration in seconds",
                    "default": 30,
                    "minimum": 5,
                    "maximum": 300
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
                "bd_addr": {
                    "type": ["string", "null"]
                },
                "afh_map": {
                    "type": "string",
                    "description": "79-bit channel map in hex"
                },
                "channels_used": {
                    "type": "array",
                    "items": { "type": "integer" }
                },
                "channels_avoided": {
                    "type": "array",
                    "items": { "type": "integer" }
                },
                "used_count": {
                    "type": "integer"
                },
                "avoided_count": {
                    "type": "integer"
                },
                "interpretation": {
                    "type": "string"
                }
            },
            "required": ["success", "afh_map", "channels_used", "channels_avoided"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing afh_analyze");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("afh_analyze", params).await?;

        tracing::info!("afh_analyze completed successfully");
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
            if method == "afh_analyze" {
                Ok(json!({
                    "success": true,
                    "bd_addr": "AA:BB:CC:DD:EE:FF",
                    "afh_map": "0x7FFFFFFFFFFF9F",
                    "channels_used": [0, 1, 5, 10, 15, 20, 25, 30],
                    "channels_avoided": [2, 3, 4, 6, 7],
                    "used_count": 62,
                    "avoided_count": 17,
                    "interpretation": "Avoiding WiFi interference on channels 2-4 (2404-2406 MHz)"
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
    async fn test_afh_analyze() {
        let backend = Arc::new(MockBackend);
        let tool = AfhAnalyzeTool::new(backend);

        let result = tool.execute(json!({
            "bd_addr": "AA:BB:CC:DD:EE:FF",
            "duration_sec": 30
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert!(result["channels_used"].is_array());
        assert!(result["channels_avoided"].is_array());
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = AfhAnalyzeTool::new(backend);

        assert_eq!(tool.name(), "afh_analyze");
        assert_eq!(tool.category(), "bt-recon");
    }
}
