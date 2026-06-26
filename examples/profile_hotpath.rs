//! Tight-loop profiling harness for finding intra-function hotspots.
//!
//! Criterion (`cargo bench`) tells you *which* component is slow; a sampling profiler run
//! against this binary tells you *where inside* that component the time goes. It runs a single
//! hot path in a tight loop with no measurement overhead, which is exactly what flamegraph /
//! samply / perf want.
//!
//! Build optimized, then profile:
//! ```text
//! cargo build --release --example profile_hotpath
//!
//! # macOS / Linux, no sudo, opens a flamegraph in the browser:
//! samply record ./target/release/examples/profile_hotpath recv-ref 50000000
//!
//! # or via cargo-flamegraph:
//! cargo flamegraph --example profile_hotpath -- send 50000000
//! ```
//!
//! Modes:
//!   - `recv-ref`  zero-copy parse  (`PacketRef::from_bytes`, the no_std core path)
//!   - `recv-owned` owned parse     (`Packet::from_bytes`, alloc + optional JSON)
//!   - `send`       frame building  (`FrameBuilder::for_each_frame`)

use std::hint::black_box;

use ddp_rs::packet::{Packet, PacketRef};
use ddp_rs::protocol::{FrameBuilder, PixelConfig, ID};

fn make_pixel_packet(num_pixels: usize) -> Vec<u8> {
    let len = num_pixels * 3;
    let mut v = vec![0x41, 0x01, 0x0D, 0x01];
    v.extend_from_slice(&0u32.to_be_bytes());
    v.extend_from_slice(&(len as u16).to_be_bytes());
    v.extend((0..len).map(|i| (i % 256) as u8));
    v
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| "recv-ref".to_string());
    let iters: u64 = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50_000_000);

    // One full-MTU frame (480 RGB pixels = 1440 bytes payload).
    let packet = make_pixel_packet(480);
    let payload = packet[10..].to_vec();
    let mut scratch = [0u8; 1500];
    let mut acc = 0u64;

    eprintln!("mode={mode} iters={iters}");

    match mode.as_str() {
        "recv-ref" => {
            for _ in 0..iters {
                let p = PacketRef::from_bytes(black_box(packet.as_slice())).unwrap();
                acc += p.data.len() as u64 + p.header.offset as u64;
            }
        }
        "recv-owned" => {
            for _ in 0..iters {
                let p = Packet::from_bytes(black_box(packet.as_slice()));
                acc += p.data.len() as u64 + p.header.offset as u64;
            }
        }
        "send" => {
            let mut fb = FrameBuilder::new(PixelConfig::default(), ID::Default);
            for _ in 0..iters {
                fb.for_each_frame(black_box(payload.as_slice()), 0, &mut scratch, |frame| {
                    acc += frame.len() as u64;
                    Ok::<(), ()>(())
                })
                .unwrap();
            }
        }
        other => {
            eprintln!("unknown mode {other:?}; use recv-ref | recv-owned | send");
            std::process::exit(2);
        }
    }

    // Keep `acc` observable so nothing is optimized out.
    println!("{}", black_box(acc));
}
