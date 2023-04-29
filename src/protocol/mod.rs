pub mod packet_type;
pub use packet_type::*;

pub mod pixel_config;
pub use pixel_config::{PixelConfig, PixelFormat};

pub mod id;
pub use id::ID;

use std::net::UdpSocket;
use thiserror::Error;

pub struct Header {
    pub packet_type: PacketType,
    pub sequence_number: u8,
    pub pixel_config: PixelConfig,
    pub id: ID,
    pub offset: u32,
    pub length: u16,
}
