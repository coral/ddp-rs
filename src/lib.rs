//! # Distributed Display Protocol (DDP) in Rust
//!
//! This crate allows you to write pixel data to LED strips over the
//! [Distributed Display Protocol (DDP)](http://www.3waylabs.com/ddp/) by 3waylabs.
//!
//! You can use this to stream pixel data to [WLED](https://github.com/Aircoookie/WLED)
//! or any other DDP-capable receiver.
//!
//! ## Quick Start
//!
//! ```no_run
//! use ddp_rs::connection::DDPConnection;
//! use ddp_rs::protocol::{PixelConfig, ID};
//! use std::net::UdpSocket;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a connection to your LED controller
//! let mut conn = DDPConnection::try_new(
//!     "192.168.1.40:4048",              // Device IP and DDP port
//!     PixelConfig::default(),            // RGB, 8 bits per channel
//!     ID::Default,                       // Default ID
//!     UdpSocket::bind("0.0.0.0:6969")?  // Local socket
//! )?;
//!
//! // Send RGB pixel data (2 pixels: red and blue)
//! conn.write(&[
//!     255, 0, 0,    // First pixel: Red
//!     0, 0, 255,    // Second pixel: Blue
//! ])?;
//! # Ok(())
//! # }
//! ```
//!
//!
//! ## Modules
//!
//! - [`connection`] - Main connection type for sending pixel data
//! - [`protocol`] - DDP protocol types and structures
//! - [`packet`] - Packet parsing for receiving data from displays
//! - [`error`] - Error types used throughout the crate
//!
//! 
pub mod connection;
pub mod error;
pub mod packet;
pub mod protocol;

#[cfg(test)]
mod testing;
