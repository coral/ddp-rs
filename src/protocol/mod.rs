pub mod packet_type;
pub use packet_type::*;

pub mod pixel_config;
pub use pixel_config::{PixelConfig, PixelFormat};

pub enum ID {
    Reserved,
    Default,
    Custom(u8),
    Control,
    Config,
    Status,
    DMX,
    Broadcast,
}

impl From<u8> for ID {
    fn from(value: u8) -> Self {
        match value {
            0 => ID::Reserved,
            1 => ID::Default,
            2..=246 => ID::Custom(value),
            249 => ID::Control,
            250 => ID::Config,
            251 => ID::Status,
            254 => ID::DMX,
            255 => ID::Broadcast,
            _ => ID::Reserved,
        }
    }
}

impl Into<u8> for ID {
    fn into(self) -> u8 {
        match self {
            ID::Reserved => 0,
            ID::Default => 1,
            ID::Custom(value) if value >= 2 && value <= 246 => value,
            ID::Control => 249,
            ID::Config => 250,
            ID::Status => 251,
            ID::DMX => 254,
            ID::Broadcast => 255,
            ID::Custom(_) => 1,
        }
    }
}

impl Default for ID {
    fn default() -> Self {
        ID::Default
    }
}

pub struct Header {
    pub packet_type: PacketType,
    pub sequence_number: u8,
    pub pixel_config: PixelConfig,
    pub id: u8,
    pub offset: u32,
    pub length: u16,
}
