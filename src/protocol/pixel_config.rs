/// Pixel data type (color space).
///
/// Defines how pixel color values should be interpreted.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
pub enum DataType {
    /// Undefined or custom data type
    Undefined,
    /// Red, Green, Blue
    RGB,
    /// Hue, Saturation, Lightness
    HSL,
    /// Red, Green, Blue, White
    RGBW,
    /// Grayscale/monochrome
    Grayscale,
}

/// Number of bits per pixel.
///
/// Defines the bit depth for each pixel's data.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
pub enum PixelFormat {
    /// Undefined format
    Undefined,
    /// 1 bit per pixel
    Pixel1Bits,
    /// 4 bits per pixel
    Pixel4Bits,
    /// 8 bits per pixel (1 byte)
    Pixel8Bits,
    /// 16 bits per pixel (2 bytes)
    Pixel16Bits,
    /// 24 bits per pixel (3 bytes) - standard RGB
    Pixel24Bits,
    /// 32 bits per pixel (4 bytes) - RGBA or RGBW
    Pixel32Bits,
}

/// Pixel format configuration.
///
/// Describes how pixel data is encoded in the packet. The default configuration
/// is RGB with 8 bits per channel (24 bits total per pixel).
///
/// # Examples
///
/// ```
/// use ddp_rs::protocol::{PixelConfig, DataType, PixelFormat};
///
/// // Default: RGB, 8 bits per channel
/// let config = PixelConfig::default();
/// assert_eq!(config.data_type, DataType::RGB);
/// assert_eq!(config.data_size, PixelFormat::Pixel24Bits);
///
/// // Custom: RGBW pixels
/// let rgbw_config = PixelConfig {
///     data_type: DataType::RGBW,
///     data_size: PixelFormat::Pixel32Bits,
///     customer_defined: false,
/// };
/// ```
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy)]
#[allow(dead_code)]
pub struct PixelConfig {
    /// Color space / data type
    pub data_type: DataType,

    /// Bits per pixel
    pub data_size: PixelFormat,

    /// Whether this is a custom/vendor-specific configuration
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
        // WLED oddities
        {
            let byte = 0x0A;
            let pixel_config = PixelConfig::from(byte);

            assert_eq!(pixel_config.data_type, DataType::RGB);
            assert_eq!(pixel_config.data_size, PixelFormat::Pixel4Bits);
            assert!(!pixel_config.customer_defined);
        }

        // RGB 24
        {
            let byte = 0x0D;
            let pixel_config = PixelConfig::from(byte);

            assert_eq!(pixel_config.data_type, DataType::RGB);
            assert_eq!(pixel_config.data_size, PixelFormat::Pixel24Bits);
            assert!(!pixel_config.customer_defined);
        }

        // RGBW 32
        {
            let byte = 0x1E;
            let pixel_config = PixelConfig::from(byte);

            assert_eq!(pixel_config.data_type, DataType::RGBW);
            assert_eq!(pixel_config.data_size, PixelFormat::Pixel32Bits);
            assert!(!pixel_config.customer_defined);
        }
    }
}
