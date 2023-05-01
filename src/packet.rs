use crate::protocol::{response::Response, Header};

#[derive(Debug, PartialEq, Hash, Clone)]
/// Packet is our internal representation of a recieved packet
/// Used to parse messages sent back by displays
/// This struct does allocate!
pub struct Packet {
    /// The packet header
    pub header: Header,
    /// Raw data, if you're getting pixels this is the one you want
    pub data: Vec<u8>,
    /// For anything that's messaging, we try to parse it or cast it to string here
    pub parsed: Option<Response>,
}

impl Packet {
    pub fn from_data(h: Header, d: &[u8]) -> Packet {
        Packet {
            header: h,
            data: d.to_vec(),
            parsed: None,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let header_bytes = &bytes[0..10];
        let header = Header::from(header_bytes);
        let data = &bytes[10..];

        let mut parsed: Option<Response> = None;

        if header.packet_type.reply {
            parsed = match match header.id {
                crate::protocol::ID::Control => match serde_json::from_slice(data) {
                    Ok(v) => Some(Response::Control(v)),
                    Err(_) => None,
                },
                crate::protocol::ID::Config => match serde_json::from_slice(data) {
                    Ok(v) => Some(Response::Config(v)),
                    Err(_) => None,
                },
                crate::protocol::ID::Status => match serde_json::from_slice(data) {
                    Ok(v) => Some(Response::Status(v)),
                    Err(_) => None,
                },
                _ => None,
            } {
                Some(v) => Some(v),
                None => match header.id {
                    crate::protocol::ID::Control
                    | crate::protocol::ID::Config
                    | crate::protocol::ID::Status => match std::str::from_utf8(&data) {
                        Ok(v) => Some(Response::Unparsed(v.to_string())),
                        Err(_) => None,
                    },
                    _ => None,
                },
            }
        }
        Packet {
            header,
            data: data.to_vec(),
            parsed: parsed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json() {
        {
            let data = vec![
                0x44, 0x00, 0x0D, 0xFA, 0x00, 0x00, 0x00, 0x00, 0x01, 0x8E, 0x7b, 0x0a, 0x20, 0x20,
                0x20, 0x20, 0x22, 0x63, 0x6f, 0x6e, 0x66, 0x69, 0x67, 0x22, 0x3a, 0x0a, 0x20, 0x20,
                0x20, 0x20, 0x7b, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x67,
                0x77, 0x22, 0x3a, 0x20, 0x22, 0x61, 0x2e, 0x62, 0x2e, 0x63, 0x2e, 0x64, 0x22, 0x2c,
                0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x69, 0x70, 0x22, 0x3a,
                0x20, 0x22, 0x61, 0x2e, 0x62, 0x2e, 0x63, 0x2e, 0x64, 0x22, 0x2c, 0x0a, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x6e, 0x6d, 0x22, 0x3a, 0x20, 0x22, 0x61,
                0x2e, 0x62, 0x2e, 0x63, 0x2e, 0x64, 0x22, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x22, 0x70, 0x6f, 0x72, 0x74, 0x73, 0x22, 0x3a, 0x0a, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x5b, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x7b, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x6c, 0x22, 0x3a,
                0x20, 0x33, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x70, 0x6f, 0x72, 0x74, 0x22, 0x3a, 0x20,
                0x31, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x73, 0x73, 0x22, 0x3a, 0x20, 0x34, 0x2c, 0x0a,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x22, 0x74, 0x73, 0x22, 0x3a, 0x20, 0x32, 0x0a, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x7d, 0x2c, 0x0a, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x7b, 0x0a, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22,
                0x6c, 0x22, 0x3a, 0x20, 0x37, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x70, 0x6f, 0x72, 0x74,
                0x22, 0x3a, 0x20, 0x35, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x73, 0x73, 0x22, 0x3a, 0x20,
                0x38, 0x2c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x74, 0x73, 0x22, 0x3a, 0x20, 0x36, 0x0a, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x7d, 0x0a, 0x20,
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x5d, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x7d,
                0x0a, 0x7d,
            ];
            let packet = Packet::from_bytes(&data);

            assert_eq!(packet.header.length, 398);

            match packet.parsed {
                Some(p) => match p {
                    Response::Config(c) => {
                        assert_eq!(c.config.gw.unwrap(), "a.b.c.d");
                        assert_eq!(c.config.nm.unwrap(), "a.b.c.d");
                        assert_eq!(c.config.ports.len(), 2);
                    }
                    _ => panic!("not the right packet parsed"),
                },
                None => panic!("Packet parsing failed"),
            }
        }
    }

    #[test]
    fn test_unparsed() {
        {
            let data = vec![
                0x44, 0x00, 0x0D, 0xFA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0E, 0x53, 0x4C, 0x49, 0x43,
                0x4B, 0x44, 0x45, 0x4E, 0x49, 0x53, 0x34, 0x30, 0x30, 0x30,
            ];
            let packet = Packet::from_bytes(&data);

            match packet.parsed {
                Some(p) => match p {
                    Response::Unparsed(p) => {
                        assert_eq!(p, "SLICKDENIS4000");
                    }
                    _ => panic!("not the right packet parsed"),
                },
                None => panic!("Packet parsing failed"),
            }
        }
    }
}