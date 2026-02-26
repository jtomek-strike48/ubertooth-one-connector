//! BLE device scanning tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for scanning BLE devices and capturing advertisements.
///
/// This is the most important tool for BLE reconnaissance.
pub struct BtleScanTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtleScanTool {
    /// Create a new BLE scan tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtleScanTool {
    fn name(&self) -> &str {
        "btle_scan"
    }

    fn category(&self) -> &str {
        "bt-recon"
    }

    fn description(&self) -> &str {
        "Scan for BLE devices and capture advertisements"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "duration_sec": {
                    "type": "integer",
                    "description": "Scan duration in seconds",
                    "default": 30,
                    "minimum": 1,
                    "maximum": 300
                },
                "channel": {
                    "type": "integer",
                    "description": "BLE advertising channel (37, 38, or 39)",
                    "default": 37,
                    "enum": [37, 38, 39]
                },
                "promiscuous": {
                    "type": "boolean",
                    "description": "Capture all advertisements vs targeted",
                    "default": true
                },
                "save_pcap": {
                    "type": "boolean",
                    "description": "Save capture to PCAP file",
                    "default": true
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
                "scan_duration_sec": {
                    "type": "integer",
                    "description": "Actual scan duration"
                },
                "channel": {
                    "type": "integer",
                    "description": "Channel scanned"
                },
                "devices_found": {
                    "type": "array",
                    "description": "List of discovered BLE devices",
                    "items": {
                        "type": "object",
                        "properties": {
                            "mac_address": { "type": "string" },
                            "address_type": { "type": "string" },
                            "device_name": { "type": "string" },
                            "rssi_avg": { "type": "integer" },
                            "packet_count": { "type": "integer" }
                        }
                    }
                },
                "total_packets": {
                    "type": "integer",
                    "description": "Total packets captured"
                },
                "pcap_path": {
                    "type": "string",
                    "description": "Path to saved PCAP file"
                },
                "preview": {
                    "type": "array",
                    "description": "Preview of captured data (first few packets)",
                    "items": { "type": "string" }
                }
            },
            "required": ["success", "capture_id"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing btle_scan");
        tracing::debug!("Parameters: {}", params);

        // Call the backend
        let result = self.backend.call("btle_scan", params).await?;

        tracing::info!("btle_scan completed successfully");
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
            if method == "btle_scan" {
                Ok(json!({
                    "success": true,
                    "capture_id": "cap-btle-test123",
                    "scan_duration_sec": 30,
                    "channel": 37,
                    "devices_found": [
                        {
                            "mac_address": "AA:BB:CC:DD:EE:FF",
                            "device_name": "Test Device",
                            "rssi_avg": -65,
                            "packet_count": 10
                        }
                    ],
                    "total_packets": 50,
                    "pcap_path": "/home/user/.ubertooth/captures/cap-btle-test123.pcap"
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
    async fn test_btle_scan() {
        let backend = Arc::new(MockBackend);
        let tool = BtleScanTool::new(backend);

        let result = tool.execute(json!({
            "duration_sec": 30,
            "channel": 37
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["capture_id"], "cap-btle-test123");
        assert_eq!(result["channel"], 37);
        assert_eq!(result["total_packets"], 50);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtleScanTool::new(backend);

        assert_eq!(tool.name(), "btle_scan");
        assert_eq!(tool.category(), "bt-recon");
        assert!(!tool.description().is_empty());
    }
}
