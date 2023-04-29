use crate::protocol::status::StatusResponse;
use crossbeam::channel::{unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use thiserror::Error;

#[derive(Debug)]
pub struct Controller {
    //socket: UdpSocket,
    connections: HashMap<SocketAddr, Sender<StatusResponse>>,
}
fn listen(socket: &std::net::UdpSocket, mut buffer: &mut [u8]) -> usize {
    let (number_of_bytes, src_addr) = socket.recv_from(&mut buffer).expect("no data received");

    println!("{:?}", number_of_bytes);
    println!("{:?}", src_addr);

    number_of_bytes
}

impl Controller {
    /// Create a new DDP controller instance
    ///
    /// Listens to the world on UDP port 4048.
    /// If that's not desired, use `Controller::new_with_socket` instead.
    pub fn new() -> Result<Controller, DDPError> {
        let socket = UdpSocket::bind("0.0.0.0:4048")?;

        thread::spawn(move || {
            let s = socket;
            s.connect("0.0.0.0:4048").unwrap();
            let mut buf: Vec<u8> = Vec::with_capacity(100);

            loop {
                while listen(&s, &mut buf) != 0 {
                    println!("boo");
                }
                //send(&socket, &client_arg, &msg_bytes);
                println!("spin2win");
            }
        });

        Ok(Controller {
            // socket,
            connections: HashMap::new(),
        })
    }

    // Basically new but you get to define your own socket if you want to use another port.
    // This is useful for binding localhost or other port or tbh whatever tf you want.
    // pub fn new_with_socket(socket: UdpSocket) -> Controller {
    //     Controller {
    //         socket,
    //         connections: HashSet::new(),
    //     }
    // }

    pub fn connect<A>(&mut self, addr: A) -> Result<Receiver<StatusResponse>, DDPError>
    where
        A: std::net::ToSocketAddrs,
    {
        let socket: SocketAddr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(DDPError::NoValidSocketAddr)?;
        let (s, r) = unbounded();
        self.connections.insert(socket, s);

        Ok(r)
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
