// Protocol specification: http://www.3waylabs.com/ddp/

//! DDP protocol types and structures.
//!
//! This module contains all the types defined by the Distributed Display Protocol specification,
//! including headers, packet types, pixel configurations, and control messages.
//!
//! # Protocol Overview
//!
//! The DDP protocol uses a 10 or 14 byte header (depending on whether timecode is included)
//! followed by pixel data or JSON control messages.
//!
//! ## Header Structure (10 bytes)
//!
//! - Byte 0: Packet type flags (version, timecode, storage, reply, query, push)
//! - Byte 1: Sequence number (1-15, wraps around)
//! - Byte 2: Pixel format configuration
//! - Byte 3: Protocol ID
//! - Bytes 4-7: Data offset (32-bit, big-endian)
//! - Bytes 8-9: Data length (16-bit, big-endian)
//! - Bytes 10-13: Optional timecode (32-bit, big-endian) if timecode flag is set

pub mod packet_type;
pub use packet_type::*;

pub mod pixel_config;
pub use pixel_config::{DataType, PixelConfig, PixelFormat};

pub mod id;
pub use id::ID;

pub mod message;

pub mod timecode;
use timecode::TimeCode;

/// DDP packet header containing metadata and control flags.
///
/// The header is 10 bytes (or 14 with timecode) and contains all the information
/// needed to interpret the packet payload.
///
/// # Examples
///
/// ```
/// use ddp_rs::protocol::{Header, PacketType, PixelConfig, ID, timecode::TimeCode};
///
/// let header = Header {
///     packet_type: PacketType::default(),
///     sequence_number: 1,
///     pixel_config: PixelConfig::default(),
///     id: ID::Default,
///     offset: 0,
///     length: 9,
///     time_code: TimeCode(None),
/// };
///
/// // Convert to bytes for transmission
/// let bytes: [u8; 10] = header.into();
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub struct Header {
    /// Packet type flags (version, timecode, storage, reply, query, push)
    pub packet_type: PacketType,

    /// Sequence number (1-15, wraps around to 1)
    pub sequence_number: u8,

    /// Pixel format configuration (RGB, RGBW, etc.)
    pub pixel_config: PixelConfig,

    /// Protocol message ID
    pub id: ID,

    /// Byte offset into the display buffer
    pub offset: u32,

    /// Length of data in this packet (in bytes)
    pub length: u16,

    /// Optional timecode for synchronization (if timecode flag is set)
    pub time_code: TimeCode,
}

impl Into<[u8; 10]> for Header {
    fn into(self) -> [u8; 10] {
        // Define a byte array with the size of the header
        let mut buffer: [u8; 10] = [0u8; 10];

        // Write the packet type field to the buffer

        let packet_type_byte: u8 = self.packet_type.into();
        buffer[0] = packet_type_byte;

        // Write the sequence number field to the buffer
        buffer[1] = self.sequence_number;

        // Write the pixel config field to the buffer
        buffer[2] = self.pixel_config.into();

        // Write the id field to the buffer
        buffer[3] = self.id.into();

        // Write the offset field to the buffer
        let offset_bytes = self.offset.to_be_bytes();
        buffer[4..8].copy_from_slice(&offset_bytes);

        // Write the length field to the buffer
        let length_bytes = self.length.to_be_bytes();
        buffer[8..10].copy_from_slice(&length_bytes);

        // Return a slice of the buffer representing the entire header
        buffer
    }
}
impl Into<[u8; 14]> for Header {
    fn into(self) -> [u8; 14] {
        // Define a byte array with the size of the header
        let mut buffer = [0u8; 14];

        // Write the packet type field to the buffer

        let packet_type_byte: u8 = self.packet_type.into();
        buffer[0] = packet_type_byte;

        // Write the sequence number field to the buffer
        buffer[1] = self.sequence_number;

        // Write the pixel config field to the buffer
        buffer[2] = self.pixel_config.into();

        // Write the id field to the buffer
        buffer[3] = self.id.into();

        // Write the offset field to the buffer
        let offset_bytes: [u8; 4] = self.offset.to_be_bytes();
        buffer[4..8].copy_from_slice(&offset_bytes);

        // Write the length field to the buffer
        let length_bytes: [u8; 2] = self.length.to_be_bytes();
        buffer[8..10].copy_from_slice(&length_bytes);

        let time_code: [u8; 4] = self.time_code.to_bytes();
        buffer[10..14].copy_from_slice(&time_code);

        // Return a slice of the buffer representing the entire header
        buffer
    }
}

impl<'a> From<&'a [u8]> for Header {
    fn from(bytes: &'a [u8]) -> Self {
        // Extract the packet type field from the buffer
        let packet_type = PacketType::from(bytes[0]);

        // Extract the sequence number field from the buffer
        let sequence_number = bytes[1];

        // Extract the pixel config field from the buffer
        let pixel_config = PixelConfig::from(bytes[2]);

        // Extract the id field from the buffer
        let id = ID::from(bytes[3]);

        // Extract the offset field from the buffer
        let offset = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        // Extract the length field from the buffer
        let length = u16::from_be_bytes([bytes[8], bytes[9]]);

        if packet_type.timecode && bytes.len() >= 14 {
            let time_code = TimeCode::from_4_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);

            Header {
                packet_type,
                sequence_number,
                pixel_config,
                id,
                offset,
                length,
                time_code,
            }
        } else {
            Header {
                packet_type,
                sequence_number,
                pixel_config,
                id,
                offset,
                length,
                time_code: TimeCode(None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        // Normal
        {
            let data: [u8; 10] = [65, 6, 10, 1, 0, 0, 0, 0, 0, 3];
            let header = Header::from(&data[..]);

            assert_eq!(
                header.packet_type,
                PacketType {
                    version: 1,
                    timecode: false,
                    storage: false,
                    reply: false,
                    query: false,
                    push: true
                }
            );
            assert_eq!(header.sequence_number, 6);
            assert_eq!(header.length, 3);
            assert_eq!(header.offset, 0);
        }

        // oddity
        {
            let data: [u8; 10] = [255, 12, 13, 1, 0, 0, 0x99, 0xd5, 0x01, 0x19];
            let header = Header::from(&data[..]);

            assert_eq!(
                header.packet_type,
                PacketType {
                    version: 3,
                    timecode: true,
                    storage: true,
                    reply: true,
                    query: true,
                    push: true
                }
            );

            assert_eq!(header.sequence_number, 12);
            assert_eq!(
                header.pixel_config,
                PixelConfig {
                    data_type: pixel_config::DataType::RGB,
                    data_size: PixelFormat::Pixel24Bits,
                    customer_defined: false
                }
            );
            assert_eq!(header.length, 281);
            assert_eq!(header.offset, 39381);
        }
    }

    // Property-based tests
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_header_10_byte_roundtrip(
            packet_type_byte in any::<u8>(),
            seq_num in any::<u8>(),
            pixel_config in any::<u8>(),
            id in any::<u8>(),
            offset in any::<u32>(),
            length in any::<u16>(),
        ) {
            // Create a 10-byte header from arbitrary values
            let mut bytes = vec![packet_type_byte, seq_num, pixel_config, id];
            bytes.extend_from_slice(&offset.to_be_bytes());
            bytes.extend_from_slice(&length.to_be_bytes());

            // Parse it
            let header = Header::from(&bytes[..]);

            // Convert back to bytes
            let roundtrip_bytes: [u8; 10] = header.into();

            // Verify the roundtrip
            prop_assert_eq!(header.sequence_number, seq_num);
            prop_assert_eq!(header.offset, offset);
            prop_assert_eq!(header.length, length);

            // The roundtrip should produce the same parsed values
            let roundtrip_header = Header::from(&roundtrip_bytes[..]);
            prop_assert_eq!(header.sequence_number, roundtrip_header.sequence_number);
            prop_assert_eq!(header.offset, roundtrip_header.offset);
            prop_assert_eq!(header.length, roundtrip_header.length);
        }

        #[test]
        fn test_header_14_byte_with_timecode_roundtrip(
            seq_num in any::<u8>(),
            pixel_config in any::<u8>(),
            id in any::<u8>(),
            offset in any::<u32>(),
            length in any::<u16>(),
            timecode in any::<u32>(),
        ) {
            // Create a header with timecode bit set
            let packet_type_byte = 0b01010000u8; // timecode bit set
            let mut bytes = vec![packet_type_byte, seq_num, pixel_config, id];
            bytes.extend_from_slice(&offset.to_be_bytes());
            bytes.extend_from_slice(&length.to_be_bytes());
            bytes.extend_from_slice(&timecode.to_be_bytes());

            // Parse it
            let header = Header::from(&bytes[..]);

            // Verify timecode was parsed
            prop_assert_eq!(header.time_code.0, Some(timecode));
            prop_assert_eq!(header.sequence_number, seq_num);
            prop_assert_eq!(header.offset, offset);
            prop_assert_eq!(header.length, length);
            prop_assert!(header.packet_type.timecode);

            // Convert back to 14-byte format
            let roundtrip_bytes: [u8; 14] = header.into();

            // Parse again and verify
            let roundtrip_header = Header::from(&roundtrip_bytes[..]);
            prop_assert_eq!(header.time_code, roundtrip_header.time_code);
            prop_assert_eq!(header.sequence_number, roundtrip_header.sequence_number);
        }

        #[test]
        fn test_header_parsing_never_panics(
            bytes in prop::collection::vec(any::<u8>(), 10..20)
        ) {
            // Parsing arbitrary bytes should never panic
            let _ = Header::from(&bytes[..]);
        }

        #[test]
        fn test_header_offset_range(
            offset in 0u32..=0xFFFFFFFF,
        ) {
            let mut header = Header::default();
            header.offset = offset;

            let bytes: [u8; 10] = header.into();
            let parsed = Header::from(&bytes[..]);

            prop_assert_eq!(parsed.offset, offset);
        }

        #[test]
        fn test_header_length_range(
            length in 0u16..=1500,
        ) {
            let mut header = Header::default();
            header.length = length;

            let bytes: [u8; 10] = header.into();
            let parsed = Header::from(&bytes[..]);

            prop_assert_eq!(parsed.length, length);
        }
    }
}
