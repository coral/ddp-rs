pub mod packet_type;
pub use packet_type::*;

pub mod pixel_config;
pub use pixel_config::{PixelConfig, PixelFormat};

pub mod id;
pub use id::ID;

pub mod status;

pub struct Header {
    pub packet_type: PacketType,
    pub sequence_number: u8,
    pub pixel_config: PixelConfig,
    pub id: ID,
    pub offset: u32,
    pub length: u16,

    // TODO - Implement timecode
    pub timecode: u32,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            packet_type: Default::default(),
            sequence_number: Default::default(),
            pixel_config: Default::default(),
            id: Default::default(),
            offset: Default::default(),
            length: Default::default(),

            timecode: 0,
        }
    }
}

impl Into<[u8; 10]> for Header {
    fn into(self) -> [u8; 10] {
        // Define a byte array with the size of the header
        let mut buffer = [0u8; 10];

        // Write the packet type field to the buffer
        let packet_type_byte = self.packet_type.into();
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
