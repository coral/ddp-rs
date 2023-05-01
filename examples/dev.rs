use anyhow::Result;
use ddp_rs::controller;
use std::{thread, time};

fn main() -> Result<()> {
    // Create new controller, listens on UDP 4080 by default
    let mut v = controller::Controller::new()?;

    // Connect to a DDP display with default pixel settings (RGB, 24 bits, ID 1)
    let (mut c, _) = v.connect(
        "10.0.1.9:4048",
        ddp_rs::protocol::PixelConfig::default(),
        ddp_rs::protocol::ID::default(),
    )?;

    // Write 4 pixels with no offset
    c.write(
        &vec![255, 255, 255, 128, 128, 12, 128, 128, 12, 128, 255, 12],
        0,
    )?;

    // WLED clears if we close the server so keeping it open to see result
    let ten_seconds = time::Duration::from_secs(10);
    thread::sleep(ten_seconds);

    Ok(())
}
