#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[allow(dead_code)]
pub struct PacketType {
    version: u8, // 0x40 ( 2 bit value, can be a value between 1-4 depending on version mask, )
    pub timecode: bool, // 0x10
    pub storage: bool, // 0x08
    pub reply: bool, // 0x04
    pub query: bool, //0x02
    pub push: bool, //0x01
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
            0x00 => 1,
            0x40 => 2,
            0x80 => 3,
            0xc0 => 4,
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
                version: 1,
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
