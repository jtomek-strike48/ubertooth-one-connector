//! BLE connection following tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for following a specific BLE connection by access address.
///
/// Monitors a targeted BLE connection using its access address.
/// Captures connection packets and events.
pub struct BtleFollowTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtleFollowTool {
    /// Create a new btle_follow tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtleFollowTool {
    fn name(&self) -> &str {
        "btle_follow"
    }

    fn category(&self) -> &str {
        "bt-recon"
    }

    fn description(&self) -> &str {
        "Follow a specific BLE connection using access address"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "access_address": {
                    "type": "string",
                    "description": "BLE access address in hex (e.g., 0x8E89BED6)",
                    "pattern": "^0x[0-9A-Fa-f]{8}$"
                },
                "duration_sec": {
                    "type": "integer",
                    "description": "Follow duration in seconds",
                    "default": 60,
                    "minimum": 10,
                    "maximum": 600
                },
                "crc_verify": {
                    "type": "boolean",
                    "description": "Verify CRC checksums",
                    "default": true
                },
                "follow_connections": {
                    "type": "boolean",
                    "description": "Follow connection events",
                    "default": true
                }
            },
            "required": ["access_address"]
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
                "access_address": {
                    "type": "string"
                },
                "packets_captured": {
                    "type": "integer"
                },
                "connection_events": {
                    "type": "integer"
                },
                "crc_valid_percent": {
                    "type": "number"
                },
                "pcap_path": {
                    "type": "string"
                }
            },
            "required": ["success", "capture_id", "access_address", "packets_captured"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing btle_follow");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("btle_follow", params).await?;

        tracing::info!("btle_follow completed successfully");
        Ok(result)
    }

    fn requires_authorization(&self) -> bool {
        true
    }

    fn authorization_category(&self) -> &str {
        "bt-recon-targeted"
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
            if method == "btle_follow" {
                let access_address = params["access_address"].as_str().unwrap_or("0x8E89BED6");
                Ok(json!({
                    "success": true,
                    "capture_id": "cap-btle-follow-test123",
                    "access_address": access_address,
                    "packets_captured": 350,
                    "connection_events": 120,
                    "crc_valid_percent": 98.5,
                    "pcap_path": "/home/user/.ubertooth/captures/cap-btle-follow-test123.pcap"
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
    async fn test_btle_follow() {
        let backend = Arc::new(MockBackend);
        let tool = BtleFollowTool::new(backend);

        let result = tool.execute(json!({
            "access_address": "0x8E89BED6",
            "duration_sec": 60,
            "crc_verify": true
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["access_address"], "0x8E89BED6");
        assert_eq!(result["packets_captured"], 350);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtleFollowTool::new(backend);

        assert_eq!(tool.name(), "btle_follow");
        assert_eq!(tool.category(), "bt-recon");
        assert_eq!(tool.requires_authorization(), true);
        assert_eq!(tool.authorization_category(), "bt-recon-targeted");
    }
}
