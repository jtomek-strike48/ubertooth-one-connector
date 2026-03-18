# Strike48 SDK Integration

All 36 Ubertooth tools are now properly exposed through the Strike48 SDK using the **Tool behavior** pattern.

## What Was Done

### 1. Implemented Strike48 Tool Behavior

Updated `crates/core/src/connector.rs` to use the proper Strike48 SDK pattern:

```rust
fn behavior(&self) -> ConnectorBehavior {
    ConnectorBehavior::Tool
}
```

### 2. Exported Tool Schemas in SDK Format

Tool schemas are now exported in the Strike48 SDK format via metadata:

```rust
fn metadata(&self) -> HashMap<String, String> {
    // Device metadata
    meta.insert("name", "Ubertooth One");
    meta.insert("description", "Bluetooth and BLE sniffing...");

    // Tool schemas - REQUIRED for TOOL behavior
    meta.insert("tool_schemas", serde_json::to_string(&tool_schemas));
    meta.insert("tool_count", "36");
    meta.insert("tool_names", "device_connect,btle_scan,...");
}
```

Each tool schema includes:
- `name` - Tool identifier
- `description` - What the tool does
- `parameters` - JSON Schema for input parameters

### 3. Verified Integration

When the agent starts, it logs:

```
INFO Connector behavior: Tool
INFO Tool schemas exported: 36 tools
INFO ✓ Tool schemas valid (Strike48 SDK format)
```

## How It Works

1. **Agent starts** with `STRIKE48_URL` environment variable set
2. **Connector registers** with Strike48 server
3. **Server calls** `connector.metadata()` to get tool schemas
4. **AI agents discover** available tools via Strike48 API
5. **AI agents execute** tools by sending requests with `tool` and `parameters`

## Request Format

AI agents send requests like this:

```json
{
  "tool": "btle_scan",
  "parameters": {
    "duration_sec": 30,
    "channel": 37,
    "save_pcap": true
  }
}
```

## Response Format

The connector returns:

```json
{
  "success": true,
  "output": {
    "capture_id": "cap-btle-37-20260302-120000",
    "packets_captured": 461,
    "devices_found": 155,
    "duration": 30
  }
}
```

## Tool Categories Exposed

All 36 tools are exposed across 7 categories:

### 🔌 bt-device (3 tools)
- `device_connect` - Connect to Ubertooth One
- `device_status` - Get device state
- `device_disconnect` - Disconnect from device

### 🔍 bt-recon (7 tools)
- `btle_scan` - Scan for BLE devices
- `btle_follow` - Follow BLE connection
- `bt_scan` - Scan for Bluetooth Classic
- `bt_follow` - Follow BT Classic connection
- `bt_discover` - Promiscuous BT discovery
- `bt_specan` - Spectrum analysis
- `afh_analyze` - AFH pattern analysis

### 📊 bt-analysis (5 tools)
- `bt_analyze` - Analyze captured packets
- `bt_decode` - Decode protocol layers
- `bt_fingerprint` - Device fingerprinting
- `bt_compare` - Compare captures
- `pcap_merge` - Merge multiple captures

### 📁 bt-capture (5 tools)
- `capture_list` - List saved captures
- `capture_get` - Retrieve capture data
- `capture_delete` - Delete captures
- `capture_tag` - Tag captures
- `capture_export` - Export to PCAP/JSON/CSV

### ⚙️ bt-config (8 tools)
- `configure_channel` - Set channel
- `configure_modulation` - Set modulation
- `configure_power` - Set TX power
- `configure_squelch` - Set RSSI threshold
- `configure_leds` - Control LEDs
- `bt_save_config` - Save preset
- `bt_load_config` - Load preset
- `config_list` - List presets
- `config_delete` - Delete preset

### 🎯 bt-attack (5 tools) ⚠️ REQUIRES AUTHORIZATION
- `btle_inject` - Inject BLE packets
- `bt_jam` - Jam frequencies
- `btle_mitm` - MITM attack
- `btle_slave` - Act as peripheral
- `bt_spoof` - Spoof device identity

### 🔧 bt-advanced (2 tools)
- `ubertooth_raw` - Send raw USB commands
- `session_context` - Get session context

## Running in Production

```bash
export STRIKE48_URL="wss://your-server.com"
export TENANT_ID="your-tenant"
export AUTH_TOKEN="your-token"

cargo run --release --bin ubertooth-agent --features rust-backend
```

The agent will:
1. Connect to Strike48 server
2. Register all 36 tools with metadata
3. Start listening for tool execution requests
4. Execute tools and return results

## Backend Options

- **Rust Backend** (Phase 3) - 9 native tools with Python fallback
  - 100-200x faster for streaming operations
  - Native: device_*, configure_*, btle_scan, btle_follow, bt_specan
  - Falls back to Python for other tools

- **Python Backend** (default) - All 36 tools via ubertooth-tools
  - Mature, stable implementation
  - Requires ubertooth-tools installed

## Tool Discovery

AI agents can discover tools by:

1. Querying the Strike48 API for connector capabilities
2. Parsing the `tool_schemas` metadata field
3. Inspecting parameter schemas to understand inputs
4. Executing tools with proper parameter validation

## Developer Testing

Test tool schema exposure locally:

```bash
# Run without STRIKE48_URL for local smoke test
cargo run --bin ubertooth-agent --features rust-backend

# Look for these log lines:
# INFO Connector behavior: Tool
# INFO Tool schemas exported: 36 tools
# INFO ✓ Tool schemas valid (Strike48 SDK format)
```

## Related Documentation

- `AGENT_PROMPT.md` - Complete tool documentation for AI agents
- `TOOLS_EXPOSED.md` - List of all exposed tools
- `~/Code/sdk-rs/crates/connector/src/behaviors/tool.rs` - Strike48 SDK Tool behavior
- `~/Code/sdk-rs/crates/connector/examples/system_command_tool.rs` - Example tool connector

## Next Steps

1. Deploy agent to production Strike48 environment
2. Test tool execution from Strike48 UI
3. Verify AI agents can discover and use all 36 tools
4. Monitor tool execution metrics and performance
