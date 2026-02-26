//! Tool event broadcasting system.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Events emitted during tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolEvent {
    /// Tool execution started.
    Started {
        tool_name: String,
        timestamp: String,
    },

    /// Progress update during tool execution.
    Progress {
        tool_name: String,
        data: Value,
    },

    /// Tool execution completed successfully.
    Completed {
        tool_name: String,
        duration_ms: u64,
        success: bool,
        timestamp: String,
    },

    /// Tool execution failed.
    Failed {
        tool_name: String,
        duration_ms: u64,
        error: String,
    },
}
