use crate::protocol;
use crate::protocol::status::StatusResponse;
use crossbeam::channel::{unbounded, Receiver, Sender};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use thiserror::Error;

#[derive(Debug)]
pub struct Controller {
    socket: UdpSocket,
    connections: Arc<DashMap<IpAddr, Sender<StatusResponse>>>,
}
const MAX_DATA_LENGTH: usize = 480 * 3;

pub struct Connection {
    pub pixel_config: protocol::PixelConfig,
    pub id: protocol::ID,

    sequence_number: u8,
    socket: UdpSocket,
    addr: SocketAddr,
    recv: Receiver<StatusResponse>,
}

impl Connection {
    pub fn write(&mut self, data: &[u8], offset: u32) -> Result<usize, DDPError> {
        let mut h = protocol::Header::default();

        h.packet_type.push(true);
        h.sequence_number = self.sequence_number;
        h.pixel_config = self.pixel_config;
        h.id = self.id;
        h.offset = offset;
        h.length = data.len() as u16;

        //Send data
        let p: Vec<u8> = protocol::Packet::from_data(h, data).into();
        let sent = self.socket.send_to(&p, self.addr)?;

        // Increment sequence number
        if self.sequence_number > 15 {
            self.sequence_number = 1;
        } else {
            self.sequence_number += 1;
        }

        Ok(sent)
    }
}

impl Controller {
    /// Create a new DDP controller instance
    ///
    /// Listens to the world on UDP port 4048.
    /// If that's not desired, use `Controller::new_with_socket` instead.
    pub fn new() -> Result<Controller, DDPError> {
        // Listen to world on 4048
        let socket = UdpSocket::bind("0.0.0.0:4048")?;

        let conn = Arc::new(DashMap::new());

        let socket_reciever = socket.try_clone()?;
        let conn_rec = conn.clone();

        thread::spawn(move || {
            // Define our receieve buffer, "1500 bytes should be enough for anyone".
            // Github copilot actually suggested that, so sassy LOL.
            let mut buf: [u8; 1500] = [0; 1500];

            loop {
                let (number_of_bytes, src_addr) = socket_reciever.recv_from(&mut buf).unwrap();
                //send(&socket, &client_arg, &msg_bytes);
                //dbg!(&buf[0..number_of_bytes]);
            }
        });

        Ok(Controller {
            socket,
            connections: conn,
        })
    }

    // fn handle_data(data: &[u8]) -> Result<protocol::Packet, DDPError> {

    // }

    // Basically new but you get to define your own socket if you want to use another port.
    // This is useful for binding localhost or other port or tbh whatever tf you want.
    // pub fn new_with_socket(socket: UdpSocket) -> Controller {
    //     Controller {
    //         socket,
    //         connections: HashSet::new(),
    //     }
    // }

    pub fn connect<A>(
        &mut self,
        addr: A,
        pixel_config: protocol::PixelConfig,
        id: protocol::ID,
    ) -> Result<Connection, DDPError>
    where
        A: std::net::ToSocketAddrs,
    {
        let socket_addr: SocketAddr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(DDPError::NoValidSocketAddr)?;
        let (s, recv) = unbounded();
        self.connections.insert(socket_addr.ip(), s);

        let socket = self.socket.try_clone()?;

        Ok(Connection {
            addr: socket_addr,
            pixel_config,
            id,
            socket,
            recv,
            sequence_number: 1,
        })
    }

    pub fn discover<A>(&mut self, addr: A)
    where
        A: std::net::ToSocketAddrs,
    {
    }
}

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
    phones: Vec<String>,
}

#[derive(Error, Debug)]
pub enum DDPError {
    #[error("socket error")]
    Disconnect(#[from] std::io::Error),
    #[error("No valid socket addr found")]
    NoValidSocketAddr,
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
