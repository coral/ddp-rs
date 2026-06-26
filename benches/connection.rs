//! Full `std` send-path benchmark: framing + the UDP `send_to` syscall.
//!
//! This isolates how much of [`DDPConnection::write`] is the library's framing work (measured
//! standalone in `benches/send.rs`) versus the kernel syscall. We send to a loopback socket we
//! bind but never read: once its receive buffer fills the kernel silently drops datagrams, so
//! `send_to` keeps returning `Ok` and we measure the steady-state send cost without a receiver
//! thread perturbing the numbers.
//!
//! Run with: `cargo bench --bench connection`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ddp_rs::connection::DDPConnection;
use ddp_rs::protocol::{PixelConfig, ID};
use std::net::UdpSocket;

fn make_data(num_pixels: usize) -> Vec<u8> {
    (0..num_pixels * 3).map(|i| (i % 256) as u8).collect()
}

fn bench_loopback_send(c: &mut Criterion) {
    // Display socket: bound so the address is routable, never drained.
    let display = UdpSocket::bind("127.0.0.1:0").expect("bind display socket");
    let addr = display.local_addr().unwrap();
    let client = UdpSocket::bind("127.0.0.1:0").expect("bind client socket");
    let mut conn =
        DDPConnection::try_new(addr, PixelConfig::default(), ID::Default, client).unwrap();

    let mut group = c.benchmark_group("connection_send");
    // 60-LED ring and a full MTU: both fit in one frame -> exactly one syscall per write.
    for &n in &[60usize, 480] {
        let data = make_data(n);
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::new("write", n), &data, |b, data| {
            b.iter(|| conn.write(black_box(data.as_slice())).unwrap())
        });
    }
    group.finish();
}

criterion_group!(benches, bench_loopback_send);
criterion_main!(benches);
