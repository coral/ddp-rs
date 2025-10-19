use serde::{Deserialize, Serialize};

/// Protocol ID used to identify the purpose of a packet.
///
/// IDs are used to differentiate between pixel data, control messages, configuration
/// queries, and other packet types. The ID space is divided into several ranges:
///
/// - 0: Reserved
/// - 1: Default (standard pixel data)
/// - 2-245, 247-249: Custom IDs for application-specific use
/// - 246: Control messages (JSON control read/write)
/// - 250: Configuration messages (JSON config read/write)
/// - 251: Status messages (JSON status read-only)
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

    /// Custom ID in ranges 2-245 and 247-249
    Custom(u8),

    /// Control message ID (246) for JSON control read/write
    Control,

    /// Configuration message ID (250) for JSON config read/write
    Config,

    /// Status message ID (251) for JSON status read-only
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
            246 => ID::Control,
            250 => ID::Config,
            251 => ID::Status,
            254 => ID::DMX,
            255 => ID::Broadcast,
            // All other values are custom IDs (2-245, 247-249, 252-253)
            _ => ID::Custom(value),
        }
    }
}

impl Into<u8> for ID {
    fn into(self) -> u8 {
        match self {
            ID::Reserved => 0,
            ID::Default => 1,
            ID::Control => 246,
            ID::Config => 250,
            ID::Status => 251,
            ID::DMX => 254,
            ID::Broadcast => 255,
            ID::Custom(value) => {
                // Valid custom ranges: 2-245, 247-249, 252-253
                if matches!(value, 2..=245 | 247..=249 | 252..=253) {
                    value
                } else {
                    1 // Default to ID 1 if invalid
                }
            }
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
