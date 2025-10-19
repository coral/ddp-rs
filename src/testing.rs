//! Test utilities and helpers for ddp-rs
//!
//! This module provides helper functions and builders for creating test fixtures,
//! making it easier to write comprehensive tests across the codebase.

#![cfg(test)]

use crate::protocol::*;
use crate::packet::Packet;

/// Builder for creating test Headers with sensible defaults
pub struct HeaderBuilder {
    packet_type: PacketType,
    sequence_number: u8,
    pixel_config: PixelConfig,
    id: ID,
    offset: u32,
    length: u16,
    time_code: timecode::TimeCode,
}

impl Default for HeaderBuilder {
    fn default() -> Self {
        HeaderBuilder {
            packet_type: PacketType::default(),
            sequence_number: 1,
            pixel_config: PixelConfig::default(),
            id: ID::default(),
            offset: 0,
            length: 0,
            time_code: timecode::TimeCode(None),
        }
    }
}

impl HeaderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sequence_number(mut self, sequence_number: u8) -> Self {
        self.sequence_number = sequence_number;
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    pub fn length(mut self, length: u16) -> Self {
        self.length = length;
        self
    }

    pub fn with_timecode(mut self, timecode_value: u32) -> Self {
        self.packet_type.timecode = true;
        self.time_code = timecode::TimeCode(Some(timecode_value));
        self
    }

    pub fn with_push(mut self) -> Self {
        self.packet_type.push = true;
        self
    }

    pub fn build(self) -> Header {
        Header {
            packet_type: self.packet_type,
            sequence_number: self.sequence_number,
            pixel_config: self.pixel_config,
            id: self.id,
            offset: self.offset,
            length: self.length,
            time_code: self.time_code,
        }
    }
}

/// Builder for creating test Packets with sensible defaults
pub struct PacketBuilder {
    header: Header,
    data: Vec<u8>,
}

impl Default for PacketBuilder {
    fn default() -> Self {
        PacketBuilder {
            header: Header::default(),
            data: Vec::new(),
        }
    }
}

impl PacketBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn header(mut self, header: Header) -> Self {
        self.header = header;
        self
    }

    pub fn rgb_data(mut self, pixels: &[(u8, u8, u8)]) -> Self {
        let mut data = Vec::new();
        for (r, g, b) in pixels {
            data.push(*r);
            data.push(*g);
            data.push(*b);
        }
        self.data = data;
        self
    }

    pub fn build(self) -> Packet {
        Packet::from_data(self.header, &self.data)
    }
}

/// Creates a simple RGB pixel data array for testing
pub fn rgb_test_data(num_pixels: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(num_pixels * 3);
    for i in 0..num_pixels {
        data.push((i % 256) as u8);        // R
        data.push(((i * 2) % 256) as u8);  // G
        data.push(((i * 3) % 256) as u8);  // B
    }
    data
}

/// Creates a test packet with valid DDP header and optional data
pub fn create_test_packet(data_length: usize) -> Vec<u8> {
    let header = HeaderBuilder::new()
        .with_push()
        .length(data_length as u16)
        .build();

    let header_bytes: [u8; 10] = header.into();
    let mut packet = header_bytes.to_vec();
    packet.extend_from_slice(&rgb_test_data(data_length / 3));
    packet
}

/// Creates a test packet with timecode
pub fn create_test_packet_with_timecode(data_length: usize, timecode: u32) -> Vec<u8> {
    let header = HeaderBuilder::new()
        .with_push()
        .with_timecode(timecode)
        .length(data_length as u16)
        .build();

    let header_bytes: [u8; 14] = header.into();
    let mut packet = header_bytes.to_vec();
    packet.extend_from_slice(&rgb_test_data(data_length / 3));
    packet
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_builder_defaults() {
        let header = HeaderBuilder::new().build();
        assert_eq!(header.sequence_number, 1);
        assert_eq!(header.offset, 0);
        assert_eq!(header.length, 0);
    }

    #[test]
    fn test_header_builder_custom() {
        let header = HeaderBuilder::new()
            .sequence_number(5)
            .offset(100)
            .length(50)
            .with_push()
            .build();

        assert_eq!(header.sequence_number, 5);
        assert_eq!(header.offset, 100);
        assert_eq!(header.length, 50);
        assert!(header.packet_type.push);
    }

    #[test]
    fn test_header_builder_with_timecode() {
        let header = HeaderBuilder::new()
            .with_timecode(12345)
            .build();

        assert!(header.packet_type.timecode);
        assert_eq!(header.time_code.0, Some(12345));
    }

    #[test]
    fn test_rgb_test_data() {
        let data = rgb_test_data(2);
        assert_eq!(data.len(), 6);
        assert_eq!(data[0], 0);  // First pixel R
        assert_eq!(data[1], 0);  // First pixel G
        assert_eq!(data[2], 0);  // First pixel B
        assert_eq!(data[3], 1);  // Second pixel R
        assert_eq!(data[4], 2);  // Second pixel G
        assert_eq!(data[5], 3);  // Second pixel B
    }

    #[test]
    fn test_create_test_packet() {
        let packet = create_test_packet(6);
        assert!(packet.len() >= 10);  // At least header size
        assert_eq!(packet[0] & 0b01000000, 0b01000000);  // Push bit set
    }

    #[test]
    fn test_create_test_packet_with_timecode() {
        let packet = create_test_packet_with_timecode(6, 42);
        assert!(packet.len() >= 14);  // Header with timecode is 14 bytes
        assert_eq!(packet[0] & 0b00010000, 0b00010000);  // Timecode bit set
    }

    #[test]
    fn test_packet_builder() {
        let packet = PacketBuilder::new()
            .header(HeaderBuilder::new().with_push().build())
            .rgb_data(&[(255, 0, 0), (0, 255, 0), (0, 0, 255)])
            .build();

        assert_eq!(packet.data.len(), 9);  // 3 pixels * 3 bytes
        assert_eq!(packet.data[0], 255);   // First pixel red
        assert_eq!(packet.data[4], 255);   // Second pixel green
        assert_eq!(packet.data[8], 255);   // Third pixel blue
    }
}
