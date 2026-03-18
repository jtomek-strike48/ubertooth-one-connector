# Ubertooth Connector - Exposed Tools

All 36 tools are exposed through the Strike48 connector and automatically registered when the agent starts.

## Tool Categories

### 🔌 bt-device (3 tools)
- `device_connect` - Connect to an Ubertooth One USB device
- `device_status` - Get current device state and configuration
- `device_disconnect` - Disconnect from Ubertooth One and release USB device

### 🔍 bt-recon (7 tools)
- `btle_scan` - Scan for BLE devices and capture advertisements
- `btle_follow` - Follow a specific BLE connection using access address
- `bt_scan` - Scan for Bluetooth Classic devices (inquiry scan)
- `bt_follow` - Follow a specific Bluetooth connection and capture packets
- `bt_discover` - Promiscuous Bluetooth discovery - capture any BR/EDR traffic
- `bt_specan` - Spectrum analysis of 2.4 GHz ISM band
- `afh_analyze` - Analyze Adaptive Frequency Hopping (AFH) channel usage

### 📊 bt-analysis (5 tools)
- `bt_analyze` - Analyze captured packets and extract insights
- `bt_decode` - Decode specific Bluetooth packet types (L2CAP, ATT, SMP, etc.)
- `bt_fingerprint` - Device fingerprinting based on protocol behavior
- `bt_compare` - Compare two captures to find differences
- `pcap_merge` - Merge multiple captures into a single PCAP file

### 📁 bt-capture (5 tools)
- `capture_list` - List stored packet captures with filtering
- `capture_get` - Retrieve packet data from a capture with pagination
- `capture_delete` - Delete a stored capture
- `capture_tag` - Add tags and notes to a capture
- `capture_export` - Export capture to standard formats (PCAP, JSON, CSV)

### ⚙️ bt-config (8 tools)
- `configure_channel` - Set Bluetooth channel (0-78)
- `configure_modulation` - Set modulation type (BT Basic Rate, BT Low Energy, etc.)
- `configure_power` - Set TX power level and amplifier settings
- `configure_squelch` - Set RSSI squelch threshold to filter weak signals
- `configure_leds` - Control LED indicators
- `bt_save_config` - Save current radio configuration as a named preset
- `bt_load_config` - Load a saved configuration preset
- `config_list` - List all saved configuration presets
- `config_delete` - Delete a saved configuration preset

### 🎯 bt-attack (5 tools) ⚠️ REQUIRES AUTHORIZATION
- `btle_inject` - Inject BLE packets into a connection
- `bt_jam` - Jam Bluetooth frequencies (denial of service) - HIGHLY REGULATED
- `btle_mitm` - Perform Man-in-the-Middle attack on BLE connection
- `btle_slave` - Act as a BLE peripheral/slave device
- `bt_spoof` - Spoof a Bluetooth device identity

### 🔧 bt-advanced (2 tools)
- `ubertooth_raw` - Send raw USB commands to Ubertooth
- `session_context` - Comprehensive orientation for AI agents

## How Tools Are Exposed

Tools are exposed through the Strike48 connector infrastructure:

1. **Tool Registry** (`crates/tools/src/lib.rs`)
   - All 36 tools registered in `create_tool_registry()`
   - Each tool implements the `Tool` trait with schema definitions

2. **Connector** (`crates/core/src/connector.rs`)
   - Implements `BaseConnector` trait from Strike48 SDK
   - `capabilities()` method returns all tool schemas
   - `execute()` method routes requests to appropriate tools

3. **Agent** (`apps/headless/src/main.rs`)
   - Creates tool registry with chosen backend (Python or Rust)
   - Wraps in `UbertoothConnector`
   - Runs via `ConnectorRunner` when `STRIKE48_URL` is set

## Running the Agent

### Production Mode (connects to Strike48)
```bash
export STRIKE48_URL="wss://your-server.com"
export TENANT_ID="your-tenant"
export AUTH_TOKEN="your-token"

cargo run --release --bin ubertooth-agent --features rust-backend
```

### Local Test Mode (no Strike48 server)
```bash
cargo run --bin ubertooth-agent --features rust-backend
```

## Backend Options

The agent supports two backends:

- **Rust USB Backend** (Phase 3) - 100-200x faster, 9 native tools
  - Native: device_*, configure_*, btle_scan, btle_follow, bt_specan
  - Falls back to Python for other tools
  - Enable with: `--backend rust` or `UBERTOOTH_BACKEND=rust`

- **Python Backend** (default) - All 36 tools via ubertooth-tools
  - Enable with: `--backend python` (default)

## Tool Discovery

When the agent connects to Strike48, it automatically registers all capabilities. AI agents can then:

1. Query available tools via Strike48 API
2. Execute tools by sending requests with tool name and parameters
3. Receive structured responses with success/error status

See `AGENT_PROMPT.md` for detailed tool documentation and usage examples.
