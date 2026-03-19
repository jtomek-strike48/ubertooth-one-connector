# Plugin Development Guide

Guide for developing plugins to extend Ubertooth One functionality.

## Overview

The Ubertooth plugin system enables:
- **Packet Processing** - Filter, modify, or tag packets in real-time
- **Capture Analysis** - Analyze complete captures and generate reports
- **Custom Export** - Export data in custom formats
- **CLI Commands** - Add custom commands to the CLI

Plugins are dynamically loaded shared libraries (.so/.dll/.dylib) that implement the `Plugin` trait.

## Quick Start

### 1. Create Plugin Project

```bash
cargo new --lib my-plugin
cd my-plugin
```

### 2. Add Dependencies

```toml
[dependencies]
ubertooth-plugin = { path = "../crates/plugin" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[lib]
crate-type = ["cdylib"]  # Dynamic library
```

### 3. Implement Plugin

```rust
use ubertooth_plugin::*;

pub struct MyPlugin {
    metadata: PluginMetadata,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "my-plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "My awesome plugin".to_string(),
                author: "Your Name".to_string(),
                api_version: "1.0".to_string(),
            },
        }
    }
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::PacketProcessor]
    }

    fn initialize(&mut self, context: PluginContext) -> Result<(), PluginError> {
        // Initialize plugin
        Ok(())
    }

    fn process_packet(&mut self, packet: &PacketData) -> Result<ProcessResult, PluginError> {
        let mut result = ProcessResult::default();
        result.tags.push("processed".to_string());
        Ok(result)
    }
}

// Required: Declare plugin for dynamic loading
declare_plugin!(MyPlugin, MyPlugin::new);
```

### 4. Build Plugin

```bash
cargo build --release
```

Output: `target/release/libmy_plugin.so` (or .dll/.dylib)

### 5. Install Plugin

```bash
cp target/release/libmy_plugin.so ~/.ubertooth/plugins/
```

## Plugin API Reference

### Plugin Trait

Core trait all plugins must implement:

```rust
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn capabilities(&self) -> Vec<PluginCapability>;
    fn initialize(&mut self, context: PluginContext) -> Result<(), PluginError>;

    // Optional capabilities
    fn process_packet(&mut self, packet: &PacketData) -> Result<ProcessResult, PluginError>;
    fn analyze_capture(&mut self, capture_id: &str, packets: &[PacketData])
        -> Result<AnalysisResult, PluginError>;
    fn export(&mut self, capture_id: &str, packets: &[PacketData], format: &str)
        -> Result<Vec<u8>, PluginError>;
    fn execute_command(&mut self, command: &str, args: &[String])
        -> Result<Value, PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;
}
```

### Plugin Capabilities

```rust
pub enum PluginCapability {
    PacketProcessor,   // Real-time packet processing
    CaptureAnalyzer,   // Post-capture analysis
    Exporter,          // Custom export formats
    Command,           // Custom CLI commands
}
```

### Plugin Metadata

```rust
pub struct PluginMetadata {
    pub name: String,         // Unique plugin name
    pub version: String,      // Semantic version
    pub description: String,  // Short description
    pub author: String,       // Author name
    pub api_version: String,  // Compatible API version
}
```

### Plugin Context

Context provided during initialization:

```rust
pub struct PluginContext {
    pub config: HashMap<String, Value>,  // Configuration
    pub data_dir: PathBuf,                // Plugin data directory
}
```

Access configuration:
```rust
fn initialize(&mut self, context: PluginContext) -> Result<(), PluginError> {
    if let Some(value) = context.get_config("my_setting") {
        // Use configuration value
    }
    Ok(())
}
```

### Packet Data

```rust
pub struct PacketData {
    pub sequence: u64,           // Packet sequence number
    pub timestamp_ms: u64,       // Unix timestamp in milliseconds
    pub data: Vec<u8>,           // Raw packet bytes
    pub packet_type: String,     // Packet type (e.g., "LE_ADV")
    pub channel: u8,             // Channel number
    pub rssi: Option<i8>,        // Signal strength
    pub metadata: HashMap<String, Value>,  // Additional metadata
}
```

## Capability Implementations

### 1. Packet Processor

Process packets in real-time during capture:

```rust
fn process_packet(&mut self, packet: &PacketData) -> Result<ProcessResult, PluginError> {
    let mut result = ProcessResult::default();

    // Filter packets
    if let Some(rssi) = packet.rssi {
        if rssi < -70 {
            result.keep = false;  // Drop weak packets
            return Ok(result);
        }
    }

    // Add tags
    if packet.channel >= 37 && packet.channel <= 39 {
        result.tags.push("advertising".to_string());
    }

    // Add metadata
    result.metadata.insert(
        "processed_at".to_string(),
        serde_json::json!(std::time::SystemTime::now())
    );

    Ok(result)
}
```

**ProcessResult:**
```rust
pub struct ProcessResult {
    pub keep: bool,                         // Keep or drop packet
    pub modified_packet: Option<PacketData>, // Modified packet data
    pub tags: Vec<String>,                   // Tags to add
    pub metadata: HashMap<String, Value>,    // Additional metadata
}
```

### 2. Capture Analyzer

Analyze complete captures and generate reports:

```rust
fn analyze_capture(
    &mut self,
    capture_id: &str,
    packets: &[PacketData],
) -> Result<AnalysisResult, PluginError> {
    let mut findings = Vec::new();
    let mut statistics = HashMap::new();

    // Analyze packets
    let total_packets = packets.len();
    let weak_signals = packets.iter()
        .filter(|p| p.rssi.map_or(false, |r| r < -70))
        .count();

    // Create findings
    if weak_signals > total_packets / 2 {
        findings.push(Finding {
            severity: Severity::Medium,
            title: "High packet loss".to_string(),
            description: format!("{} of {} packets have weak signal",
                weak_signals, total_packets),
            packet_indices: vec![],
        });
    }

    // Collect statistics
    statistics.insert("total_packets".to_string(), serde_json::json!(total_packets));
    statistics.insert("weak_signals".to_string(), serde_json::json!(weak_signals));

    Ok(AnalysisResult {
        summary: format!("Analyzed {} packets", total_packets),
        findings,
        statistics,
        recommendations: vec![
            "Move closer to target device".to_string()
        ],
    })
}
```

**AnalysisResult:**
```rust
pub struct AnalysisResult {
    pub summary: String,
    pub findings: Vec<Finding>,
    pub statistics: HashMap<String, Value>,
    pub recommendations: Vec<String>,
}

pub struct Finding {
    pub severity: Severity,      // Info, Low, Medium, High, Critical
    pub title: String,
    pub description: String,
    pub packet_indices: Vec<usize>,
}
```

### 3. Exporter

Export data in custom formats:

```rust
fn export(
    &mut self,
    capture_id: &str,
    packets: &[PacketData],
    format: &str,
) -> Result<Vec<u8>, PluginError> {
    match format {
        "csv" => {
            let mut csv = String::new();
            csv.push_str("seq,timestamp,channel,rssi,type\n");

            for packet in packets {
                csv.push_str(&format!(
                    "{},{},{},{},{}\n",
                    packet.sequence,
                    packet.timestamp_ms,
                    packet.channel,
                    packet.rssi.unwrap_or(0),
                    packet.packet_type
                ));
            }

            Ok(csv.into_bytes())
        }
        "json" => {
            let json = serde_json::to_vec_pretty(packets)?;
            Ok(json)
        }
        _ => Err(PluginError::NotSupported(format!("Unknown format: {}", format)))
    }
}
```

### 4. Command

Add custom CLI commands:

```rust
fn execute_command(
    &mut self,
    command: &str,
    args: &[String],
) -> Result<Value, PluginError> {
    match command {
        "stats" => {
            Ok(serde_json::json!({
                "processed_packets": self.packet_count,
                "filtered_packets": self.filtered_count
            }))
        }
        "set-threshold" => {
            if args.is_empty() {
                return Err(PluginError::ExecutionError(
                    "Missing threshold value".to_string()
                ));
            }

            let threshold: i8 = args[0].parse()
                .map_err(|_| PluginError::ExecutionError(
                    "Invalid threshold".to_string()
                ))?;

            self.threshold = threshold;

            Ok(serde_json::json!({
                "success": true,
                "new_threshold": threshold
            }))
        }
        _ => Err(PluginError::ExecutionError(
            format!("Unknown command: {}", command)
        ))
    }
}
```

## Plugin Lifecycle

### 1. Initialization

Called once when plugin is loaded:

```rust
fn initialize(&mut self, context: PluginContext) -> Result<(), PluginError> {
    // Read configuration
    if let Some(value) = context.get_config("setting") {
        self.setting = value.as_str().unwrap_or("default");
    }

    // Set up resources
    self.cache = HashMap::new();

    // Create data directory if needed
    std::fs::create_dir_all(&context.data_dir)?;

    tracing::info!("Plugin initialized");
    Ok(())
}
```

### 2. Operation

Plugin methods called based on capabilities:
- `process_packet()` - For each packet during capture
- `analyze_capture()` - When user requests analysis
- `export()` - When user requests export
- `execute_command()` - When user invokes command

### 3. Shutdown

Called before plugin is unloaded:

```rust
fn shutdown(&mut self) -> Result<(), PluginError> {
    // Clean up resources
    self.cache.clear();

    // Save state if needed
    self.save_state()?;

    tracing::info!("Plugin shutting down");
    Ok(())
}
```

## Error Handling

### Plugin Errors

```rust
pub enum PluginError {
    NotFound(String),         // Plugin not found
    LoadError(String),        // Failed to load plugin
    InitError(String),        // Initialization failed
    ExecutionError(String),   // Execution failed
    NotSupported(String),     // Capability not supported
    ConfigError(String),      // Configuration error
    IoError(std::io::Error),  // I/O error
    SerializationError(serde_json::Error),  // JSON error
}
```

### Error Handling Best Practices

1. **Return descriptive errors:**
```rust
if args.is_empty() {
    return Err(PluginError::ExecutionError(
        "Command requires at least one argument".to_string()
    ));
}
```

2. **Use Result types:**
```rust
fn process_packet(&mut self, packet: &PacketData) -> Result<ProcessResult, PluginError>
```

3. **Handle recoverable errors gracefully:**
```rust
match self.parse_data(&packet.data) {
    Ok(parsed) => {
        // Process successfully
    }
    Err(e) => {
        // Log error but continue
        tracing::warn!("Failed to parse packet: {}", e);
        return Ok(ProcessResult::default());
    }
}
```

## Configuration

### Plugin Configuration

Plugins can be configured via JSON:

```json
{
  "plugins": {
    "my-plugin": {
      "enabled": true,
      "config": {
        "rssi_threshold": -70,
        "max_packets": 1000,
        "log_level": "debug"
      }
    }
  }
}
```

Access in plugin:

```rust
fn initialize(&mut self, context: PluginContext) -> Result<(), PluginError> {
    if let Some(threshold) = context.get_config("rssi_threshold") {
        self.threshold = threshold.as_i64().unwrap_or(-70) as i8;
    }
    Ok(())
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_processing() {
        let mut plugin = MyPlugin::new();
        let context = PluginContext::new(
            HashMap::new(),
            PathBuf::from("/tmp")
        );
        plugin.initialize(context).unwrap();

        let packet = PacketData {
            sequence: 1,
            timestamp_ms: 1000,
            data: vec![1, 2, 3],
            packet_type: "LE_ADV".to_string(),
            channel: 37,
            rssi: Some(-50),
            metadata: HashMap::new(),
        };

        let result = plugin.process_packet(&packet).unwrap();
        assert!(result.keep);
    }
}
```

### Integration Tests

Build and load plugin:

```bash
# Build plugin
cargo build --release

# Copy to plugins directory
cp target/release/libmy_plugin.so ~/.ubertooth/plugins/

# Test loading
ubertooth-cli --list-plugins
```

## Example: Sample Plugin

See `examples/sample_plugin.rs` for a complete working example that demonstrates:
- RSSI-based packet filtering
- Packet type counting
- Capture analysis with findings
- Custom commands
- Configuration handling

Build and install:

```bash
cargo build --example sample_plugin --release
cp target/release/examples/libsample_plugin.so ~/.ubertooth/plugins/
```

## Plugin Registry

### Loading Plugins

```rust
use ubertooth_plugin::{PluginRegistry, PluginContext};

// Create registry
let mut registry = PluginRegistry::new("/path/to/plugins");

// Load single plugin
let context = PluginContext::new(config, data_dir);
registry.load_plugin("my_plugin.so", context.clone())?;

// Load all plugins
let results = registry.load_all(context);

// List loaded plugins
for name in registry.list_plugins() {
    println!("Loaded: {}", name);
}
```

### Using Plugins

```rust
// Get plugin
if let Some(plugin) = registry.get_plugin("my-plugin") {
    let metadata = plugin.metadata();
    println!("{}: {}", metadata.name, metadata.description);
}

// Get mutable plugin for operations
if let Some(plugin) = registry.get_plugin_mut("my-plugin") {
    let result = plugin.process_packet(&packet)?;
}

// Find plugins with capability
let analyzers = registry.get_plugins_with_capability(
    PluginCapability::CaptureAnalyzer
);
```

## Best Practices

### 1. Keep Plugins Focused
- One responsibility per plugin
- Small, composable plugins > monolithic
- Clear capability declaration

### 2. Handle Errors Gracefully
- Don't panic in plugin code
- Return descriptive errors
- Log warnings for non-critical issues

### 3. Be Efficient
- Minimize allocations in hot paths
- Cache expensive computations
- Use appropriate data structures

### 4. Document Thoroughly
- Clear README
- Usage examples
- Configuration options
- Expected input/output

### 5. Version Carefully
- Follow semantic versioning
- Test API compatibility
- Document breaking changes

## Troubleshooting

### Plugin Won't Load

**Check file extension:**
```bash
ls -l ~/.ubertooth/plugins/
# Should be .so (Linux), .dylib (macOS), or .dll (Windows)
```

**Check plugin symbols:**
```bash
nm -D libmy_plugin.so | grep _plugin_create
# Should show: _plugin_create
```

**Check logs:**
```bash
RUST_LOG=debug ubertooth-cli
```

### Plugin Crashes

- Check for null pointer dereferences
- Verify Send + Sync safety
- Use proper error handling
- Test in isolation first

### Performance Issues

- Profile with `perf` or `flamegraph`
- Check allocation patterns
- Optimize hot paths
- Consider caching

## API Versioning

Current API version: **1.0**

Plugins must specify compatible API version:
```rust
api_version: "1.0".to_string()
```

Breaking changes will increment major version.

## References

- [Sample Plugin](../examples/sample_plugin.rs)
- [Plugin Crate](../crates/plugin/)
- [libloading Documentation](https://docs.rs/libloading)
- [Dynamic Loading in Rust](https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html)

---

**Last Updated:** 2026-03-19
**Phase:** 4.2 - Plugin System
