# Ubertooth One Connector - Tool Schemas

Complete tool specifications for GitHub issue creation. Each tool includes input schema, output schema, authorization requirements, and implementation notes.

---

## Table of Contents

1. [bt-device (4 tools)](#bt-device)
2. [bt-config (8 tools)](#bt-config)
3. [bt-recon (7 tools)](#bt-recon)
4. [bt-capture (5 tools)](#bt-capture)
5. [bt-analysis (5 tools)](#bt-analysis)
6. [bt-attack (5 tools)](#bt-attack)
7. [bt-advanced (2 tools)](#bt-advanced)

**Total: 36 tools across 7 categories**

---

## Category: bt-device

Device connection, status, and session management.

### Tool: device_connect

**Description:** Connect to an Ubertooth One USB device.

**Category:** `bt-device`

**Input Schema:**
```json
{
  "device_index": 0  // Optional: which Ubertooth if multiple connected (default: 0)
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
  "board_id": 1,  // 0=Ubertooth Zero, 1=Ubertooth One, 2=TC13Badge
  "capabilities": ["btle_sniff", "bt_rx", "specan", "jam"],
  "message": "Connected to Ubertooth One (serial: 0000000012AB)"
}
```

**Error Cases:**
- `DEVICE_NOT_FOUND` - No Ubertooth devices detected
- `DEVICE_IN_USE` - Device already connected by another process
- `USB_PERMISSION_DENIED` - Need udev rules or sudo
- `FIRMWARE_TOO_OLD` - Firmware update required

**Backend Implementation:**
- **Python:** Call `ubertooth-util -v` to get version info
- **Rust:** `ubertooth_init()` + `ubertooth_connect()` + `cmd_get_api()`

**Authorization:** None (read-only connection)

---

### Tool: device_disconnect

**Description:** Disconnect from Ubertooth One and release USB device.

**Category:** `bt-device`

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

**Error Cases:**
- `NO_DEVICE_CONNECTED` - No active connection

**Backend Implementation:**
- **Python:** Kill sidecar process
- **Rust:** `ubertooth_stop()` + `libusb_close()`

**Authorization:** None

---

### Tool: device_status

**Description:** Get current device state and configuration.

**Category:** `bt-device`

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
  "current_mode": "idle",  // idle, rx_symbols, btle_sniff, specan, etc.
  "channel": 37,
  "modulation": "BT_LOW_ENERGY",
  "power_level": 7,
  "paen": true,  // Power amplifier enabled
  "hgm": true,   // High gain mode
  "led_state": {
    "usr": false,
    "rx": false,
    "tx": false
  },
  "stats": {
    "packets_received": 12450,
    "packets_transmitted": 0,
    "fifo_overflows": 0,
    "dma_errors": 0
  }
}
```

**Error Cases:**
- `NO_DEVICE_CONNECTED`

**Backend Implementation:**
- **Python:** Multiple `ubertooth-util` calls
- **Rust:** `cmd_get_channel()`, `cmd_get_modulation()`, `cmd_get_palevel()`, etc.

**Authorization:** None

---

### Tool: session_context

**Description:** Comprehensive orientation for AI - device state + recent captures + configurations.

**Category:** `bt-device`

**Input Schema:**
```json
{
  "include_recent_captures": true,  // default: true
  "max_captures": 5,                // default: 5
  "include_configs": true           // default: true
}
```

**Output Schema:**
```json
{
  "success": true,
  "timestamp": "2026-02-26T15:30:45Z",
  "device": {
    "connected": true,
    "serial": "0000000012AB",
    "firmware": "2020-12-R1",
    "mode": "idle",
    "channel": 37
  },
  "recent_captures": [
    {
      "capture_id": "cap-abc123",
      "timestamp": "2026-02-26T15:25:00Z",
      "type": "btle_sniff",
      "packet_count": 142,
      "duration_sec": 30,
      "tags": ["ble", "advertisements"]
    }
  ],
  "saved_configs": [
    {
      "config_name": "ble_channel_37",
      "channel": 37,
      "modulation": "BT_LOW_ENERGY",
      "description": "BLE advertising channel 37"
    }
  ],
  "storage": {
    "captures_dir": "/home/user/.ubertooth/captures",
    "captures_count": 23,
    "total_size_mb": 145.3
  }
}
```

**Backend Implementation:**
- Combines `device_status` + `capture_list` + `config_list`
- Single tool call for AI context loading

**Authorization:** None

---

## Category: bt-config

Radio configuration and preset management.

### Tool: configure_channel

**Description:** Set Bluetooth channel (0-78 for Classic BR, 37-39 for BLE advertising).

**Category:** `bt-config`

**Input Schema:**
```json
{
  "channel": 37,  // 0-78
  "validate": true  // default: true - check if channel is valid for current mode
}
```

**Output Schema:**
```json
{
  "success": true,
  "channel": 37,
  "frequency_mhz": 2402,  // Calculated: 2402 + channel MHz
  "message": "Channel set to 37 (2402 MHz)"
}
```

**Error Cases:**
- `NO_DEVICE_CONNECTED`
- `INVALID_CHANNEL` - Channel out of range (0-78)
- `CHANNEL_MODE_MISMATCH` - e.g., using BR channel in BLE mode

**Backend Implementation:**
- **Python:** `ubertooth-util -c <channel>`
- **Rust:** `cmd_set_channel(channel)`

**Authorization:** None (configuration only, no RF emission)

---

### Tool: configure_modulation

**Description:** Set modulation type (BT Basic Rate, BT Low Energy, or generic).

**Category:** `bt-config`

**Input Schema:**
```json
{
  "modulation": "BT_LOW_ENERGY"  // "BT_BASIC_RATE", "BT_LOW_ENERGY", "80211_FHSS", "NONE"
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

**Error Cases:**
- `NO_DEVICE_CONNECTED`
- `INVALID_MODULATION`

**Backend Implementation:**
- **Python:** `ubertooth-util -m <mode>`
- **Rust:** `cmd_set_modulation(mod)`

**Authorization:** None

---

### Tool: configure_power

**Description:** Set TX power level and amplifier settings.

**Category:** `bt-config`

**Input Schema:**
```json
{
  "power_level": 7,    // 0-7, where 7 is maximum (~20 dBm with PA)
  "paen": true,        // Power amplifier enable (default: true)
  "hgm": false         // High gain mode for RX (default: false)
}
```

**Output Schema:**
```json
{
  "success": true,
  "power_level": 7,
  "paen": true,
  "hgm": false,
  "estimated_power_dbm": 20,  // Approximate TX power
  "message": "Power configured: Level 7 with PA enabled (~20 dBm)"
}
```

**Error Cases:**
- `NO_DEVICE_CONNECTED`
- `INVALID_POWER_LEVEL` - Must be 0-7

**Backend Implementation:**
- **Python:** `ubertooth-util -p <level>`, `ubertooth-util -P <0|1>`, `ubertooth-util -H <0|1>`
- **Rust:** `cmd_set_palevel()`, `cmd_set_paen()`, `cmd_set_hgm()`

**Authorization:** ⚠️ WARNING level (modifying TX power)

---

### Tool: configure_squelch

**Description:** Set RSSI squelch threshold to filter weak signals.

**Category:** `bt-config`

**Input Schema:**
```json
{
  "squelch_level": -90  // RSSI threshold in dBm (-128 to 0)
}
```

**Output Schema:**
```json
{
  "success": true,
  "squelch_level": -90,
  "message": "Squelch set to -90 dBm"
}
```

**Backend Implementation:**
- **Python:** `ubertooth-util -q <level>`
- **Rust:** `cmd_set_squelch(level)`

**Authorization:** None

---

### Tool: configure_leds

**Description:** Control LED indicators (useful for headless operation).

**Category:** `bt-config`

**Input Schema:**
```json
{
  "usr_led": true,   // User LED
  "rx_led": false,   // RX activity LED
  "tx_led": false    // TX activity LED
}
```

**Output Schema:**
```json
{
  "success": true,
  "leds": {
    "usr": true,
    "rx": false,
    "tx": false
  }
}
```

**Backend Implementation:**
- **Rust:** `cmd_set_usrled()`, `cmd_set_rxled()`, `cmd_set_txled()`

**Authorization:** None

---

### Tool: bt_save_config

**Description:** Save current radio configuration as a named preset.

**Category:** `bt-config`

**Input Schema:**
```json
{
  "config_name": "ble_adv_ch37",  // Unique name
  "description": "BLE advertising on channel 37 with default settings",
  "overwrite": false  // Allow overwriting existing config
}
```

**Output Schema:**
```json
{
  "success": true,
  "config_name": "ble_adv_ch37",
  "config_path": "/home/user/.ubertooth/configs/ble_adv_ch37.json",
  "saved_settings": {
    "channel": 37,
    "modulation": "BT_LOW_ENERGY",
    "power_level": 7,
    "paen": true,
    "hgm": false,
    "squelch": -90
  }
}
```

**Backend Implementation:**
- Query all config parameters via device_status
- Write to `~/.ubertooth/configs/<name>.json`

**Authorization:** None

---

### Tool: bt_load_config

**Description:** Load a saved configuration preset.

**Category:** `bt-config`

**Input Schema:**
```json
{
  "config_name": "ble_adv_ch37"
}
```

**Output Schema:**
```json
{
  "success": true,
  "config_name": "ble_adv_ch37",
  "applied_settings": {
    "channel": 37,
    "modulation": "BT_LOW_ENERGY",
    "power_level": 7
  },
  "message": "Configuration 'ble_adv_ch37' loaded successfully"
}
```

**Error Cases:**
- `CONFIG_NOT_FOUND`
- `CONFIG_PARSE_ERROR`

**Backend Implementation:**
- Read from `~/.ubertooth/configs/<name>.json`
- Apply each setting via corresponding configure_* tools

**Authorization:** None

---

### Tool: config_list

**Description:** List all saved configuration presets.

**Category:** `bt-config`

**Input Schema:**
```json
{}
```

**Output Schema:**
```json
{
  "success": true,
  "configs": [
    {
      "name": "ble_adv_ch37",
      "description": "BLE advertising on channel 37",
      "created": "2026-02-26T10:00:00Z",
      "settings_preview": {
        "channel": 37,
        "modulation": "BT_LOW_ENERGY"
      }
    },
    {
      "name": "bt_classic_ch10",
      "description": "Bluetooth Classic on channel 10",
      "created": "2026-02-25T14:30:00Z",
      "settings_preview": {
        "channel": 10,
        "modulation": "BT_BASIC_RATE"
      }
    }
  ],
  "count": 2
}
```

**Backend Implementation:**
- List files in `~/.ubertooth/configs/`

**Authorization:** None

---

### Tool: config_delete

**Description:** Delete a saved configuration preset.

**Category:** `bt-config`

**Input Schema:**
```json
{
  "config_name": "ble_adv_ch37"
}
```

**Output Schema:**
```json
{
  "success": true,
  "message": "Configuration 'ble_adv_ch37' deleted"
}
```

**Backend Implementation:**
- Delete file from `~/.ubertooth/configs/`

**Authorization:** None

---

## Category: bt-recon

Reconnaissance and signal discovery operations.

### Tool: btle_scan

**Description:** Scan for BLE devices and capture advertisements.

**Category:** `bt-recon`

**Input Schema:**
```json
{
  "duration_sec": 30,     // Scan duration (default: 30)
  "channel": 37,           // BLE ad channel 37, 38, or 39 (default: 37)
  "promiscuous": true,     // Capture all ads vs targeted (default: true)
  "save_pcap": true        // Save to PCAP file (default: true)
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
      "address_type": "random",  // "public" or "random"
      "device_name": "Fitbit Charge",
      "rssi_avg": -65,
      "rssi_min": -72,
      "rssi_max": -58,
      "packet_count": 45,
      "first_seen": "2026-02-26T15:30:00Z",
      "last_seen": "2026-02-26T15:30:30Z",
      "adv_data": {
        "flags": ["LE_LIMITED_DISCOVERABLE"],
        "services": ["180F"],  // Battery Service UUID
        "manufacturer_data": "4C00..."
      }
    }
  ],
  "total_packets": 142,
  "pcap_path": "/home/user/.ubertooth/captures/cap-btle-abc123.pcap",
  "preview": [
    "AA:BB:CC:DD:EE:FF | ADV_IND | RSSI -65 dBm | Name: Fitbit Charge",
    "11:22:33:44:55:66 | ADV_NONCONN_IND | RSSI -72 dBm | Beacon"
  ]
}
```

**Error Cases:**
- `NO_DEVICE_CONNECTED`
- `INVALID_CHANNEL` - Must be 37, 38, or 39 for BLE advertising
- `SCAN_TIMEOUT`

**Backend Implementation:**
- **Python:** `ubertooth-btle -f -c <channel> -t <duration>`
- **Rust:** `cmd_set_channel()` + `cmd_btle_promisc()` + bulk RX loop

**Authorization:** None (passive scanning)

---

### Tool: bt_scan

**Description:** Scan for Bluetooth Classic devices (inquiry scan).

**Category:** `bt-recon`

**Input Schema:**
```json
{
  "duration_sec": 30,      // Inquiry duration
  "extended_inquiry": true  // Capture Extended Inquiry Response (EIR)
}
```

**Output Schema:**
```json
{
  "success": true,
  "capture_id": "cap-bt-abc123",
  "devices_found": [
    {
      "bd_addr": "AA:BB:CC:DD:EE:FF",
      "class_of_device": "0x5A020C",  // Hex CoD
      "device_class": "Phone, Smartphone",
      "device_name": "John's iPhone",
      "rssi": -60,
      "clock_offset": 12345,
      "page_scan_mode": 1,
      "eir_data": {
        "name": "John's iPhone",
        "services": ["1108", "110B"]  // Audio Sink, A/V Remote Control
      }
    }
  ],
  "total_devices": 5
}
```

**Backend Implementation:**
- **Python:** `ubertooth-scan -t <duration>`
- **Rust:** Inquiry scan mode (requires btctl firmware mode)

**Authorization:** None (passive)

---

### Tool: bt_specan

**Description:** Spectrum analysis of 2.4 GHz ISM band.

**Category:** `bt-recon`

**Input Schema:**
```json
{
  "low_freq": 2402,        // Start frequency in MHz
  "high_freq": 2480,       // End frequency in MHz
  "duration_sec": 10,      // Scan duration
  "rssi_threshold": -90    // RSSI floor in dBm
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
      "rssi_max": -58,
      "rssi_min": -72,
      "activity_percent": 45.2
    },
    {
      "frequency_mhz": 2403,
      "channel": 1,
      "rssi_avg": -70,
      "rssi_max": -65,
      "rssi_min": -80,
      "activity_percent": 32.1
    }
  ],
  "hotspots": [
    {
      "frequency_mhz": 2402,
      "rssi_max": -58,
      "interpretation": "Likely BLE advertising channel 37"
    }
  ],
  "visualization_data": {
    "frequencies": [2402, 2403, ...],
    "rssi_values": [-65, -70, ...]
  }
}
```

**Backend Implementation:**
- **Python:** `ubertooth-specan -l <low> -u <high>`
- **Rust:** `cmd_specan(low, high)` + packet parsing

**Authorization:** None (passive)

---

### Tool: bt_follow

**Description:** Follow a specific Bluetooth connection and capture packets.

**Category:** `bt-recon`

**Input Schema:**
```json
{
  "bd_addr": "AA:BB:CC:DD:EE:FF",  // Target Bluetooth address
  "duration_sec": 60,               // Follow duration
  "channel_hopping": true           // Follow AFH hopping (default: true)
}
```

**Output Schema:**
```json
{
  "success": true,
  "capture_id": "cap-follow-abc123",
  "bd_addr": "AA:BB:CC:DD:EE:FF",
  "connection_found": true,
  "packet_count": 1250,
  "duration_sec": 60,
  "channels_used": [12, 15, 18, 22, ...],  // AFH map
  "pcap_path": "/home/user/.ubertooth/captures/cap-follow-abc123.pcap"
}
```

**Error Cases:**
- `TARGET_NOT_FOUND` - No connection with specified BD_ADDR
- `LOST_CONNECTION` - Connection dropped during capture

**Backend Implementation:**
- **Python:** `ubertooth-follow -t <bdaddr>`
- **Rust:** `cmd_set_bdaddr()` + `cmd_start_hopping()` + capture

**Authorization:** ⚠️ WARNING (monitoring specific connection)

---

### Tool: afh_analyze

**Description:** Analyze Adaptive Frequency Hopping (AFH) channel usage for a Bluetooth piconet.

**Category:** `bt-recon`

**Input Schema:**
```json
{
  "bd_addr": "AA:BB:CC:DD:EE:FF",  // Piconet master address (optional)
  "duration_sec": 30
}
```

**Output Schema:**
```json
{
  "success": true,
  "bd_addr": "AA:BB:CC:DD:EE:FF",
  "afh_map": "0xFFFFFFFFFF...",  // 79-bit channel map
  "channels_used": [0, 1, 5, 10, ...],  // Active channels
  "channels_avoided": [2, 3, 4, 6, ...],  // Unused channels
  "used_count": 62,
  "avoided_count": 17,
  "interpretation": "Avoiding WiFi interference on channels 2-4 (2404-2406 MHz)"
}
```

**Backend Implementation:**
- **Python:** `ubertooth-afh -t <bdaddr>`
- **Rust:** `cmd_afh()` + AFH map parsing

**Authorization:** None (passive)

---

### Tool: bt_discover

**Description:** Promiscuous Bluetooth discovery - capture any BR/EDR traffic.

**Category:** `bt-recon`

**Input Schema:**
```json
{
  "duration_sec": 60,
  "channel": null,  // null = hop all channels, or specify 0-78
  "save_pcap": true
}
```

**Output Schema:**
```json
{
  "success": true,
  "capture_id": "cap-discover-abc123",
  "duration_sec": 60,
  "piconets_found": [
    {
      "bd_addr": "AA:BB:CC:DD:EE:FF",
      "uap": 170,  // Upper Address Part
      "packet_count": 450
    }
  ],
  "total_packets": 2500,
  "pcap_path": "/home/user/.ubertooth/captures/cap-discover-abc123.pcap"
}
```

**Backend Implementation:**
- **Python:** `ubertooth-rx -t <duration>`
- **Rust:** `cmd_rx_syms()` + bulk RX loop

**Authorization:** None (passive)

---

### Tool: btle_follow

**Description:** Follow a specific BLE connection using access address.

**Category:** `bt-recon`

**Input Schema:**
```json
{
  "access_address": "0x8E89BED6",  // BLE access address (hex)
  "duration_sec": 60,
  "crc_verify": true,  // Verify CRC (default: true)
  "follow_connections": true  // Follow connection events (default: true)
}
```

**Output Schema:**
```json
{
  "success": true,
  "capture_id": "cap-btle-follow-abc123",
  "access_address": "0x8E89BED6",
  "packets_captured": 350,
  "connection_events": 120,
  "crc_valid_percent": 98.5,
  "pcap_path": "/home/user/.ubertooth/captures/cap-btle-follow-abc123.pcap"
}
```

**Backend Implementation:**
- **Python:** `ubertooth-btle -f -a <aa>`
- **Rust:** `cmd_set_access_address()` + `cmd_btle_sniffing(true)`

**Authorization:** ⚠️ WARNING (targeted connection sniffing)

---

## Category: bt-capture

Packet capture storage and management.

### Tool: capture_list

**Description:** List stored packet captures with filtering.

**Category:** `bt-capture`

**Input Schema:**
```json
{
  "filter_type": null,  // "btle_sniff", "bt_rx", "specan", etc.
  "limit": 50,          // Max results
  "offset": 0,          // Pagination offset
  "sort_by": "timestamp",  // "timestamp", "size", "packet_count"
  "sort_order": "desc"  // "asc" or "desc"
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
      "duration_sec": 30,
      "file_size_bytes": 45320,
      "pcap_path": "/home/user/.ubertooth/captures/cap-btle-abc123.pcap",
      "tags": ["ble", "scan", "channel_37"],
      "description": "BLE advertisement scan on channel 37"
    }
  ],
  "total_count": 23,
  "offset": 0,
  "limit": 50
}
```

**Backend Implementation:**
- Read metadata from `~/.ubertooth/captures/*.json`

**Authorization:** None

---

### Tool: capture_get

**Description:** Retrieve packet data from a capture with pagination.

**Category:** `bt-capture`

**Input Schema:**
```json
{
  "capture_id": "cap-btle-abc123",
  "offset": 0,      // Packet offset
  "limit": 100,     // Max packets to return
  "format": "json"  // "json" or "hex"
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
      "pkt_type": "LE_PACKET",
      "data_hex": "0201061AFF4C000215...",
      "parsed": {
        "pdu_type": "ADV_IND",
        "mac_address": "AA:BB:CC:DD:EE:FF",
        "adv_data": {...}
      }
    }
  ],
  "has_more": true
}
```

**Backend Implementation:**
- Parse PCAP file or read from JSON cache

**Authorization:** None

---

### Tool: capture_delete

**Description:** Delete a stored capture.

**Category:** `bt-capture`

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

**Backend Implementation:**
- Delete PCAP + JSON metadata files

**Authorization:** None

---

### Tool: capture_tag

**Description:** Add tags and notes to a capture.

**Category:** `bt-capture`

**Input Schema:**
```json
{
  "capture_id": "cap-btle-abc123",
  "tags": ["ble", "scan", "target_device"],
  "description": "Captured advertisements from target Fitbit",
  "append_tags": true  // Append vs replace (default: true)
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

**Backend Implementation:**
- Update JSON metadata file

**Authorization:** None

---

### Tool: capture_export

**Description:** Export capture to standard formats (PCAP, JSON, CSV).

**Category:** `bt-capture`

**Input Schema:**
```json
{
  "capture_id": "cap-btle-abc123",
  "format": "pcap",  // "pcap", "pcapng", "json", "csv"
  "output_path": "/tmp/export.pcap"  // Optional: defaults to captures dir
}
```

**Output Schema:**
```json
{
  "success": true,
  "export_path": "/tmp/export.pcap",
  "format": "pcap",
  "packet_count": 142,
  "file_size_bytes": 45320
}
```

**Backend Implementation:**
- Copy PCAP or convert JSON → PCAP/CSV

**Authorization:** None

---

## Category: bt-analysis

Packet analysis and comparison tools.

### Tool: bt_analyze

**Description:** Analyze captured packets and extract insights.

**Category:** `bt-analysis`

**Input Schema:**
```json
{
  "capture_id": "cap-btle-abc123",
  "analysis_type": "auto",  // "auto", "protocol", "timing", "security"
  "target_mac": null  // Optional: focus on specific device
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
      "version": "5.0",
      "pdu_types": {
        "ADV_IND": 45,
        "ADV_NONCONN_IND": 30,
        "SCAN_REQ": 12
      }
    },
    "devices": [
      {
        "mac_address": "AA:BB:CC:DD:EE:FF",
        "packet_count": 45,
        "device_name": "Fitbit Charge",
        "manufacturer": "Fitbit Inc.",
        "services": ["180F"],
        "security": {
          "address_randomization": true,
          "privacy": "resolvable_private"
        }
      }
    ],
    "timing_analysis": {
      "avg_interval_ms": 100,
      "min_interval_ms": 95,
      "max_interval_ms": 110,
      "regularity_score": 0.95
    },
    "security_observations": [
      "All devices use address randomization",
      "No legacy pairing observed",
      "Manufacturer data is unencrypted"
    ],
    "anomalies": []
  }
}
```

**Backend Implementation:**
- Parse PCAP with libbtbb (C) or pure Rust parser
- Statistical analysis
- Protocol fingerprinting

**Authorization:** None

---

### Tool: bt_compare

**Description:** Compare two captures to find differences (useful for replay attacks).

**Category:** `bt-analysis`

**Input Schema:**
```json
{
  "capture_id_a": "cap-btle-abc123",
  "capture_id_b": "cap-btle-def456",
  "compare_mode": "packets"  // "packets", "devices", "timing"
}
```

**Output Schema:**
```json
{
  "success": true,
  "comparison": {
    "mode": "packets",
    "similarity_percent": 87.5,
    "differences": [
      {
        "type": "packet_content",
        "packet_index_a": 5,
        "packet_index_b": 5,
        "field": "payload_byte_3",
        "value_a": "0x42",
        "value_b": "0x43",
        "interpretation": "Rolling code difference"
      }
    ],
    "unique_to_a": 12,
    "unique_to_b": 8,
    "common_packets": 122
  }
}
```

**Backend Implementation:**
- Load both captures
- Byte-by-byte or semantic comparison

**Authorization:** None

---

### Tool: bt_decode

**Description:** Decode specific Bluetooth packet types (L2CAP, ATT, SMP, etc.).

**Category:** `bt-analysis`

**Input Schema:**
```json
{
  "capture_id": "cap-btle-abc123",
  "packet_index": 5,  // Optional: specific packet, or decode all
  "protocol_layer": "auto"  // "auto", "l2cap", "att", "smp", "gatt"
}
```

**Output Schema:**
```json
{
  "success": true,
  "decoded_packets": [
    {
      "index": 5,
      "timestamp": "2026-02-26T15:30:00.123456Z",
      "layers": {
        "link_layer": {
          "pdu_type": "LL_DATA_PDU",
          "llid": "LL_DATA_START",
          "length": 27
        },
        "l2cap": {
          "channel_id": 4,
          "length": 23,
          "protocol": "ATT"
        },
        "att": {
          "opcode": "Read By Type Request",
          "uuid": "2A00",
          "uuid_name": "Device Name"
        }
      },
      "interpretation": "Client reading device name characteristic"
    }
  ]
}
```

**Backend Implementation:**
- Protocol dissectors for each layer
- libbtbb for BT Classic

**Authorization:** None

---

### Tool: bt_fingerprint

**Description:** Device fingerprinting based on protocol behavior.

**Category:** `bt-analysis`

**Input Schema:**
```json
{
  "capture_id": "cap-btle-abc123",
  "target_mac": "AA:BB:CC:DD:EE:FF"
}
```

**Output Schema:**
```json
{
  "success": true,
  "device": {
    "mac_address": "AA:BB:CC:DD:EE:FF",
    "fingerprint": {
      "manufacturer": "Apple Inc.",
      "device_type": "iPhone",
      "os_version": "iOS 17.x",
      "confidence": 0.92
    },
    "indicators": [
      "Manufacturer data prefix: 0x4C00 (Apple)",
      "Advertisement interval: 100ms (Apple-standard)",
      "Service UUIDs: Apple continuity services"
    ]
  }
}
```

**Backend Implementation:**
- Pattern matching against fingerprint database
- Heuristic analysis

**Authorization:** None

---

### Tool: pcap_merge

**Description:** Merge multiple captures into a single PCAP file.

**Category:** `bt-analysis`

**Input Schema:**
```json
{
  "capture_ids": ["cap-1", "cap-2", "cap-3"],
  "output_name": "merged_capture",
  "sort_by_timestamp": true
}
```

**Output Schema:**
```json
{
  "success": true,
  "capture_id": "cap-merged-abc123",
  "source_captures": 3,
  "total_packets": 450,
  "pcap_path": "/home/user/.ubertooth/captures/cap-merged-abc123.pcap"
}
```

**Backend Implementation:**
- Parse multiple PCAPs
- Sort by timestamp
- Write combined PCAP

**Authorization:** None

---

## Category: bt-attack

⚠️ **All tools in this category require explicit authorization**

### Tool: btle_inject

**Description:** Inject BLE packets into a connection.

**Category:** `bt-attack`

**Input Schema:**
```json
{
  "access_address": "0x8E89BED6",
  "channel": 37,
  "packet_hex": "0201061AFF4C00...",  // Raw packet data
  "repeat": 1  // Number of times to transmit
}
```

**Output Schema:**
```json
{
  "success": true,
  "packets_sent": 1,
  "access_address": "0x8E89BED6",
  "channel": 37,
  "message": "Packet injected successfully"
}
```

**Error Cases:**
- `UNAUTHORIZED` - Authorization required
- `TX_FAILED`

**Backend Implementation:**
- **Python:** `ubertooth-btle -t <data>`
- **Rust:** `cmd_btle_slave()` + packet construction

**Authorization:** 🔴 REQUIRED - Active RF transmission

---

### Tool: bt_jam

**Description:** Jam Bluetooth frequencies (denial of service).

**Category:** `bt-attack`

**Input Schema:**
```json
{
  "jam_mode": "continuous",  // "none", "once", "continuous"
  "channel": null,  // null = all channels, or specific channel
  "duration_sec": 10
}
```

**Output Schema:**
```json
{
  "success": true,
  "jam_mode": "continuous",
  "duration_sec": 10,
  "channels_jammed": 79,
  "message": "Jamming completed"
}
```

**Error Cases:**
- `UNAUTHORIZED` - Authorization required
- `ILLEGAL_OPERATION` - Jamming is illegal in most jurisdictions

**Backend Implementation:**
- **Python:** Not available in standard tools
- **Rust:** `cmd_set_jam_mode(mode)` + carrier wave TX

**Authorization:** 🔴🔴 STRICTLY REQUIRED - Highly regulated operation

⚠️ **WARNING:** Bluetooth jamming is illegal in most countries without proper authorization.

---

### Tool: btle_slave

**Description:** Act as a BLE peripheral/slave device.

**Category:** `bt-attack`

**Input Schema:**
```json
{
  "mac_address": "AA:BB:CC:DD:EE:FF",
  "adv_data": "0201061AFF4C00...",  // Advertisement payload
  "adv_interval_ms": 100,
  "connectable": true
}
```

**Output Schema:**
```json
{
  "success": true,
  "mac_address": "AA:BB:CC:DD:EE:FF",
  "advertising": true,
  "connections_received": 0,
  "message": "BLE peripheral mode active"
}
```

**Backend Implementation:**
- **Python:** Not available
- **Rust:** `cmd_btle_slave(mac)` + `cmd_le_set_adv_data()`

**Authorization:** 🔴 REQUIRED - Spoofing/impersonation risk

---

### Tool: btle_mitm

**Description:** Perform Man-in-the-Middle attack on BLE connection.

**Category:** `bt-attack`

**Input Schema:**
```json
{
  "target_mac": "AA:BB:CC:DD:EE:FF",
  "access_address": "0x8E89BED6",
  "duration_sec": 60,
  "intercept_mode": "passive"  // "passive" (log only) or "active" (inject)
}
```

**Output Schema:**
```json
{
  "success": true,
  "capture_id": "cap-mitm-abc123",
  "target_mac": "AA:BB:CC:DD:EE:FF",
  "packets_intercepted": 350,
  "packets_injected": 0,
  "connection_disrupted": false
}
```

**Backend Implementation:**
- Complex: requires connection following + packet injection
- **Rust:** Combine `btle_follow` + `btle_inject`

**Authorization:** 🔴🔴 STRICTLY REQUIRED - Active attack

---

### Tool: bt_spoof

**Description:** Spoof a Bluetooth device identity.

**Category:** `bt-attack`

**Input Schema:**
```json
{
  "spoof_mac": "AA:BB:CC:DD:EE:FF",
  "device_name": "Spoofed Device",
  "class_of_device": "0x5A020C",
  "duration_sec": 60
}
```

**Output Schema:**
```json
{
  "success": true,
  "spoof_mac": "AA:BB:CC:DD:EE:FF",
  "duration_sec": 60,
  "message": "Device identity spoofed"
}
```

**Backend Implementation:**
- Requires firmware support
- **Rust:** `cmd_set_bdaddr()` + inquiry scan mode

**Authorization:** 🔴 REQUIRED - Impersonation

---

## Category: bt-advanced

Advanced operations and low-level access.

### Tool: ubertooth_raw

**Description:** Send raw USB commands to Ubertooth (escape hatch for advanced users).

**Category:** `bt-advanced`

**Input Schema:**
```json
{
  "command": "UBERTOOTH_PING",  // Command name or numeric ID
  "command_id": 0,  // Optional: numeric command ID
  "data": "",  // Hex string of command data
  "timeout_ms": 5000
}
```

**Output Schema:**
```json
{
  "success": true,
  "command": "UBERTOOTH_PING",
  "response_hex": "00",
  "response_length": 1
}
```

**Error Cases:**
- `INVALID_COMMAND`
- `USB_TRANSFER_FAILED`

**Backend Implementation:**
- **Rust:** Direct `ubertooth_cmd_sync()` call

**Authorization:** ⚠️ WARNING - Direct hardware access

---

### Tool: firmware_update

**Description:** Update Ubertooth firmware via DFU.

**Category:** `bt-advanced`

**Input Schema:**
```json
{
  "firmware_path": "/path/to/ubertooth-one-R1.bin",
  "verify": true  // Verify after flashing
}
```

**Output Schema:**
```json
{
  "success": true,
  "old_version": "2020-08-R1",
  "new_version": "2020-12-R1",
  "message": "Firmware updated successfully. Replug device to activate."
}
```

**Error Cases:**
- `FIRMWARE_FILE_NOT_FOUND`
- `FLASH_FAILED`
- `VERIFY_FAILED`

**Backend Implementation:**
- **Python:** `ubertooth-dfu` wrapper
- **Rust:** DFU protocol implementation

**Authorization:** ⚠️ WARNING - Can brick device

---

## Implementation Priority

### Phase 1 (Week 1-2): Core Operations - Python Wrapper
1. ✅ device_connect
2. ✅ device_disconnect
3. ✅ device_status
4. ✅ session_context
5. ✅ btle_scan (most important!)
6. ✅ bt_specan
7. ✅ configure_channel
8. ✅ configure_modulation
9. ✅ configure_power
10. ✅ capture_list
11. ✅ capture_get
12. ✅ capture_delete
13. ✅ capture_tag
14. ✅ bt_analyze (basic)

**Deliverable: 14 working tools via Python wrapper**

### Phase 2 (Week 3-4): Advanced Recon
15. ✅ bt_scan
16. ✅ bt_follow
17. ✅ afh_analyze
18. ✅ bt_discover
19. ✅ btle_follow
20. ✅ configure_squelch
21. ✅ configure_leds
22. ✅ bt_save_config
23. ✅ bt_load_config
24. ✅ config_list
25. ✅ config_delete

**Deliverable: 25 tools total**

### Phase 3 (Week 5-6): Analysis Tools
26. ✅ bt_compare
27. ✅ bt_decode
28. ✅ bt_fingerprint
29. ✅ pcap_merge
30. ✅ capture_export

**Deliverable: 30 tools total**

### Phase 4 (Week 7-8): Attack Operations (WITH AUTHORIZATION)
31. ✅ btle_inject
32. ✅ bt_jam
33. ✅ btle_slave
34. ✅ btle_mitm
35. ✅ bt_spoof
36. ✅ ubertooth_raw

**Deliverable: All 36 tools complete**

### Phase 5 (Week 9-12): Native Rust USB Backend
- Reimplement core operations in Rust
- Performance optimization
- Backend selection: `UBERTOOTH_BACKEND=rust`

---

## GitHub Issue Template

```markdown
## Tool: [TOOL_NAME]

**Category:** [CATEGORY]

**Priority:** P[0-2]

**Estimated Time:** [X hours]

### Description
[Brief description from schema]

### Input Schema
```json
[Input JSON]
```

### Output Schema
```json
[Output JSON]
```

### Backend Implementation
- [ ] Python wrapper
- [ ] Native Rust (optional)

### Test Cases
- [ ] Happy path: [description]
- [ ] Error case: [description]
- [ ] Edge case: [description]

### Authorization
[None / WARNING / REQUIRED]

### Dependencies
- Depends on: #[issue numbers]
- Blocks: #[issue numbers]

### Acceptance Criteria
- [ ] Input validation works
- [ ] Output matches schema
- [ ] Error cases handled
- [ ] Unit tests written
- [ ] Integration test passes
- [ ] Documentation updated
```

---

## Summary

**Total Tools: 36**

| Category | Count | Auth Level |
|----------|-------|------------|
| bt-device | 4 | None |
| bt-config | 8 | None (power: WARNING) |
| bt-recon | 7 | None/WARNING |
| bt-capture | 5 | None |
| bt-analysis | 5 | None |
| bt-attack | 5 | REQUIRED 🔴 |
| bt-advanced | 2 | WARNING |

**Auth Summary:**
- 🟢 **None:** 29 tools (passive, read-only)
- 🟡 **WARNING:** 4 tools (configuration, monitoring)
- 🔴 **REQUIRED:** 5 tools (active attacks)

All schemas are ready for conversion to GitHub issues! 🚀
