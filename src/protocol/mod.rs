pub mod packet_type;
pub use packet_type::*;

pub mod pixel_config;
pub use pixel_config::{PixelConfig, PixelFormat};

pub mod id;
pub use id::ID;

pub mod status;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Packet<'a> {
    pub header: Header,
    pub data: &'a [u8],
}

impl<'a> Packet<'a> {
    pub fn from_data(h: Header, d: &'a [u8]) -> Packet<'a> {
        Packet { header: h, data: d }
    }
}

impl<'a> Into<Vec<u8>> for Packet<'a> {
    fn into(self) -> Vec<u8> {
        let header_bytes: [u8; 10] = self.header.into();
        let mut bytes = Vec::with_capacity(10 + self.data.len());
        bytes.extend_from_slice(&header_bytes);
        bytes.extend_from_slice(self.data);
        bytes
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Header {
    pub packet_type: PacketType,
    pub sequence_number: u8,
    pub pixel_config: PixelConfig,
    pub id: ID,
    pub offset: u32,
    pub length: u16,
    // TODO - Implement timecode
    //pub timecode: u32,
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
            //timecode: 0,
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

        Header {
            packet_type,
            sequence_number,
            pixel_config,
            id,
            offset,
            length,
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
}
