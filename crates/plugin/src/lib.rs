//! Plugin system for extending Ubertooth functionality.
//!
//! Plugins can:
//! - Process captured packets
//! - Add custom analysis
//! - Implement custom export formats
//! - Extend CLI commands

pub mod loader;
pub mod registry;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin API version
    pub api_version: String,
}

/// Plugin context provided to plugins
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Configuration passed to plugin
    pub config: HashMap<String, Value>,
    /// Plugin data directory
    pub data_dir: std::path::PathBuf,
}

impl PluginContext {
    /// Create new plugin context
    pub fn new(config: HashMap<String, Value>, data_dir: std::path::PathBuf) -> Self {
        Self { config, data_dir }
    }

    /// Get configuration value
    pub fn get_config(&self, key: &str) -> Option<&Value> {
        self.config.get(key)
    }
}

/// Plugin capability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCapability {
    /// Process packets
    PacketProcessor,
    /// Analyze captures
    CaptureAnalyzer,
    /// Export data
    Exporter,
    /// Custom command
    Command,
}

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Get plugin capabilities
    fn capabilities(&self) -> Vec<PluginCapability>;

    /// Initialize plugin with context
    fn initialize(&mut self, context: PluginContext) -> Result<(), PluginError>;

    /// Process packet (if capability includes PacketProcessor)
    fn process_packet(&mut self, _packet: &PacketData) -> Result<ProcessResult, PluginError> {
        Err(PluginError::NotSupported(
            "PacketProcessor capability not implemented".to_string(),
        ))
    }

    /// Analyze capture (if capability includes CaptureAnalyzer)
    fn analyze_capture(
        &mut self,
        _capture_id: &str,
        _packets: &[PacketData],
    ) -> Result<AnalysisResult, PluginError> {
        Err(PluginError::NotSupported(
            "CaptureAnalyzer capability not implemented".to_string(),
        ))
    }

    /// Export data (if capability includes Exporter)
    fn export(
        &mut self,
        _capture_id: &str,
        _packets: &[PacketData],
        _format: &str,
    ) -> Result<Vec<u8>, PluginError> {
        Err(PluginError::NotSupported(
            "Exporter capability not implemented".to_string(),
        ))
    }

    /// Execute command (if capability includes Command)
    fn execute_command(&mut self, _command: &str, _args: &[String]) -> Result<Value, PluginError> {
        Err(PluginError::NotSupported(
            "Command capability not implemented".to_string(),
        ))
    }

    /// Shutdown plugin
    fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Packet data passed to plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketData {
    /// Packet sequence number
    pub sequence: u64,
    /// Timestamp (unix timestamp in milliseconds)
    pub timestamp_ms: u64,
    /// Raw packet bytes
    pub data: Vec<u8>,
    /// Packet type
    pub packet_type: String,
    /// Channel
    pub channel: u8,
    /// RSSI
    pub rssi: Option<i8>,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

/// Packet processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    /// Whether to keep the packet
    pub keep: bool,
    /// Modified packet data (if any)
    pub modified_packet: Option<PacketData>,
    /// Additional tags to add
    pub tags: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

impl Default for ProcessResult {
    fn default() -> Self {
        Self {
            keep: true,
            modified_packet: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Analysis summary
    pub summary: String,
    /// Detailed findings
    pub findings: Vec<Finding>,
    /// Statistics
    pub statistics: HashMap<String, Value>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Individual finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Finding severity
    pub severity: Severity,
    /// Finding title
    pub title: String,
    /// Finding description
    pub description: String,
    /// Related packet indices
    pub packet_indices: Vec<usize>,
}

/// Finding severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Plugin error types
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin load error: {0}")]
    LoadError(String),

    #[error("Plugin initialization error: {0}")]
    InitError(String),

    #[error("Plugin execution error: {0}")]
    ExecutionError(String),

    #[error("Capability not supported: {0}")]
    NotSupported(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Type alias for plugin results
pub type PluginResult<T> = Result<T, PluginError>;

/// Plugin declaration macro for plugin authors
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut dyn $crate::Plugin {
            let constructor: fn() -> $plugin_type = $constructor;
            let plugin = constructor();
            let boxed: Box<dyn $crate::Plugin> = Box::new(plugin);
            Box::into_raw(boxed)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        metadata: PluginMetadata,
    }

    impl TestPlugin {
        fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    name: "test-plugin".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Test plugin".to_string(),
                    author: "Test Author".to_string(),
                    api_version: "1.0".to_string(),
                },
            }
        }
    }

    impl Plugin for TestPlugin {
        fn metadata(&self) -> PluginMetadata {
            self.metadata.clone()
        }

        fn capabilities(&self) -> Vec<PluginCapability> {
            vec![PluginCapability::PacketProcessor]
        }

        fn initialize(&mut self, _context: PluginContext) -> Result<(), PluginError> {
            Ok(())
        }

        fn process_packet(&mut self, packet: &PacketData) -> Result<ProcessResult, PluginError> {
            let mut result = ProcessResult::default();
            result.tags.push("processed".to_string());
            result
                .metadata
                .insert("test".to_string(), serde_json::json!(packet.sequence));
            Ok(result)
        }
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = TestPlugin::new();
        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "test-plugin");
        assert_eq!(metadata.version, "1.0.0");
    }

    #[test]
    fn test_plugin_capabilities() {
        let plugin = TestPlugin::new();
        let caps = plugin.capabilities();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0], PluginCapability::PacketProcessor);
    }

    #[test]
    fn test_plugin_process_packet() {
        let mut plugin = TestPlugin::new();
        let packet = PacketData {
            sequence: 123,
            timestamp_ms: 1000,
            data: vec![1, 2, 3],
            packet_type: "TEST".to_string(),
            channel: 37,
            rssi: Some(-50),
            metadata: HashMap::new(),
        };

        let result = plugin.process_packet(&packet).unwrap();
        assert!(result.keep);
        assert_eq!(result.tags.len(), 1);
        assert_eq!(result.tags[0], "processed");
    }

    #[test]
    fn test_plugin_context() {
        let mut config = HashMap::new();
        config.insert("key".to_string(), serde_json::json!("value"));
        let data_dir = std::path::PathBuf::from("/tmp");

        let context = PluginContext::new(config, data_dir.clone());
        assert_eq!(
            context.get_config("key").unwrap(),
            &serde_json::json!("value")
        );
        assert_eq!(context.data_dir, data_dir);
    }
}
