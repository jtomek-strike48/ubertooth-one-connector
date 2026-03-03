//! Tool categories and selection logic

use std::sync::Arc;
use ubertooth_core::{PentestTool, ToolRegistry};

/// Tool categories matching ISSUE_TUI.md spec
#[derive(Debug, Clone, PartialEq)]
pub enum Category {
    DeviceManagement,
    Reconnaissance,
    Analysis,
    CaptureManagement,
    Configuration,
    AttackOperations,
    Advanced,
}

impl Category {
    /// Create category from menu index (0-6)
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Category::DeviceManagement,
            1 => Category::Reconnaissance,
            2 => Category::Analysis,
            3 => Category::CaptureManagement,
            4 => Category::Configuration,
            5 => Category::AttackOperations,
            6 => Category::Advanced,
            _ => Category::DeviceManagement,
        }
    }

    /// Get tools in this category
    pub fn get_tools(&self, registry: &Arc<ToolRegistry>) -> Vec<Arc<dyn PentestTool>> {
        let category_prefix = self.category_prefix();
        registry
            .tools()
            .iter()
            .filter(|tool| {
                let name = tool.name();
                match category_prefix {
                    "device_" => name.starts_with("device_"),
                    "recon" => {
                        name.starts_with("btle_scan")
                            || name.starts_with("btle_follow")
                            || name.starts_with("bt_scan")
                            || name.starts_with("bt_follow")
                            || name.starts_with("bt_discover")
                            || name.starts_with("bt_specan")
                            || name.starts_with("afh_analyze")
                    }
                    "analysis" => {
                        name.starts_with("bt_analyze")
                            || name.starts_with("bt_decode")
                            || name.starts_with("bt_fingerprint")
                            || name.starts_with("bt_compare")
                            || name.starts_with("pcap_merge")
                    }
                    "capture_" => name.starts_with("capture_"),
                    "config" => {
                        name.starts_with("configure_")
                            || name.starts_with("bt_save_config")
                            || name.starts_with("bt_load_config")
                            || name.starts_with("config_list")
                            || name.starts_with("config_delete")
                    }
                    "attack" => {
                        name.starts_with("btle_inject")
                            || name.starts_with("bt_jam")
                            || name.starts_with("btle_mitm")
                            || name.starts_with("btle_slave")
                            || name.starts_with("bt_spoof")
                    }
                    "advanced" => {
                        name.starts_with("ubertooth_raw") || name.starts_with("session_context")
                    }
                    _ => false,
                }
            })
            .cloned()
            .collect()
    }

    /// Get tool count in this category
    pub fn tool_count(&self, registry: &Arc<ToolRegistry>) -> usize {
        self.get_tools(registry).len()
    }

    /// Get tool at specific index in this category
    pub fn get_tool_at_index(&self, registry: &Arc<ToolRegistry>, index: usize) -> Option<String> {
        self.get_tools(registry)
            .get(index)
            .map(|tool| tool.name().to_string())
    }

    /// Get category prefix for filtering
    fn category_prefix(&self) -> &str {
        match self {
            Category::DeviceManagement => "device_",
            Category::Reconnaissance => "recon",
            Category::Analysis => "analysis",
            Category::CaptureManagement => "capture_",
            Category::Configuration => "config",
            Category::AttackOperations => "attack",
            Category::Advanced => "advanced",
        }
    }
}

