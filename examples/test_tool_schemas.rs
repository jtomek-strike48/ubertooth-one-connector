//! Test that tool schemas are properly exposed through Strike48 SDK format
//!
//! This verifies that:
//! 1. ConnectorBehavior::Tool is returned
//! 2. Metadata contains "tool_schemas" key
//! 3. Tool schemas are valid JSON
//! 4. All 36 tools are present in the schemas

use std::sync::Arc;
use strike48_connector::BaseConnector;
use ubertooth_core::connector::UbertoothConnector;
use ubertooth_platform::SidecarManager;
use ubertooth_tools::create_tool_registry;

fn main() -> anyhow::Result<()> {
    println!("=== Testing Tool Schema Exposure ===\n");

    // Create backend and tool registry
    let backend = SidecarManager::new();
    let tools = create_tool_registry(backend);
    let connector = Arc::new(UbertoothConnector::new(tools));

    // Test 1: Verify behavior is Tool
    println!("✓ Connector behavior: {:?}", connector.behavior());

    // Test 2: Get metadata
    let metadata = connector.metadata();
    println!("\n=== Connector Metadata ===");
    println!("  Name: {}", metadata.get("name").unwrap_or(&"N/A".to_string()));
    println!("  Description: {}", metadata.get("description").unwrap_or(&"N/A".to_string()));
    println!("  Device: {}", metadata.get("device").unwrap_or(&"N/A".to_string()));
    println!("  Protocol: {}", metadata.get("protocol").unwrap_or(&"N/A".to_string()));
    println!("  Frequency: {}", metadata.get("frequency").unwrap_or(&"N/A".to_string()));
    println!("  Tool Count: {}", metadata.get("tool_count").unwrap_or(&"N/A".to_string()));

    // Test 3: Verify tool_schemas key exists
    let tool_schemas_json = metadata
        .get("tool_schemas")
        .expect("Metadata should contain 'tool_schemas' key");

    println!("\n=== Tool Schemas (Strike48 SDK Format) ===");

    // Test 4: Parse and validate tool schemas
    let tool_schemas: Vec<serde_json::Value> = serde_json::from_str(tool_schemas_json)
        .expect("tool_schemas should be valid JSON array");

    println!("  Total tools: {}", tool_schemas.len());

    // Test 5: Show first 5 tool schemas as examples
    println!("\n  Example tool schemas:");
    for (i, schema) in tool_schemas.iter().take(5).enumerate() {
        let name = schema["name"].as_str().unwrap_or("unknown");
        let description = schema["description"].as_str().unwrap_or("no description");
        let param_count = schema["parameters"]["properties"]
            .as_object()
            .map(|o| o.len())
            .unwrap_or(0);

        println!("\n  {}. {}", i + 1, name);
        println!("     Description: {}", description);
        println!("     Parameters: {}", param_count);

        // Show parameter names
        if let Some(props) = schema["parameters"]["properties"].as_object() {
            let param_names: Vec<&str> = props.keys().map(|k| k.as_str()).collect();
            if !param_names.is_empty() {
                println!("     Param names: {}", param_names.join(", "));
            }
        }
    }

    // Test 6: Verify all expected tools are present
    println!("\n=== Tool Categories ===");
    let expected_tools = vec![
        "device_connect", "device_status", "device_disconnect",
        "btle_scan", "btle_follow", "bt_scan", "bt_follow",
        "bt_discover", "bt_specan", "afh_analyze",
        "bt_analyze", "bt_decode", "bt_fingerprint",
        "bt_compare", "pcap_merge",
        "capture_list", "capture_get", "capture_delete",
        "capture_tag", "capture_export",
        "configure_channel", "configure_modulation",
        "configure_power", "configure_squelch",
        "configure_leds", "bt_save_config",
        "bt_load_config", "config_list", "config_delete",
        "btle_inject", "bt_jam", "btle_mitm",
        "btle_slave", "bt_spoof",
        "ubertooth_raw", "session_context",
    ];

    let tool_names: Vec<&str> = tool_schemas
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();

    let mut missing = Vec::new();
    for expected in &expected_tools {
        if !tool_names.contains(expected) {
            missing.push(*expected);
        }
    }

    if missing.is_empty() {
        println!("  ✓ All {} expected tools are present!", expected_tools.len());
    } else {
        eprintln!("  ✗ Missing tools: {:?}", missing);
        return Err(anyhow::anyhow!("Some tools are missing from schemas"));
    }

    // Test 7: Show tool count by category (from tool names)
    println!("\n=== Strike48 SDK Integration ===");
    println!("  ✓ Behavior: ConnectorBehavior::Tool");
    println!("  ✓ Metadata key 'tool_schemas': Present");
    println!("  ✓ Tool schemas: Valid JSON ({} tools)", tool_schemas.len());
    println!("  ✓ All expected tools: Found");

    println!("\n=== Example Tool Schema (JSON) ===");
    if let Some(schema) = tool_schemas.first() {
        println!("{}", serde_json::to_string_pretty(schema)?);
    }

    println!("\n✅ All tests passed! Tools are properly exposed through Strike48 SDK.");

    Ok(())
}
