use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum ID {
    Reserved,
    #[default]
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
            ID::Custom(value) if (2..=246).contains(&value) => value,
            ID::Control => 249,
            ID::Config => 250,
            ID::Status => 251,
            ID::DMX => 254,
            ID::Broadcast => 255,
            ID::Custom(_) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_parse() {
        // Custom
        let id = ID::from(212);
        assert_eq!(id, ID::Custom(212));

        // Default
        let id = ID::from(1);
        assert_eq!(id, ID::Default);

        // Status
        let id = ID::from(251);
        assert_eq!(id, ID::Status);

        // Reserved
        let id = ID::from(0);
        assert_eq!(id, ID::Reserved);
    }
}
