use anyhow::Result;
use ddp_rs::connection;
use std::{thread, time};
use std::net::UdpSocket;
use ddp_rs::protocol::{ID, PixelConfig};

fn main() -> Result<()> {

    let mut conn = connection::DDPConnection::try_new
        (
            "192.168.1.40:4048",
            PixelConfig::default(),
            ID::Default,
            UdpSocket::bind("0.0.0.0:4048").unwrap()
        )?;

    thread::sleep(time::Duration::from_millis(100));

    for i in 0u8..100u8{
        let high = (10u8.overflowing_mul(i).0) % 255;

        // loop through some colors

        let temp: usize = conn.write(&vec![
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
