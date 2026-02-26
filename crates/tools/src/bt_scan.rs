//! Bluetooth Classic scanning tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for scanning Bluetooth Classic devices (inquiry scan).
///
/// Performs BR/EDR device discovery using inquiry scan.
/// Captures Extended Inquiry Response (EIR) data when available.
pub struct BtScanTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtScanTool {
    /// Create a new bt_scan tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtScanTool {
    fn name(&self) -> &str {
        "bt_scan"
    }

    fn category(&self) -> &str {
        "bt-recon"
    }

    fn description(&self) -> &str {
        "Scan for Bluetooth Classic devices (inquiry scan)"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "duration_sec": {
                    "type": "integer",
                    "description": "Inquiry scan duration in seconds",
                    "default": 30,
                    "minimum": 5,
                    "maximum": 300
                },
                "extended_inquiry": {
                    "type": "boolean",
                    "description": "Capture Extended Inquiry Response (EIR) data",
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
                    "type": "boolean"
                },
                "capture_id": {
                    "type": "string"
                },
                "devices_found": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "bd_addr": { "type": "string" },
                            "class_of_device": { "type": "string" },
                            "device_class": { "type": "string" },
                            "device_name": { "type": ["string", "null"] },
                            "rssi": { "type": "integer" },
                            "clock_offset": { "type": "integer" },
                            "page_scan_mode": { "type": "integer" },
                            "eir_data": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": ["string", "null"] },
                                    "services": {
                                        "type": "array",
                                        "items": { "type": "string" }
                                    }
                                }
                            }
                        }
                    }
                },
                "total_devices": {
                    "type": "integer"
                },
                "pcap_path": {
                    "type": "string"
                }
            },
            "required": ["success", "devices_found", "total_devices"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing bt_scan");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("bt_scan", params).await?;

        tracing::info!("bt_scan completed successfully");
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
            if method == "bt_scan" {
                Ok(json!({
                    "success": true,
                    "capture_id": "cap-bt-test123",
                    "devices_found": [
                        {
                            "bd_addr": "AA:BB:CC:DD:EE:FF",
                            "class_of_device": "0x5A020C",
                            "device_class": "Phone, Smartphone",
                            "device_name": "Test Phone",
                            "rssi": -60,
                            "clock_offset": 12345,
                            "page_scan_mode": 1,
                            "eir_data": {
                                "name": "Test Phone",
                                "services": ["1108", "110B"]
                            }
                        }
                    ],
                    "total_devices": 1,
                    "pcap_path": "/home/user/.ubertooth/captures/cap-bt-test123.pcap"
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
    async fn test_bt_scan() {
        let backend = Arc::new(MockBackend);
        let tool = BtScanTool::new(backend);

        let result = tool.execute(json!({
            "duration_sec": 30,
            "extended_inquiry": true
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["total_devices"], 1);
        assert!(result["devices_found"].is_array());
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtScanTool::new(backend);

        assert_eq!(tool.name(), "bt_scan");
        assert_eq!(tool.category(), "bt-recon");
    }
}
