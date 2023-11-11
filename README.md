# Fork of [this](https://github.com/coral/ddp-rs) project by the wonderful Coral

This package allows you to write pixel data to a LED strip over [Distributed Display Protocol (DDP)](http://www.3waylabs.com/ddp/) by 3waylabs.

You can use this to stream pixel data to [WLED](https://github.com/Aircoookie/WLED) or any other DDP capable reciever.

## Example

```rust
use anyhow::Result;
use ddp_connection::connection;
use ddp_connection::protocol;

fn main() -> Result<()> {

    let mut conn = connection::DDPConnection::try_new
        (
            "192.168.1.40:4048",
            protocol::PixelConfig::default(),
            protocol::ID::Default,
            std::net::UdpSocket::bind("0.0.0.0:4048").unwrap()
        )?;

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

        std::thread::sleep(std::time::Duration::from_millis(10));

        println!("sent {temp} packets");
    }

    Ok(())
}

```

or try it by running `cargo run --example dev`

## Is it trash?

yes. but it works for WLED, and thats all I can test or care about.

## Why?

I wanted to stream color values to a WLED controller, and DDP has the highest framerate of any protocol I could find

## Contributing

m8 just open a PR with some gucchimucchi code and I'll review it.

![KADSBUGGEL](https://raw.githubusercontent.com/coral/fluidsynth2/master/kadsbuggel.png)
