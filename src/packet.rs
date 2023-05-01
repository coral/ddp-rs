use crate::protocol::{response::Response, Header};

#[derive(Debug, PartialEq, Hash, Clone)]
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

        if header.packet_type.query {
            parsed = match header.id {
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
                _ => match std::str::from_utf8(&data) {
                    Ok(v) => Some(Response::Unparsed(v.to_string())),
                    Err(_) => None,
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
