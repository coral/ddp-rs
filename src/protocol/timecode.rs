#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub struct TimeCode(pub Option<u32>);

impl TimeCode {
    pub fn from_4_bytes(bytes: [u8; 4]) -> Self {
        TimeCode(Some(u32::from_be_bytes(bytes)))
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.0.unwrap_or(0u32).to_be_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timecode_default() {
        let tc = TimeCode::default();
        assert_eq!(tc.0, None);
    }

    #[test]
    fn test_timecode_from_4_bytes() {
        let bytes = [0x00, 0x00, 0x30, 0x39]; // 12345 in big-endian
        let tc = TimeCode::from_4_bytes(bytes);
        assert_eq!(tc.0, Some(12345));
    }

    #[test]
    fn test_timecode_from_4_bytes_zero() {
        let bytes = [0x00, 0x00, 0x00, 0x00];
        let tc = TimeCode::from_4_bytes(bytes);
        assert_eq!(tc.0, Some(0));
    }

    #[test]
    fn test_timecode_from_4_bytes_max() {
        let bytes = [0xFF, 0xFF, 0xFF, 0xFF];
        let tc = TimeCode::from_4_bytes(bytes);
        assert_eq!(tc.0, Some(u32::MAX));
    }

    #[test]
    fn test_timecode_to_bytes_some() {
        let tc = TimeCode(Some(12345));
        let bytes = tc.to_bytes();
        assert_eq!(bytes, [0x00, 0x00, 0x30, 0x39]);
    }

    #[test]
    fn test_timecode_to_bytes_none() {
        let tc = TimeCode(None);
        let bytes = tc.to_bytes();
        assert_eq!(bytes, [0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_timecode_to_bytes_zero() {
        let tc = TimeCode(Some(0));
        let bytes = tc.to_bytes();
        assert_eq!(bytes, [0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_timecode_to_bytes_max() {
        let tc = TimeCode(Some(u32::MAX));
        let bytes = tc.to_bytes();
        assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn test_timecode_roundtrip() {
        let original = TimeCode(Some(98765));
        let bytes = original.to_bytes();
        let roundtrip = TimeCode::from_4_bytes(bytes);
        assert_eq!(original, roundtrip);
    }

    #[test]
    fn test_timecode_roundtrip_various_values() {
        let test_values = vec![0, 1, 255, 256, 65535, 65536, 16777215, 16777216, u32::MAX];

        for value in test_values {
            let original = TimeCode(Some(value));
            let bytes = original.to_bytes();
            let roundtrip = TimeCode::from_4_bytes(bytes);
            assert_eq!(original, roundtrip, "Failed roundtrip for value {}", value);
        }
    }

    #[test]
    fn test_timecode_equality() {
        let tc1 = TimeCode(Some(100));
        let tc2 = TimeCode(Some(100));
        let tc3 = TimeCode(Some(200));

        assert_eq!(tc1, tc2);
        assert_ne!(tc1, tc3);
    }

    #[test]
    fn test_timecode_clone() {
        let tc1 = TimeCode(Some(12345));
        let tc2 = tc1.clone();
        assert_eq!(tc1, tc2);
    }

    #[test]
    fn test_timecode_copy() {
        let tc1 = TimeCode(Some(12345));
        let tc2 = tc1; // Copy occurs here
        assert_eq!(tc1, tc2);
        assert_eq!(tc1.0, Some(12345)); // tc1 is still valid
    }

    #[test]
    fn test_timecode_debug_format() {
        let tc = TimeCode(Some(12345));
        let debug_str = format!("{:?}", tc);
        assert_eq!(debug_str, "TimeCode(Some(12345))");
    }

    #[test]
    fn test_timecode_none_debug_format() {
        let tc = TimeCode(None);
        let debug_str = format!("{:?}", tc);
        assert_eq!(debug_str, "TimeCode(None)");
    }

    #[test]
    fn test_timecode_big_endian_encoding() {
        // Verify big-endian byte order
        let tc = TimeCode(Some(0x12345678));
        let bytes = tc.to_bytes();
        assert_eq!(bytes[0], 0x12); // Most significant byte first
        assert_eq!(bytes[1], 0x34);
        assert_eq!(bytes[2], 0x56);
        assert_eq!(bytes[3], 0x78); // Least significant byte last
    }
}
