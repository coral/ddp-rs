# Distributed Display Protocol (DDP) in Rust

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
            "192.168.1.40:4048", // The IP address of the device followed by :4048
            protocol::PixelConfig::default(), // Default is RGB, 8 bits ber channel
            protocol::ID::Default,
            std::net::UdpSocket::bind("0.0.0.0:6969")
                .unwrap() // can be any unused port on 0.0.0.0, but protocol recommends 4048
        )?;

    // loop sets some colors for the first 6 pixels to see if it works
    for i in 0u8..100u8{
        let high = 10u8.overflowing_mul(i).0;

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
        // this crate is non blocking, so with out the sleep, it will send them all instantly

        println!("sent {temp} packets");
    }

    Ok(())
}
```

or try it by running `cargo run --example dev`

## Why?

I wish I could tell you. I've gone back and forth on these bespoke LED protocols and DDP seems like the most "sane" one although the "specification" leaves some to be desired. [TPM2.net](https://gist.github.com/jblang/89e24e2655be6c463c56) was another possible protocol which [i started to implement](https://github.com/coral/tpm2net) but stopped after I realized how bad it is. Artnet and E1.31 is great but then you have framerate problem (approx 40-44 FPS) to maintain backwards compatbility with DMX.

DDP sits in the middle here as "sane" but not perfect, hence why I implemented it for whatever it is I'm doing. It doesn't mandate a framerate, it's spec agnostic to if you send it over UDP or TCP (although I suspect most vendors only accept UDP) and it's open ended in that it relies on JSON for messaging. On top of that the author shoved so much data into the 10 byte header it's almost impressive. Only drawback is that clients needs to implement JSON parsing if they want to be smart but that's tablestakes at this point for anything connected.

For any future "i'm going to invent my own LED protocol" people out there, take note from the people in broadcast video instead of your jank ham radio serial protocol. I like the "freeform pixel struture" but there would probably be value to a more structured "session" where you standardize on how to communicate pixel size etc.

## Is it trash?

yes. but it works for WLED, and thats all I can test or care about.

## Contributing

m8 just open a PR with some gucchimucchi code and I'll review it.

![KADSBUGGEL](https://raw.githubusercontent.com/coral/fluidsynth2/master/kadsbuggel.png)

## Contributors

- [coral](https://www.youtube.com/@coral1), main clown of this library
- [paulwrath1223](https://github.com/paulwrath1223) coming in with a [hot potato PR](https://github.com/coral/ddp-rs/pull/1) Absolute legendary PR right there.
