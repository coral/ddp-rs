use anyhow::Result;
use ddp_rs::controller;
use std::{thread, time};

fn main() -> Result<()> {

    let mut v = controller::Controller::new()?;

    // Connect to a DDP display with default pixel settings (RGB, 24 bits, ID 1)
    let (mut c, _) = v.connect(

        "192.168.1.40:4048",    // the port is specified to always be 4048 in the DDP protocol

        ddp_rs::protocol::PixelConfig::default(),
        ddp_rs::protocol::ID::default(),
    )?;

    thread::sleep(time::Duration::from_millis(100));

    for i in 0u8..100u8{
        let high = (10u8.overflowing_mul(i).0) % 255;

        // loop through some colors

        let temp: usize = c.write(&vec![
            high/*red value*/, 0/*green value*/, 0/*blue value*/,
            high/*red value*/, 0/*green value*/, 0/*blue value*/,
            0/*red value*/, high/*green value*/, 0/*blue value*/,
            0/*red value*/, high/*green value*/, 0/*blue value*/,
            0/*red value*/, 0/*green value*/, high/*blue value*/,
            0/*red value*/, 0/*green value*/, high/*blue value*/
        ])?;

        thread::sleep(time::Duration::from_millis(10));

        println!("sent {temp} packets");
    }

    Ok(())
}
