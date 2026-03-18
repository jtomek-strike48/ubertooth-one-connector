# GitHub Issues for Ubertooth One Connector

This document contains all 36 tool implementation issues ready to create in GitHub.

## Milestones

Create these milestones first:
- **v0.1.0** - Phase 1: Python Backend Foundation (14 tools)
- **v0.2.0** - Phase 2: Complete Python Implementation (22 additional tools)
- **v1.0.0** - Phase 3: Native Rust USB (all 36 tools)

## Labels

Create these labels:
- `P0` (red) - Critical priority
- `P1` (orange) - High priority
- `P2` (yellow) - Normal priority
- `phase-1` (blue)
- `phase-2` (blue)
- `phase-3` (blue)
- `bt-device` (purple)
- `bt-config` (purple)
- `bt-recon` (purple)
- `bt-capture` (purple)
- `bt-analysis` (purple)
- `bt-attack` (red)
- `bt-advanced` (purple)
- `backend-python` (green)
- `backend-rust` (green)
- `security` (red)
- `tool-implementation` (gray)

---

## Phase 1 Issues (v0.1.0 - Week 1-2)

### Issue #1: ✅ device_connect (COMPLETE)

**Title:** Implement device_connect tool

**Labels:** `P0`, `phase-1`, `bt-device`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Status:** ✅ **COMPLETED** in v0.0.1

**Description:**
Connect to an Ubertooth One USB device.

**Implementation:**
- [x] Tool implementation in `crates/tools/src/device_connect.rs`
- [x] Backend method in `SidecarManager::device_connect()`
- [x] Unit tests with mock backend
- [x] Integration test

**Input Schema:**
```json
{
  "device_index": 0  // Optional: which Ubertooth if multiple connected
}
```

**Output Schema:**
```json
{
  "success": true,
  "device_id": "ubertooth-001",
  "serial": "0000000012AB",
  "firmware_version": "2020-12-R1",
  "api_version": "1.07",
  "board_id": 1,
  "capabilities": ["btle_sniff", "bt_rx", "specan"],
  "message": "Connected to Ubertooth One"
}
```

**Authorization:** None

**Acceptance Criteria:**
- [x] Detects and connects to Ubertooth One via USB
- [x] Returns firmware and device information
- [x] Clear error if ubertooth-tools not installed
- [x] Clear error if device not found
- [x] Unit tests pass

---

### Issue #2: device_disconnect

**Title:** Implement device_disconnect tool

**Labels:** `P0`, `phase-1`, `bt-device`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Disconnect from Ubertooth One and release USB device.

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/device_disconnect.rs`
- [ ] Backend method in `SidecarManager::device_disconnect()`
- [ ] Unit tests with mock backend
- [ ] Integration test
- [ ] Register in tool registry

**Input Schema:**
```json
{}
```

**Output Schema:**
```json
{
  "success": true,
  "message": "Device disconnected"
}
```

**Authorization:** None

**Acceptance Criteria:**
- [ ] Cleanly disconnects from device
- [ ] Returns success confirmation
- [ ] Handles already-disconnected state gracefully
- [ ] Unit tests pass

---

### Issue #3: device_status

**Title:** Implement device_status tool

**Labels:** `P0`, `phase-1`, `bt-device`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Get current device state and configuration.

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/device_status.rs`
- [ ] Backend method in `SidecarManager::device_status()`
- [ ] Parse ubertooth-util output
- [ ] Unit tests with mock backend
- [ ] Register in tool registry

**Input Schema:**
```json
{}
```

**Output Schema:**
```json
{
  "success": true,
  "connected": true,
  "device_id": "ubertooth-001",
  "serial": "0000000012AB",
  "firmware": "2020-12-R1",
  "board_id": 1,
  "current_mode": "idle",
  "channel": 37,
  "modulation": "BT_LOW_ENERGY",
  "power_level": 7
}
```

**Authorization:** None

**Acceptance Criteria:**
- [ ] Returns comprehensive device status
- [ ] Includes current configuration
- [ ] Error if not connected
- [ ] Unit tests pass

---

### Issue #4: session_context

**Title:** Implement session_context tool

**Labels:** `P1`, `phase-1`, `bt-device`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Comprehensive orientation for AI - device state + recent captures + configurations.

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/session_context.rs`
- [ ] Combines device_status + capture_list + config_list
- [ ] Unit tests with mock backend
- [ ] Register in tool registry

**Input Schema:**
```json
{
  "include_recent_captures": true,
  "max_captures": 5,
  "include_configs": true
}
```

**Output Schema:**
```json
{
  "success": true,
  "timestamp": "2026-02-26T15:30:45Z",
  "device": { "connected": true, "serial": "..." },
  "recent_captures": [...],
  "saved_configs": [...],
  "storage": { "captures_count": 23, "total_size_mb": 145.3 }
}
```

**Authorization:** None

**Dependencies:** device_status, capture_list, config_list

**Acceptance Criteria:**
- [ ] Returns complete session overview
- [ ] Handles missing components gracefully
- [ ] Unit tests pass

---

### Issue #5: btle_scan

**Title:** Implement btle_scan tool

**Labels:** `P0`, `phase-1`, `bt-recon`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Scan for BLE devices and capture advertisements. **Most important tool!**

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/btle_scan.rs`
- [ ] Backend executes `ubertooth-btle -f -c <channel> -t <duration>`
- [ ] Parse output and create capture
- [ ] Save to CaptureStore
- [ ] Unit tests with mock backend
- [ ] Register in tool registry

**Input Schema:**
```json
{
  "duration_sec": 30,
  "channel": 37,
  "promiscuous": true,
  "save_pcap": true
}
```

**Output Schema:**
```json
{
  "success": true,
  "capture_id": "cap-btle-abc123",
  "scan_duration_sec": 30,
  "channel": 37,
  "devices_found": [
    {
      "mac_address": "AA:BB:CC:DD:EE:FF",
      "device_name": "Fitbit Charge",
      "rssi_avg": -65,
      "packet_count": 45
    }
  ],
  "total_packets": 142,
  "pcap_path": "/home/user/.ubertooth/captures/cap-btle-abc123.pcap"
}
```

**Authorization:** None (passive)

**Acceptance Criteria:**
- [ ] Scans BLE advertising channels
- [ ] Parses device information
- [ ] Saves PCAP file
- [ ] Returns capture metadata
- [ ] Unit tests pass

---

### Issue #6: bt_specan

**Title:** Implement bt_specan tool

**Labels:** `P1`, `phase-1`, `bt-recon`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Spectrum analysis of 2.4 GHz ISM band.

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/bt_specan.rs`
- [ ] Backend executes `ubertooth-specan -l <low> -u <high>`
- [ ] Parse RSSI data
- [ ] Save capture
- [ ] Unit tests with mock backend
- [ ] Register in tool registry

**Input Schema:**
```json
{
  "low_freq": 2402,
  "high_freq": 2480,
  "duration_sec": 10,
  "rssi_threshold": -90
}
```

**Output Schema:**
```json
{
  "success": true,
  "capture_id": "cap-specan-abc123",
  "frequency_range": [2402, 2480],
  "duration_sec": 10,
  "scan_results": [
    {
      "frequency_mhz": 2402,
      "channel": 0,
      "rssi_avg": -65,
      "activity_percent": 45.2
    }
  ],
  "hotspots": [...]
}
```

**Authorization:** None (passive)

**Acceptance Criteria:**
- [ ] Performs spectrum sweep
- [ ] Returns RSSI data per frequency
- [ ] Identifies activity hotspots
- [ ] Unit tests pass

---

### Issue #7: configure_channel

**Title:** Implement configure_channel tool

**Labels:** `P1`, `phase-1`, `bt-config`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Set Bluetooth channel (0-78).

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/configure_channel.rs`
- [ ] Backend executes `ubertooth-util -c <channel>`
- [ ] Validate channel range
- [ ] Unit tests with mock backend
- [ ] Register in tool registry

**Input Schema:**
```json
{
  "channel": 37,
  "validate": true
}
```

**Output Schema:**
```json
{
  "success": true,
  "channel": 37,
  "frequency_mhz": 2402,
  "message": "Channel set to 37 (2402 MHz)"
}
```

**Authorization:** None

**Acceptance Criteria:**
- [ ] Sets channel via ubertooth-util
- [ ] Validates channel range (0-78)
- [ ] Returns frequency calculation
- [ ] Unit tests pass

---

### Issue #8: configure_modulation

**Title:** Implement configure_modulation tool

**Labels:** `P1`, `phase-1`, `bt-config`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Set modulation type (BT Basic Rate, BT Low Energy, etc.).

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/configure_modulation.rs`
- [ ] Backend executes `ubertooth-util -m <mode>`
- [ ] Validate modulation type
- [ ] Unit tests with mock backend
- [ ] Register in tool registry

**Input Schema:**
```json
{
  "modulation": "BT_LOW_ENERGY"
}
```

**Output Schema:**
```json
{
  "success": true,
  "modulation": "BT_LOW_ENERGY",
  "message": "Modulation set to BT_LOW_ENERGY"
}
```

**Authorization:** None

**Acceptance Criteria:**
- [ ] Sets modulation type
- [ ] Validates modulation value
- [ ] Unit tests pass

---

### Issue #9: configure_power

**Title:** Implement configure_power tool

**Labels:** `P1`, `phase-1`, `bt-config`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Set TX power level and amplifier settings.

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/configure_power.rs`
- [ ] Backend executes ubertooth-util power commands
- [ ] Validate power level (0-7)
- [ ] Unit tests with mock backend
- [ ] Register in tool registry

**Input Schema:**
```json
{
  "power_level": 7,
  "paen": true,
  "hgm": false
}
```

**Output Schema:**
```json
{
  "success": true,
  "power_level": 7,
  "paen": true,
  "hgm": false,
  "estimated_power_dbm": 20,
  "message": "Power configured: Level 7 with PA enabled (~20 dBm)"
}
```

**Authorization:** WARNING (modifying TX power)

**Acceptance Criteria:**
- [ ] Sets TX power level
- [ ] Controls PA enable
- [ ] Controls high gain mode
- [ ] Unit tests pass

---

### Issue #10: capture_list

**Title:** Implement capture_list tool

**Labels:** `P1`, `phase-1`, `bt-capture`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
List stored packet captures with filtering.

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/capture_list.rs`
- [ ] Use `CaptureStore::list_captures()`
- [ ] Implement filtering and sorting
- [ ] Unit tests with mock store
- [ ] Register in tool registry

**Input Schema:**
```json
{
  "filter_type": null,
  "limit": 50,
  "offset": 0,
  "sort_by": "timestamp",
  "sort_order": "desc"
}
```

**Output Schema:**
```json
{
  "success": true,
  "captures": [
    {
      "capture_id": "cap-btle-abc123",
      "timestamp": "2026-02-26T15:30:00Z",
      "type": "btle_sniff",
      "packet_count": 142,
      "file_size_bytes": 45320,
      "tags": ["ble", "scan"]
    }
  ],
  "total_count": 23,
  "offset": 0,
  "limit": 50
}
```

**Authorization:** None

**Acceptance Criteria:**
- [ ] Lists all captures
- [ ] Filters by type
- [ ] Supports pagination
- [ ] Sorts by timestamp/size
- [ ] Unit tests pass

---

### Issue #11: capture_get

**Title:** Implement capture_get tool

**Labels:** `P1`, `phase-1`, `bt-capture`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Retrieve packet data from a capture with pagination.

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/capture_get.rs`
- [ ] Use `CaptureStore::load_metadata()`
- [ ] Parse PCAP file for packet data
- [ ] Implement pagination
- [ ] Unit tests with mock store
- [ ] Register in tool registry

**Input Schema:**
```json
{
  "capture_id": "cap-btle-abc123",
  "offset": 0,
  "limit": 100,
  "format": "json"
}
```

**Output Schema:**
```json
{
  "success": true,
  "capture_id": "cap-btle-abc123",
  "offset": 0,
  "limit": 100,
  "packet_count": 142,
  "packets": [
    {
      "index": 0,
      "timestamp": "2026-02-26T15:30:00.123456Z",
      "channel": 37,
      "rssi": -65,
      "data_hex": "0201061AFF4C00..."
    }
  ],
  "has_more": true
}
```

**Authorization:** None

**Acceptance Criteria:**
- [ ] Retrieves capture metadata
- [ ] Parses PCAP packets
- [ ] Supports pagination
- [ ] Returns JSON format
- [ ] Unit tests pass

---

### Issue #12: capture_delete

**Title:** Implement capture_delete tool

**Labels:** `P2`, `phase-1`, `bt-capture`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Delete a stored capture.

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/capture_delete.rs`
- [ ] Use `CaptureStore::delete_capture()`
- [ ] Unit tests with mock store
- [ ] Register in tool registry

**Input Schema:**
```json
{
  "capture_id": "cap-btle-abc123"
}
```

**Output Schema:**
```json
{
  "success": true,
  "message": "Capture 'cap-btle-abc123' deleted"
}
```

**Authorization:** None

**Acceptance Criteria:**
- [ ] Deletes PCAP file
- [ ] Deletes metadata file
- [ ] Handles non-existent capture
- [ ] Unit tests pass

---

### Issue #13: capture_tag

**Title:** Implement capture_tag tool

**Labels:** `P2`, `phase-1`, `bt-capture`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Add tags and notes to a capture.

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/capture_tag.rs`
- [ ] Load metadata, update tags, save
- [ ] Support append vs replace
- [ ] Unit tests with mock store
- [ ] Register in tool registry

**Input Schema:**
```json
{
  "capture_id": "cap-btle-abc123",
  "tags": ["ble", "scan", "target_device"],
  "description": "Captured advertisements from target Fitbit",
  "append_tags": true
}
```

**Output Schema:**
```json
{
  "success": true,
  "capture_id": "cap-btle-abc123",
  "tags": ["ble", "scan", "target_device"],
  "description": "Captured advertisements from target Fitbit"
}
```

**Authorization:** None

**Acceptance Criteria:**
- [ ] Updates capture metadata
- [ ] Appends or replaces tags
- [ ] Updates description
- [ ] Unit tests pass

---

### Issue #14: bt_analyze

**Title:** Implement bt_analyze tool

**Labels:** `P1`, `phase-1`, `bt-analysis`, `backend-python`, `tool-implementation`

**Milestone:** v0.1.0

**Description:**
Analyze captured packets and extract insights (basic version for Phase 1).

**Implementation Checklist:**
- [ ] Tool implementation in `crates/tools/src/bt_analyze.rs`
- [ ] Load capture from store
- [ ] Basic packet statistics
- [ ] Device enumeration
- [ ] Unit tests with mock data
- [ ] Register in tool registry

**Input Schema:**
```json
{
  "capture_id": "cap-btle-abc123",
  "analysis_type": "auto",
  "target_mac": null
}
```

**Output Schema:**
```json
{
  "success": true,
  "capture_id": "cap-btle-abc123",
  "analysis": {
    "protocol_summary": {
      "type": "BLE",
      "pdu_types": { "ADV_IND": 45 }
    },
    "devices": [
      {
        "mac_address": "AA:BB:CC:DD:EE:FF",
        "packet_count": 45,
        "device_name": "Fitbit Charge"
      }
    ]
  }
}
```

**Authorization:** None

**Acceptance Criteria:**
- [ ] Parses PCAP file
- [ ] Generates basic statistics
- [ ] Identifies devices
- [ ] Unit tests pass

---

## Phase 2 Issues (v0.2.0 - Week 3-6)

### Week 3: Advanced Recon (Issues #15-21)

**Issue #15:** `bt_scan` - Bluetooth Classic device scanning
**Issue #16:** `bt_follow` - Follow specific connection
**Issue #17:** `afh_analyze` - AFH channel analysis
**Issue #18:** `bt_discover` - Promiscuous BR/EDR discovery
**Issue #19:** `btle_follow` - Follow BLE connection by access address
**Issue #20:** `configure_squelch` - Set RSSI squelch threshold
**Issue #21:** `configure_leds` - Control LED indicators

### Week 4: Config Management (Issues #22-25)

**Issue #22:** `bt_save_config` - Save current radio configuration
**Issue #23:** `bt_load_config` - Load saved configuration
**Issue #24:** `config_list` - List all saved configs
**Issue #25:** `config_delete` - Delete a configuration

### Week 5: Analysis Tools (Issues #26-30)

**Issue #26:** `bt_compare` - Compare two captures
**Issue #27:** `bt_decode` - Decode specific packet types
**Issue #28:** `bt_fingerprint` - Device fingerprinting
**Issue #29:** `pcap_merge` - Merge multiple captures
**Issue #30:** `capture_export` - Export to PCAP/JSON/CSV

### Week 6: Attack Operations (Issues #31-36)

**Issue #31:** `btle_inject` - Inject BLE packets ⚠️
**Issue #32:** `bt_jam` - Jam Bluetooth frequencies ⚠️⚠️
**Issue #33:** `btle_slave` - BLE peripheral mode ⚠️
**Issue #34:** `btle_mitm` - BLE MITM attack ⚠️⚠️
**Issue #35:** `bt_spoof` - Spoof device identity ⚠️
**Issue #36:** `ubertooth_raw` - Raw command execution

---

## Phase 3 Issues (v1.0.0 - Week 7-18)

### Infrastructure Issues

**Issue #37:** Native Rust USB device connection
**Issue #38:** USB command protocol implementation
**Issue #39:** Packet streaming and buffering
**Issue #40:** PCAP writer in pure Rust
**Issue #41:** Protocol dissectors (BLE/BR)

### Tool Reimplementation (Issues #42-77)

Reimplement all 36 tools using RustUsbBackend for 100-200x performance improvement.

---

## Summary

**Total Issues:** 77+
- Phase 1: 14 tools (Issues #1-14)
- Phase 2: 22 tools (Issues #15-36)
- Phase 3: 36 tools + 5 infrastructure (Issues #37-77)

**Priority Breakdown:**
- P0 (Critical): 5 issues (device_connect, device_status, btle_scan, etc.)
- P1 (High): 20 issues
- P2 (Normal): 11 issues

**Authorization:**
- None: 29 tools
- WARNING: 4 tools
- REQUIRED: 5 tools (attack operations)
