pub mod packet_type;
pub use packet_type::*;

pub mod pixel_config;
pub use pixel_config::{PixelConfig, PixelFormat};

pub struct Header {
    pub packet_type: PacketType,
    pub sequence_number: u8,
    pub pixel_config: PixelConfig,
    pub id: u8,
    pub offset: u32,
    pub length: u16,
}
