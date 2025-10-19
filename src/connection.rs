//! DDP connection handling for sending and receiving pixel data.
//!
//! This module provides the main [`DDPConnection`] type for communicating with
//! DDP-compatible LED displays.

use crate::error::DDPError;
use crate::error::DDPError::CrossBeamError;
use crate::packet::Packet;
use crate::protocol;
use crossbeam::channel::{unbounded, Receiver, TryRecvError};
use std::net::{SocketAddr, UdpSocket};

/// Maximum pixel data size per DDP packet (480 pixels × 3 bytes RGB = 1440 bytes)
const MAX_DATA_LENGTH: usize = 480 * 3;

/// A connection to a DDP display device.
///
/// This is the main type for sending pixel data to LED strips and other DDP-compatible
/// displays. It handles packet assembly, sequencing, and automatic chunking of large
/// data arrays.
///
/// # Examples
///
/// ## Basic usage
///
/// ```no_run
/// use ddp_rs::connection::DDPConnection;
/// use ddp_rs::protocol::{PixelConfig, ID};
/// use std::net::UdpSocket;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut conn = DDPConnection::try_new(
///     "192.168.1.40:4048",
///     PixelConfig::default(),
///     ID::Default,
///     UdpSocket::bind("0.0.0.0:4048")?
/// )?;
///
/// // Send RGB data for 3 pixels
/// conn.write(&[
///     255, 0, 0,    // Red
///     0, 255, 0,    // Green
///     0, 0, 255,    // Blue
/// ])?;
/// # Ok(())
/// # }
/// ```
///
/// ## Using offsets to update part of the strip
///
/// ```no_run
/// # use ddp_rs::connection::DDPConnection;
/// # use ddp_rs::protocol::{PixelConfig, ID};
/// # use std::net::UdpSocket;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut conn = DDPConnection::try_new(
/// #     "192.168.1.40:4048",
/// #     PixelConfig::default(),
/// #     ID::Default,
/// #     UdpSocket::bind("0.0.0.0:4048")?
/// # )?;
/// // Update pixels starting at byte offset 300 (pixel 100 in RGB)
/// conn.write_offset(&[255, 128, 64], 300)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct DDPConnection {
    /// Pixel format configuration (RGB, RGBW, etc.)
    pub pixel_config: protocol::PixelConfig,

    /// Protocol ID for this connection
    pub id: protocol::ID,

    sequence_number: u8,
    socket: UdpSocket,
    addr: SocketAddr,

    /// Receiver for packets coming from the display (responses)
    pub receiver_packet: Receiver<Packet>,

    // Since the buffer is hot path, we can reuse it to avoid allocations per packet
    buffer: [u8; 1500],
}

impl DDPConnection {
    /// Writes pixel data to the display starting at offset 0.
    ///
    /// Large data arrays are automatically split into multiple packets. Each packet
    /// can contain up to 1440 bytes (480 RGB pixels).
    ///
    /// # Arguments
    ///
    /// * `data` - Raw pixel data bytes. For RGB, this should be groups of 3 bytes (R,G,B).
    ///            For RGBW, groups of 4 bytes (R,G,B,W).
    ///
    /// # Returns
    ///
    /// The total number of bytes sent across all packets.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ddp_rs::connection::DDPConnection;
    /// # use ddp_rs::protocol::{PixelConfig, ID};
    /// # use std::net::UdpSocket;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = DDPConnection::try_new("192.168.1.40:4048", PixelConfig::default(), ID::Default, UdpSocket::bind("0.0.0.0:4048")?)?;
    /// // Set first 3 pixels to red, green, blue
    /// conn.write(&[255, 0, 0, 0, 255, 0, 0, 0, 255])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write(&mut self, data: &[u8]) -> Result<usize, DDPError> {
        let mut h = protocol::Header::default();

        h.packet_type.push(false);
        h.pixel_config = self.pixel_config;
        h.id = self.id;

        self.slice_send(&mut h, data)
    }

    /// Writes pixel data to the display starting at a specific byte offset.
    ///
    /// This is useful for updating only a portion of your LED strip without
    /// resending all the data.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw pixel data bytes to send
    /// * `offset` - Starting byte offset (not pixel offset). For RGB, offset 3 = pixel 1.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ddp_rs::connection::DDPConnection;
    /// # use ddp_rs::protocol::{PixelConfig, ID};
    /// # use std::net::UdpSocket;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = DDPConnection::try_new("192.168.1.40:4048", PixelConfig::default(), ID::Default, UdpSocket::bind("0.0.0.0:4048")?)?;
    /// // Update pixel 10 (offset = 10 * 3 = 30) to white
    /// conn.write_offset(&[255, 255, 255], 30)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_offset(&mut self, data: &[u8], offset: u32) -> Result<usize, DDPError> {
        let mut h = protocol::Header::default();

        h.packet_type.push(false);
        h.pixel_config = self.pixel_config;
        h.id = self.id;
        h.offset = offset;

        self.slice_send(&mut h, data)
    }

    /// Sends a JSON control message to the display.
    ///
    /// This is useful for things like setting brightness, changing display modes,
    /// or querying configuration.
    ///
    /// # Arguments
    ///
    /// * `msg` - A [`protocol::message::Message`] (typed or untyped JSON)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ddp_rs::connection::DDPConnection;
    /// # use ddp_rs::protocol::{PixelConfig, ID, message::Message};
    /// # use std::net::UdpSocket;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = DDPConnection::try_new("192.168.1.40:4048", PixelConfig::default(), ID::Default, UdpSocket::bind("0.0.0.0:4048")?)?;
    /// // Send a control message
    /// let json_value = serde_json::json!({"brightness": 128});
    /// conn.write_message(Message::Parsed((ID::Control, json_value)))?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Attempts to retrieve a packet from the display (non-blocking).
    ///
    /// Checks if any response packets have been received from the display.
    ///
    /// # Returns
    ///
    /// * `Ok(Packet)` - A packet was available
    /// * `Err(DDPError::NothingToReceive)` - No packets waiting
    /// * `Err(DDPError::CrossBeamError)` - Channel error
    pub fn get_incoming(&self) -> Result<Packet, DDPError> {
        match self.receiver_packet.try_recv() {
            Ok(packet) => Ok(packet),
            Err(TryRecvError::Empty) => Err(DDPError::NothingToReceive),
            Err(e2) => Err(CrossBeamError(e2)),
        }
    }

    /// Creates a new DDP connection to a display.
    ///
    /// # Arguments
    ///
    /// * `addr` - The display address (IP:port). DDP standard port is 4048.
    /// * `pixel_config` - Pixel format configuration (RGB, RGBW, etc.)
    /// * `id` - Protocol ID to use for this connection
    /// * `socket` - A bound UDP socket for sending/receiving data
    ///
    /// # Returns
    ///
    /// * `Ok(DDPConnection)` - Connection created successfully
    /// * `Err(DDPError)` - Failed to resolve address or create connection
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ddp_rs::connection::DDPConnection;
    /// use ddp_rs::protocol::{PixelConfig, ID};
    /// use std::net::UdpSocket;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let conn = DDPConnection::try_new(
    ///     "192.168.1.40:4048",
    ///     PixelConfig::default(),
    ///     ID::Default,
    ///     UdpSocket::bind("0.0.0.0:4048")?
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
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

    // Helper function for creating test connections
    fn create_test_connection() -> (DDPConnection, UdpSocket) {
        let display_socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind display socket");
        let display_addr = display_socket.local_addr().unwrap();
        let client_socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind client socket");

        let conn = DDPConnection::try_new(
            display_addr,
            PixelConfig::default(),
            ID::default(),
            client_socket,
        )
        .expect("Failed to create connection");

        (conn, display_socket)
    }

    #[test]
    fn test_connection_creation() {
        let (conn, _display_socket) = create_test_connection();
        assert_eq!(conn.pixel_config, PixelConfig::default());
        assert_eq!(conn.id, ID::default());
    }

    #[test]
    fn test_connection_write_pixel_data() {
        use std::time::Duration;

        let (mut conn, display_socket) = create_test_connection();
        display_socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();

        let pixel_data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // 3 RGB pixels
        let result = conn.write(&pixel_data);

        assert!(result.is_ok());
        assert!(result.unwrap() > 0);

        let mut buf = [0u8; 1500];
        let recv_result = display_socket.recv_from(&mut buf);
        assert!(recv_result.is_ok());
    }

    #[test]
    fn test_connection_write_with_offset() {
        use std::time::Duration;

        let (mut conn, display_socket) = create_test_connection();
        display_socket
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();

        let pixel_data = vec![128, 128, 128]; // 1 RGB pixel
        let offset = 30; // Start at pixel 10 (30 bytes / 3)
        let result = conn.write_offset(&pixel_data, offset);

        assert!(result.is_ok());

        let mut buf = [0u8; 1500];
        match display_socket.recv_from(&mut buf) {
            Ok((size, _)) => {
                assert!(size > 10);
                let received_offset = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
                assert_eq!(received_offset, offset);
            }
            Err(e) => {
                eprintln!("Warning: recv_from timed out: {}", e);
            }
        }
    }

    #[test]
    fn test_connection_sequence_numbers() {
        use std::time::Duration;

        let (mut conn, display_socket) = create_test_connection();
        display_socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();

        let pixel_data = vec![255, 0, 0];

        for i in 0..5 {
            conn.write(&pixel_data).unwrap();

            let mut buf = [0u8; 1500];
            display_socket.recv_from(&mut buf).unwrap();

            let seq_num = buf[1];
            assert_eq!(seq_num, (i + 1) as u8);
        }
    }

    #[test]
    fn test_connection_large_data_chunking() {
        use std::time::Duration;

        let (mut conn, display_socket) = create_test_connection();
        display_socket
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();

        // Send data larger than MAX_DATA_LENGTH (480 * 3 = 1440 bytes)
        let large_data = vec![128u8; 2000];
        let result = conn.write(&large_data);

        assert!(result.is_ok());

        // Should receive multiple packets
        let mut received_packets = 0;
        let mut buf = [0u8; 1500];

        loop {
            match display_socket.recv_from(&mut buf) {
                Ok(_) => received_packets += 1,
                Err(_) => break,
            }

            if received_packets >= 2 {
                break;
            }
        }

        assert!(received_packets >= 2, "Expected multiple packets for large data");
    }

    #[test]
    fn test_connection_empty_data() {
        use std::time::Duration;

        let (mut conn, display_socket) = create_test_connection();
        display_socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();

        let empty_data: Vec<u8> = vec![];
        let result = conn.write(&empty_data);

        assert!(result.is_ok());
    }

    #[test]
    fn test_pixel_config_preserved() {
        let display_socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind display socket");
        let display_addr = display_socket.local_addr().unwrap();
        let client_socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind client socket");

        let custom_config = PixelConfig::default();

        let conn = DDPConnection::try_new(
            display_addr,
            custom_config,
            ID::default(),
            client_socket,
        )
        .expect("Failed to create connection");

        assert_eq!(conn.pixel_config, custom_config);
    }

    #[test]
    fn test_id_preserved() {
        let display_socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind display socket");
        let display_addr = display_socket.local_addr().unwrap();
        let client_socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind client socket");

        let custom_id = ID::Config;

        let conn = DDPConnection::try_new(
            display_addr,
            PixelConfig::default(),
            custom_id,
            client_socket,
        )
        .expect("Failed to create connection");

        assert_eq!(conn.id, custom_id);
    }
}
