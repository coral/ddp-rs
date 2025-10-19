//! Console DDP Server Example
//!
//! This example creates a DDP server that listens for incoming packets
//! and displays them as a virtual LED strip in the terminal using ANSI colors.
//!
//! Usage:
//! ```
//! cargo run --example consoleserver
//! ```
//!
//! Then send DDP packets to localhost:4048 using the dev example or any DDP client.

use anyhow::Result;
use ddp_rs::packet::Packet;
use std::io::{self, Write};
use std::net::UdpSocket;

/// Console renderer that displays LED pixels as colored blocks in the terminal
struct ConsoleRenderer {
    num_pixels: usize,
    pixels: Vec<u8>,
}

impl ConsoleRenderer {
    /// Create a new console renderer with the specified number of pixels
    fn new(num_pixels: usize) -> Self {
        Self {
            num_pixels,
            pixels: vec![0; num_pixels * 3], // RGB data
        }
    }

    /// Update pixel data from a DDP packet
    fn update_from_packet(&mut self, packet: &Packet) {
        let offset = packet.header.offset as usize;
        let data = &packet.data;

        // Calculate the starting pixel index
        let start_pixel_idx = offset;

        // Copy the data into our pixel buffer at the correct offset
        if start_pixel_idx < self.pixels.len() {
            let end_idx = (start_pixel_idx + data.len()).min(self.pixels.len());
            let copy_len = end_idx - start_pixel_idx;
            self.pixels[start_pixel_idx..end_idx].copy_from_slice(&data[..copy_len]);
        }
    }

    /// Render the current pixel state to the console
    fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Move cursor to beginning of line and clear it
        write!(stdout, "\r\x1b[0m")?;

        // Render each pixel as a colored block using ANSI 24-bit color
        for i in 0..self.num_pixels {
            let idx = i * 3;
            if idx + 2 < self.pixels.len() {
                let r = self.pixels[idx];
                let g = self.pixels[idx + 1];
                let b = self.pixels[idx + 2];

                // Use ANSI 24-bit true color escape codes for background
                // Format: \x1b[48;2;R;G;Bm
                write!(stdout, "\x1b[48;2;{};{};{}m ", r, g, b)?;
            }
        }

        // Reset color and add space at end
        write!(stdout, "\x1b[0m ")?;
        stdout.flush()?;

        Ok(())
    }

    /// Clear the display
    fn clear(&mut self) -> io::Result<()> {
        self.pixels.fill(0);
        writeln!(io::stdout(), "\n\x1b[0m")?;
        Ok(())
    }
}

fn main() -> Result<()> {
    println!("DDP Console Server");
    println!("==================");
    println!("Listening on 0.0.0.0:4048");

    // Create UDP socket to listen for DDP packets
    let socket = UdpSocket::bind("0.0.0.0:4048")?;
    println!("Bound to {}\n", socket.local_addr()?);

    // Create a console renderer with 100 pixels (adjustable)
    let mut renderer = ConsoleRenderer::new(100);

    // Buffer for incoming packets (DDP max packet size is ~1500 bytes)
    let mut buf = [0u8; 2048];

    loop {
        // Receive a packet
        match socket.recv_from(&mut buf) {
            Ok((size, src)) => {
                // Parse the DDP packet
                let packet = Packet::from_bytes(&buf[..size]);

                // Update the display with new pixel data
                renderer.update_from_packet(&packet);

                // Render to console
                if let Err(e) = renderer.render() {
                    eprintln!("Render error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
            }
        }
    }

    // Clean up (unreachable in this infinite loop, but good practice)
    #[allow(unreachable_code)]
    {
        renderer.clear()?;
        Ok(())
    }
}
