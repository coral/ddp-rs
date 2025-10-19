use serde::{Deserialize, Serialize};

/// Protocol ID used to identify the purpose of a packet.
///
/// IDs are used to differentiate between pixel data, control messages, configuration
/// queries, and other packet types. The ID space is divided into several ranges:
///
/// - 0: Reserved
/// - 1: Default (standard pixel data)
/// - 2-246: Custom IDs for application-specific use
/// - 249: Control messages
/// - 250: Configuration messages
/// - 251: Status messages
/// - 254: DMX data
/// - 255: Broadcast to all displays
///
/// # Examples
///
/// ```
/// use ddp_rs::protocol::ID;
///
/// // Standard pixel data
/// let pixel_id = ID::Default;
///
/// // Control message
/// let control_id = ID::Control;
///
/// // Custom application ID
/// let custom_id = ID::Custom(42);
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum ID {
    /// Reserved, should not be used
    Reserved,

    /// Default ID for standard pixel data
    #[default]
    Default,

    /// Custom ID in the range 2-246
    Custom(u8),

    /// Control message ID (249)
    Control,

    /// Configuration message ID (250)
    Config,

    /// Status message ID (251)
    Status,

    /// DMX data ID (254)
    DMX,

    /// Broadcast to all displays (255)
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
