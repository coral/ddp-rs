//! Packet parsing for receiving data from DDP displays.
//!
//! This module provides the [`Packet`] type for parsing incoming DDP packets,
//! typically used when receiving responses from displays.

use crate::protocol::{message::Message, Header};

/// A parsed DDP packet received from a display.
///
/// This struct represents packets sent back by displays, such as status updates,
/// configuration responses, or acknowledgments.
///
/// # Examples
///
/// ```
/// use ddp_rs::packet::Packet;
///
/// // Parse a packet from raw bytes
/// let bytes = vec![
///     0x41, 0x01, 0x0D, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
///     0xFF, 0x00, 0x00  // 1 RGB pixel: red
/// ];
/// let packet = Packet::from_bytes(&bytes);
///
/// assert_eq!(packet.header.sequence_number, 1);
/// assert_eq!(packet.data, vec![0xFF, 0x00, 0x00]);
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Packet {
    /// The parsed packet header with metadata
    pub header: Header,

    /// Raw pixel data (if this packet contains pixels)
    pub data: Vec<u8>,

    /// Parsed JSON message (if this packet contains a message)
    pub parsed: Option<Message>,
}

impl Packet {
    /// Creates a packet from a header and data slice (without parsing).
    pub fn from_data(h: Header, d: &[u8]) -> Packet {
        Packet {
            header: h,
            data: d.to_vec(),
            parsed: None,
        }
    }

    /// Parses a DDP packet from raw bytes.
    ///
    /// This method handles both 10-byte and 14-byte headers (with timecode),
    /// and attempts to parse JSON messages if the packet is a reply/query.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw packet bytes including header and data
    ///
    /// # Returns
    ///
    /// A parsed `Packet`. If parsing fails, returns a default packet with empty data.
    ///
    /// # Examples
    ///
    /// ```
    /// use ddp_rs::packet::Packet;
    ///
    /// let bytes = vec![
    ///     0x41, 0x01, 0x0D, 0x01,           // Packet type, seq, config, id
    ///     0x00, 0x00, 0x00, 0x00,           // Offset
    ///     0x00, 0x06,                        // Length = 6
    ///     0xFF, 0x00, 0x00,                 // Pixel 1: Red
    ///     0x00, 0xFF, 0x00,                 // Pixel 2: Green
    /// ];
    /// let packet = Packet::from_bytes(&bytes);
    /// assert_eq!(packet.data.len(), 6);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Self {
        // Ensure we have at least 10 bytes for the minimum header
        if bytes.len() < 10 {
            return Packet {
                header: Header::default(),
                data: Vec::new(),
                parsed: None,
            };
        }

        // First, parse just enough to check if timecode is present
        let has_timecode = (bytes[0] & 0b00010000) != 0;
        let header_size = if has_timecode { 14 } else { 10 };

        // Ensure we have enough bytes for the header
        if bytes.len() < header_size {
            return Packet {
                header: Header::default(),
                data: Vec::new(),
                parsed: None,
            };
        }

        let header_bytes = &bytes[0..header_size];
        let header = Header::from(header_bytes);
        let data = &bytes[header_size..];

        let mut parsed: Option<Message> = None;

        if header.packet_type.reply {
            // Try to parse the data into typed structs in the spec
            parsed = match match header.id {
                crate::protocol::ID::Control => match serde_json::from_slice(data) {
                    Ok(v) => Some(Message::Control(v)),
                    Err(_) => None,
                },
                crate::protocol::ID::Config => match serde_json::from_slice(data) {
                    Ok(v) => Some(Message::Config(v)),
                    Err(_) => None,
                },
                crate::protocol::ID::Status => match serde_json::from_slice(data) {
                    Ok(v) => Some(Message::Status(v)),
                    Err(_) => None,
                },
                _ => None,
            } {
                // Worked, return the typed struct
                Some(v) => Some(v),

                // OK, no bueno, lets try just untyped JSON
                None => match header.id {
                    crate::protocol::ID::Control
                    | crate::protocol::ID::Config
                    | crate::protocol::ID::Status => match serde_json::from_slice(data) {
                        // JSON Value it is
                        Ok(v) => Some(Message::Parsed((header.id, v))),
                        // Ok we're really screwed, lets just return the raw data as a string
                        Err(_) => match std::str::from_utf8(data) {
                            Ok(v) => Some(Message::Unparsed((header.id, v.to_string()))),
                            // I guess it's... just bytes?
                            Err(_) => None,
                        },
                    },
                    _ => None,
                },
            }
        }
        Packet {
            header,
            data: data.to_vec(),
            parsed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json() {
        {
            let data = vec![
                0x44, 0x00, 0x0D, 0xFA, 0x00, 0x00, 0x00, 0x00, 0x01, 0x8E, 0x7b, 0x0a, 0x20, 0x20,
                0x20, 0x20, 0x22, 0x63, 0x6f, 0x6e, 0x66, 0x69, 0x67, 0x22, 0x3a, 0x0a, 0x20, 0x20,
                0x20, 0x20, 0x7b, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x67,
                0x77, 0x22, 0x3a, 0x20, 0x22, 0x61, 0x2e, 0x62, 0x2e, 0x63, 0x2e, 0x64, 0x22, 0x2c,
                0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x69, 0x70, 0x22, 0x3a,
                0x20, 0x22, 0x61, 0x2e, 0x62, 0x2e, 0x63, 0x2e, 0x64, 0x22, 0x2c, 0x0a, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x6e, 0x6d, 0x22, 0x3a, 0x20, 0x22, 0x61,
                0x2e, 0x62, 0x2e, 0x63, 0x2e, 0x64, 0x22, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x22, 0x70, 0x6f, 0x72, 0x74, 0x73, 0x22, 0x3a, 0x0a, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x5b, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x7b, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x6c, 0x22, 0x3a,
                0x20, 0x33, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x70, 0x6f, 0x72, 0x74, 0x22, 0x3a, 0x20,
                0x31, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x73, 0x73, 0x22, 0x3a, 0x20, 0x34, 0x2c, 0x0a,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x22, 0x74, 0x73, 0x22, 0x3a, 0x20, 0x32, 0x0a, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x7d, 0x2c, 0x0a, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x7b, 0x0a, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22,
                0x6c, 0x22, 0x3a, 0x20, 0x37, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x70, 0x6f, 0x72, 0x74,
                0x22, 0x3a, 0x20, 0x35, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x73, 0x73, 0x22, 0x3a, 0x20,
                0x38, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x74, 0x73, 0x22, 0x3a, 0x20, 0x36, 0x0a, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x7d, 0x0a, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x5d, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x7d,
                0x0a, 0x7d,
            ];
            let packet = Packet::from_bytes(&data);

            assert_eq!(packet.header.length, 398);

            match packet.parsed {
                Some(p) => match p {
                    Message::Config(c) => {
                        assert_eq!(c.config.gw.unwrap(), "a.b.c.d");
                        assert_eq!(c.config.nm.unwrap(), "a.b.c.d");
                        assert_eq!(c.config.ports.len(), 2);
                    }
                    _ => panic!("not the right packet parsed"),
                },
                None => panic!("Packet parsing failed"),
            }
        }
    }

    #[test]
    fn test_untyped() {
        {
            let data = vec![
                0x44, 0x00, 0x0D, 0xFA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x7B, 0x22, 0x68, 0x65,
                0x6C, 0x6C, 0x6F, 0x22, 0x3A, 0x20, 0x22, 0x6F, 0x6B, 0x22, 0x7D,
            ];
            let packet = Packet::from_bytes(&data);

            match packet.parsed {
                Some(p) => match p {
                    Message::Parsed((_, p)) => {
                        assert_eq!(p["hello"], "ok");
                    }
                    _ => panic!("not the right packet parsed"),
                },
                None => panic!("Packet parsing failed"),
            }
        }
    }

    #[test]
    fn test_unparsed() {
        {
            let data = vec![
                0x44, 0x00, 0x0D, 0xFA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0E, 0x53, 0x4C, 0x49, 0x43,
                0x4B, 0x44, 0x45, 0x4E, 0x49, 0x53, 0x34, 0x30, 0x30, 0x30,
            ];
            let packet = Packet::from_bytes(&data);

            match packet.parsed {
                Some(p) => match p {
                    Message::Unparsed((_, p)) => {
                        assert_eq!(p, "SLICKDENIS4000");
                    }
                    _ => panic!("not the right packet parsed"),
                },
                None => panic!("Packet parsing failed"),
            }
        }
    }

    // Property-based tests using proptest
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_packet_header_roundtrip(
            version in 0u8..4,
            seq_num in 0u8..=15,
            pixel_config in 0u8..=255,
            id in 0u8..=255,
            offset in 0u32..1000000,
            length in 0u16..1500,
        ) {
            // Create a header with random but valid values
            let mut packet_bytes = vec![
                (version << 6) | 0b00000001, // packet_type with push bit set
                seq_num,
                pixel_config,
                id,
            ];
            packet_bytes.extend_from_slice(&offset.to_be_bytes());
            packet_bytes.extend_from_slice(&length.to_be_bytes());

            // Add some data
            let data = vec![0u8; length.min(100) as usize];
            packet_bytes.extend_from_slice(&data);

            // Parse the packet
            let packet = Packet::from_bytes(&packet_bytes);

            // Verify the header fields were parsed correctly
            prop_assert_eq!(packet.header.sequence_number, seq_num);
            prop_assert_eq!(packet.header.offset, offset);
            prop_assert_eq!(packet.header.length, length);
        }

        #[test]
        fn test_packet_with_arbitrary_data(
            data_len in 0usize..500,
            seq_num in 1u8..=15,
        ) {
            // Generate random pixel data
            let data: Vec<u8> = (0..data_len).map(|i| (i % 256) as u8).collect();

            // Create a minimal valid header
            let mut packet_bytes = vec![
                0x41, // version 1, push bit set
                seq_num,
                0x00, // pixel config
                0x01, // ID
                0x00, 0x00, 0x00, 0x00, // offset
            ];
            let length = data.len() as u16;
            packet_bytes.extend_from_slice(&length.to_be_bytes());
            packet_bytes.extend_from_slice(&data);

            // Parse it
            let packet = Packet::from_bytes(&packet_bytes);

            // Verify
            prop_assert_eq!(packet.data, data);
            prop_assert_eq!(packet.header.sequence_number, seq_num);
        }

        #[test]
        fn test_packet_parsing_never_panics(
            bytes in prop::collection::vec(any::<u8>(), 10..1500)
        ) {
            // This test ensures that parsing arbitrary bytes never panics
            // Even with completely random data, we should handle it gracefully
            let _ = Packet::from_bytes(&bytes);
        }

        #[test]
        fn test_packet_with_timecode_roundtrip(
            timecode in any::<u32>(),
            seq_num in 1u8..=15,
            data_len in 0usize..100,
        ) {
            // Create header with timecode bit set
            let mut packet_bytes = vec![
                0x51, // version 1, push bit set, timecode bit set (0b01010001)
                seq_num,
                0x00, // pixel config
                0x01, // ID
                0x00, 0x00, 0x00, 0x00, // offset
            ];

            let data: Vec<u8> = (0..data_len).map(|i| (i % 256) as u8).collect();
            let length = data.len() as u16;
            packet_bytes.extend_from_slice(&length.to_be_bytes());
            packet_bytes.extend_from_slice(&timecode.to_be_bytes());
            packet_bytes.extend_from_slice(&data);

            let packet = Packet::from_bytes(&packet_bytes);

            prop_assert_eq!(packet.header.time_code.0, Some(timecode));
            prop_assert_eq!(packet.data, data);
        }

        #[test]
        fn test_offset_values_preserved(
            offset in 0u32..4000000,
        ) {
            let mut packet_bytes = vec![
                0x41, 1, 0x00, 0x01,
            ];
            packet_bytes.extend_from_slice(&offset.to_be_bytes());
            packet_bytes.extend_from_slice(&[0x00, 0x03]); // length = 3
            packet_bytes.extend_from_slice(&[255, 0, 0]); // 1 pixel

            let packet = Packet::from_bytes(&packet_bytes);
            prop_assert_eq!(packet.header.offset, offset);
        }
    }

    // Integration tests for full packet roundtrips
    #[test]
    fn test_full_packet_roundtrip() {
        use crate::protocol::{Header, PacketType, PixelConfig, ID, timecode::TimeCode};

        // Create a header
        let header = Header {
            packet_type: PacketType {
                version: 1,
                timecode: false,
                storage: false,
                reply: false,
                query: false,
                push: true,
            },
            sequence_number: 5,
            pixel_config: PixelConfig::default(),
            id: ID::default(),
            offset: 0,
            length: 9,
            time_code: TimeCode(None),
        };

        // Create RGB data
        let data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];

        // Convert header to bytes
        let header_bytes: [u8; 10] = header.into();

        // Create full packet
        let mut packet_bytes = header_bytes.to_vec();
        packet_bytes.extend_from_slice(&data);

        // Parse it back
        let parsed_packet = Packet::from_bytes(&packet_bytes);

        // Verify
        assert_eq!(parsed_packet.header.sequence_number, 5);
        assert_eq!(parsed_packet.header.length, 9);
        assert_eq!(parsed_packet.data, data);
    }

    #[test]
    fn test_packet_with_timecode_integration() {
        use crate::protocol::{Header, PacketType, PixelConfig, ID, timecode::TimeCode};

        // Create a header with timecode
        let header = Header {
            packet_type: PacketType {
                version: 1,
                timecode: true,
                storage: false,
                reply: false,
                query: false,
                push: true,
            },
            sequence_number: 3,
            pixel_config: PixelConfig::default(),
            id: ID::default(),
            offset: 100,
            length: 6,
            time_code: TimeCode(Some(12345)),
        };

        // Create RGB data
        let data = vec![128, 128, 128, 64, 64, 64];

        // Convert header to bytes (14 bytes with timecode)
        let header_bytes: [u8; 14] = header.into();

        // Create full packet
        let mut packet_bytes = header_bytes.to_vec();
        packet_bytes.extend_from_slice(&data);

        // Parse it back
        let parsed_packet = Packet::from_bytes(&packet_bytes);

        // Verify
        assert_eq!(parsed_packet.header.sequence_number, 3);
        assert_eq!(parsed_packet.header.length, 6);
        assert_eq!(parsed_packet.header.offset, 100);
        assert_eq!(parsed_packet.header.time_code.0, Some(12345));
        assert_eq!(parsed_packet.data, data);
    }

    #[test]
    fn test_packet_with_config_message_integration() {
        use crate::protocol::{Header, PacketType, PixelConfig, ID, timecode::TimeCode};

        let json = r#"{"config":{"gw":"192.168.1.1","ip":"192.168.1.100"}}"#;

        let header = Header {
            packet_type: PacketType {
                version: 1,
                timecode: false,
                storage: false,
                reply: true,
                query: false,
                push: false,
            },
            sequence_number: 1,
            pixel_config: PixelConfig::default(),
            id: ID::Config,
            offset: 0,
            length: json.len() as u16,
            time_code: TimeCode(None),
        };

        let header_bytes: [u8; 10] = header.into();
        let mut packet_bytes = header_bytes.to_vec();
        packet_bytes.extend_from_slice(json.as_bytes());

        let parsed_packet = Packet::from_bytes(&packet_bytes);

        assert_eq!(parsed_packet.header.id, ID::Config);
        assert!(parsed_packet.parsed.is_some());
    }

    #[test]
    fn test_multiple_packets_different_sequences_integration() {
        use crate::protocol::{Header, PacketType, PixelConfig, ID, timecode::TimeCode};

        // Test that we can parse multiple packets with different sequence numbers
        let test_cases = vec![
            (1, vec![255, 0, 0]),
            (5, vec![0, 255, 0]),
            (10, vec![0, 0, 255]),
            (15, vec![128, 128, 128]),
        ];

        for (seq_num, data) in test_cases {
            let header = Header {
                packet_type: PacketType {
                    version: 1,
                    timecode: false,
                    storage: false,
                    reply: false,
                    query: false,
                    push: true,
                },
                sequence_number: seq_num,
                pixel_config: PixelConfig::default(),
                id: ID::default(),
                offset: 0,
                length: data.len() as u16,
                time_code: TimeCode(None),
            };

            let header_bytes: [u8; 10] = header.into();
            let mut packet_bytes = header_bytes.to_vec();
            packet_bytes.extend_from_slice(&data);

            let parsed = Packet::from_bytes(&packet_bytes);
            assert_eq!(parsed.header.sequence_number, seq_num);
            assert_eq!(parsed.data, data);
        }
    }

    #[test]
    fn test_large_pixel_data_integration() {
        use crate::protocol::{Header, PacketType, PixelConfig, ID, timecode::TimeCode};

        // Test with a large number of pixels
        let num_pixels = 480; // Max size in connection
        let mut data = Vec::with_capacity(num_pixels * 3);

        for i in 0..num_pixels {
            data.push((i % 256) as u8);
            data.push(((i * 2) % 256) as u8);
            data.push(((i * 3) % 256) as u8);
        }

        let header = Header {
            packet_type: PacketType {
                version: 1,
                timecode: false,
                storage: false,
                reply: false,
                query: false,
                push: true,
            },
            sequence_number: 1,
            pixel_config: PixelConfig::default(),
            id: ID::default(),
            offset: 0,
            length: data.len() as u16,
            time_code: TimeCode(None),
        };

        let header_bytes: [u8; 10] = header.into();
        let mut packet_bytes = header_bytes.to_vec();
        packet_bytes.extend_from_slice(&data);

        let parsed = Packet::from_bytes(&packet_bytes);
        assert_eq!(parsed.data.len(), data.len());
        assert_eq!(parsed.data, data);
    }
}
