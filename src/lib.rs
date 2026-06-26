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
//! - [`connection`] - Main connection type for sending pixel data (requires `std`)
//! - [`protocol`] - DDP protocol types and structures (always available, `no_std`)
//! - [`packet`] - Packet parsing for receiving data from displays
//! - [`error`] - Error types used throughout the crate
//!
//! ## `no_std` / embedded use
//!
//! The crate is `no_std`-compatible. The default `std` feature gives the full library
//! (UDP [`connection`], JSON messages, the full error type) and is unchanged from previous
//! releases. For embedded targets:
//!
//! - `--no-default-features` — bare-metal core, **no allocator required**. Parse incoming
//!   packets with [`packet::PacketRef`] and build outgoing frames with
//!   [`protocol::FrameBuilder`], writing into your own buffers. Suitable for e.g. Teensy 4.1.
//! - `--no-default-features --features alloc` (alias: `embedded`) — adds the owned
//!   [`packet::Packet`] type for targets with a heap (e.g. ESP32).
//!
//! Socket I/O is the caller's responsibility in `no_std` builds.
//!
//!
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
pub mod connection;
pub mod error;
pub mod packet;
pub mod protocol;

#[cfg(all(test, feature = "std"))]
mod testing;
