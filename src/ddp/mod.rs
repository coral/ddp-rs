pub mod packet_type;
pub use packet_type::*;

pub struct Header {
    pub packet_type: PacketType,
    pub sequence_number: u8,
    pub pixel_config: PixelConfig,
    pub id: u8,
    pub offset: u32,
    pub length: u16,
}

#[derive(Debug)]
#[repr(u8)]
#[allow(dead_code)]
pub enum DataType {
    UndefinedType,
    RGB,
    HSL,
    RGBW,
    Grayscale,
}

#[derive(Debug)]
#[repr(u8)]
#[allow(dead_code)]
pub enum PixelFormat {
    UndefinedPixelFormat,
    Pixel1Bits,
    Pixel4Bits,
    Pixel8Bits,
    Pixel16Bits,
    Pixel24Bits,
    Pixel32Bits,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PixelConfig {
    pub data_type: DataType,
    pub data_size: PixelFormat,
    pub customer_defined: bool,
}
