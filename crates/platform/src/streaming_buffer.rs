//! Ring buffer for real-time packet streaming.
//!
//! Provides bounded memory usage with configurable limits and thread-safe access.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// Maximum buffer size in packets (default: 10,000)
pub const DEFAULT_MAX_PACKETS: usize = 10_000;

/// Maximum memory usage in bytes (default: 100MB)
pub const DEFAULT_MAX_MEMORY_BYTES: usize = 100 * 1024 * 1024;

/// Packet data stored in streaming buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketData {
    /// Packet sequence number
    pub sequence: u64,
    /// Timestamp when packet was received
    pub timestamp: DateTime<Utc>,
    /// Raw packet bytes
    pub data: Vec<u8>,
    /// Packet type (e.g., "LE_ADV", "LE_DATA", "BR_EDR")
    pub packet_type: String,
    /// Channel or frequency
    pub channel: u8,
    /// RSSI (signal strength)
    pub rssi: Option<i8>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl PacketData {
    /// Estimate memory usage of this packet
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.len()
            + self.packet_type.len()
            + self.metadata.to_string().len()
    }
}

/// Buffer statistics
#[derive(Debug, Clone, Default)]
pub struct BufferStats {
    /// Total packets received since start
    pub total_packets: u64,
    /// Packets currently in buffer
    pub buffered_packets: usize,
    /// Total bytes received
    pub total_bytes: u64,
    /// Current memory usage in bytes
    pub memory_usage: usize,
    /// Number of packets dropped due to buffer full
    pub dropped_packets: u64,
    /// Packets per second (rolling average)
    pub packets_per_second: f64,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// Last packet timestamp
    pub last_packet_at: Option<DateTime<Utc>>,
}

impl BufferStats {
    /// Calculate capture duration in seconds
    pub fn duration_seconds(&self) -> f64 {
        if let Some(last) = self.last_packet_at {
            (last - self.started_at).num_milliseconds() as f64 / 1000.0
        } else {
            0.0
        }
    }

    /// Calculate average bytes per packet
    pub fn avg_bytes_per_packet(&self) -> f64 {
        if self.total_packets > 0 {
            self.total_bytes as f64 / self.total_packets as f64
        } else {
            0.0
        }
    }
}

/// Buffer capacity limits
#[derive(Debug, Clone)]
pub struct CaptureLimits {
    /// Maximum packets in buffer
    pub max_packets: usize,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum capture duration in seconds (None = unlimited)
    pub max_duration_seconds: Option<u64>,
    /// Maximum total packets (None = unlimited)
    pub max_total_packets: Option<u64>,
}

impl Default for CaptureLimits {
    fn default() -> Self {
        Self {
            max_packets: DEFAULT_MAX_PACKETS,
            max_memory_bytes: DEFAULT_MAX_MEMORY_BYTES,
            max_duration_seconds: None,
            max_total_packets: None,
        }
    }
}

impl CaptureLimits {
    /// Check if limits have been reached
    pub fn is_exceeded(&self, stats: &BufferStats) -> bool {
        // Check duration limit
        if let Some(max_duration) = self.max_duration_seconds {
            if stats.duration_seconds() >= max_duration as f64 {
                return true;
            }
        }

        // Check total packets limit
        if let Some(max_total) = self.max_total_packets {
            if stats.total_packets >= max_total {
                return true;
            }
        }

        false
    }
}

/// Thread-safe ring buffer for streaming packets
pub struct StreamingBuffer {
    /// Packet ring buffer
    packets: Arc<RwLock<VecDeque<PacketData>>>,
    /// Buffer statistics
    stats: Arc<RwLock<BufferStats>>,
    /// Capacity limits
    limits: CaptureLimits,
    /// Next sequence number
    sequence: Arc<RwLock<u64>>,
}

impl StreamingBuffer {
    /// Create a new streaming buffer with default limits
    pub fn new() -> Self {
        Self::with_limits(CaptureLimits::default())
    }

    /// Create a new streaming buffer with custom limits
    pub fn with_limits(limits: CaptureLimits) -> Self {
        let mut stats = BufferStats::default();
        stats.started_at = Utc::now();

        Self {
            packets: Arc::new(RwLock::new(VecDeque::with_capacity(limits.max_packets))),
            stats: Arc::new(RwLock::new(stats)),
            limits,
            sequence: Arc::new(RwLock::new(0)),
        }
    }

    /// Push a packet into the buffer
    ///
    /// If buffer is full, oldest packet is dropped.
    pub fn push(&self, mut packet: PacketData) -> Result<(), String> {
        // Assign sequence number
        let mut seq = self.sequence.write().map_err(|e| e.to_string())?;
        packet.sequence = *seq;
        *seq += 1;
        drop(seq);

        let packet_size = packet.memory_size();
        let packet_data_len = packet.data.len();

        let mut packets = self.packets.write().map_err(|e| e.to_string())?;
        let mut stats = self.stats.write().map_err(|e| e.to_string())?;

        // Check if buffer is full
        let should_drop = packets.len() >= self.limits.max_packets
            || stats.memory_usage + packet_size > self.limits.max_memory_bytes;

        if should_drop && !packets.is_empty() {
            // Drop oldest packet
            if let Some(dropped) = packets.pop_front() {
                stats.memory_usage = stats.memory_usage.saturating_sub(dropped.memory_size());
                stats.dropped_packets += 1;
            }
        }

        // Add new packet
        packets.push_back(packet);

        // Update statistics
        stats.total_packets += 1;
        stats.buffered_packets = packets.len();
        stats.total_bytes += packet_data_len as u64;
        stats.memory_usage += packet_size;
        stats.last_packet_at = Some(Utc::now());

        // Update packets per second (simple moving average)
        let duration = stats.duration_seconds();
        if duration > 0.0 {
            stats.packets_per_second = stats.total_packets as f64 / duration;
        }

        Ok(())
    }

    /// Get all packets currently in buffer
    pub fn get_packets(&self) -> Result<Vec<PacketData>, String> {
        let packets = self.packets.read().map_err(|e| e.to_string())?;
        Ok(packets.iter().cloned().collect())
    }

    /// Get packets in a specific range
    pub fn get_packets_range(&self, start: usize, count: usize) -> Result<Vec<PacketData>, String> {
        let packets = self.packets.read().map_err(|e| e.to_string())?;
        Ok(packets
            .iter()
            .skip(start)
            .take(count)
            .cloned()
            .collect())
    }

    /// Get the most recent N packets
    pub fn get_recent_packets(&self, count: usize) -> Result<Vec<PacketData>, String> {
        let packets = self.packets.read().map_err(|e| e.to_string())?;
        let total = packets.len();
        let start = if total > count { total - count } else { 0 };
        Ok(packets.iter().skip(start).cloned().collect())
    }

    /// Get current statistics
    pub fn get_stats(&self) -> Result<BufferStats, String> {
        let stats = self.stats.read().map_err(|e| e.to_string())?;
        Ok(stats.clone())
    }

    /// Get buffer limits
    pub fn get_limits(&self) -> &CaptureLimits {
        &self.limits
    }

    /// Clear all packets from buffer
    pub fn clear(&self) -> Result<(), String> {
        let mut packets = self.packets.write().map_err(|e| e.to_string())?;
        let mut stats = self.stats.write().map_err(|e| e.to_string())?;

        packets.clear();
        stats.buffered_packets = 0;
        stats.memory_usage = 0;

        Ok(())
    }

    /// Reset statistics but keep packets
    pub fn reset_stats(&self) -> Result<(), String> {
        let mut stats = self.stats.write().map_err(|e| e.to_string())?;

        stats.total_packets = stats.buffered_packets as u64;
        stats.total_bytes = 0;
        stats.dropped_packets = 0;
        stats.packets_per_second = 0.0;
        stats.started_at = Utc::now();
        stats.last_packet_at = None;

        Ok(())
    }

    /// Check if limits have been exceeded
    pub fn is_limit_exceeded(&self) -> Result<bool, String> {
        let stats = self.stats.read().map_err(|e| e.to_string())?;
        Ok(self.limits.is_exceeded(&stats))
    }

    /// Get current buffer size (number of packets)
    pub fn len(&self) -> Result<usize, String> {
        let packets = self.packets.read().map_err(|e| e.to_string())?;
        Ok(packets.len())
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> Result<bool, String> {
        let packets = self.packets.read().map_err(|e| e.to_string())?;
        Ok(packets.is_empty())
    }
}

impl Default for StreamingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for StreamingBuffer {
    fn clone(&self) -> Self {
        Self {
            packets: Arc::clone(&self.packets),
            stats: Arc::clone(&self.stats),
            limits: self.limits.clone(),
            sequence: Arc::clone(&self.sequence),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_packet(data_size: usize) -> PacketData {
        PacketData {
            sequence: 0,
            timestamp: Utc::now(),
            data: vec![0u8; data_size],
            packet_type: "LE_ADV".to_string(),
            channel: 37,
            rssi: Some(-50),
            metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn test_buffer_creation() {
        let buffer = StreamingBuffer::new();
        assert!(buffer.is_empty().unwrap());
        assert_eq!(buffer.len().unwrap(), 0);
    }

    #[test]
    fn test_push_packet() {
        let buffer = StreamingBuffer::new();
        let packet = create_test_packet(100);

        buffer.push(packet).unwrap();

        assert_eq!(buffer.len().unwrap(), 1);
        assert!(!buffer.is_empty().unwrap());

        let stats = buffer.get_stats().unwrap();
        assert_eq!(stats.total_packets, 1);
        assert_eq!(stats.buffered_packets, 1);
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let limits = CaptureLimits {
            max_packets: 5,
            ..Default::default()
        };
        let buffer = StreamingBuffer::with_limits(limits);

        // Add 10 packets (buffer max is 5)
        for _ in 0..10 {
            buffer.push(create_test_packet(100)).unwrap();
        }

        // Should only have 5 packets (oldest dropped)
        assert_eq!(buffer.len().unwrap(), 5);

        let stats = buffer.get_stats().unwrap();
        assert_eq!(stats.total_packets, 10);
        assert_eq!(stats.buffered_packets, 5);
        assert_eq!(stats.dropped_packets, 5);
    }

    #[test]
    fn test_get_recent_packets() {
        let buffer = StreamingBuffer::new();

        // Add 10 packets
        for _ in 0..10 {
            buffer.push(create_test_packet(100)).unwrap();
        }

        // Get 3 most recent
        let recent = buffer.get_recent_packets(3).unwrap();
        assert_eq!(recent.len(), 3);

        // Sequences should be 7, 8, 9
        assert_eq!(recent[0].sequence, 7);
        assert_eq!(recent[1].sequence, 8);
        assert_eq!(recent[2].sequence, 9);
    }

    #[test]
    fn test_clear_buffer() {
        let buffer = StreamingBuffer::new();

        for _ in 0..5 {
            buffer.push(create_test_packet(100)).unwrap();
        }

        assert_eq!(buffer.len().unwrap(), 5);

        buffer.clear().unwrap();

        assert_eq!(buffer.len().unwrap(), 0);
        assert!(buffer.is_empty().unwrap());
    }

    #[test]
    fn test_memory_limit() {
        let limits = CaptureLimits {
            max_packets: 1000,
            max_memory_bytes: 1000, // Very small limit
            ..Default::default()
        };
        let buffer = StreamingBuffer::with_limits(limits);

        // Add packets until memory limit hit
        for _ in 0..10 {
            buffer.push(create_test_packet(200)).unwrap();
        }

        // Should have dropped packets due to memory limit
        let stats = buffer.get_stats().unwrap();
        assert!(stats.dropped_packets > 0);
        assert!(stats.memory_usage <= 1000);
    }

    #[test]
    fn test_stats_calculation() {
        let buffer = StreamingBuffer::new();

        for _ in 0..5 {
            buffer.push(create_test_packet(100)).unwrap();
        }

        let stats = buffer.get_stats().unwrap();
        assert_eq!(stats.total_packets, 5);
        assert_eq!(stats.total_bytes, 500);
        assert!(stats.packets_per_second >= 0.0);
        assert!(stats.last_packet_at.is_some());
    }
}
