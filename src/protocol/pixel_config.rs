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

        let data_size = match (byte & 0x07) as usize {
            0 => PixelFormat::Undefined,
            1 => PixelFormat::Pixel1Bits,
            2 => PixelFormat::Pixel4Bits,
            3 => PixelFormat::Pixel8Bits,
            4 => PixelFormat::Pixel16Bits,
            5 => PixelFormat::Pixel24Bits,
            6 => PixelFormat::Pixel32Bits,
            _ => PixelFormat::Undefined,
        };

        let customer_defined = (byte >> 7) != 0;

        PixelConfig {
            data_type,
            data_size,
            customer_defined,
        }
    }
}

impl Into<u8> for PixelConfig {
    fn into(self) -> u8 {
        let mut byte = 0u8;

        byte |= match self.data_type {
            DataType::Undefined => 0,
            DataType::RGB => 1,
            DataType::HSL => 2,
            DataType::RGBW => 3,
            DataType::Grayscale => 4,
        } << 3;

        byte |= match self.data_size {
            PixelFormat::Undefined => 0,
            PixelFormat::Pixel1Bits => 1,
            PixelFormat::Pixel4Bits => 2,
            PixelFormat::Pixel8Bits => 3,
            PixelFormat::Pixel16Bits => 4,
            PixelFormat::Pixel24Bits => 5,
            PixelFormat::Pixel32Bits => 6,
        };

        if self.customer_defined {
            byte |= 0x80;
        }

        byte
    }
}

impl Default for PixelConfig {
    fn default() -> Self {
        Self {
            data_type: DataType::RGB,
            data_size: PixelFormat::Pixel24Bits,
            customer_defined: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_config_from_u8() {
        let byte = 10;
        let pixel_config = PixelConfig::from(byte);

        assert_eq!(pixel_config.data_type, DataType::RGB);
        assert_eq!(pixel_config.data_size, PixelFormat::Pixel4Bits);
        assert!(!pixel_config.customer_defined);
    }
}
