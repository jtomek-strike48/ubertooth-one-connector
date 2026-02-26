# Ubertooth One Connector — Product Requirements Document

> **Connector ID:** `ubertooth`
> **Version:** 0.1.0
> **Date:** 2026-02-26
> **Status:** Planning → Implementation

---

## 1. Overview

The Ubertooth One Connector integrates the [Ubertooth One](https://greatscottgadgets.com/ubertoothone/) Bluetooth monitoring device with [Prospector Studio](https://prospectorstudio.com) via the Strike48 connector SDK. It exposes the full capabilities of Ubertooth — Bluetooth Classic and BLE sniffing, spectrum analysis, packet injection, and more — as AI-invocable tools for wireless security research and assessment.

### Goals

- Give Prospector Studio's AI full control over an Ubertooth One device
- Expose 36 Bluetooth operations as well-organized, category-based tools
- Maintain persistent device state across tool calls
- Follow the established connector architecture from the yardstick-one-connector reference (PentestTool trait, ToolRegistry, BaseConnector, ToolEvent broadcasting)
- Support both passive reconnaissance and active attack operations (with authorization)
- Use the published Strike48 SDK crate for portability

### Hardware Support

The primary target is the Ubertooth One, but the architecture supports any compatible Ubertooth device:
- Ubertooth One (Product ID: 0x6002)
- Ubertooth Zero (Product ID: 0x6000)
- TC13Badge (Product ID: 0x0004)

---

## 2. Ubertooth One Capabilities

### Hardware Specifications

| Spec | Value |
|------|-------|
| Chipset | Custom Bluetooth baseband (CC2400 + LPC175x) |
| Frequency | 2.4 GHz ISM band (2402-2480 MHz) |
| Protocols | Bluetooth Classic (BR/EDR), Bluetooth Low Energy (BLE 4.0+) |
| Channels | 79 channels (1 MHz spacing) |
| RX Sensitivity | -94 dBm (typ) |
| TX Power | Up to +20 dBm (with PA) |
| Duplex | Half-duplex |
| Interface | USB 2.0 (Vendor ID: 0x1d50) |

### Key Capabilities

**Passive Operations:**
- BLE advertisement sniffing (channels 37, 38, 39)
- BLE connection following (using access address)
- Bluetooth Classic packet capture (promiscuous mode)
- Spectrum analysis (2402-2480 MHz with RSSI)
- AFH (Adaptive Frequency Hopping) analysis
- Device scanning and discovery

**Active Operations:**
- BLE packet injection
- BLE peripheral/slave mode
- Bluetooth jamming (highly regulated!)
- BLE MITM attacks
- Device spoofing

---

## 3. Tool Architecture

### Tool Categories (36 tools)

| Category | Count | Description | Auth Level |
|----------|-------|-------------|------------|
| **bt-device** | 4 | Device connection, status, session management | None |
| **bt-config** | 8 | Radio configuration and preset management | None/WARNING |
| **bt-recon** | 7 | Scanning, sniffing, spectrum analysis | None/WARNING |
| **bt-capture** | 5 | Capture storage and management | None |
| **bt-analysis** | 5 | Protocol analysis, fingerprinting, comparison | None |
| **bt-attack** | 5 | Active RF operations (injection, jamming, MITM) | REQUIRED |
| **bt-advanced** | 2 | Raw commands, firmware updates | WARNING |

**Total: 36 tools**

See [TOOL_SCHEMAS.md](TOOL_SCHEMAS.md) for complete tool specifications.

### Authorization Levels

**🟢 None (29 tools)** - Passive operations, no authorization required
- Read-only operations
- Passive scanning and sniffing
- Analysis and decoding
- Configuration (non-destructive)

**🟡 WARNING (4 tools)** - Recommended logging
- TX power adjustment
- Targeted connection monitoring
- Configuration changes that affect RF behavior
- Requires user acknowledgment in UI

**🔴 REQUIRED (5 tools)** - Strict authorization and audit logging
- Packet injection
- Jamming (illegal in most jurisdictions!)
- MITM attacks
- Device spoofing
- Must have explicit authorization token
- Full audit trail required

### Tool Naming Convention

All tools follow the pattern: `{category}_{operation}`

Examples:
- `device_connect` - Connect to Ubertooth
- `btle_scan` - BLE device scanning
- `bt_specan` - Spectrum analysis
- `capture_list` - List stored captures
- `bt_analyze` - Analyze captured packets

---

## 4. Backend Architecture

### Dual Backend Support

Following the yardstick-one-connector pattern, the connector supports two backends:

**Phase 1: Python Sidecar (Default)**
- Wraps existing `ubertooth-*` command-line tools
- Full feature set (all 36 tools)
- JSON-RPC bridge for command execution
- Proven, stable, battle-tested
- Requires: `ubertooth` package installed

**Phase 2: Native Rust USB (Future)**
- Direct libusb access via `rusb` crate
- Core operations only (estimated 7-10 tools)
- 100-200x faster than Python wrapper
- Zero-copy packet streaming
- No Python dependency

```rust
pub trait UbertoothBackendProvider: Send + Sync {
    async fn call(&self, method: &str, params: serde_json::Value)
        -> Result<serde_json::Value>;

    async fn is_alive(&self) -> bool;
    async fn restart(&self) -> Result<()>;
}
```

### Backend Selection

Via environment variable:

```bash
# Python wrapper (default, full features)
UBERTOOTH_BACKEND=python ubertooth-agent

# Native Rust (Phase 2, high performance)
UBERTOOTH_BACKEND=rust ubertooth-agent
```

---

## 5. Capture Storage

### File System Layout

```
~/.ubertooth/
  captures/
    cap-btle-abc123.pcap      # Raw packet capture (Wireshark-compatible)
    cap-btle-abc123.json      # Metadata (devices, stats, tags)
    cap-specan-def456.pcap
    cap-specan-def456.json
  configs/
    ble_adv_ch37.json         # Saved radio configurations
    bt_classic_ch10.json
```

### Capture Metadata Format

```json
{
  "capture_id": "cap-btle-abc123",
  "timestamp": "2026-02-26T15:30:00Z",
  "type": "btle_sniff",
  "channel": 37,
  "duration_sec": 30,
  "packet_count": 142,
  "file_size_bytes": 45320,
  "pcap_path": "/home/user/.ubertooth/captures/cap-btle-abc123.pcap",
  "tags": ["ble", "scan", "channel_37"],
  "description": "BLE advertisement scan on channel 37",
  "devices_found": [
    {
      "mac_address": "AA:BB:CC:DD:EE:FF",
      "device_name": "Fitbit Charge",
      "packet_count": 45,
      "rssi_avg": -65
    }
  ]
}
```

### Capture Management

- **Auto-save**: All data-producing tools automatically save captures
- **Pagination**: Use `capture_get` with `offset`/`limit` for large captures
- **Tagging**: Add tags and descriptions via `capture_tag`
- **Export**: Convert to PCAP/PCAPNG/JSON/CSV via `capture_export`
- **Persistence**: Survives session restarts

---

## 6. Configuration Management

### Configuration Presets

Radio configurations can be saved and loaded:

```json
{
  "config_name": "ble_adv_ch37",
  "description": "BLE advertising on channel 37",
  "created": "2026-02-26T10:00:00Z",
  "settings": {
    "channel": 37,
    "modulation": "BT_LOW_ENERGY",
    "power_level": 7,
    "paen": true,
    "hgm": false,
    "squelch": -90
  }
}
```

### Configuration Tools

- `bt_save_config` - Save current radio state
- `bt_load_config` - Restore saved configuration
- `config_list` - List all saved configs
- `config_delete` - Delete a configuration

---

## 7. Implementation Phases

### Phase 1: Python Wrapper (Week 1-2)

**Goal:** Ship working connector ASAP

**Deliverables:**
- ✅ 14 core tools operational
- ✅ AI can perform BLE scanning and basic analysis
- ✅ Captures stored and retrievable
- ✅ Device connection and status working

**Tools (Priority Order):**

**Week 1 (7 tools):**
1. `device_connect` ⭐⭐⭐
2. `device_disconnect` ⭐⭐⭐
3. `device_status` ⭐⭐⭐
4. `btle_scan` ⭐⭐⭐ (Most important!)
5. `bt_specan` ⭐⭐
6. `configure_channel` ⭐⭐
7. `capture_list` ⭐⭐

**Week 2 (7 more tools):**
8. `capture_get` ⭐⭐
9. `capture_delete` ⭐
10. `capture_tag` ⭐
11. `configure_modulation` ⭐⭐
12. `configure_power` ⭐⭐
13. `bt_analyze` ⭐⭐
14. `session_context` ⭐⭐⭐

**Technical Approach:**
- Python sidecar process (similar to yardstick-one-connector)
- Subprocess calls to `ubertooth-btle`, `ubertooth-specan`, etc.
- Parse text output to JSON
- Embedded Python script via `include_str!()`

---

### Phase 2: Full Feature Set (Week 3-6)

**Goal:** Complete all 36 tools with Python backend

**Week 3 - Advanced Recon (7 tools):**
15. `bt_scan`
16. `bt_follow`
17. `afh_analyze`
18. `bt_discover`
19. `btle_follow`
20. `configure_squelch`
21. `configure_leds`

**Week 4 - Config Management (4 tools):**
22. `bt_save_config`
23. `bt_load_config`
24. `config_list`
25. `config_delete`

**Week 5 - Analysis Tools (5 tools):**
26. `bt_compare`
27. `bt_decode`
28. `bt_fingerprint`
29. `pcap_merge`
30. `capture_export`

**Week 6 - Attack Operations + Advanced (6 tools):**
31. `btle_inject` ⚠️
32. `bt_jam` ⚠️⚠️
33. `btle_slave` ⚠️
34. `btle_mitm` ⚠️⚠️
35. `bt_spoof` ⚠️
36. `ubertooth_raw`

**Note:** Attack tools require explicit authorization and audit logging.

**Additional Week 6 Tasks:**
- Implement authorization checks
- Add audit logging
- Comprehensive testing
- Documentation

---

### Phase 3: Native Rust USB (Week 7-12, Optional)

**Goal:** High-performance native implementation for core operations

**Scope:** Estimated 7-10 tools
- Device connection/status
- BLE sniffing (streaming)
- Spectrum analysis (real-time)
- Basic configuration
- Packet injection

**Performance Target:** 100-200x faster than Python for supported operations

**Deliverables:**
- `crates/usb/` - Native USB implementation
- Backend selection via `UBERTOOTH_BACKEND` env var
- Zero regressions vs Python backend
- Performance benchmarks

**Technical Approach:**
- Direct libusb access via `rusb` crate
- Async packet streaming with tokio
- Zero-copy buffer management
- See [LIBUBERTOOTH_RUST_FEASIBILITY.md](LIBUBERTOOTH_RUST_FEASIBILITY.md)

---

## 8. Crate Structure

Following the yardstick-one-connector pattern:

```
ubertooth-one-connector/
├── Cargo.toml                    # Workspace definition
├── justfile                      # Build commands
├── README.md
├── PRD.md                        # This document
├── TOOL_SCHEMAS.md              # Complete tool specifications
├── LIBUBERTOOTH_RUST_FEASIBILITY.md
│
├── crates/
│   ├── core/
│   │   ├── src/
│   │   │   ├── connector.rs     # UbertoothConnector (BaseConnector)
│   │   │   ├── error.rs         # Error types
│   │   │   ├── events.rs        # ToolEvent broadcasting
│   │   │   ├── logging.rs       # Tracing setup
│   │   │   ├── tools.rs         # PentestTool trait
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── platform/
│   │   ├── src/
│   │   │   ├── sidecar.rs       # SidecarManager (Python wrapper)
│   │   │   ├── rust_usb.rs      # RustUsbBackend (Phase 3)
│   │   │   ├── backend.rs       # UbertoothBackendProvider trait
│   │   │   ├── capture_store.rs # ~/.ubertooth/ storage
│   │   │   ├── system_info.rs   # Platform utilities
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── usb/                      # Phase 3 only
│   │   ├── src/
│   │   │   ├── device.rs        # UbertoothDevice struct
│   │   │   ├── commands.rs      # 73 USB commands
│   │   │   ├── protocol.rs      # Packet structures
│   │   │   ├── error.rs
│   │   │   ├── constants.rs     # USB IDs, endpoints
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── tools/
│   │   ├── src/
│   │   │   ├── lib.rs           # create_tool_registry()
│   │   │   │
│   │   │   ├── device_connect.rs
│   │   │   ├── device_disconnect.rs
│   │   │   ├── device_status.rs
│   │   │   ├── session_context.rs
│   │   │   │
│   │   │   ├── configure_channel.rs
│   │   │   ├── configure_modulation.rs
│   │   │   ├── configure_power.rs
│   │   │   ├── configure_squelch.rs
│   │   │   ├── configure_leds.rs
│   │   │   ├── bt_save_config.rs
│   │   │   ├── bt_load_config.rs
│   │   │   ├── config_list.rs
│   │   │   ├── config_delete.rs
│   │   │   │
│   │   │   ├── btle_scan.rs
│   │   │   ├── bt_scan.rs
│   │   │   ├── bt_specan.rs
│   │   │   ├── bt_follow.rs
│   │   │   ├── afh_analyze.rs
│   │   │   ├── bt_discover.rs
│   │   │   ├── btle_follow.rs
│   │   │   │
│   │   │   ├── capture_list.rs
│   │   │   ├── capture_get.rs
│   │   │   ├── capture_delete.rs
│   │   │   ├── capture_tag.rs
│   │   │   ├── capture_export.rs
│   │   │   │
│   │   │   ├── bt_analyze.rs
│   │   │   ├── bt_compare.rs
│   │   │   ├── bt_decode.rs
│   │   │   ├── bt_fingerprint.rs
│   │   │   ├── pcap_merge.rs
│   │   │   │
│   │   │   ├── btle_inject.rs
│   │   │   ├── bt_jam.rs
│   │   │   ├── btle_slave.rs
│   │   │   ├── btle_mitm.rs
│   │   │   ├── bt_spoof.rs
│   │   │   │
│   │   │   ├── ubertooth_raw.rs
│   │   │   └── firmware_update.rs
│   │   │
│   │   └── Cargo.toml
│   │
│   └── sidecar/
│       ├── ubertooth_bridge.py  # Python JSON-RPC bridge
│       └── requirements.txt     # Python dependencies (if any)
│
├── apps/
│   ├── headless/
│   │   ├── src/
│   │   │   └── main.rs          # Production agent
│   │   └── Cargo.toml
│   │
│   └── cli/
│       ├── src/
│       │   └── main.rs          # Standalone CLI
│       └── Cargo.toml
│
├── docs/
│   ├── quickstart.md
│   ├── tool-reference.md
│   ├── AUTHORIZATION.md
│   ├── troubleshooting.md
│   ├── guide/
│   │   ├── getting-started.md
│   │   ├── ble-scanning.md
│   │   ├── spectrum-analysis.md
│   │   ├── packet-analysis.md
│   │   └── attack-operations.md
│   └── examples/
│       ├── ble-device-scan.md
│       ├── bt-connection-follow.md
│       └── spectrum-survey.md
│
└── .github/
    └── workflows/
        └── ci.yml               # GitHub Actions CI
```

---

## 9. Dependencies

### Workspace Dependencies

```toml
[workspace.dependencies]
# Async
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Error handling
thiserror = "1"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Platform-specific
sysinfo = "0.32"

# Temp files
tempfile = "3"

# UUID
uuid = { version = "1", features = ["v4"] }

# Home directory
dirs = "5"

# CLI (apps only)
clap = { version = "4", features = ["derive"] }

# USB (Phase 3 only)
rusb = "0.9"

# PCAP
pcap-file = "2"

# Strike48 Connector SDK
strike48-connector = { path = "../sdk-rs/crates/connector" }

# Internal crates
ubertooth-core = { path = "crates/core" }
ubertooth-platform = { path = "crates/platform", default-features = false }
ubertooth-tools = { path = "crates/tools", default-features = false }
ubertooth-usb = { path = "crates/usb" }  # Phase 3
```

### System Dependencies

**Required:**
- libusb-1.0 (for Rust USB access)
- ubertooth-tools (for Python backend)

**Installation:**

```bash
# Debian/Ubuntu
sudo apt-get install ubertooth libusb-1.0-0-dev

# Arch Linux
sudo pacman -S ubertooth libusb

# macOS
brew install ubertooth libusb

# Or build from source:
git clone https://github.com/greatscottgadgets/ubertooth.git
cd ubertooth/host
mkdir build && cd build
cmake ..
make && sudo make install
```

---

## 10. Build & Test Strategy

### Just Commands

```bash
just check          # cargo check --workspace
just lint           # cargo clippy --workspace -- -D warnings
just fmt            # cargo fmt --all
just fmt-check      # cargo fmt --all -- --check
just test           # cargo test --workspace
just test-python    # Python sidecar unit tests
just build          # cargo build --workspace
just build-release  # cargo build --release
just run            # cargo run --package ubertooth-agent
just run-release    # cargo run --release --package ubertooth-agent
just ci             # check + fmt-check + lint + test + test-python
just install-udev   # Install udev rules for Ubertooth One
just clean          # cargo clean
```

### Testing Layers

| Layer | Command | Hardware? | Coverage |
|-------|---------|-----------|----------|
| Rust unit tests | `just test` | No | Core logic, data structures |
| Python unit tests | `just test-python` | No | Sidecar command parsing |
| Integration tests | `just test` | No | Tool registry, mocked backend |
| Hardware smoke | Manual | Yes | Real Ubertooth One required |

### Hardware Test Setup

```bash
# Install udev rules
just install-udev

# Replug device

# Verify device detected
lsusb | grep "1d50:6002"

# Run smoke test
UBERTOOTH_BACKEND=python cargo run --package ubertooth-agent
```

---

## 11. Security & Authorization

### Authorization Architecture

**Three-tier authorization model:**

1. **Tool-level authorization** (declared in schema)
   ```rust
   impl PentestTool for BtJamTool {
       fn requires_authorization(&self) -> bool {
           true
       }

       fn authorization_category(&self) -> &str {
           "bt-attack"
       }
   }
   ```

2. **Runtime authorization check** (enforced in executor)
   ```rust
   async fn execute(&self, params: Value, ctx: ToolContext) -> Result<ToolResult> {
       if self.requires_authorization() && !ctx.has_authorization(self.authorization_category()) {
           return Err(ToolError::Unauthorized {
               tool: self.name(),
               required: self.authorization_category(),
           });
       }
       // ... proceed with execution
   }
   ```

3. **Audit logging** (automatic for authorized tools)
   ```rust
   ctx.audit_log(AuditEvent {
       timestamp: Utc::now(),
       tool: self.name(),
       user_id: ctx.user_id,
       parameters: serde_json::to_value(&params)?,
       result: "success",
       authorization_token: ctx.authorization_token,
   });
   ```

### Audit Log Format

```json
{
  "timestamp": "2026-02-26T15:30:45Z",
  "tool": "bt_jam",
  "user_id": "user-123",
  "tenant_id": "tenant-456",
  "session_id": "session-789",
  "parameters": {
    "jam_mode": "continuous",
    "channel": 37,
    "duration_sec": 10
  },
  "result": "success",
  "authorization_token": "auth-token-abc",
  "client_ip": "192.168.1.100"
}
```

### Authorization Matrix

| Level | Tools | Requirements |
|-------|-------|--------------|
| 🟢 None | 29 | No authorization needed |
| 🟡 WARNING | 4 | User acknowledgment in UI |
| 🔴 REQUIRED | 5 | Explicit authorization token + audit log |

### Regulatory Compliance

**⚠️ CRITICAL:** Bluetooth jamming is **illegal in most countries** without proper authorization.

Tools that perform jamming must:
1. Require explicit authorization
2. Log all operations
3. Display clear warnings
4. Check for regulatory flags (future enhancement)

---

## 12. Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum UbertoothError {
    #[error("Device not found")]
    DeviceNotFound,

    #[error("Device already connected")]
    AlreadyConnected,

    #[error("No device connected")]
    NotConnected,

    #[error("USB error: {0}")]
    UsbError(String),

    #[error("Firmware too old: {current}, required: {required}")]
    FirmwareTooOld { current: String, required: String },

    #[error("Permission denied - check udev rules")]
    PermissionDenied,

    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Capture not found: {0}")]
    CaptureNotFound(String),

    #[error("Authorization required for {tool}")]
    Unauthorized { tool: String, required: String },

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}
```

### Error Recovery

**Device disconnection:**
- Automatic reconnection attempts (up to 3 retries)
- Clear error messages to user
- State cleanup

**USB errors:**
- Detect permission issues → suggest udev rules
- Detect device unplugged → graceful shutdown
- Detect firmware incompatibility → suggest update

**Backend crashes:**
- Python sidecar auto-restart on next command
- Device reconnection after sidecar restart
- No state loss (captures/configs on disk)

---

## 13. Performance Requirements

### Python Backend (Phase 1-2)

| Operation | Target Latency | Notes |
|-----------|----------------|-------|
| device_connect | < 1 second | Subprocess spawn |
| device_status | < 100ms | Quick status query |
| btle_scan (30s) | ~30.5s | Subprocess + parsing overhead |
| capture_list | < 50ms | Disk metadata read |
| bt_analyze | < 500ms | Parse + analysis |

### Native Rust Backend (Phase 3)

| Operation | Target Latency | Speedup |
|-----------|----------------|---------|
| device_connect | < 10ms | 100x |
| device_status | < 1ms | 100x |
| btle_scan (30s) | ~30.001s | Minimal overhead |
| Packet streaming | Real-time | N/A (Python can't do this) |

---

## 14. Documentation Requirements

### User Documentation

**Getting Started:**
- Hardware setup guide
- Software installation
- First scan tutorial
- Troubleshooting common issues

**Tool Reference:**
- Complete tool catalog (generated from schemas)
- Input/output examples
- Error cases
- Authorization requirements

**Guides:**
- BLE device scanning
- Spectrum analysis
- Bluetooth Classic monitoring
- Protocol analysis
- Attack operations (with warnings)

### Developer Documentation

**Architecture:**
- Crate structure
- Backend abstraction
- Tool implementation pattern
- Testing strategy

**Contributing:**
- Code style
- Adding new tools
- Backend implementation
- Pull request process

---

## 15. Success Metrics

### Phase 1 (Week 2)

- ✅ 14 tools operational
- ✅ AI can scan BLE devices
- ✅ Captures stored and retrievable
- ✅ Zero crashes on happy path
- ✅ Tests passing (unit + integration)

### Phase 2 (Week 6)

- ✅ All 36 tools operational
- ✅ Full feature parity with ubertooth-tools
- ✅ Authorization enforced for attack tools
- ✅ Audit logging implemented
- ✅ Test coverage > 80%
- ✅ Documentation complete

### Phase 3 (Week 12, Optional)

- ✅ Native Rust backend for core operations
- ✅ 100x+ performance improvement demonstrated
- ✅ Backend selection working
- ✅ Zero regressions vs Python
- ✅ Performance benchmarks published

---

## 16. Open Questions

### For Resolution During Implementation

1. **libbtbb dependency:**
   - Use FFI bindings to C library?
   - Reimplement packet parsing in Rust?
   - Python-only for analysis tools?

2. **PCAP format:**
   - Store PCAP only?
   - Parse to JSON for AI?
   - Both (hybrid approach)?

3. **Device pooling:**
   - Support multiple Ubertooth devices simultaneously?
   - Single device per connector instance?

4. **Firmware update:**
   - Include in connector?
   - Separate manual process?
   - Version check and warning only?

5. **Authorization storage:**
   - Tokens in environment variables?
   - Config file?
   - Runtime-only (no persistence)?

---

## 17. Risk Mitigation

### Risk: Python wrapper performance inadequate

**Probability:** Medium
**Impact:** Medium
**Mitigation:** Implement native Rust backend (Phase 3)

### Risk: libbtbb dependency complicates Rust implementation

**Probability:** Medium
**Impact:** Medium
**Mitigation:** Use FFI bindings initially, pure Rust later

### Risk: Firmware compatibility issues

**Probability:** Low
**Impact:** High
**Mitigation:** Version check in device_connect, clear error messages

### Risk: Authorization bypass

**Probability:** Low
**Impact:** Critical
**Mitigation:** Defense-in-depth (tool + connector + audit logging)

### Risk: Regulatory compliance issues

**Probability:** Low
**Impact:** Critical
**Mitigation:** Clear warnings, authorization requirements, documentation

---

## 18. Future Enhancements

**Not in scope for v0.1.0, but possible future additions:**

- Multi-device support (parallel operations)
- Real-time streaming to Studio (WebSocket)
- Cloud capture storage
- Machine learning-based device fingerprinting
- Automated protocol fuzzing
- Integration with Wireshark for live analysis
- Mobile app support (Android/iOS)
- BLE 5.0+ features (long range, 2M PHY)
- Bluetooth Classic baseband analysis (advanced)

---

## 19. Conclusion

The Ubertooth One connector brings powerful Bluetooth monitoring and security testing capabilities to Prospector Studio. By following the proven yardstick-one-connector architecture and implementing in phases, we can:

1. **Ship fast** - 14 working tools in 2 weeks
2. **Scale up** - 36 tools in 6 weeks
3. **Optimize** - Native Rust for performance in 12 weeks

The dual backend architecture provides flexibility: Python wrapper for fast development and full features, native Rust for performance-critical operations.

**Status: Ready for implementation** ✅

---

## Appendix A: Tool Summary

See [TOOL_SCHEMAS.md](TOOL_SCHEMAS.md) for complete specifications.

**Phase 1 (Week 1-2): 14 tools**
- device_connect, device_disconnect, device_status, session_context
- btle_scan, bt_specan
- configure_channel, configure_modulation, configure_power
- capture_list, capture_get, capture_delete, capture_tag
- bt_analyze

**Phase 2 (Week 3-6): 22 additional tools**
- bt_scan, bt_follow, afh_analyze, bt_discover, btle_follow
- configure_squelch, configure_leds
- bt_save_config, bt_load_config, config_list, config_delete
- bt_compare, bt_decode, bt_fingerprint, pcap_merge, capture_export
- btle_inject, bt_jam, btle_slave, btle_mitm, bt_spoof
- ubertooth_raw, firmware_update

---

## Appendix B: GitHub Issue Workflow

### Issue Template

```markdown
## Tool: [TOOL_NAME]

**Category:** [bt-device/bt-config/bt-recon/bt-capture/bt-analysis/bt-attack/bt-advanced]

**Priority:** [P0/P1/P2]

**Phase:** [phase-1/phase-2/phase-3]

**Estimated Time:** [X hours]

### Description
[Brief description]

### Input Schema
\`\`\`json
[Input JSON]
\`\`\`

### Output Schema
\`\`\`json
[Output JSON]
\`\`\`

### Backend Implementation
- [ ] Python wrapper
- [ ] Native Rust (optional)

### Test Cases
- [ ] Happy path
- [ ] Error cases
- [ ] Edge cases

### Authorization
[None / WARNING / REQUIRED]

### Dependencies
- Depends on: #[issue]
- Blocks: #[issue]

### Acceptance Criteria
- [ ] Input validation works
- [ ] Output matches schema
- [ ] Error cases handled
- [ ] Unit tests written
- [ ] Integration test passes
- [ ] Documentation updated
```

### Issue Labels

**Priority:**
- `P0` - Critical (device_connect, btle_scan, device_status)
- `P1` - Important (most tools)
- `P2` - Nice-to-have (advanced features)

**Phase:**
- `phase-1` - Week 1-2 (14 tools)
- `phase-2` - Week 3-6 (22 tools)
- `phase-3` - Week 7-12 (Rust USB)

**Category:**
- `bt-device`
- `bt-config`
- `bt-recon`
- `bt-capture`
- `bt-analysis`
- `bt-attack`
- `bt-advanced`

**Backend:**
- `backend-python`
- `backend-rust`

**Security:**
- `security` - Requires authorization

**Type:**
- `tool-implementation`
- `infrastructure`
- `documentation`
- `testing`

---

**Document Version:** 1.0
**Last Updated:** 2026-02-26
**Status:** Approved for Implementation ✅
