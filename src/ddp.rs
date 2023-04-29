
#[derive(Debug)]
#[repr(u8)]
#[allow(dead_code)]
enum DataType {
    UndefinedType,
    RGB,
    HSL,
    RGBW,
    Grayscale,
}


#[derive(Debug)]
#[repr(u8)]
#[allow(dead_code)]
enum PixelFormat {
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
struct PixelConfig {
    data_type: DataType,
    data_size: PixelFormat,
    customer_defined: bool,
}

#[derive(Debug)]
struct PacketType {
    timecode: bool,
    storage: bool,
    reply: bool,
    query: bool,
    push: bool,
}

pub struct Header {
    sequence_number: u8,
    data_type: PixelConfig,
    id: u8,
    offset: u32,
    length: u16,
}