# Higher level version of this (https://github.com/coral/ddp-rs) project by the wonderful Coral

This package allows you to write pixel data to a LED strip over [Distributed Display Protocol (DDP)](http://www.3waylabs.com/ddp/) by 3waylabs.

You can use this to stream pixel data to [WLED](https://github.com/Aircoookie/WLED) or any other DDP capable reciever.

## Example

```rust
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
    c.write(&vec![
        255, 255, 255, 128, 128, 12, 128, 128, 12, 128, 255, 12,
    ])?;

    // WLED clears if we close the server so keeping it open to see result
    let ten_seconds = time::Duration::from_secs(10);
    thread::sleep(ten_seconds);

    Ok(())
}
```

or try it by running `cargo run --example dev`

## Why?

I wish I could tell you. I've gone back and forth on these bespoke LED protocols and DDP seems like the most "sane" one although the "specification" leaves some to be desired. [TPM2.net](https://gist.github.com/jblang/89e24e2655be6c463c56) was another possible protocol which [i started to implement](https://github.com/coral/tpm2net) but stopped after I realized how bad it is. Artnet and E1.31 is great but then you have framerate problem (approx 40-44 FPS) to maintain backwards compatbility with DMX.

DDP sits in the middle here as "sane" but not perfect, hence why I implemented it for whatever it is I'm doing. It doesn't mandate a framerate, it's spec agnostic to if you send it over UDP or TCP (although I suspect most vendors only accept UDP) and it's open ended in that it relies on JSON for messaging. On top of that the author shoved so much data into the 10 byte header it's almost impressive. Only drawback is that clients needs to implement JSON parsing if they want to be smart but that's tablestakes at this point for anything connected.

For any future "i'm going to invent my own LED protocol" people out there, take note from the people in broadcast video instead of your jank ham radio serial protocol. I like the "freeform pixel struture" but there would probably be value to a more structured "session" where you standardize on how to communicate pixel size etc.

## Implemented

- [x] Multiple connections
- [x] Data slicing for more than 1500 bytes
- [x] JSON parsing for recieving
- [x] Sending JSON messages
- [x] Tests for most of spec
- [ ] Timecode
- [ ] Broadcast
- [ ] TCP

## Contributing

m8 just open a PR with some gucchimucchi code and I'll review it.

![KADSBUGGEL](https://raw.githubusercontent.com/coral/fluidsynth2/master/kadsbuggel.png)
