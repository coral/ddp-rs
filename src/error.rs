use thiserror::Error;

#[derive(Error, Debug)]
pub enum DDPError {
    #[error("socket error")]
    Disconnect(#[from] std::io::Error),
    #[error("No valid socket addr found")]
    NoValidSocketAddr,
    #[error("parse error")]
    ParseError(#[from] serde_json::Error),
    #[error("invalid sender, did you forget to connect() ( data from {from:?} - {data:?})")]
    UnknownClient {
        from: std::net::SocketAddr,
        data: Vec<u8>,
    },
    #[error("Invalid packet")]
    InvalidPacket,
    #[error("There are no packets waiting to be read. This error should be handled explicitly")]
    NothingToReceive,
    #[error("Error receiving packet: {0}")]
    CrossBeamError(#[from] crossbeam::channel::TryRecvError),
}
