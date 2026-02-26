//! Device fingerprinting tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for device fingerprinting based on protocol behavior.
///
/// Identifies manufacturer, device type, and OS version from
/// Bluetooth/BLE packet patterns and characteristics.
pub struct BtFingerprintTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtFingerprintTool {
    /// Create a new bt_fingerprint tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtFingerprintTool {
    fn name(&self) -> &str {
        "bt_fingerprint"
    }

    fn category(&self) -> &str {
        "bt-analysis"
    }

    fn description(&self) -> &str {
        "Device fingerprinting based on protocol behavior"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "capture_id": {
                    "type": "string",
                    "description": "Capture ID to analyze"
                },
                "target_mac": {
                    "type": "string",
                    "description": "Target device MAC address",
                    "pattern": "^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$"
                }
            },
            "required": ["capture_id", "target_mac"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean"
                },
                "device": {
                    "type": "object",
                    "properties": {
                        "mac_address": { "type": "string" },
                        "fingerprint": {
                            "type": "object",
                            "properties": {
                                "manufacturer": { "type": "string" },
                                "device_type": { "type": "string" },
                                "os_version": { "type": ["string", "null"] },
                                "confidence": { "type": "number" }
                            }
                        },
                        "indicators": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    }
                }
            },
            "required": ["success", "device"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing bt_fingerprint");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("bt_fingerprint", params).await?;

        tracing::info!("bt_fingerprint completed successfully");
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
            if method == "bt_fingerprint" {
                Ok(json!({
                    "success": true,
                    "device": {
                        "mac_address": "AA:BB:CC:DD:EE:FF",
                        "fingerprint": {
                            "manufacturer": "Apple Inc.",
                            "device_type": "iPhone",
                            "os_version": "iOS 17.x",
                            "confidence": 0.92
                        },
                        "indicators": ["Manufacturer data prefix: 0x4C00 (Apple)"]
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
    async fn test_bt_fingerprint() {
        let backend = Arc::new(MockBackend);
        let tool = BtFingerprintTool::new(backend);

        let result = tool.execute(json!({
            "capture_id": "cap-test-123",
            "target_mac": "AA:BB:CC:DD:EE:FF"
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert!(result["device"].is_object());
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtFingerprintTool::new(backend);

        assert_eq!(tool.name(), "bt_fingerprint");
        assert_eq!(tool.category(), "bt-analysis");
    }
}
