# Device Fingerprinting

Device fingerprinting identifies Bluetooth/BLE devices based on their advertising data, packet patterns, and behavioral characteristics.

## Overview

The fingerprinting system matches observed device characteristics against a signature database to identify:
- **Manufacturer** (e.g., Apple Inc., Samsung Electronics)
- **Device Type** (e.g., iPhone, Galaxy Watch, AirPods)
- **OS Family** (e.g., iOS, Android, watchOS)
- **Confidence Score** (0.0-1.0)

## Architecture

### Components

1. **FingerprintEngine** (`crates/platform/src/fingerprint.rs`)
   - Signature matching engine
   - OUI (Organizationally Unique Identifier) lookup
   - Confidence scoring

2. **Device Signatures** (`resources/device_signatures.json`)
   - Extensible signature database
   - 20 built-in signatures for common devices
   - JSON format for easy updates

3. **bt_fingerprint Tool** (`crates/tools/src/bt_fingerprint.rs`)
   - CLI tool interface
   - PCAP parsing integration

## Usage

### Basic Fingerprinting

```bash
# Fingerprint a device from capture
ubertooth-cli bt_fingerprint --capture-id cap-ble-scan-123 --target-mac AA:BB:CC:DD:EE:FF
```

### Programmatic Usage

```rust
use ubertooth_platform::{FingerprintEngine, FingerprintPacketData};

let engine = FingerprintEngine::new();

let packet = FingerprintPacketData {
    mac_address: Some("00:17:F2:11:22:33".to_string()),
    manufacturer_data: Some(ManufacturerData {
        company_id: 0x004C,  // Apple
        data: vec![0x02, 0x01, 0x05],
    }),
    ..Default::default()
};

if let Some(fp) = engine.fingerprint(&packet) {
    println!("Device: {} {}", fp.manufacturer, fp.device_type);
    println!("Confidence: {:.0}%", fp.confidence * 100.0);
    println!("Indicators: {:?}", fp.indicators);
}
```

## Matching Rules

Signatures use multiple rule types for robust matching:

### 1. OUI Match
Matches the first 3 bytes of MAC address (manufacturer identifier).

```json
{
  "type": "Oui",
  "prefix": "00:17:F2"
}
```

**Example OUIs:**
- `00:17:F2` - Apple Inc.
- `F4:F5:E8` - Google LLC
- `C8:F2:30` - Samsung Electronics

### 2. Manufacturer Data
Matches Bluetooth SIG company ID in advertising data.

```json
{
  "type": "ManufacturerData",
  "company_id": 76
}
```

**Common Company IDs:**
- `76` (0x004C) - Apple Inc.
- `224` (0x00E0) - Google LLC
- `117` (0x0075) - Samsung Electronics

### 3. Service UUID
Matches advertised service UUIDs.

```json
{
  "type": "ServiceUuid",
  "uuid": "180f"
}
```

### 4. Device Name Pattern
Matches device name substring.

```json
{
  "type": "DeviceName",
  "pattern": "AirPods"
}
```

### 5. Advertising Interval
Matches advertising interval range (milliseconds).

```json
{
  "type": "AdvertisingInterval",
  "min_ms": 100,
  "max_ms": 200
}
```

### 6. TX Power
Matches exact TX power level.

```json
{
  "type": "TxPower",
  "value": 4
}
```

### 7. Flags
Matches BLE advertising flags byte.

```json
{
  "type": "Flags",
  "value": 6
}
```

## Signature Format

Signatures are defined in JSON:

```json
{
  "id": "apple-iphone",
  "manufacturer": "Apple Inc.",
  "device_type": "iPhone",
  "os_family": "iOS",
  "rules": [
    { "type": "ManufacturerData", "company_id": 76 },
    { "type": "Oui", "prefix": "00:17:F2" }
  ]
}
```

### Fields

- **id** (string) - Unique signature identifier
- **manufacturer** (string) - Manufacturer name
- **device_type** (string) - Device type/model
- **os_family** (string|null) - Operating system family
- **rules** (array) - Match rules (AND logic)

## Confidence Scoring

Confidence is calculated as: `matches / total_rules`

- **1.0 (100%)** - All rules match
- **0.75 (75%)** - 3 of 4 rules match
- **0.50 (50%)** - Minimum for positive identification
- **< 0.50** - No match returned

Minimum confidence threshold: **50%**

## Built-in Signatures

### Apple Devices
- iPhone (3 variants for different OUIs)
- AirPods
- AirPods Pro
- Apple Watch

### Android Devices
- Google Pixel Phone
- Google Pixel Buds
- Samsung Galaxy Phone (2 variants)
- Samsung Galaxy Watch
- Samsung Galaxy Buds
- Xiaomi Mi Band (2 variants)

### Other Devices
- Microsoft Surface
- Sony WH-1000XM Headphones
- Amazon Echo
- Google Nest Thermostat
- Fitbit Fitness Tracker
- Tile Tracker

## Adding Custom Signatures

### 1. Edit `resources/device_signatures.json`

Add new signature to array:

```json
{
  "id": "my-device",
  "manufacturer": "Custom Manufacturer",
  "device_type": "Smart Device",
  "os_family": null,
  "rules": [
    { "type": "Oui", "prefix": "AA:BB:CC" },
    { "type": "DeviceName", "pattern": "MyDevice" }
  ]
}
```

### 2. Reload Application

No recompilation needed - signatures are loaded at runtime.

### 3. Load from Custom File

```rust
let engine = FingerprintEngine::from_json(&std::fs::read_to_string("my_signatures.json")?)?;
```

## Packet Data Extraction

The system extracts BLE advertising data from PCAP files:

### Supported AD Types

| Type | Description | Field |
|------|-------------|-------|
| 0x01 | Flags | `flags` |
| 0x08/0x09 | Local Name | `device_name` |
| 0x0A | TX Power | `tx_power` |
| 0x02/0x03 | Service UUIDs (16-bit) | `service_uuids` |
| 0xFF | Manufacturer Data | `manufacturer_data` |

### Example Advertising Packet

```
AD Structure 1: [length=02][type=01][flags=06]
AD Structure 2: [length=0C][type=09][name="AirPods Pro"]
AD Structure 3: [length=1A][type=FF][company=4C00][data=...]
```

Parsed as:

```rust
PacketData {
    mac_address: Some("AA:BB:CC:DD:EE:FF"),
    device_name: Some("AirPods Pro"),
    flags: Some(0x06),
    manufacturer_data: Some(ManufacturerData {
        company_id: 0x004C,
        data: vec![...],
    }),
    ...
}
```

## API Response Format

```json
{
  "success": true,
  "device": {
    "mac_address": "AA:BB:CC:DD:EE:FF",
    "fingerprint": {
      "manufacturer": "Apple Inc.",
      "device_type": "AirPods Pro",
      "os_version": null,
      "confidence": 1.0
    },
    "indicators": [
      "Manufacturer data: 0x004C",
      "Device name pattern: AirPods"
    ]
  }
}
```

### Fallback Response (OUI Only)

If no signature matches but OUI is known:

```json
{
  "success": true,
  "device": {
    "mac_address": "AA:BB:CC:DD:EE:FF",
    "fingerprint": {
      "manufacturer": "Apple Inc.",
      "device_type": "Unknown",
      "os_version": null,
      "confidence": 0.5
    },
    "indicators": ["OUI match only"]
  }
}
```

## Performance

- **Signature matching:** O(n) where n = number of signatures
- **OUI lookup:** O(1) hash table lookup
- **PCAP parsing:** Single pass through file
- **Memory:** ~50KB for signature database

Typical fingerprinting time: **< 10ms** for small captures

## Limitations

1. **BLE Only** - Currently supports BLE advertising packets only (Classic Bluetooth support planned)
2. **Static Signatures** - Doesn't detect behavioral patterns or dynamic characteristics
3. **Single Packet** - Uses first matching packet (doesn't aggregate multiple packets)
4. **No Passive OS Fingerprinting** - Doesn't analyze timing or protocol quirks

## Future Enhancements

- [ ] Classic Bluetooth (BR/EDR) support
- [ ] Behavioral fingerprinting (connection patterns, timing)
- [ ] Statistical analysis across multiple packets
- [ ] Machine learning-based classification
- [ ] Passive OS fingerprinting
- [ ] Integration with online signature databases
- [ ] Fuzzy matching for partial signatures

## References

- [Bluetooth SIG Assigned Numbers](https://www.bluetooth.com/specifications/assigned-numbers/)
- [BLE Advertising Data Format](https://www.bluetooth.com/specifications/specs/core-specification/)
- [IEEE OUI Database](https://standards-oui.ieee.org/)
- [Ubertooth One Documentation](https://ubertooth.readthedocs.io/)

## Testing

Run fingerprinting tests:

```bash
# Unit tests
cargo test --package ubertooth-platform --lib fingerprint

# Integration test with real capture
ubertooth-cli btle_scan --duration 10
ubertooth-cli bt_fingerprint --capture-id <id> --target-mac <mac>
```

---

**Last Updated:** 2026-03-18
**Phase:** 3.1 - Device Fingerprinting
