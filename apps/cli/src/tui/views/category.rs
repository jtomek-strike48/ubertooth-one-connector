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
            1 => Category::CaptureManagement,
            2 => Category::Reconnaissance,
            3 => Category::Analysis,
            4 => Category::AttackOperations,
            5 => Category::Configuration,
            6 => Category::Advanced,
            _ => Category::DeviceManagement,
        }
    }

    /// Get tools in this category
    pub fn get_tools(&self, registry: &Arc<ToolRegistry>) -> Vec<Arc<dyn PentestTool>> {
        let category_prefix = self.category_prefix();
        let mut tools: Vec<Arc<dyn PentestTool>> = registry
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
                    "capture_" => name == "capture_list",  // Only show capture_list (other ops are hotkeys)
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
            .collect();

        // Custom ordering for specific categories
        match self {
            Category::DeviceManagement => {
                // Order: device_connect, device_disconnect, device_status
                tools.sort_by_key(|tool| {
                    match tool.name() {
                        "device_connect" => 0,
                        "device_disconnect" => 1,
                        "device_status" => 2,
                        _ => 999,
                    }
                });
            }
            Category::CaptureManagement => {
                // Order: capture_list, capture_tag, capture_get, capture_export, capture_delete
                tools.sort_by_key(|tool| {
                    match tool.name() {
                        "capture_list" => 0,
                        "capture_tag" => 1,
                        "capture_get" => 2,
                        "capture_export" => 3,
                        "capture_delete" => 4,
                        _ => 999,
                    }
                });
            }
            _ => {
                // Default: sort alphabetically
                tools.sort_by_key(|tool| tool.name().to_string());
            }
        }

        tools
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

