/// Packet type flags that control protocol behavior.
///
/// The packet type is encoded in byte 0 of the header and contains flags
/// that determine how the packet should be interpreted and processed.
///
/// # Flag Bits
///
/// - Bits 6-7: Protocol version (1-4)
/// - Bit 4: Timecode present (if set, header is 14 bytes instead of 10)
/// - Bit 3: Storage (for persisting settings)
/// - Bit 2: Reply (packet is a response)
/// - Bit 1: Query (request information)
/// - Bit 0: Push (final packet in sequence)
///
/// # Examples
///
/// ```
/// use ddp_rs::protocol::PacketType;
///
/// let mut pkt = PacketType::default();
/// pkt.push = true;  // Mark as final packet
/// pkt.version = 1;  // DDP version 1
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[allow(dead_code)]
pub struct PacketType {
    /// Protocol version (1-4)
    pub version: u8,

    /// Timecode is present (extends header to 14 bytes)
    pub timecode: bool,

    /// Storage flag (persist settings)
    pub storage: bool,

    /// Reply flag (this is a response packet)
    pub reply: bool,

    /// Query flag (request information)
    pub query: bool,

    /// Push flag (final packet in a sequence)
    pub push: bool,
}

impl PacketType {
    /// Sets the push flag, indicating this is the final packet in a sequence.
    pub fn push(&mut self, push: bool) {
        self.push = push;
    }
}

const VERSION_MASK: u8 = 0xc0;
//const RESERVED: u8 = 0x20;
const TIMECODE: u8 = 0x10;
const STORAGE: u8 = 0x08;
const REPLY: u8 = 0x04;
const QUERY: u8 = 0x02;
const PUSH: u8 = 0x01;

impl Default for PacketType {
    fn default() -> Self {
        Self {
            version: 1,
            timecode: false,
            storage: false,
            reply: false,
            query: false,
            push: false,
        }
    }
}

impl From<u8> for PacketType {
    fn from(byte: u8) -> Self {
        let version = match byte & VERSION_MASK {
            0x00 => 0,
            0x40 => 1,
            0x80 => 2,
            0xc0 => 3,
            _ => 0,
        };
        let timecode = byte & TIMECODE == TIMECODE;
        let storage = byte & STORAGE == STORAGE;
        let reply = byte & REPLY == REPLY;
        let query = byte & QUERY == QUERY;
        let push = byte & PUSH == PUSH;

        PacketType {
            version,
            timecode,
            storage,
            reply,
            query,
            push,
        }
    }
}

impl Into<u8> for PacketType {
    fn into(self) -> u8 {
        let mut byte: u8 = 0;
        let v = match self.version {
            1 => self.version,
            2 => self.version,
            3 => self.version,
            4 => self.version,
            _ => 0,
        };
        byte |= v << 6;
        // Set the flag bits
        if self.timecode {
            byte |= TIMECODE
        };
        if self.storage {
            byte |= STORAGE
        };
        if self.reply {
            byte |= REPLY
        };
        if self.query {
            byte |= QUERY
        };
        if self.push {
            byte |= PUSH
        };

        byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_type_parse() {
        let byte: u8 = 0b00010110;
        let packet_type = PacketType::from(byte);

        assert_eq!(
            packet_type,
            PacketType {
                version: 0,
                timecode: true,
                storage: false,
                reply: true,
                query: true,
                push: false,
            }
        );
    }

    #[test]
    fn test_packet_type_into_u8() {
        let packet_type = PacketType {
            version: 1,
            timecode: true,
            storage: false,
            reply: true,
            query: true,
            push: false,
        };

        let byte: u8 = packet_type.into();
        assert_eq!(byte, 0x56);
    }
}
