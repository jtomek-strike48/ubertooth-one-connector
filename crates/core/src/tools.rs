//! Tool trait and registry for Ubertooth One operations.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;

/// A penetration testing tool that can be executed by the connector.
#[async_trait]
pub trait PentestTool: Send + Sync {
    /// Returns the unique identifier for this tool (e.g., "device_connect", "btle_scan").
    fn name(&self) -> &str;

    /// Returns the category this tool belongs to (e.g., "bt-device", "bt-recon").
    fn category(&self) -> &str;

    /// Returns a human-readable description of what this tool does.
    fn description(&self) -> &str;

    /// Returns the JSON schema for input parameters.
    fn input_schema(&self) -> Value;

    /// Returns the JSON schema for output results.
    fn output_schema(&self) -> Value;

    /// Returns whether this tool requires explicit authorization.
    ///
    /// Phase 1: All tools return false (no authorization enforcement yet)
    /// Phase 2: Attack tools return true
    fn requires_authorization(&self) -> bool {
        false
    }

    /// Returns the authorization category for this tool (e.g., "bt-attack").
    /// Only used if `requires_authorization()` returns true.
    fn authorization_category(&self) -> &str {
        "none"
    }

    /// Execute the tool with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - JSON object containing tool-specific parameters
    ///
    /// # Returns
    ///
    /// JSON object containing tool-specific results
    async fn execute(&self, params: Value) -> Result<Value>;
}

/// Registry of all available tools.
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn PentestTool>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool with the registry.
    pub fn register(&mut self, tool: Arc<dyn PentestTool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn PentestTool>> {
        self.tools.get(name).cloned()
    }

    /// Get all tool names.
    pub fn names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get all tools.
    pub fn tools(&self) -> Vec<Arc<dyn PentestTool>> {
        self.tools.values().cloned().collect()
    }

    /// Get tools by category.
    pub fn tools_by_category(&self, category: &str) -> Vec<Arc<dyn PentestTool>> {
        self.tools
            .values()
            .filter(|t| t.category() == category)
            .cloned()
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
