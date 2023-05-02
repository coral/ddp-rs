use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Clone)]
pub struct StatusRoot {
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Clone)]
pub struct Status {
    pub update: Option<String>,
    pub state: Option<String>,
    pub man: Option<String>,
    #[serde(rename = "mod")]
    pub model: Option<String>,
    pub ver: Option<String>,
    pub mac: Option<String>,
    pub push: Option<bool>,
    pub ntp: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Clone)]
pub struct ConfigRoot {
    pub config: Config,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Clone)]
pub struct Config {
    pub ip: Option<String>,
    pub nm: Option<String>,
    pub gw: Option<String>,
    pub ports: Vec<Port>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Clone)]
pub struct Port {
    pub port: u32,
    pub ts: u32,
    pub l: u32,
    pub ss: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Clone)]
pub struct ControlRoot {
    pub control: Control,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Clone)]
pub struct Control {
    pub fx: Option<String>,
    pub int: Option<u32>,
    pub spd: Option<u32>,
    pub dir: Option<u32>,
    pub colors: Option<Vec<Color>>,
    pub save: Option<u32>,
    pub power: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Clone)]
pub struct Color {
    pub r: u32,
    pub g: u32,
    pub b: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Message {
    Control(ControlRoot),
    Status(StatusRoot),
    Config(ConfigRoot),
    Parsed(Value),
    Unparsed(String),
}

impl TryInto<Vec<u8>> for Message {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        match self {
            Message::Control(c) => serde_json::to_vec(&c),
            Message::Status(s) => serde_json::to_vec(&s),
            Message::Config(c) => serde_json::to_vec(&c),
            Message::Parsed(v) => serde_json::to_vec(&v),
            Message::Unparsed(s) => Ok(s.as_bytes().to_vec()),
        }
    }
}

impl Into<crate::protocol::ID> for Message {
    fn into(self) -> crate::protocol::ID {
        match self {
            Message::Control(_) => crate::protocol::ID::Control,
            Message::Status(_) => crate::protocol::ID::Status,
            Message::Config(_) => crate::protocol::ID::Config,
            Message::Parsed(_) => crate::protocol::ID::Control,
            Message::Unparsed(_) => crate::protocol::ID::Control,
        }
    }
}
