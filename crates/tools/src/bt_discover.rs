//! Promiscuous Bluetooth discovery tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for promiscuous Bluetooth BR/EDR discovery.
///
/// Captures any Bluetooth Classic traffic without targeting specific devices.
/// Useful for discovering hidden piconets and analyzing BT activity.
pub struct BtDiscoverTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtDiscoverTool {
    /// Create a new bt_discover tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtDiscoverTool {
    fn name(&self) -> &str {
        "bt_discover"
    }

    fn category(&self) -> &str {
        "bt-recon"
    }

    fn description(&self) -> &str {
        "Promiscuous Bluetooth discovery - capture any BR/EDR traffic"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "duration_sec": {
                    "type": "integer",
                    "description": "Capture duration in seconds",
                    "default": 60,
                    "minimum": 10,
                    "maximum": 600
                },
                "channel": {
                    "type": ["integer", "null"],
                    "description": "Specific channel (0-78) or null to hop all channels",
                    "minimum": 0,
                    "maximum": 78
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
                    "type": "boolean"
                },
                "capture_id": {
                    "type": "string"
                },
                "duration_sec": {
                    "type": "integer"
                },
                "piconets_found": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "bd_addr": { "type": "string" },
                            "uap": { "type": "integer" },
                            "packet_count": { "type": "integer" }
                        }
                    }
                },
                "total_packets": {
                    "type": "integer"
                },
                "pcap_path": {
                    "type": "string"
                }
            },
            "required": ["success", "capture_id", "total_packets"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing bt_discover");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("bt_discover", params).await?;

        tracing::info!("bt_discover completed successfully");
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
            if method == "bt_discover" {
                Ok(json!({
                    "success": true,
                    "capture_id": "cap-discover-test123",
                    "duration_sec": 60,
                    "piconets_found": [
                        {
                            "bd_addr": "AA:BB:CC:DD:EE:FF",
                            "uap": 170,
                            "packet_count": 450
                        }
                    ],
                    "total_packets": 2500,
                    "pcap_path": "/home/user/.ubertooth/captures/cap-discover-test123.pcap"
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
    async fn test_bt_discover() {
        let backend = Arc::new(MockBackend);
        let tool = BtDiscoverTool::new(backend);

        let result = tool.execute(json!({
            "duration_sec": 60,
            "save_pcap": true
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["total_packets"], 2500);
        assert!(result["piconets_found"].is_array());
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtDiscoverTool::new(backend);

        assert_eq!(tool.name(), "bt_discover");
        assert_eq!(tool.category(), "bt-recon");
    }
}
