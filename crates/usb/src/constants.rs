//! USB protocol constants for Ubertooth One.
//!
//! Based on the Ubertooth One firmware specification.
//! Reference: https://github.com/greatscottgadgets/ubertooth/tree/master/firmware

/// Ubertooth One USB Vendor ID
pub const USB_VENDOR_ID: u16 = 0x1d50;

/// Ubertooth One USB Product ID
pub const USB_PRODUCT_ID: u16 = 0x6002;

/// USB endpoint for bulk data IN (device -> host)
pub const ENDPOINT_DATA_IN: u8 = 0x82;

/// USB endpoint for bulk data OUT (host -> device)
pub const ENDPOINT_DATA_OUT: u8 = 0x05;

/// USB control endpoint
pub const ENDPOINT_CONTROL: u8 = 0x00;

/// Maximum USB packet size (64 bytes)
pub const USB_PKT_SIZE: usize = 64;

/// Payload size within USB packet (50 bytes after 14-byte header)
pub const USB_PAYLOAD_SIZE: usize = 50;

/// Default USB timeout in milliseconds
pub const USB_TIMEOUT_MS: u64 = 20_000;

/// Short timeout for quick operations (ms)
pub const USB_TIMEOUT_SHORT_MS: u64 = 1_000;

/// Minimum firmware version required
pub const MIN_FIRMWARE_VERSION: &str = "2018-12-R1";

// USB Command Opcodes (Vendor Requests)
// These match the Ubertooth firmware command definitions

/// Ping command - test device responsiveness
pub const CMD_PING: u8 = 0;

/// Reset device
pub const CMD_RESET: u8 = 1;

/// Get device info (board ID, serial, firmware version)
pub const CMD_GET_BOARD_ID: u8 = 6;

/// Set modulation type
pub const CMD_SET_MODULATION: u8 = 7;

/// Get modulation type
pub const CMD_GET_MODULATION: u8 = 8;

/// Set channel
pub const CMD_SET_CHANNEL: u8 = 12;

/// Get channel
pub const CMD_GET_CHANNEL: u8 = 13;

/// Set transmit power
pub const CMD_SET_POWER: u8 = 14;

/// Get transmit power
pub const CMD_GET_POWER: u8 = 15;

/// LED control
pub const CMD_LED_SPECAN: u8 = 26;

/// Start spectrum analysis
pub const CMD_SPECAN: u8 = 27;

/// BLE promiscuous mode
pub const CMD_BTLE_PROMISC: u8 = 37;

/// BLE set target (follow connection)
pub const CMD_BTLE_SET_TARGET: u8 = 38;

/// BLE slave mode
pub const CMD_BTLE_SLAVE: u8 = 39;

/// Stop current operation
pub const CMD_STOP: u8 = 48;

/// Get firmware revision string
pub const CMD_GET_REV_NUM: u8 = 49;

/// Get compile info
pub const CMD_GET_COMPILE_INFO: u8 = 50;

/// Get serial number
pub const CMD_GET_SERIAL: u8 = 51;

/// Set squelch level
pub const CMD_SET_SQUELCH: u8 = 52;

/// Get squelch level
pub const CMD_GET_SQUELCH: u8 = 53;

/// Set access address for BLE
pub const CMD_BTLE_SET_ACCESS_ADDRESS: u8 = 54;

/// Get access address
pub const CMD_BTLE_GET_ACCESS_ADDRESS: u8 = 55;

/// Set CRC initialization
pub const CMD_BTLE_SET_CRC_INIT: u8 = 56;

/// Get CRC initialization
pub const CMD_BTLE_GET_CRC_INIT: u8 = 57;

/// Clear AFH map
pub const CMD_CLEAR_AFH_MAP: u8 = 58;

/// BLE sniff advertisements
pub const CMD_BTLE_SNIFF_AA: u8 = 59;

/// Get register value
pub const CMD_GET_REGISTER: u8 = 60;

/// Set register value
pub const CMD_SET_REGISTER: u8 = 61;

/// Get USB API version
pub const CMD_GET_API_VERSION: u8 = 62;

// Modulation Types

/// Bluetooth Basic Rate (BR) modulation
pub const MOD_BT_BASIC_RATE: u8 = 0;

/// Bluetooth Low Energy (BLE) modulation
pub const MOD_BT_LOW_ENERGY: u8 = 1;

/// 802.15.4 modulation
pub const MOD_80211_FHSS: u8 = 2;

// BLE Channels

/// BLE advertising channel 37 (2402 MHz)
pub const BLE_CHANNEL_37: u8 = 37;

/// BLE advertising channel 38 (2426 MHz)
pub const BLE_CHANNEL_38: u8 = 38;

/// BLE advertising channel 39 (2480 MHz)
pub const BLE_CHANNEL_39: u8 = 39;

/// BLE advertising access address (standard for all advertising)
pub const BLE_ADV_ACCESS_ADDRESS: u32 = 0x8E89BED6;

/// Minimum BLE data channel
pub const BLE_CHANNEL_MIN: u8 = 0;

/// Maximum BLE data channel
pub const BLE_CHANNEL_MAX: u8 = 39;

// Power Levels (approximate dBm values)

/// Minimum transmit power
pub const TX_POWER_MIN: i8 = -30;

/// Maximum transmit power
pub const TX_POWER_MAX: i8 = 20;

/// Default transmit power (0 dBm)
pub const TX_POWER_DEFAULT: i8 = 0;

// Board IDs

/// Ubertooth Zero
pub const BOARD_ID_UBERTOOTH_ZERO: u8 = 0;

/// Ubertooth One
pub const BOARD_ID_UBERTOOTH_ONE: u8 = 1;

/// TC13 Badge
pub const BOARD_ID_TC13BADGE: u8 = 2;

// USB Packet Types

/// Status packet
pub const PKT_TYPE_STATUS: u8 = 0;

/// Spectrum analysis data
pub const PKT_TYPE_SPECAN: u8 = 1;

/// BLE packet
pub const PKT_TYPE_LE_PACKET: u8 = 2;

/// BR/EDR packet
pub const PKT_TYPE_BR_PACKET: u8 = 3;

// USB Request Types (for control transfers)

/// Host to device, vendor request
pub const USB_REQ_TYPE_OUT: u8 = 0x40;

/// Device to host, vendor request
pub const USB_REQ_TYPE_IN: u8 = 0xC0;

// Streaming Configuration

/// Ring buffer size for packet streaming (must be power of 2)
pub const STREAM_RING_BUFFER_SIZE: usize = 8192;

/// Maximum packets to buffer before backpressure
pub const STREAM_MAX_BUFFERED_PACKETS: usize = 1024;

/// Number of concurrent USB transfers for streaming
pub const STREAM_CONCURRENT_TRANSFERS: usize = 4;

// PCAP Configuration

/// PCAP linktype for Bluetooth Low Energy
pub const PCAP_LINKTYPE_BLUETOOTH_LE_LL: u32 = 251;

/// PCAP linktype for Bluetooth BR/EDR
pub const PCAP_LINKTYPE_BLUETOOTH_HCI_H4: u32 = 201;

/// PCAP snapshot length (max packet size to capture)
pub const PCAP_SNAPLEN: u32 = 65535;
