//! USB packet protocol structures and parsing.

use crate::constants::*;
use crate::error::{Result, UsbError};
use serde::{Deserialize, Serialize};

/// USB packet header structure (14 bytes).
///
/// This is the common header for all USB packets from Ubertooth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbPacketHeader {
    /// Packet type (PKT_TYPE_*)
    pub pkt_type: u8,

    /// Status/error code
    pub status: u8,

    /// Channel number
    pub channel: u8,

    /// Clock value (4 bytes, little-endian)
    pub clock: u32,

    /// RSSI value (signed)
    pub rssi: i8,

    /// Reserved bytes (6 bytes)
    pub reserved: [u8; 6],
}

impl UsbPacketHeader {
    /// Parse header from raw bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 14 {
            return Err(UsbError::InvalidPacket(format!(
                "Header too short: {} bytes",
                data.len()
            )));
        }

        Ok(Self {
            pkt_type: data[0],
            status: data[1],
            channel: data[2],
            clock: u32::from_le_bytes([data[3], data[4], data[5], data[6]]),
            rssi: data[7] as i8,
            reserved: [data[8], data[9], data[10], data[11], data[12], data[13]],
        })
    }

    /// Serialize header to bytes.
    pub fn to_bytes(&self) -> [u8; 14] {
        let mut bytes = [0u8; 14];
        bytes[0] = self.pkt_type;
        bytes[1] = self.status;
        bytes[2] = self.channel;
        let clock_bytes = self.clock.to_le_bytes();
        bytes[3..7].copy_from_slice(&clock_bytes);
        bytes[7] = self.rssi as u8;
        bytes[8..14].copy_from_slice(&self.reserved);
        bytes
    }
}

/// Complete USB packet (header + payload).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbPacket {
    /// Packet header
    pub header: UsbPacketHeader,

    /// Payload data (up to 50 bytes)
    pub payload: Vec<u8>,
}

impl UsbPacket {
    /// Parse packet from raw USB transfer.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 14 {
            return Err(UsbError::InvalidPacket(format!(
                "Packet too short: {} bytes",
                data.len()
            )));
        }

        let header = UsbPacketHeader::from_bytes(&data[0..14])?;
        let payload = data[14..].to_vec();

        Ok(Self { header, payload })
    }

    /// Get the full packet size.
    pub fn size(&self) -> usize {
        14 + self.payload.len()
    }

    /// Check if this is a status packet.
    pub fn is_status(&self) -> bool {
        self.header.pkt_type == PKT_TYPE_STATUS
    }

    /// Check if this is a spectrum analysis packet.
    pub fn is_specan(&self) -> bool {
        self.header.pkt_type == PKT_TYPE_SPECAN
    }

    /// Check if this is a BLE packet.
    pub fn is_ble(&self) -> bool {
        self.header.pkt_type == PKT_TYPE_LE_PACKET
    }

    /// Check if this is a BR/EDR packet.
    pub fn is_bredr(&self) -> bool {
        self.header.pkt_type == PKT_TYPE_BR_PACKET
    }
}

/// BLE packet data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlePacket {
    /// Access address (4 bytes)
    pub access_address: u32,

    /// PDU header
    pub pdu_header: u8,

    /// Payload length
    pub length: u8,

    /// Payload data
    pub payload: Vec<u8>,

    /// CRC (3 bytes)
    pub crc: [u8; 3],

    /// Metadata
    pub rssi: i8,
    pub channel: u8,
    pub timestamp: u32,
}

impl BlePacket {
    /// Parse BLE packet from USB packet payload.
    pub fn from_usb_packet(pkt: &UsbPacket) -> Result<Self> {
        if !pkt.is_ble() {
            return Err(UsbError::InvalidPacket(
                "Not a BLE packet".to_string(),
            ));
        }

        let payload = &pkt.payload;
        if payload.len() < 10 {
            return Err(UsbError::InvalidPacket(format!(
                "BLE payload too short: {} bytes",
                payload.len()
            )));
        }

        // Parse BLE packet structure
        let access_address = u32::from_le_bytes([
            payload[0],
            payload[1],
            payload[2],
            payload[3],
        ]);

        let pdu_header = payload[4];
        let length = payload[5];

        let payload_end = 6 + length as usize;
        if payload.len() < payload_end + 3 {
            return Err(UsbError::InvalidPacket(format!(
                "BLE packet incomplete: expected {} bytes, got {}",
                payload_end + 3,
                payload.len()
            )));
        }

        let ble_payload = payload[6..payload_end].to_vec();
        let crc = [
            payload[payload_end],
            payload[payload_end + 1],
            payload[payload_end + 2],
        ];

        Ok(Self {
            access_address,
            pdu_header,
            length,
            payload: ble_payload,
            crc,
            rssi: pkt.header.rssi,
            channel: pkt.header.channel,
            timestamp: pkt.header.clock,
        })
    }

    /// Get the advertiser address if this is an advertisement.
    pub fn advertiser_address(&self) -> Option<[u8; 6]> {
        // Check if this is an ADV_IND, ADV_DIRECT_IND, ADV_NONCONN_IND, or ADV_SCAN_IND
        let pdu_type = self.pdu_header & 0x0F;
        if pdu_type <= 0x06 && self.payload.len() >= 6 {
            let mut addr = [0u8; 6];
            addr.copy_from_slice(&self.payload[0..6]);
            Some(addr)
        } else {
            None
        }
    }

    /// Get the device name from advertisement data if present.
    pub fn device_name(&self) -> Option<String> {
        // Skip the address (6 bytes) and parse AD structures
        let ad_data = &self.payload.get(6..)?;

        let mut offset = 0;
        while offset < ad_data.len() {
            if offset + 1 >= ad_data.len() {
                break;
            }

            let length = ad_data[offset] as usize;
            if length == 0 {
                break;
            }

            let ad_type = ad_data[offset + 1];

            // 0x08 = Shortened Local Name, 0x09 = Complete Local Name
            if (ad_type == 0x08 || ad_type == 0x09) && offset + 2 + length - 1 <= ad_data.len() {
                let name_bytes = &ad_data[offset + 2..offset + 1 + length];
                if let Ok(name) = String::from_utf8(name_bytes.to_vec()) {
                    return Some(name);
                }
            }

            offset += 1 + length;
        }

        None
    }
}

/// Spectrum analysis data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumPoint {
    /// Frequency in MHz
    pub frequency_mhz: u16,

    /// RSSI value
    pub rssi: i8,

    /// Channel number
    pub channel: u8,
}

impl SpectrumPoint {
    /// Parse spectrum data from USB packet.
    pub fn from_usb_packet(pkt: &UsbPacket) -> Result<Vec<Self>> {
        if !pkt.is_specan() {
            return Err(UsbError::InvalidPacket(
                "Not a spectrum analysis packet".to_string(),
            ));
        }

        // Spectrum data is RSSI values for consecutive channels
        let mut points = Vec::new();
        let base_channel = pkt.header.channel;

        for (i, &rssi_byte) in pkt.payload.iter().enumerate() {
            let channel = base_channel + i as u8;
            let frequency_mhz = 2402 + (channel as u16 * 1); // Bluetooth frequency mapping

            points.push(SpectrumPoint {
                frequency_mhz,
                rssi: rssi_byte as i8,
                channel,
            });
        }

        Ok(points)
    }
}

/// Device information from GET_BOARD_ID and related commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Board ID (0=Zero, 1=One, 2=TC13Badge)
    pub board_id: u8,

    /// Firmware version string
    pub firmware_version: String,

    /// API version
    pub api_version: String,

    /// Serial number
    pub serial_number: String,

    /// Compile information
    pub compile_info: String,
}

impl DeviceInfo {
    /// Get board name from ID.
    pub fn board_name(&self) -> &'static str {
        match self.board_id {
            BOARD_ID_UBERTOOTH_ZERO => "Ubertooth Zero",
            BOARD_ID_UBERTOOTH_ONE => "Ubertooth One",
            BOARD_ID_TC13BADGE => "TC13 Badge",
            _ => "Unknown",
        }
    }

    /// Check if firmware meets minimum version requirement.
    pub fn is_firmware_compatible(&self) -> bool {
        // Simple string comparison (should work for YYYY-MM-RX format)
        self.firmware_version >= MIN_FIRMWARE_VERSION.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_header_parse() {
        let data = vec![
            2, 0, 37, 0x12, 0x34, 0x56, 0x78, 0xE0, 0, 0, 0, 0, 0, 0,
        ];
        let header = UsbPacketHeader::from_bytes(&data).unwrap();

        assert_eq!(header.pkt_type, 2);
        assert_eq!(header.status, 0);
        assert_eq!(header.channel, 37);
        assert_eq!(header.clock, 0x78563412);
        assert_eq!(header.rssi, -32);
    }

    #[test]
    fn test_packet_header_roundtrip() {
        let header = UsbPacketHeader {
            pkt_type: PKT_TYPE_LE_PACKET,
            status: 0,
            channel: 38,
            clock: 0xDEADBEEF,
            rssi: -45,
            reserved: [0; 6],
        };

        let bytes = header.to_bytes();
        let parsed = UsbPacketHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.pkt_type, header.pkt_type);
        assert_eq!(parsed.channel, header.channel);
        assert_eq!(parsed.clock, header.clock);
        assert_eq!(parsed.rssi, header.rssi);
    }

    #[test]
    fn test_usb_packet_parse() {
        let mut data = vec![2, 0, 37, 0, 0, 0, 0, 0xD0, 0, 0, 0, 0, 0, 0];
        data.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // Some payload

        let packet = UsbPacket::from_bytes(&data).unwrap();

        assert!(packet.is_ble());
        assert_eq!(packet.header.channel, 37);
        assert_eq!(packet.payload.len(), 4);
    }
}
