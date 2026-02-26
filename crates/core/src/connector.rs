//! Ubertooth One connector implementation.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use strike48_connector::{BaseConnector, ConnectorError, Result, TaskTypeSchema};
use tokio::sync::broadcast;

use crate::events::ToolEvent;
use crate::tools::ToolRegistry;

/// Ubertooth One connector that implements the Strike48 BaseConnector trait.
pub struct UbertoothConnector {
    registry: Arc<ToolRegistry>,
    event_tx: broadcast::Sender<ToolEvent>,
}

impl UbertoothConnector {
    /// Create a new Ubertooth connector with the given tool registry.
    pub fn new(registry: ToolRegistry) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            registry: Arc::new(registry),
            event_tx,
        }
    }

    /// Get a sender for broadcasting tool events.
    pub fn event_sender(&self) -> broadcast::Sender<ToolEvent> {
        self.event_tx.clone()
    }

    /// Subscribe to tool events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<ToolEvent> {
        self.event_tx.subscribe()
    }

    /// Emit a tool event.
    fn emit_event(&self, event: ToolEvent) {
        let _ = self.event_tx.send(event);
    }
}

impl BaseConnector for UbertoothConnector {
    fn connector_type(&self) -> &str {
        "ubertooth"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn execute(
        &self,
        request: Value,
        _capability_id: Option<&str>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send + '_>> {
        Box::pin(async move {
            // Extract tool name from request
            let tool_name = request
                .get("tool")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ConnectorError::InvokeFailed("Missing 'tool' field".to_string())
                })?;

            // Get the tool from registry
            let tool = self.registry.get(tool_name).ok_or_else(|| {
                ConnectorError::InvokeFailed(format!("Tool not found: {}", tool_name))
            })?;

            // Extract parameters
            let params = request.get("parameters").cloned().unwrap_or_else(|| json!({}));

            // Emit started event
            let start_time = std::time::Instant::now();
            self.emit_event(ToolEvent::Started {
                tool_name: tool_name.to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });

            // Execute the tool
            let tool_result: std::result::Result<Value, crate::error::UbertoothError> = tool.execute(params).await;

            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Emit completion event
            match &tool_result {
                Ok(_) => {
                    self.emit_event(ToolEvent::Completed {
                        tool_name: tool_name.to_string(),
                        duration_ms,
                        success: true,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
                Err(e) => {
                    self.emit_event(ToolEvent::Failed {
                        tool_name: tool_name.to_string(),
                        duration_ms,
                        error: e.to_string(),
                    });
                }
            }

            // Convert result to Strike48 format
            match tool_result {
                Ok(output) => Ok(json!({
                    "success": true,
                    "output": output,
                })),
                Err(e) => Ok(json!({
                    "success": false,
                    "error": e.to_string(),
                })),
            }
        })
    }

    fn capabilities(&self) -> Vec<TaskTypeSchema> {
        self.registry
            .tools()
            .iter()
            .map(|tool| TaskTypeSchema {
                task_type_id: tool.name().to_string(),
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                category: tool.category().to_string(),
                icon: "bluetooth".to_string(),
                input_schema_json: tool.input_schema().to_string(),
                output_schema_json: tool.output_schema().to_string(),
            })
            .collect()
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("device".to_string(), "Ubertooth One".to_string());
        metadata.insert("protocol".to_string(), "Bluetooth/BLE".to_string());
        metadata.insert("frequency".to_string(), "2.4 GHz".to_string());
        metadata
    }
}
