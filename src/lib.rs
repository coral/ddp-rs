/// # Distributed Display Protocol (DDP) in Rust
///
///  This package allows you to write pixel data to a LED strip over [DDP](http://www.3waylabs.com/ddp/)
///
//You can use this to stream pixel data to [WLED](https://github.com/Aircoookie/WLED) or any other DDP capable reciever.
/// ## Example
/// ```rust
/// use anyhow::Result;
/// use ddp_rs::controller;
/// use std::{thread, time};
///
/// fn main() -> Result<()> {
///     // Create new controller, listens on UDP 4080 by default
///     let mut v = controller::Controller::new()?;
///
///     // Connect to a DDP display with default pixel settings (RGB, 24 bits, ID 1)
///     let (mut c, _) = v.connect(
///         "10.0.1.9:4048",
///         ddp_rs::protocol::PixelConfig::default(),
///         ddp_rs::protocol::ID::default(),
///     )?;
///
///     // Write 4 pixels with no offset
///     c.write(&vec![
///     255, 255, 255, 128, 128, 12, 128, 128, 12, 128, 255, 12,
///     ])?;
///
///     // WLED clears if we close the server so keeping it open to see result
///     let ten_seconds = time::Duration::from_secs(10);
///     thread::sleep(ten_seconds);
///
///     Ok(())
/// }
/// ```
///
pub mod controller;
pub mod error;
pub mod packet;
pub mod protocol;
