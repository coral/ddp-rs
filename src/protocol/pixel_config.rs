#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
pub enum DataType {
    Undefined,
    RGB,
    HSL,
    RGBW,
    Grayscale,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
pub enum PixelFormat {
    Undefined,
    Pixel1Bits,
    Pixel4Bits,
    Pixel8Bits,
    Pixel16Bits,
    Pixel24Bits,
    Pixel32Bits,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy)]
#[allow(dead_code)]
pub struct PixelConfig {
    pub data_type: DataType,
    pub data_size: PixelFormat,
    pub customer_defined: bool,
}

impl From<u8> for PixelConfig {
    fn from(byte: u8) -> Self {
        let data_type = match (byte >> 3) & 0x07 {
            0 => DataType::Undefined,
            1 => DataType::RGB,
            2 => DataType::HSL,
            3 => DataType::RGBW,
            4 => DataType::Grayscale,
            _ => DataType::Undefined,
        };

        let data_size = match byte & 0x07 {
            0 => PixelFormat::Undefined,
            1 => PixelFormat::Pixel1Bits,
            2 => PixelFormat::Pixel4Bits,
            3 => PixelFormat::Pixel8Bits,
            4 => PixelFormat::Pixel16Bits,
            5 => PixelFormat::Pixel24Bits,
            6 => PixelFormat::Pixel32Bits,
            _ => PixelFormat::Undefined,
        };

        let customer_defined = (byte & 0x80) != 0;

        PixelConfig {
            data_type,
            data_size,
            customer_defined,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_config_from_u8() {
        let byte = 0b10100000;
        let pixel_config = PixelConfig::from(byte);

        assert_eq!(pixel_config.data_type, DataType::RGB);
        assert_eq!(pixel_config.data_size, PixelFormat::Pixel8Bits);
        assert!(pixel_config.customer_defined);
    }
}
