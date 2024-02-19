use crate::error::DDPError;
use crate::error::DDPError::CrossBeamError;
use crate::packet::Packet;
use crate::protocol;
use crossbeam::channel::{unbounded, Receiver, TryRecvError};
use std::net::{SocketAddr, UdpSocket};

const MAX_DATA_LENGTH: usize = 480 * 3;

/// Represents a connection to a DDP display
#[derive(Debug)]
pub struct DDPConnection {
    pub pixel_config: protocol::PixelConfig,
    pub id: protocol::ID,

    sequence_number: u8,
    socket: UdpSocket,
    addr: SocketAddr,

    pub receiver_packet: Receiver<Packet>,

    // Since the buffer is hot path, we can reuse it to avoid allocations per packet
    buffer: [u8; 1500],
}

impl DDPConnection {
    /// Writes pixel data to the display
    ///
    /// You send the data
    pub fn write(&mut self, data: &[u8]) -> Result<usize, DDPError> {
        let mut h = protocol::Header::default();

        h.packet_type.push(false);
        h.pixel_config = self.pixel_config;
        h.id = self.id;

        self.slice_send(&mut h, data)
    }

    /// Writes pixel data to the display with offset
    ///
    /// You send the data with offset
    pub fn write_offset(&mut self, data: &[u8], offset: u32) -> Result<usize, DDPError> {
        let mut h = protocol::Header::default();

        h.packet_type.push(false);
        h.pixel_config = self.pixel_config;
        h.id = self.id;
        h.offset = offset;

        self.slice_send(&mut h, data)
    }

    /// Allows you to send JSON messages to display
    /// This is useful for things like setting the brightness
    /// or changing the display mode
    ///
    /// You provide a Message (either typed or untyped) and it will be sent to the display
    pub fn write_message(&mut self, msg: protocol::message::Message) -> Result<usize, DDPError> {
        let mut h = protocol::Header::default();
        h.packet_type.push(false);
        h.id = msg.get_id();
        let msg_data: Vec<u8> = msg.try_into()?;
        h.length = msg_data.len() as u16;

        self.slice_send(&mut h, &msg_data)
    }

    fn slice_send(
        &mut self,
        header: &mut protocol::Header,
        data: &[u8],
    ) -> Result<usize, DDPError> {
        let mut offset = header.offset as usize;
        let mut sent = 0;

        let num_iterations = (data.len() + MAX_DATA_LENGTH - 1) / MAX_DATA_LENGTH;
        let mut iter = 0;

        while offset < data.len() {
            iter += 1;

            if iter == num_iterations {
                header.packet_type.push(true);
            }

            header.sequence_number = self.sequence_number;

            let chunk_end = std::cmp::min(offset + MAX_DATA_LENGTH, data.len());
            let chunk = &data[offset..chunk_end];
            header.length = chunk.len() as u16;
            let len = self.assemble_packet(*header, chunk);

            // Send to socket
            sent += self.socket.send_to(&self.buffer[0..len], self.addr)?;

            // Increment sequence number
            if self.sequence_number > 15 {
                self.sequence_number = 1;
            } else {
                self.sequence_number += 1;
            }
            offset += MAX_DATA_LENGTH;
            header.offset = offset as u32;
        }

        Ok(sent)
    }

    pub fn get_incoming(&self) -> Result<Packet, DDPError> {
        match self.receiver_packet.try_recv() {
            Ok(packet) => Ok(packet),
            Err(TryRecvError::Empty) => Err(DDPError::NothingToReceive),
            Err(e2) => Err(CrossBeamError(e2)),
        }
    }

    pub fn try_new<A>(
        addr: A,
        pixel_config: protocol::PixelConfig,
        id: protocol::ID,
        socket: UdpSocket,
    ) -> Result<DDPConnection, DDPError>
    where
        A: std::net::ToSocketAddrs,
    {
        let socket_addr: SocketAddr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(DDPError::NoValidSocketAddr)?;
        let (_s, recv) = unbounded();

        Ok(DDPConnection {
            addr: socket_addr,
            pixel_config,
            id,
            socket,
            receiver_packet: recv,
            sequence_number: 1,
            buffer: [0u8; 1500],
        })
    }

    // doing this to avoid allocations per frame
    // micro optimization, but it's a hot path
    // esp running this embedded
    #[inline(always)]
    fn assemble_packet(&mut self, header: protocol::Header, data: &[u8]) -> usize {
        let header_bytes: usize = if header.packet_type.timecode {
            let header_bytes: [u8; 14] = header.into();
            self.buffer[0..14].copy_from_slice(&header_bytes);
            14usize
        } else {
            let header_bytes: [u8; 10] = header.into();
            self.buffer[0..10].copy_from_slice(&header_bytes);
            10usize
        };
        self.buffer[header_bytes..(header_bytes + data.len())].copy_from_slice(data);

        header_bytes + data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{PixelConfig, ID};
    use crossbeam::channel::unbounded;
    use std::thread;

    #[test]
    // Test sending to a loopback device
    fn test_conn() {
        let data_to_send = &vec![255, 0, 0, 255, 0, 0, 255, 0, 0];
        let (s, r) = unbounded();

        thread::spawn(move || {
            let socket = UdpSocket::bind("127.0.0.1:4048").unwrap();

            let mut buf = [0; 1500];
            let (amt, _) = socket.recv_from(&mut buf).unwrap();
            let buf = &mut buf[..amt];

            s.send(buf.to_vec()).unwrap();
        });

        let mut conn = DDPConnection::try_new(
            "127.0.0.1:4048",
            PixelConfig::default(),
            ID::Default,
            UdpSocket::bind("0.0.0.0:4049").unwrap(),
        )
        .unwrap();

        // Test simple send
        conn.write(data_to_send).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let recv_data = r.recv().unwrap();
        assert_eq!(
            &vec![
                0x41, 0x01, 0x0D, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0xFF, 0x00, 0x00, 0xFF,
                0x00, 0x00, 0xFF, 0x00, 0x00
            ],
            &recv_data
        );
    }
}
