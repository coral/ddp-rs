use std::net::UdpSocket;
use thiserror::Error;

#[derive(Debug)]
pub struct Controller {
    socket: UdpSocket,
}

impl Controller {
    /// Create a new DDP controller instance
    ///
    /// Listens to the world on UDP port 4048.
    /// If that's not desired, use `Controller::new_with_socket` instead.
    pub fn new() -> Result<Controller, DDPError> {
        let socket = UdpSocket::bind("0.0.0.0:4048")?;
        Ok(Controller { socket })
    }

    // Basically new but you get to define your own socket if you want to use another port.
    pub fn new_with_socket(socket: UdpSocket) -> Controller {
        Controller { socket }
    }
}

#[derive(Error, Debug)]
pub enum DDPError {
    #[error("socket error")]
    Disconnect(#[from] std::io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown data store error")]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conn() {
        let conn = Controller::new();
        assert!(conn.is_ok());
    }
}
