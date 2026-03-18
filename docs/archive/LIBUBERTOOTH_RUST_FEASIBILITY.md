# libubertooth C API → Rust Implementation Feasibility Analysis

## Executive Summary

**Verdict: ✅ HIGHLY FEASIBLE**

Native Rust implementation is completely viable using `rusb` crate (Rust libusb 1.0 bindings). The libubertooth API is well-structured, documented, and follows standard USB patterns that map cleanly to Rust.

---

## API Architecture Overview

### USB Communication Layer

libubertooth uses **libusb 1.0** for all USB communication:

```c
// Device identification
#define U1_VENDORID    0x1d50
#define U1_PRODUCTID   0x6002

// USB endpoints
#define DATA_IN     (0x82 | LIBUSB_ENDPOINT_IN)   // Bulk IN for packet RX
#define DATA_OUT    (0x05 | LIBUSB_ENDPOINT_OUT)  // Bulk OUT for packet TX
#define TIMEOUT     20000                          // 20 second timeout
```

**Rust Equivalent:**
```rust
const U1_VENDOR_ID: u16 = 0x1d50;
const U1_PRODUCT_ID: u16 = 0x6002;
const DATA_IN: u8 = 0x82;
const DATA_OUT: u8 = 0x05;
const TIMEOUT: Duration = Duration::from_secs(20);

// Using rusb crate
use rusb::{Context, DeviceHandle, Direction, TransferType};
```

---

## Core Data Structures

### 1. Main Device Handle

```c
typedef struct {
    fifo_t* fifo;                           // Packet buffer
    struct libusb_device_handle* devh;      // USB device handle
    struct libusb_transfer* rx_xfer;        // Async RX transfer
    uint8_t stop_ubertooth;                 // Shutdown flag
    uint64_t abs_start_ns;                  // Timing reference
    uint32_t start_clk100ns;
    uint64_t last_clk100ns;
    uint64_t clk100ns_upper;
    btbb_pcap_handle* h_pcap_bredr;         // PCAP handles
    lell_pcap_handle* h_pcap_le;
    btbb_pcapng_handle* h_pcapng_bredr;
    lell_pcapng_handle* h_pcapng_le;
} ubertooth_t;
```

**Rust Equivalent:**
```rust
pub struct UbertoothDevice {
    handle: DeviceHandle<rusb::GlobalContext>,
    rx_buffer: VecDeque<UsbPacketRx>,          // FIFO replacement
    shutdown: Arc<AtomicBool>,
    timing: TimingContext,
    pcap_writer: Option<PcapWriter>,
}

struct TimingContext {
    abs_start_ns: u64,
    start_clk100ns: u32,
    last_clk100ns: u64,
    clk100ns_upper: u64,
}
```

### 2. USB Packet Format (64 bytes)

```c
#define DMA_SIZE 50

typedef struct {
    uint8_t  pkt_type;       // BR_PACKET, LE_PACKET, MESSAGE, etc.
    uint8_t  status;         // DMA_OVERFLOW, FIFO_OVERFLOW, etc.
    uint8_t  channel;        // BT channel (0-78)
    uint8_t  clkn_high;
    uint32_t clk100ns;
    int8_t   rssi_max;       // Max RSSI
    int8_t   rssi_min;       // Min RSSI
    int8_t   rssi_avg;       // Average RSSI
    uint8_t  rssi_count;
    uint8_t  reserved[2];
    uint8_t  data[DMA_SIZE]; // 50 bytes of packet data
} usb_pkt_rx;
```

**Rust Equivalent:**
```rust
const DMA_SIZE: usize = 50;
const PKT_LEN: usize = 64;

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct UsbPacketRx {
    pub pkt_type: PacketType,
    pub status: PacketStatus,
    pub channel: u8,
    pub clkn_high: u8,
    pub clk100ns: u32,
    pub rssi_max: i8,
    pub rssi_min: i8,
    pub rssi_avg: i8,
    pub rssi_count: u8,
    pub reserved: [u8; 2],
    pub data: [u8; DMA_SIZE],
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PacketType {
    BrPacket = 0,
    LePacket = 1,
    Message = 2,
    KeepAlive = 3,
    Specan = 4,
    LePromisc = 5,
    EgoPacket = 6,
}

bitflags! {
    pub struct PacketStatus: u8 {
        const DMA_OVERFLOW = 0x01;
        const DMA_ERROR = 0x02;
        const FIFO_OVERFLOW = 0x04;
        const CS_TRIGGER = 0x08;
        const RSSI_TRIGGER = 0x10;
        const DISCARD = 0x20;
    }
}
```

---

## USB Command Interface

### Command Enumeration (73 commands)

```c
enum ubertooth_usb_commands {
    UBERTOOTH_PING               = 0,
    UBERTOOTH_RX_SYMBOLS         = 1,
    UBERTOOTH_TX_SYMBOLS         = 2,
    UBERTOOTH_GET_CHANNEL        = 11,
    UBERTOOTH_SET_CHANNEL        = 12,
    UBERTOOTH_RESET              = 13,
    UBERTOOTH_SPECAN             = 27,
    UBERTOOTH_BTLE_SNIFFING      = 42,
    UBERTOOTH_BTLE_PROMISC       = 50,
    UBERTOOTH_JAM_MODE           = 59,
    // ... 73 total commands
};
```

**Rust Equivalent:**
```rust
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum UbertoothCommand {
    Ping = 0,
    RxSymbols = 1,
    TxSymbols = 2,
    GetChannel = 11,
    SetChannel = 12,
    Reset = 13,
    Specan = 27,
    BtleSniffing = 42,
    BtlePromisc = 50,
    JamMode = 59,
    // ...
}
```

### Command Execution Pattern

```c
int ubertooth_cmd_sync(struct libusb_device_handle* devh,
                       uint8_t type,
                       uint8_t command,
                       uint8_t* data,
                       uint16_t size)
{
    return libusb_control_transfer(devh,
        type,                    // bmRequestType
        command,                 // bRequest
        0, 0,                    // wValue, wIndex
        data, size,              // data buffer
        TIMEOUT);                // timeout
}
```

**Rust Equivalent:**
```rust
impl UbertoothDevice {
    fn cmd_sync(
        &self,
        request_type: u8,
        command: UbertoothCommand,
        data: &mut [u8],
    ) -> Result<usize, UbertoothError> {
        let bytes_transferred = self.handle.write_control(
            request_type,
            command as u8,
            0,      // wValue
            0,      // wIndex
            data,
            TIMEOUT,
        )?;
        Ok(bytes_transferred)
    }

    fn cmd_ping(&self) -> Result<(), UbertoothError> {
        let mut buf = [0u8; 0];
        self.cmd_sync(0x40, UbertoothCommand::Ping, &mut buf)?;
        Ok(())
    }

    fn cmd_set_channel(&self, channel: u16) -> Result<(), UbertoothError> {
        let mut buf = channel.to_le_bytes();
        self.cmd_sync(0x40, UbertoothCommand::SetChannel, &mut buf)?;
        Ok(())
    }
}
```

---

## Bulk Transfer Pattern (Packet Reception)

### C Implementation

```c
// Callback for async bulk transfer
static void cb_xfer(struct libusb_transfer *xfer) {
    ubertooth_t* ut = (ubertooth_t*)xfer->user_data;

    if (xfer->status != LIBUSB_TRANSFER_COMPLETED) {
        // handle error
        return;
    }

    fifo_inc_write_ptr(ut->fifo);
    ut->rx_xfer->buffer = (uint8_t*)fifo_get_write_element(ut->fifo);

    libusb_submit_transfer(ut->rx_xfer);  // re-submit
}

int ubertooth_bulk_init(ubertooth_t* ut) {
    ut->rx_xfer = libusb_alloc_transfer(0);
    libusb_fill_bulk_transfer(
        ut->rx_xfer,
        ut->devh,
        DATA_IN,
        (uint8_t*)fifo_get_write_element(ut->fifo),
        PKT_LEN,
        cb_xfer,
        ut,
        TIMEOUT
    );
    return libusb_submit_transfer(ut->rx_xfer);
}
```

**Rust Equivalent:**
```rust
impl UbertoothDevice {
    pub async fn start_bulk_rx(&mut self) -> Result<(), UbertoothError> {
        let mut buffer = vec![0u8; PKT_LEN];

        loop {
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }

            let bytes_read = self.handle.read_bulk(
                DATA_IN,
                &mut buffer,
                TIMEOUT,
            )?;

            if bytes_read == PKT_LEN {
                let packet = self.parse_packet(&buffer)?;
                self.rx_buffer.push_back(packet);
            }
        }

        Ok(())
    }

    fn parse_packet(&self, buf: &[u8]) -> Result<UsbPacketRx, UbertoothError> {
        if buf.len() != PKT_LEN {
            return Err(UbertoothError::InvalidPacketSize);
        }

        // Safe because we've verified the size and repr(C, packed)
        let packet: UsbPacketRx = unsafe {
            std::ptr::read(buf.as_ptr() as *const UsbPacketRx)
        };

        Ok(packet)
    }
}
```

---

## Key Operations Mapping

### Device Enumeration

| C API | Rust Equivalent |
|-------|----------------|
| `libusb_get_device_list()` | `rusb::Context::new()?.devices()?.iter()` |
| `libusb_get_device_descriptor()` | `device.device_descriptor()?` |
| `libusb_open()` | `device.open()?` |
| Vendor ID: `0x1d50`, Product ID: `0x6002` | Same constants |

### Control Commands

| Operation | C Function | Rust Method |
|-----------|-----------|-------------|
| Ping device | `cmd_ping(devh)` | `device.cmd_ping()` |
| Set channel | `cmd_set_channel(devh, chan)` | `device.cmd_set_channel(chan)` |
| Get serial | `cmd_get_serial(devh, serial)` | `device.cmd_get_serial()` |
| Reset | `cmd_reset(devh)` | `device.cmd_reset()` |
| Stop | `cmd_stop(devh)` | `device.cmd_stop()` |

### BLE-Specific Commands

| Operation | C Function | Rust Method |
|-----------|-----------|-------------|
| Start BLE sniffing | `cmd_btle_sniffing(devh, do_follow)` | `device.start_btle_sniffing(follow)` |
| Promiscuous mode | `cmd_btle_promisc(devh)` | `device.btle_promiscuous()` |
| Set access address | `cmd_set_access_address(devh, aa)` | `device.set_access_address(aa)` |
| Set target MAC | `cmd_btle_set_target(devh, mac, mask)` | `device.set_btle_target(mac, mask)` |

### Spectrum Analysis

| Operation | C Function | Rust Method |
|-----------|-----------|-------------|
| Start spectrum scan | `cmd_specan(devh, low, high)` | `device.start_specan(low, high)` |
| LED spectrum | `cmd_led_specan(devh, thresh)` | `device.led_specan(threshold)` |

---

## Rust Crate Dependencies

```toml
[dependencies]
rusb = "0.9"                    # libusb 1.0 bindings
tokio = { version = "1", features = ["full"] }
thiserror = "1"
anyhow = "1"
bitflags = "2"
pcap-file = "2"                 # PCAP/PCAPNG writing
serde = { version = "1", features = ["derive"] }
tracing = "0.1"
```

---

## Implementation Phases

### Phase 1: Core USB Layer (Rust Native)
**Estimate: 2-3 days**

- ✅ Device enumeration and connection
- ✅ Control command infrastructure
- ✅ Basic commands: ping, reset, get_serial, set_channel
- ✅ Packet structure definitions
- ✅ Error types

**Files:**
```
crates/usb/
  src/
    device.rs       # UbertoothDevice struct
    commands.rs     # Command enum + implementations
    protocol.rs     # USB packet structures
    error.rs        # Error types
    constants.rs    # USB IDs, endpoints, timeouts
```

### Phase 2: Bulk Transfer & Packet Reception
**Estimate: 2-3 days**

- ✅ Async bulk IN transfers
- ✅ Packet parsing and validation
- ✅ Ring buffer for packet queue
- ✅ RSSI statistics handling
- ✅ Timeout and error recovery

### Phase 3: BLE Operations
**Estimate: 3-4 days**

- ✅ BLE sniffing (promiscuous + targeted)
- ✅ Access address configuration
- ✅ CRC verification
- ✅ Channel hopping
- ✅ Advertisement parsing

### Phase 4: Spectrum Analysis
**Estimate: 2 days**

- ✅ Spectrum scan command
- ✅ RSSI data collection
- ✅ Frequency sweeping

### Phase 5: PCAP Integration
**Estimate: 2 days**

- ✅ PCAP/PCAPNG file writing
- ✅ BLE packet formatting for Wireshark
- ✅ Timestamp synchronization

### Phase 6: Advanced Features
**Estimate: 3-4 days**

- ✅ AFH (Adaptive Frequency Hopping) analysis
- ✅ Jamming mode (requires authorization)
- ✅ Slave/peripheral mode
- ✅ Advertisement injection

---

## Advantages of Rust Implementation

### 1. **Memory Safety**
- No buffer overflows (Rust compiler prevents)
- No use-after-free bugs
- Thread-safe by default

### 2. **Better Error Handling**
```rust
// C: returns -1, sets errno, hard to propagate
int result = cmd_ping(devh);
if (result < 0) { /* handle error */ }

// Rust: explicit Result type, composable with ?
let result = device.cmd_ping()?;  // Auto-propagates errors
```

### 3. **Async/Await for Packet Processing**
```rust
// Non-blocking packet reception
tokio::spawn(async move {
    while let Some(packet) = device.recv_packet().await? {
        process_packet(packet).await?;
    }
});
```

### 4. **Type Safety**
```rust
// C: everything is uint8_t*, easy to mix up
uint8_t* data = malloc(50);
cmd_do_something(devh, data, 50);

// Rust: strongly typed
let packet: UsbPacketRx = device.recv_packet()?;
match packet.pkt_type {
    PacketType::LePacket => { /* handle BLE */ }
    PacketType::BrPacket => { /* handle Classic */ }
}
```

### 5. **Zero-Cost Abstractions**
- `#[repr(C, packed)]` structs compile to identical layout as C
- No runtime overhead vs C implementation
- Better optimization opportunities

---

## Challenges & Mitigations

### Challenge 1: Callback-based libusb → Rust async

**C Pattern:**
```c
void cb_xfer(struct libusb_transfer *xfer) {
    // Called from libusb thread
    process_packet(xfer->buffer);
}
```

**Rust Solution:**
```rust
// Use tokio channels instead of callbacks
let (tx, mut rx) = mpsc::channel(100);

std::thread::spawn(move || {
    loop {
        let mut buffer = [0u8; PKT_LEN];
        handle.read_bulk(DATA_IN, &mut buffer, TIMEOUT)?;
        tx.send(buffer).await?;
    }
});

// Async consumer
while let Some(buffer) = rx.recv().await {
    let packet = parse_packet(&buffer)?;
    process(packet).await?;
}
```

### Challenge 2: FIFO buffer management

**C Pattern:**
```c
fifo_t* fifo = fifo_init();
void* element = fifo_get_write_element(fifo);
```

**Rust Solution:**
```rust
use std::collections::VecDeque;

struct PacketFifo {
    buffer: VecDeque<UsbPacketRx>,
    capacity: usize,
}

impl PacketFifo {
    fn push(&mut self, packet: UsbPacketRx) -> Result<(), FifoError> {
        if self.buffer.len() >= self.capacity {
            return Err(FifoError::Full);
        }
        self.buffer.push_back(packet);
        Ok(())
    }

    fn pop(&mut self) -> Option<UsbPacketRx> {
        self.buffer.pop_front()
    }
}
```

### Challenge 3: External dependencies (libbtbb for packet parsing)

**Option A: FFI bindings to libbtbb**
```rust
// Use bindgen to generate Rust FFI bindings
#[link(name = "btbb")]
extern "C" {
    fn btbb_init(max_ac_errors: i32) -> i32;
    fn btbb_packet_decode(...) -> *mut btbb_packet;
}
```

**Option B: Pure Rust implementation**
- Rewrite BT baseband parsing in Rust
- More work but better long-term maintainability
- **Recommended approach**

---

## Comparison: Python Wrapper vs Native Rust

### Python Wrapper Approach (Phase 1)

**Pros:**
- ✅ Fast initial development (1-2 weeks)
- ✅ Reuses existing ubertooth-tools binaries
- ✅ Proven, stable implementation

**Cons:**
- ❌ Python + subprocess overhead (~50ms per command)
- ❌ Parsing text output from CLI tools
- ❌ No streaming packet access (tools write to files)
- ❌ Python dependency required

### Native Rust Approach (Phase 2+)

**Pros:**
- ✅ Direct USB access (0.5-1ms latency)
- ✅ Streaming packet processing
- ✅ Zero Python dependency
- ✅ Type-safe, memory-safe
- ✅ Better error handling
- ✅ **100-200x faster than Python wrapper**

**Cons:**
- ❌ More upfront development (3-4 weeks)
- ❌ Need to reimplement packet parsing
- ❌ Requires libusb system dependency

---

## Recommendation

### Hybrid Approach (Best of Both Worlds)

**Phase 1: Python Sidecar** (Week 1-2)
- Wrap `ubertooth-btle`, `ubertooth-scan`, `ubertooth-specan` CLI tools
- Get basic functionality working quickly
- Ship ~15-20 tools

**Phase 2: Native Rust USB** (Week 3-6)
- Implement core operations in native Rust
- Focus on high-frequency operations:
  - BLE sniffing (streaming)
  - Spectrum analysis (real-time)
  - Device control (low-latency)
- Keep Python wrapper as fallback for advanced features

**Backend Selection:**
```bash
# High-performance streaming
UBERTOOTH_BACKEND=rust ubertooth-agent

# Full feature set
UBERTOOTH_BACKEND=python ubertooth-agent
```

---

## Conclusion

**Native Rust implementation is not only feasible but RECOMMENDED.**

The libubertooth C API is well-designed and maps cleanly to Rust idioms. Using `rusb` crate provides excellent libusb 1.0 bindings with zero overhead. The hybrid approach (Python wrapper first, then native Rust) gives us:

1. **Fast time-to-market** (Python wrapper ships in 2 weeks)
2. **Performance path** (Native Rust for critical operations)
3. **Fallback option** (Python for edge cases)
4. **Type safety** (Rust prevents entire classes of bugs)

**Estimated Timeline:**
- Python sidecar: 2 weeks → 20 tools working
- Native Rust core: 4 weeks → 7-10 core operations at 100-200x speed
- Advanced Rust features: 2 weeks → Full parity with C implementation

**Total: 8 weeks to full native Rust parity**

But we can ship incrementally starting at Week 2. ✅
