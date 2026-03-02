//! USB constants for Ubertooth One
//!
//! CORRECTED from official GitHub source:
//! https://github.com/greatscottgadgets/ubertooth/blob/master/host/libubertooth/src/ubertooth_interface.h

// USB IDs
pub const USB_VENDOR_ID: u16 = 0x1d50;
pub const USB_PRODUCT_ID: u16 = 0x6002;

// USB Endpoints
pub const ENDPOINT_DATA_IN: u8 = 0x82;
pub const ENDPOINT_DATA_OUT: u8 = 0x05;

// USB Request Types
pub const USB_REQ_TYPE_OUT: u8 = 0x40;
pub const USB_REQ_TYPE_IN: u8 = 0xC0;

// USB Packet Size
pub const USB_PKT_SIZE: usize = 64;

// Timeouts
pub const USB_TIMEOUT_SHORT_MS: u64 = 1000;
pub const USB_TIMEOUT_LONG_MS: u64 = 20000;

// Firmware Version
pub const MIN_FIRMWARE_VERSION: &str = "2018-12-R1";

// TX Power Limits
pub const TX_POWER_MIN: i8 = -30;
pub const TX_POWER_MAX: i8 = 20;
pub const TX_POWER_DEFAULT: i8 = 0;

// Board IDs
pub const BOARD_ID_UBERTOOTH_ZERO: u8 = 0;
pub const BOARD_ID_UBERTOOTH_ONE: u8 = 1;
pub const BOARD_ID_TC13BADGE: u8 = 2;

// USB Commands (from official ubertooth_interface.h)
pub const CMD_PING: u8 = 0;
pub const CMD_RX_SYMBOLS: u8 = 1;
pub const CMD_TX_SYMBOLS: u8 = 2;
pub const CMD_GET_USRLED: u8 = 3;
pub const CMD_SET_USRLED: u8 = 4;
pub const CMD_GET_RXLED: u8 = 5;
pub const CMD_SET_RXLED: u8 = 6;
pub const CMD_GET_TXLED: u8 = 7;
pub const CMD_SET_TXLED: u8 = 8;
pub const CMD_GET_1V8: u8 = 9;
pub const CMD_SET_1V8: u8 = 10;
pub const CMD_GET_CHANNEL: u8 = 11;
pub const CMD_SET_CHANNEL: u8 = 12;
pub const CMD_RESET: u8 = 13;
pub const CMD_GET_SERIAL: u8 = 14;
pub const CMD_GET_PARTNUM: u8 = 15;
pub const CMD_GET_PAEN: u8 = 16;
pub const CMD_SET_PAEN: u8 = 17;
pub const CMD_GET_HGM: u8 = 18;
pub const CMD_SET_HGM: u8 = 19;
pub const CMD_TX_TEST: u8 = 20;
pub const CMD_STOP: u8 = 21;
pub const CMD_GET_MOD: u8 = 22;
pub const CMD_SET_MODULATION: u8 = 23;
pub const CMD_SET_ISP: u8 = 24;
pub const CMD_FLASH: u8 = 25;
pub const CMD_BOOTLOADER_FLASH: u8 = 26;
pub const CMD_SPECAN: u8 = 27;
pub const CMD_GET_PALEVEL: u8 = 28;
pub const CMD_SET_PALEVEL: u8 = 29;
pub const CMD_REPEATER: u8 = 30;
pub const CMD_RANGE_TEST: u8 = 31;
pub const CMD_RANGE_CHECK: u8 = 32;
pub const CMD_GET_REV_NUM: u8 = 33;
pub const CMD_LED_SPECAN: u8 = 34;
pub const CMD_GET_BOARD_ID: u8 = 35;
pub const CMD_SET_SQUELCH: u8 = 36;
pub const CMD_GET_SQUELCH: u8 = 37;
pub const CMD_SET_BDADDR: u8 = 38;
pub const CMD_START_HOPPING: u8 = 39;
pub const CMD_SET_CLOCK: u8 = 40;
pub const CMD_GET_CLOCK: u8 = 41;
pub const CMD_BTLE_SNIFFING: u8 = 42;
pub const CMD_GET_ACCESS_ADDRESS: u8 = 43;
pub const CMD_SET_ACCESS_ADDRESS: u8 = 44;
pub const CMD_DO_SOMETHING: u8 = 45;
pub const CMD_DO_SOMETHING_REPLY: u8 = 46;
pub const CMD_GET_CRC_VERIFY: u8 = 47;
pub const CMD_SET_CRC_VERIFY: u8 = 48;
pub const CMD_POLL: u8 = 49;
pub const CMD_BTLE_PROMISC: u8 = 50;
pub const CMD_SET_AFHMAP: u8 = 51;
pub const CMD_CLEAR_AFHMAP: u8 = 52;
pub const CMD_READ_REGISTER: u8 = 53;
pub const CMD_BTLE_SLAVE: u8 = 54;
pub const CMD_GET_COMPILE_INFO: u8 = 55;
pub const CMD_BTLE_SET_TARGET: u8 = 56;
pub const CMD_BTLE_PHY: u8 = 57;
pub const CMD_WRITE_REGISTER: u8 = 58;
pub const CMD_JAM_MODE: u8 = 59;
pub const CMD_EGO: u8 = 60;
pub const CMD_AFH: u8 = 61;
pub const CMD_HOP: u8 = 62;
pub const CMD_TRIM_CLOCK: u8 = 63;
pub const CMD_WRITE_REGISTERS: u8 = 65;
pub const CMD_READ_ALL_REGISTERS: u8 = 66;
pub const CMD_RX_GENERIC: u8 = 67;
pub const CMD_TX_GENERIC_PACKET: u8 = 68;
pub const CMD_FIX_CLOCK_DRIFT: u8 = 69;
pub const CMD_CANCEL_FOLLOW: u8 = 70;
pub const CMD_LE_SET_ADV_DATA: u8 = 71;
pub const CMD_RFCAT_SUBCMD: u8 = 72;
pub const CMD_XMAS: u8 = 73;

// Legacy/Aliased Commands (may not exist in firmware, kept for compatibility)
pub const CMD_SET_POWER: u8 = CMD_SET_PALEVEL; // Alias for PA level
pub const CMD_GET_API_VERSION: u8 = CMD_HOP; // TODO: Verify this mapping

// Jam Modes
pub const JAM_NONE: u8 = 0;
pub const JAM_ONCE: u8 = 1;
pub const JAM_CONTINUOUS: u8 = 2;

// Modulation Types
pub const MOD_BT_BASIC_RATE: u8 = 0;
pub const MOD_BT_LOW_ENERGY: u8 = 1;
pub const MOD_80211_FHSS: u8 = 2;

// BLE Constants
pub const BLE_ADV_ACCESS_ADDRESS: u32 = 0x8E89BED6;
pub const BLE_CHANNEL_37: u8 = 37;
pub const BLE_CHANNEL_38: u8 = 38;
pub const BLE_CHANNEL_39: u8 = 39;
pub const BLE_CHANNEL_MIN: u8 = 0;
pub const BLE_CHANNEL_MAX: u8 = 39;

// Packet Types
pub const PKT_TYPE_STATUS: u8 = 0;
pub const PKT_TYPE_LE_PACKET: u8 = 1;  // BLE packets - FIXED!
pub const PKT_TYPE_BR_PACKET: u8 = 2;  // BR/EDR packets
pub const PKT_TYPE_SPECAN: u8 = 3;  // Spectrum analysis (old format)
pub const PKT_TYPE_SPECAN_RAW: u8 = 4;  // Spectrum analysis (raw format with 09 markers)
