//! Bluetooth connection following tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for following a specific Bluetooth connection and capturing packets.
///
/// Monitors a targeted BR/EDR connection by BD_ADDR.
/// Follows AFH channel hopping patterns.
pub struct BtFollowTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl BtFollowTool {
    /// Create a new bt_follow tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for BtFollowTool {
    fn name(&self) -> &str {
        "bt_follow"
    }

    fn category(&self) -> &str {
        "bt-recon"
    }

    fn description(&self) -> &str {
        "Follow a specific Bluetooth connection and capture packets"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "bd_addr": {
                    "type": "string",
                    "description": "Target Bluetooth address (AA:BB:CC:DD:EE:FF)",
                    "pattern": "^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$"
                },
                "duration_sec": {
                    "type": "integer",
                    "description": "Follow duration in seconds",
                    "default": 60,
                    "minimum": 10,
                    "maximum": 600
                },
                "channel_hopping": {
                    "type": "boolean",
                    "description": "Follow AFH channel hopping",
                    "default": true
                }
            },
            "required": ["bd_addr"]
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
                "bd_addr": {
                    "type": "string"
                },
                "connection_found": {
                    "type": "boolean"
                },
                "packet_count": {
                    "type": "integer"
                },
                "duration_sec": {
                    "type": "integer"
                },
                "channels_used": {
                    "type": "array",
                    "items": { "type": "integer" }
                },
                "pcap_path": {
                    "type": "string"
                }
            },
            "required": ["success", "capture_id", "bd_addr", "connection_found"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing bt_follow");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("bt_follow", params).await?;

        tracing::info!("bt_follow completed successfully");
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
            if method == "bt_follow" {
                let bd_addr = params["bd_addr"].as_str().unwrap_or("unknown");
                Ok(json!({
                    "success": true,
                    "capture_id": "cap-follow-test123",
                    "bd_addr": bd_addr,
                    "connection_found": true,
                    "packet_count": 1250,
                    "duration_sec": 60,
                    "channels_used": [12, 15, 18, 22, 25, 30],
                    "pcap_path": "/home/user/.ubertooth/captures/cap-follow-test123.pcap"
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
    async fn test_bt_follow() {
        let backend = Arc::new(MockBackend);
        let tool = BtFollowTool::new(backend);

        let result = tool.execute(json!({
            "bd_addr": "AA:BB:CC:DD:EE:FF",
            "duration_sec": 60,
            "channel_hopping": true
        })).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["bd_addr"], "AA:BB:CC:DD:EE:FF");
        assert_eq!(result["connection_found"], true);
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = BtFollowTool::new(backend);

        assert_eq!(tool.name(), "bt_follow");
        assert_eq!(tool.category(), "bt-recon");
        assert_eq!(tool.requires_authorization(), true);
        assert_eq!(tool.authorization_category(), "bt-recon-targeted");
    }
}
