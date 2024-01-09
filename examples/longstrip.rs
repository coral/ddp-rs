use anyhow::Result;
use colorgrad::Color;
use ddp_rs::connection;
use ddp_rs::protocol;

// Testing a longer LED strip with offset

fn main() -> Result<()> {
    let mut conn = connection::DDPConnection::try_new(
        "10.0.1.184:4048",
        protocol::PixelConfig::default(),
        protocol::ID::Default,
        std::net::UdpSocket::bind("0.0.0.0:4048").unwrap(),
    )?;

    let clr = argen(1200)?;

    loop {
        conn.write_offset(&clr, 100)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn argen(length: u32) -> Result<Vec<u8>> {
    let mut vec = Vec::new();
    let g = colorgrad::CustomGradient::new()
        .colors(&[
            Color::from_rgba8(255, 0, 0, 255),
            Color::from_rgba8(0, 255, 0, 255),
        ])
        .build()?;

    for i in 0..length {
        let color = g.at(i as f64 / length as f64).to_rgba8();
        vec.push(color[0] as u8);
        vec.push(color[1] as u8);
        vec.push(color[2] as u8);
    }

    Ok(vec)
}
