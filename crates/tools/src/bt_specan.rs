//! Bluetooth spectrum analysis tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for spectrum analysis of the 2.4 GHz ISM band.
pub struct BtSpecanTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtSpecanTool {
    /// Create a new spectrum analysis tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtSpecanTool {
    fn name(&self) -> &str {
        "bt_specan"
    }

    fn category(&self) -> &str {
        "bt-recon"
    }

    fn description(&self) -> &str {
        "Spectrum analysis of 2.4 GHz ISM band"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "low_freq": {
                    "type": "integer",
                    "description": "Start frequency in MHz",
                    "default": 2402,
                    "minimum": 2400,
                    "maximum": 2483
                },
                "high_freq": {
                    "type": "integer",
                    "description": "End frequency in MHz",
                    "default": 2480,
                    "minimum": 2400,
                    "maximum": 2483
                },
                "duration_sec": {
                    "type": "integer",
                    "description": "Scan duration in seconds",
                    "default": 10,
                    "minimum": 1,
                    "maximum": 300
                },
                "rssi_threshold": {
                    "type": "integer",
                    "description": "RSSI floor in dBm",
                    "default": -90,
                    "minimum": -128,
                    "maximum": 0
                }
            }
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean",
                    "description": "Whether the scan succeeded"
                },
                "capture_id": {
                    "type": "string",
                    "description": "Unique capture identifier"
                },
                "frequency_range": {
                    "type": "array",
                    "description": "Frequency range scanned [low, high]",
                    "items": { "type": "integer" }
                },
                "duration_sec": {
                    "type": "integer",
                    "description": "Scan duration"
                },
                "scan_results": {
                    "type": "array",
                    "description": "RSSI data per frequency",
                    "items": {
                        "type": "object",
                        "properties": {
                            "frequency_mhz": { "type": "integer" },
                            "channel": { "type": "integer" },
                            "rssi_avg": { "type": "integer" },
                            "rssi_max": { "type": "integer" },
                            "rssi_min": { "type": "integer" },
                            "activity_percent": { "type": "number" }
                        }
                    }
                },
                "hotspots": {
                    "type": "array",
                    "description": "High-activity frequency ranges",
                    "items": {
                        "type": "object",
                        "properties": {
                            "frequency_mhz": { "type": "integer" },
                            "rssi_max": { "type": "integer" },
                            "interpretation": { "type": "string" }
                        }
                    }
                }
            },
            "required": ["success", "capture_id"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing bt_specan");
        tracing::debug!("Parameters: {}", params);

        // Call the backend
        let result = self.backend.call("bt_specan", params).await?;

        tracing::info!("bt_specan completed successfully");
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
            if method == "bt_specan" {
                Ok(json!({
                    "success": true,
                    "capture_id": "cap-specan-test123",
                    "frequency_range": [2402, 2480],
                    "duration_sec": 10,
                    "scan_results": [
                        {
                            "frequency_mhz": 2402,
                            "channel": 0,
                            "rssi_avg": -65,
                            "rssi_max": -58,
                            "rssi_min": -72,
                            "activity_percent": 45.2
                        }
                    ],
                    "hotspots": []
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
    async fn test_bt_specan() {
        let backend = Arc::new(MockBackend);
        let tool = BtSpecanTool::new(backend);

        let result = tool.execute(json!({
            "low_freq": 2402,
            "high_freq": 2480,
            "duration_sec": 10
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["capture_id"], "cap-specan-test123");
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtSpecanTool::new(backend);

        assert_eq!(tool.name(), "bt_specan");
        assert_eq!(tool.category(), "bt-recon");
        assert!(!tool.description().is_empty());
    }
}
