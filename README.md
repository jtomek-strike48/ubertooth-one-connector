# Ubertooth One Connector

Connect an [Ubertooth One](https://greatscottgadgets.com/ubertoothone/) Bluetooth monitoring device to [Prospector Studio](https://prospectorstudio.com) and let the AI drive it as a tool.

The connector exposes 36 Bluetooth operations — scanning, sniffing, analyzing, and more — as AI-callable tools through the [Strike48](https://github.com/jtomek/sdk-rs) connector SDK.

---

## What the Agent Can Do

- **Scan** — Discover BLE and Bluetooth Classic devices
- **Sniff** — Capture BLE advertisements and connection packets
- **Analyze** — Decode protocols, fingerprint devices, extract insights
- **Follow** — Track specific connections and analyze AFH patterns
- **Spectrum Analysis** — Visualize 2.4 GHz ISM band activity
- **Attack Operations** — Packet injection, jamming, MITM (requires authorization)
- **Configure** — Set channels, power, modulation, save presets
- **Manage captures** — List, page through, tag, and delete stored captures

All captures auto-save to `~/.ubertooth/captures/` on the connector machine and survive session restarts.

---

## Prerequisites

**Hardware**

- [Ubertooth One](https://greatscottgadgets.com/ubertoothone/) USB device
- Firmware version 2020-12-R1 or newer

**Software**

```bash
# Install ubertooth tools
sudo apt-get install ubertooth

# Or build from source:
git clone https://github.com/greatscottgadgets/ubertooth.git
cd ubertooth/host
mkdir build && cd build
cmake ..
make && sudo make install
```

**Build from source** (Rust toolchain required)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Hardware Setup

Install udev rules so Ubertooth One is accessible without root:

```bash
just install-udev
# Replug the device after running this
```

Verify the device is found:

```bash
lsusb | grep -i "1d50:6002"
# 1d50:6002 OpenMoko, Inc. Ubertooth One
```

---

## Running

### Connect to Prospector Studio

```bash
ubertooth-agent \
  --server-url wss://studio.example.com:443 \
  --tenant-id your-tenant \
  --auth-token your-jwt-token
```

| Flag | Short | Description |
|------|-------|-------------|
| `--server-url` | `-s` | Studio server URL — must use `wss://` or `ws://` |
| `--tenant-id` | `-t` | Tenant ID from your Studio account |
| `--auth-token` | | JWT authentication token |
| `--insecure` | `-k` | Skip TLS certificate verification (self-signed certs) |
| `--log-level` | | `trace`, `debug`, `info`, `warn`, `error` (default: `info`) |

### Environment variables (alternative to flags)

```bash
export STRIKE48_URL=wss://studio.example.com:443
export TENANT_ID=your-tenant
export AUTH_TOKEN=your-jwt-token
ubertooth-agent
```

---

## Backend Selection & Performance

The connector supports **two backends**:

1. **Python Wrapper** (Phase 1) ✅ **IMPLEMENTED** - Wraps ubertooth-tools CLI utilities (14 tools in v0.1.0)
2. **Rust USB** (Phase 3, future) - Native implementation via libusb (7-10 tools, **100-200x faster**)

### Choosing a Backend

**Python Backend** (default, Phase 1):
- 14 Phase 1 tools implemented (device, recon, config, capture, analysis)
- Proven, stable, battle-tested
- Requires ubertooth-tools installed
- All captures stored as PCAP + JSON metadata in `~/.ubertooth/`

**Rust Backend** (Phase 3, planned):
- Core operations only (device, scan, sniff, specan)
- **100-200x faster** than Python for supported operations
- No Python dependency, pure Rust + libusb
- Lower memory footprint

Set the backend via environment variable:

```bash
# Use Python backend (full features, default)
UBERTOOTH_BACKEND=python ubertooth-agent

# Use Rust backend (high performance, Phase 3)
UBERTOOTH_BACKEND=rust ubertooth-agent
```

---

## Capture Storage

Captures and configs are stored on the connector machine:

```
~/.ubertooth/
  captures/   # Bluetooth capture files (PCAP format)
  configs/    # Named radio configurations (saved via bt_save_config)
```

Each capture is identified by a UUID (`capture_id`). The AI holds capture IDs in context and passes them between tool calls for analysis, comparison, and tagging. Use `capture_get` with `offset`/`limit` to page through large captures.

---

## Tool Categories

**Phase 1 (v0.1.0): 14 tools** ✅

| Category | Phase 1 | Total | Description |
|----------|---------|-------|-------------|
| **bt-device** | 4/4 ✅ | 4 | Device connection and status |
| **bt-config** | 3/8 ✅ | 8 | Radio configuration and presets |
| **bt-recon** | 2/7 ✅ | 7 | Scanning and signal discovery |
| **bt-capture** | 4/5 ✅ | 5 | Capture management and storage |
| **bt-analysis** | 1/5 ✅ | 5 | Protocol analysis and fingerprinting |
| **bt-attack** | 0/5 | 5 | Active operations (Phase 2, requires authorization) |
| **bt-advanced** | 0/2 | 2 | Raw commands and firmware updates (Phase 2) |

**Phase 2: 22 more tools planned** (advanced recon, analysis, attack operations)

See [TOOL_SCHEMAS.md](TOOL_SCHEMAS.md) for complete tool specifications and [PRD.md](PRD.md) for the phased roadmap.

---

## Documentation

### Getting Started
- **[Tool Schemas](TOOL_SCHEMAS.md)** - Complete API reference for all 36 tools
- **[Rust Feasibility Analysis](LIBUBERTOOTH_RUST_FEASIBILITY.md)** - Native Rust implementation deep dive

### Development
- **[CLAUDE.md](CLAUDE.md)** - Architecture and development guide (coming soon)
- **[PRD.md](PRD.md)** - Product requirements document (coming soon)

---

## Building from Source

```bash
just build          # debug build
just build-release  # optimized release build
just run            # cargo run (connects to Studio using env vars)
just test           # unit + integration tests (no hardware required)
just test-hardware  # hardware smoke tests (requires Ubertooth One)
just ci             # run all validation checks locally
```

---

## Security & Authorization

### Tool Authorization Levels

🟢 **None (29 tools)** - Passive operations, read-only
- Scanning, sniffing, analysis, configuration

🟡 **WARNING (4 tools)** - Configuration changes, targeted monitoring
- TX power adjustment, connection following

🔴 **REQUIRED (5 tools)** - Active RF operations
- Packet injection, jamming, MITM, spoofing

**All attack operations require explicit authorization and audit logging.**

See [AUTHORIZATION.md](docs/AUTHORIZATION.md) for security guidelines (coming soon).

---

## Troubleshooting

### Device not found

1. Check the device is plugged in: `lsusb | grep 1d50:6002`
2. Check udev rules are installed: `just install-udev`, then replug.
3. Check your user is in the `plugdev` group: `groups $USER`
   If not: `sudo usermod -aG plugdev $USER` and log out/in.

### `ubertooth-* not found`

```bash
sudo apt-get install ubertooth
# Or build from source (see Prerequisites)
```

### Permission denied on USB device

Re-run `just install-udev` and replug the device.

---

## Project Status

**Version: v0.1.0 (Phase 1 Complete) 🎉**

- ✅ Template project analyzed (yardstick-one-connector)
- ✅ libubertooth C API analyzed for Rust feasibility
- ✅ 36 tool schemas designed
- ✅ PRD document complete
- ✅ **Phase 1 implementation complete: 14/14 tools working**
  - All device management, recon, config, capture, and analysis tools
  - Python backend wrapper for ubertooth-tools
  - Capture storage with PCAP + JSON metadata
  - Full test coverage (28 tests passing)
- ⏳ Phase 2 (next): Advanced tools + PCAP parsing
- ⏳ Phase 3: Native Rust USB backend (100-200x faster)

See [PRD.md](PRD.md) for complete roadmap and [LIBUBERTOOTH_RUST_FEASIBILITY.md](LIBUBERTOOTH_RUST_FEASIBILITY.md) for implementation details.

---

## For Developers

Based on the [yardstick-one-connector](https://github.com/jtomek/yardstick-one-connector) template with the following adaptations:

**Similarities:**
- Strike48 connector SDK integration
- Dual backend architecture (Python sidecar + Rust USB)
- Category-based tool organization
- Capture storage and management
- Configuration presets

**Differences:**
- Protocol: Bluetooth (2.4 GHz) vs sub-1 GHz RF
- Library: libubertooth (C) vs rfcat (Python)
- USB interface: Control + bulk transfers vs serial-like
- Security: Stricter authorization for BT operations
- Analysis: BLE/BR protocol dissectors vs generic RF analysis

---

## License

MIT

---

## Acknowledgments

- [Great Scott Gadgets](https://greatscottgadgets.com/) for Ubertooth One hardware and software
- [libbtbb](https://github.com/greatscottgadgets/libbtbb) for Bluetooth baseband library
- [yardstick-one-connector](https://github.com/jtomek/yardstick-one-connector) for the connector template
