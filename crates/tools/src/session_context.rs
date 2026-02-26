//! Session context tool.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use ubertooth_core::error::Result;
use ubertooth_core::tools::PentestTool;
use ubertooth_platform::UbertoothBackendProvider;

/// Tool for comprehensive session orientation (AI-friendly).
///
/// Combines device_status + recent captures + configurations
/// into a single comprehensive response for AI context loading.
pub struct SessionContextTool {
    backend: Arc<dyn UbertoothBackendProvider>,
}

impl SessionContextTool {
    /// Create a new session_context tool.
    pub fn new(backend: Arc<dyn UbertoothBackendProvider>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl PentestTool for SessionContextTool {
    fn name(&self) -> &str {
        "session_context"
    }

    fn category(&self) -> &str {
        "bt-device"
    }

    fn description(&self) -> &str {
        "Comprehensive orientation for AI - device state, recent captures, and configurations"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "include_recent_captures": {
                    "type": "boolean",
                    "description": "Include recent capture list",
                    "default": true
                },
                "max_captures": {
                    "type": "integer",
                    "description": "Maximum number of recent captures to include",
                    "default": 5,
                    "minimum": 1,
                    "maximum": 20
                },
                "include_configs": {
                    "type": "boolean",
                    "description": "Include saved configurations",
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
                "timestamp": {
                    "type": "string",
                    "format": "date-time"
                },
                "device": {
                    "type": "object",
                    "properties": {
                        "connected": { "type": "boolean" },
                        "serial": { "type": "string" },
                        "firmware": { "type": "string" },
                        "mode": { "type": "string" },
                        "channel": { "type": "integer" }
                    }
                },
                "recent_captures": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "capture_id": { "type": "string" },
                            "timestamp": { "type": "string" },
                            "type": { "type": "string" },
                            "packet_count": { "type": "integer" },
                            "duration_sec": { "type": ["integer", "null"] },
                            "tags": { "type": "array", "items": { "type": "string" } }
                        }
                    }
                },
                "saved_configs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "config_name": { "type": "string" },
                            "channel": { "type": "integer" },
                            "modulation": { "type": "string" },
                            "description": { "type": "string" }
                        }
                    }
                },
                "storage": {
                    "type": "object",
                    "properties": {
                        "captures_dir": { "type": "string" },
                        "captures_count": { "type": "integer" },
                        "total_size_mb": { "type": "number" }
                    }
                }
            },
            "required": ["success", "timestamp", "device"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        tracing::info!("Executing session_context");
        tracing::debug!("Parameters: {}", params);

        let result = self.backend.call("session_context", params).await?;

        tracing::info!("session_context completed successfully");
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
            if method == "session_context" {
                Ok(json!({
                    "success": true,
                    "timestamp": "2026-02-26T15:30:45Z",
                    "device": {
                        "connected": true,
                        "serial": "0000000012AB",
                        "firmware": "2020-12-R1",
                        "mode": "idle",
                        "channel": 37
                    },
                    "recent_captures": [
                        {
                            "capture_id": "cap-abc123",
                            "timestamp": "2026-02-26T15:25:00Z",
                            "type": "btle_sniff",
                            "packet_count": 142,
                            "duration_sec": 30,
                            "tags": ["ble", "advertisements"]
                        }
                    ],
                    "saved_configs": [],
                    "storage": {
                        "captures_dir": "/home/user/.ubertooth/captures",
                        "captures_count": 23,
                        "total_size_mb": 145.3
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
    async fn test_session_context() {
        let backend = Arc::new(MockBackend);
        let tool = SessionContextTool::new(backend);

        let result = tool.execute(json!({})).await.unwrap();

        assert_eq!(result["success"], true);
        assert!(result["device"].is_object());
        assert!(result["recent_captures"].is_array());
    }

    #[test]
    fn test_tool_metadata() {
        let backend = Arc::new(MockBackend);
        let tool = SessionContextTool::new(backend);

        assert_eq!(tool.name(), "session_context");
        assert_eq!(tool.category(), "bt-device");
    }
}
