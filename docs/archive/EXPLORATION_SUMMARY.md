# Ubertooth One Connector - Exploration Summary

**Date:** 2026-02-26
**Author:** Claude (Sonnet 4.5)
**Purpose:** Deep dive into libubertooth C API feasibility and tool schema design

---

## Executive Summary

✅ **Native Rust implementation is HIGHLY FEASIBLE and RECOMMENDED**

The libubertooth C API is clean, well-documented, and maps perfectly to Rust idioms using the `rusb` crate. A hybrid approach (Python wrapper first, then native Rust) provides the best of both worlds:

- **Week 2:** Ship 14 working tools via Python wrapper
- **Week 6:** Ship all 36 tools with full feature parity
- **Week 12:** Native Rust core delivering 100-200x performance for critical operations

---

## Key Deliverables

### 1. Rust Feasibility Analysis

**Document:** [LIBUBERTOOTH_RUST_FEASIBILITY.md](LIBUBERTOOTH_RUST_FEASIBILITY.md)

**Findings:**
- ✅ USB communication: Direct mapping via `rusb` crate (Rust libusb 1.0 bindings)
- ✅ Command interface: 73 USB commands, trivial to wrap in Rust enums
- ✅ Data structures: `#[repr(C, packed)]` gives byte-perfect compatibility
- ✅ Bulk transfers: Can use tokio async or blocking, both work well
- ✅ Zero overhead: Rust compiles to identical machine code as C

**Code Examples Provided:**
- Device enumeration and connection
- Control command execution
- Bulk transfer packet reception
- USB packet parsing with type safety
- Error handling with Result types

**Challenges Identified:**
1. ✅ Callback → async: Solved with tokio channels
2. ✅ FIFO management: Solved with `VecDeque<UsbPacketRx>`
3. ⚠️ libbtbb dependency: Can use FFI or rewrite in Rust

**Performance Estimates:**
- Python wrapper: ~50ms latency per command (subprocess overhead)
- Rust native: ~0.5-1ms latency (direct USB)
- **Speedup: 50-100x for single operations, 100-200x for streaming**

---

### 2. Comprehensive Tool Schemas

**Document:** [TOOL_SCHEMAS.md](TOOL_SCHEMAS.md)

**Total Tools Designed: 36**

| Category | Tools | Description |
|----------|-------|-------------|
| bt-device | 4 | Connection, status, session context |
| bt-config | 8 | Channel, modulation, power, presets |
| bt-recon | 7 | BLE/BT scanning, spectrum analysis, following |
| bt-capture | 5 | Capture list, get, delete, tag, export |
| bt-analysis | 5 | Decode, compare, fingerprint, merge |
| bt-attack | 5 | Inject, jam, MITM, slave, spoof |
| bt-advanced | 2 | Raw commands, firmware update |

**Each tool includes:**
- ✅ Input/output JSON schemas
- ✅ Error cases and handling
- ✅ Backend implementation notes (Python vs Rust)
- ✅ Authorization level (None, WARNING, REQUIRED)
- ✅ Example use cases

**Ready for GitHub Issues:**
All 36 schemas can be directly converted to GitHub issues with the provided template.

---

## Key Technical Insights

### libubertooth Architecture

```
┌─────────────────────────────────────┐
│     Application (ubertooth-btle)    │
├─────────────────────────────────────┤
│  libubertooth (C API)               │
│  - Device management                │
│  - Control commands (73 total)      │
│  - Bulk transfer callbacks          │
│  - PCAP writing                     │
├─────────────────────────────────────┤
│  libusb 1.0 (USB abstraction)       │
├─────────────────────────────────────┤
│  Linux kernel (USB drivers)         │
└─────────────────────────────────────┘
```

**Rust Equivalent:**
```
┌─────────────────────────────────────┐
│     ubertooth-agent (Rust)          │
├─────────────────────────────────────┤
│  ubertooth-core (Rust)              │
│  - UbertoothDevice struct           │
│  - Command enum + implementations   │
│  - Async packet streaming           │
│  - PCAP writing (pcap-file crate)   │
├─────────────────────────────────────┤
│  rusb (Rust libusb 1.0 bindings)    │
├─────────────────────────────────────┤
│  Linux kernel (USB drivers)         │
└─────────────────────────────────────┘
```

### USB Packet Format (64 bytes)

The core data structure is beautifully simple:

```rust
#[repr(C, packed)]
pub struct UsbPacketRx {
    pub pkt_type: u8,        // BR/LE/SPECAN/etc
    pub status: u8,          // DMA overflow, FIFO errors
    pub channel: u8,         // BT channel 0-78
    pub clkn_high: u8,
    pub clk100ns: u32,       // Timestamp
    pub rssi_max: i8,        // Signal strength stats
    pub rssi_min: i8,
    pub rssi_avg: i8,
    pub rssi_count: u8,
    pub reserved: [u8; 2],
    pub data: [u8; 50],      // Actual packet payload
}
```

This maps 1:1 from C to Rust with zero overhead. The Rust compiler will generate identical memory layout.

### Command Execution Pattern

**C:**
```c
int cmd_set_channel(struct libusb_device_handle* devh, u16 channel) {
    return libusb_control_transfer(devh, 0x40, 12, channel, 0, NULL, 0, TIMEOUT);
}
```

**Rust:**
```rust
fn set_channel(&self, channel: u16) -> Result<(), UbertoothError> {
    self.handle.write_control(
        0x40,                           // bmRequestType
        UbertoothCommand::SetChannel,   // 12
        channel,                         // wValue
        0,                               // wIndex
        &[],                             // no data
        TIMEOUT,
    )?;
    Ok(())
}
```

Virtually identical, but with type safety and error handling baked in.

---

## Comparison: Yardstick One vs Ubertooth One

### Similarities (Good news - template is highly relevant!)

| Aspect | Yardstick One | Ubertooth One |
|--------|---------------|---------------|
| Architecture | Dual backend (Python + Rust) | ✅ Same pattern |
| Tool count | 31 tools | 36 tools (similar complexity) |
| Categories | 8 categories | 7 categories |
| Capture storage | ~/.rfcat/captures/ | ~/.ubertooth/captures/ |
| Config presets | ✅ Supported | ✅ Supported |
| Authorization levels | 3 levels (None/Warning/Required) | ✅ Same 3 levels |
| PCAP output | ✅ Yes | ✅ Yes |
| Strike48 SDK | ✅ Used | ✅ Will use |

### Differences (Need to adapt)

| Aspect | Yardstick One | Ubertooth One |
|--------|---------------|---------------|
| **Protocol** | Sub-1 GHz RF (300-928 MHz) | Bluetooth/BLE (2.4 GHz) |
| **Primary library** | rfcat (Python) | libubertooth (C) |
| **USB interface** | Serial-like (CDC-ACM) | Control + bulk transfers |
| **Packet format** | Generic RF data | Bluetooth-specific (L2CAP, ATT, etc.) |
| **Main use case** | ISM band hacking | BLE sniffing, BT monitoring |
| **Attack surface** | Garage doors, key fobs | BLE devices, Bluetooth peripherals |
| **Regulatory** | Less regulated (sub-GHz) | More regulated (BT jamming illegal) |

---

## Recommended Implementation Strategy

### Phase 1: Python Wrapper (Week 1-2)

**Goal:** Ship working connector ASAP

**Approach:**
- Wrap existing `ubertooth-*` command-line tools
- Python sidecar process (like yardstick-one-connector)
- JSON-RPC bridge for command execution
- Parse text output into structured JSON

**Tools to implement (Priority order):**

**Week 1 (7 tools):**
1. device_connect ⭐⭐⭐
2. device_disconnect ⭐⭐⭐
3. device_status ⭐⭐⭐
4. btle_scan ⭐⭐⭐ (MOST IMPORTANT)
5. bt_specan ⭐⭐
6. configure_channel ⭐⭐
7. capture_list ⭐⭐

**Week 2 (7 more tools):**
8. capture_get ⭐⭐
9. capture_delete ⭐
10. capture_tag ⭐
11. configure_modulation ⭐⭐
12. configure_power ⭐⭐
13. bt_analyze ⭐⭐
14. session_context ⭐⭐⭐

**Deliverable:** 14 working tools, AI can do basic BLE scanning and analysis

---

### Phase 2: Full Feature Set (Week 3-6)

**Goal:** Complete all 36 tools with Python backend

**Week 3 (Advanced recon):**
15. bt_scan
16. bt_follow
17. afh_analyze
18. bt_discover
19. btle_follow
20. configure_squelch
21. configure_leds

**Week 4 (Config management):**
22. bt_save_config
23. bt_load_config
24. config_list
25. config_delete

**Week 5 (Analysis tools):**
26. bt_compare
27. bt_decode
28. bt_fingerprint
29. pcap_merge
30. capture_export

**Week 6 (Attack operations + advanced):**
31. btle_inject ⚠️
32. bt_jam ⚠️⚠️
33. btle_slave ⚠️
34. btle_mitm ⚠️⚠️
35. bt_spoof ⚠️
36. ubertooth_raw
37. firmware_update

**Deliverable:** All 36 tools working, full feature parity with yardstick-one-connector

---

### Phase 3: Native Rust Core (Week 7-12)

**Goal:** High-performance native implementation

**Week 7-8: Core USB layer**
```
crates/usb/
  src/
    device.rs       # UbertoothDevice struct
    commands.rs     # 73 USB commands
    protocol.rs     # Packet structures
    error.rs
    constants.rs
```

**Week 9-10: Packet processing**
- Async bulk transfers with tokio
- Ring buffer for packet queue
- RSSI statistics
- Error recovery

**Week 11-12: Critical operations**
- BLE sniffing (streaming, zero-copy)
- Spectrum analysis (real-time)
- Device control (low latency)

**Backend selection:**
```bash
# High-performance streaming (100-200x faster)
UBERTOOTH_BACKEND=rust ubertooth-agent

# Full feature set (all 36 tools)
UBERTOOTH_BACKEND=python ubertooth-agent
```

**Deliverable:** Native Rust backend for core operations, dramatic performance improvement

---

## Performance Projections

### Python Wrapper Performance

| Operation | Latency | Throughput | Notes |
|-----------|---------|------------|-------|
| device_connect | ~500ms | - | Spawn subprocess |
| device_status | ~50ms | 20 ops/sec | Parse CLI output |
| btle_scan (30s) | 30.5s | - | Subprocess + parsing |
| Packet processing | N/A | File-based | Write PCAP, read back |

**Bottlenecks:**
- Subprocess spawn overhead
- Text parsing (stdout)
- No streaming (must wait for tool to finish)

### Native Rust Performance

| Operation | Latency | Throughput | Speedup |
|-----------|---------|------------|---------|
| device_connect | ~5ms | - | **100x faster** |
| device_status | ~0.5ms | 2,000 ops/sec | **100x faster** |
| btle_scan (30s) | 30.001s | - | **500x lower overhead** |
| Packet processing | Real-time | 10,000+ pkt/sec | **Streaming!** |

**Advantages:**
- Direct USB access (no subprocess)
- Zero-copy packet processing
- Real-time streaming
- Lower memory footprint (4MB vs 50MB)

---

## Security & Authorization

### Authorization Matrix

| Level | Tools | Description | Audit Required |
|-------|-------|-------------|----------------|
| 🟢 None | 29 | Passive operations only | No |
| 🟡 WARNING | 4 | Config changes, targeted monitoring | Log recommended |
| 🔴 REQUIRED | 5 | Active RF operations | **Yes - full audit trail** |

### Attack Tool Authorization

**Must implement:**
1. ✅ Explicit authorization flags in tool schema
2. ✅ Runtime authorization check before execution
3. ✅ Audit logging (timestamp, user, tool, parameters, result)
4. ✅ Rate limiting for attack operations
5. ✅ Geographic/regulatory compliance checks (jamming is illegal!)

**Example authorization flow:**
```rust
impl BtJamTool {
    async fn execute(&self, params: BtJamParams, ctx: ToolContext) -> Result<ToolResult> {
        // Check authorization
        if !ctx.has_authorization("bt-attack") {
            return Err(ToolError::Unauthorized {
                message: "bt_jam requires explicit authorization".into(),
                required_permission: "bt-attack".into(),
            });
        }

        // Log the operation
        ctx.audit_log(AuditEvent {
            tool: "bt_jam",
            user: ctx.user_id,
            timestamp: Utc::now(),
            parameters: serde_json::to_value(params)?,
            authorized_by: ctx.authorization_token,
        });

        // Proceed with jamming...
    }
}
```

---

## Tooling & Infrastructure

### Crate Structure (Following yardstick-one-connector pattern)

```
ubertooth-one-connector/
├── Cargo.toml (workspace)
├── crates/
│   ├── core/
│   │   ├── src/
│   │   │   ├── connector.rs      # UbertoothConnector (BaseConnector impl)
│   │   │   ├── error.rs          # Error types
│   │   │   ├── events.rs         # ToolEvent broadcasting
│   │   │   ├── logging.rs        # Tracing setup
│   │   │   └── tools.rs          # PentestTool trait
│   │   └── Cargo.toml
│   ├── platform/
│   │   ├── src/
│   │   │   ├── sidecar.rs        # SidecarManager (Python wrapper)
│   │   │   ├── rust_usb.rs       # RustUsbBackend (native)
│   │   │   ├── backend.rs        # RfBackendProvider trait
│   │   │   ├── capture_store.rs  # ~/.ubertooth/ storage
│   │   │   └── system_info.rs    # Platform utilities
│   │   └── Cargo.toml
│   ├── usb/
│   │   ├── src/
│   │   │   ├── device.rs         # UbertoothDevice
│   │   │   ├── commands.rs       # 73 USB commands
│   │   │   ├── protocol.rs       # Packet structures
│   │   │   ├── error.rs
│   │   │   └── constants.rs      # USB IDs, endpoints
│   │   └── Cargo.toml
│   └── tools/
│       ├── src/
│       │   ├── lib.rs            # create_tool_registry()
│       │   ├── device_connect.rs
│       │   ├── device_status.rs
│       │   ├── btle_scan.rs
│       │   ├── bt_specan.rs
│       │   └── ... (36 total tools)
│       └── Cargo.toml
├── apps/
│   ├── headless/
│   │   ├── src/main.rs           # Production agent
│   │   └── Cargo.toml
│   └── cli/
│       ├── src/main.rs           # Standalone CLI
│       └── Cargo.toml
├── crates/sidecar/
│   └── ubertooth_bridge.py       # Python JSON-RPC bridge
├── docs/
│   ├── tool-reference.md
│   ├── quickstart.md
│   ├── AUTHORIZATION.md
│   └── examples/
│       ├── ble-scan.md
│       ├── bt-follow.md
│       └── spectrum-analysis.md
├── justfile                      # Build commands
├── README.md
├── PRD.md
├── CLAUDE.md
├── TOOL_SCHEMAS.md
└── LIBUBERTOOTH_RUST_FEASIBILITY.md
```

### Key Dependencies

```toml
[workspace.dependencies]
# Async
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# USB (Rust backend only)
rusb = "0.9"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# PCAP
pcap-file = "2"

# Error handling
thiserror = "1"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Strike48 SDK
strike48-connector = { path = "../sdk-rs/crates/connector" }

# Internal crates
ubertooth-core = { path = "crates/core" }
ubertooth-platform = { path = "crates/platform" }
ubertooth-tools = { path = "crates/tools" }
ubertooth-usb = { path = "crates/usb" }
```

---

## Testing Strategy

### Unit Tests
- Mock libusb device handle
- Test packet parsing
- Validate command encoding

### Integration Tests
- Python sidecar spawn/shutdown
- Command execution end-to-end
- Capture storage

### Hardware Tests (optional)
- Requires real Ubertooth One
- Smoke tests: connect, status, scan
- Capture validation

```bash
just test           # Unit + integration (no hardware)
just test-hardware  # Requires Ubertooth One connected
```

---

## Next Steps

### Immediate (This Session)

1. ✅ Review feasibility analysis
2. ✅ Review tool schemas
3. ⏳ Get your feedback and questions
4. ⏳ Refine any schemas based on your input
5. ⏳ Create PRD document

### Week 1 Tasks

1. Initialize Rust workspace
2. Set up Strike48 SDK dependency
3. Create basic crate structure
4. Implement Python sidecar manager
5. Implement first 7 tools (device_connect → capture_list)
6. Hardware smoke test with real Ubertooth One

### GitHub Issues

Ready to create 36 GitHub issues (one per tool) using the template in TOOL_SCHEMAS.md.

**Issue labels:**
- `P0` - Critical (device_connect, btle_scan, device_status)
- `P1` - Important (all recon + analysis tools)
- `P2` - Nice-to-have (advanced features)
- `phase-1` - Python wrapper
- `phase-2` - Full feature set
- `phase-3` - Native Rust
- `security` - Requires authorization
- `backend-python` - Python implementation
- `backend-rust` - Rust implementation

---

## Questions & Considerations

### For Discussion

1. **Authorization model:** Should we implement authorization at:
   - Connector level (global permissions)?
   - Per-tool level (fine-grained)?
   - Per-operation level (e.g., allow scan but not follow)?

2. **PCAP format:** Should we:
   - Store raw PCAP only (Wireshark-compatible)?
   - Also parse to JSON for AI consumption?
   - Use PCAPNG for better metadata?

3. **Python dependencies:** Should we:
   - Require system ubertooth-tools installation?
   - Bundle binaries in the connector?
   - Use Python bindings (if they exist)?

4. **Rust backend priority:** Should we:
   - Start Rust backend in parallel with Python (more work)?
   - Wait until Python is stable (safer)?
   - Only do Rust if performance becomes an issue?

5. **Device pooling:** If multiple Ubertooth devices are connected:
   - Support device selection via index?
   - Allow parallel operations on multiple devices?
   - Single device at a time (simpler)?

---

## Risks & Mitigations

### Risk 1: Python wrapper performance

**Impact:** Medium
**Probability:** High
**Mitigation:** Native Rust backend (Phase 3)

### Risk 2: libbtbb dependency for packet parsing

**Impact:** Medium (affects analysis tools)
**Probability:** Medium
**Mitigation:** FFI bindings or pure Rust reimplementation

### Risk 3: Firmware compatibility

**Impact:** High (device won't work)
**Probability:** Low
**Mitigation:** Version check in device_connect, clear error messages

### Risk 4: Authorization bypass

**Impact:** Critical (regulatory/legal issues)
**Probability:** Low
**Mitigation:** Defense-in-depth: tool-level + connector-level + audit logging

### Risk 5: PCAP parsing complexity

**Impact:** Medium (affects analysis tools)
**Probability:** Medium
**Mitigation:** Use existing `pcap-file` crate, fallback to raw hex

---

## Success Metrics

### Phase 1 (Week 2)
- ✅ 14 tools working via Python wrapper
- ✅ AI can perform basic BLE scanning
- ✅ Captures stored and retrievable
- ✅ Zero crashes on happy path

### Phase 2 (Week 6)
- ✅ All 36 tools working
- ✅ Full feature parity with ubertooth-tools
- ✅ Authorization enforced for attack tools
- ✅ Comprehensive test coverage (>80%)

### Phase 3 (Week 12)
- ✅ Native Rust backend for core operations
- ✅ 100x+ performance improvement demonstrated
- ✅ Backend selection working (UBERTOOTH_BACKEND env var)
- ✅ Zero regressions vs Python backend

---

## Conclusion

**The Ubertooth One connector is an excellent follow-on project to yardstick-one-connector.**

Key advantages:
1. ✅ Template is highly relevant (90% architectural reuse)
2. ✅ Native Rust is definitely feasible
3. ✅ Clear implementation path (Python → Rust)
4. ✅ Well-defined tool schemas (36 tools, ready for issues)
5. ✅ Bluetooth is a high-value security target

**Recommended approach:**
- Start with Python wrapper (fast time-to-market)
- Ship incrementally (14 tools @ Week 2, 36 tools @ Week 6)
- Add native Rust core (performance optimization @ Week 12)

**I'm ready to move forward with PRD creation and implementation!** 🚀

What would you like to tackle next?
