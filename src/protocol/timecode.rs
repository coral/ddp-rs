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
